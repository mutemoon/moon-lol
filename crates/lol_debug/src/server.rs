use std::sync::Arc;
use std::thread;

use async_channel::{Receiver, Sender};
use bevy::prelude::*;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex as TokioMutex;
use tokio_tungstenite::accept_async;

use crate::protocol::{CmdKind, WsEvent, WsRequest};

/// Channel for Bevy ← WS communication.
#[derive(Resource)]
pub struct DebugWsChannel {
    /// Receives commands from WS connections.
    pub cmd_rx: Receiver<(u64, CmdKind, serde_json::Value)>,
    /// Sends JSON-serialized WsResponse / WsEvent strings to all connected WS clients.
    pub out_tx: Sender<String>,
}

/// Start the tokio WS server on a background thread.
/// Inserts DebugWsChannel into the world so Bevy systems can poll it.
pub fn start(world: &mut World, port: u16) {
    let (cmd_tx, cmd_rx) = async_channel::unbounded::<(u64, CmdKind, serde_json::Value)>();
    let (out_tx, out_rx) = async_channel::unbounded::<String>();

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async move {
            let listener = TcpListener::bind(("127.0.0.1", port))
                .await
                .expect("failed to bind WS port");

            // Connected clients' write halves.
            let clients: Arc<TokioMutex<Vec<tokio::sync::mpsc::UnboundedSender<String>>>> =
                Arc::new(TokioMutex::new(vec![]));

            // Fan-out task: reads from async_channel, sends to every connected client.
            let clients_fanout = clients.clone();
            tokio::spawn(async move {
                loop {
                    let Ok(msg) = out_rx.recv().await else {
                        break;
                    };
                    let mut clients = clients_fanout.lock().await;
                    clients.retain(|tx| tx.send(msg.clone()).is_ok());
                }
            });

            // Accept loop.
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    continue;
                };
                let Ok(ws_stream) = accept_async(stream).await else {
                    continue;
                };

                let (mut write, mut read) = ws_stream.split();
                let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

                clients.lock().await.push(client_tx);

                let cmd_tx = cmd_tx.clone();

                // Read task: WS messages → command channel.
                tokio::spawn(async move {
                    while let Some(Ok(msg)) = read.next().await {
                        let text = match msg.to_text() {
                            Ok(t) => t,
                            Err(_) => continue,
                        };
                        let req: WsRequest = match serde_json::from_str(text) {
                            Ok(r) => r,
                            Err(_) => continue,
                        };
                        let _ = cmd_tx.send((req.id, req.cmd, req.params)).await;
                    }
                });

                // Write task: client_rx → WS frames.
                tokio::spawn(async move {
                    while let Some(msg) = client_rx.recv().await {
                        if write
                            .send(tokio_tungstenite::tungstenite::Message::Text(msg.into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                });
            }
        });
    });

    world.insert_resource(DebugWsChannel { cmd_rx, out_tx });
}

/// Bevy Update system — poll incoming commands and dispatch to handlers.
/// Runs every frame; non-blocking via try_recv.
pub fn poll_commands(world: &mut World) {
    let cmd_rx = world
        .get_resource::<DebugWsChannel>()
        .map(|ch| ch.cmd_rx.clone());

    let Some(cmd_rx) = cmd_rx else {
        return;
    };

    while let Ok((id, cmd, params)) = cmd_rx.try_recv() {
        let response = crate::handlers::dispatch(world, id, cmd, params);
        let out_tx = world
            .get_resource::<DebugWsChannel>()
            .map(|ch| ch.out_tx.clone());

        if let Some(out_tx) = out_tx {
            if let Ok(json) = serde_json::to_string(&response) {
                let _ = out_tx.try_send(json);
            }
        }
    }
}

/// Send a WsEvent to all connected WS clients.
pub fn send_event(world: &World, event: WsEvent) {
    if let Some(ch) = world.get_resource::<DebugWsChannel>() {
        if let Ok(json) = serde_json::to_string(&event) {
            let _ = ch.out_tx.try_send(json);
        }
    }
}
