pub mod events;
pub mod handlers;
pub mod protocol;
pub mod server;

use async_channel::Receiver;
use bevy::prelude::*;

use lol_core::match_events::{MatchEventChannel, MatchEventOut};

pub struct PluginServer {
    pub ws_port: u16,
}

impl Plugin for PluginServer {
    fn build(&self, app: &mut App) {
        let port = self.ws_port;

        // 创建对局事件通道：tx 交给 lol_core 的 MatchEventChannel（产出方写入），
        // rx 由本插件的转发系统轮询并推到 WS。
        // 若 MatchEventChannel 已被外部插入，则复用之；否则在此创建。
        if !app.world().contains_resource::<MatchEventChannel>() {
            let (tx, rx) = async_channel::unbounded::<MatchEventOut>();
            app.insert_resource(MatchEventChannel { tx });
            app.insert_resource(MatchEventReceiver { rx });
        } else {
            // 外部已插入 tx，但没有 rx —— 这种情况通常不会发生（本插件是唯一创建者）。
            // 为安全起见，重新建立一对并替换 tx。
            let (tx, rx) = async_channel::unbounded::<MatchEventOut>();
            app.insert_resource(MatchEventChannel { tx });
            app.insert_resource(MatchEventReceiver { rx });
        }

        app.add_systems(Startup, move |world: &mut World| {
            server::start(world, port);
            server::send_event(world, protocol::WsEvent::game_loaded());
        });

        app.add_systems(Update, |world: &mut World| {
            server::poll_commands(world);
            forward_match_events(world);
        });
    }
}

/// 持有对局事件通道的接收端，由 forward_match_events 轮询。
#[derive(Resource)]
pub struct MatchEventReceiver {
    pub rx: Receiver<MatchEventOut>,
}

/// 将 lol_core 产出的对局事件转发给所有 WS 客户端。
fn forward_match_events(world: &mut World) {
    let Some(events) = world
        .get_resource::<MatchEventReceiver>()
        .map(|r| r.rx.clone())
    else {
        return;
    };

    let mut batch = Vec::new();
    while let Ok(ev) = events.try_recv() {
        batch.push(ev);
    }

    for ev in batch {
        let payload = serde_json::to_value(&ev).unwrap_or_else(|_| serde_json::json!({}));
        server::send_event(world, protocol::WsEvent::match_event(payload));
    }
}
