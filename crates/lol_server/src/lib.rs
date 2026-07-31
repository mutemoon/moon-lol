pub mod handlers;
pub mod protocol;
pub mod server;

use async_channel::Receiver;
use bevy::prelude::*;
use lol_core::match_events::{MatchEventChannel, MatchEventOut};

/// 通用 WS 服务基础设施：因为粒子渲染 server 等非对局进程也需要 WS RPC 能力，
/// 所以把「启动监听 + 轮询命令分发」抽为独立插件，不含任何对局事件语义。
pub struct PluginWsServer {
    pub ws_port: u16,
}

impl Plugin for PluginWsServer {
    fn build(&self, app: &mut App) {
        app.insert_resource(WsServerPort(self.ws_port));

        app.add_systems(Startup, startup_ws_server);

        app.add_systems(Update, |world: &mut World| {
            server::poll_commands(world);
        });
    }
}

/// WS 监听端口，由 PluginWsServer 插入。
#[derive(Resource)]
struct WsServerPort(u16);

/// 具名 Startup 系统：因为其它插件的 Startup 系统（如 game_loaded 广播）需要
/// 通过 .after() 确保 DebugWsChannel 已插入，所以不能用匿名闭包。
fn startup_ws_server(world: &mut World) {
    let port = world.resource::<WsServerPort>().0;
    server::start(world, port);
}

pub struct PluginServer {
    pub ws_port: u16,
}

impl Plugin for PluginServer {
    fn build(&self, app: &mut App) {
        // 复用通用 WS 基础设施（监听 + 命令轮询）。
        app.add_plugins(PluginWsServer {
            ws_port: self.ws_port,
        });

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

        app.add_systems(
            Startup,
            (|world: &mut World| {
                server::send_event(world, protocol::WsEvent::game_loaded());
            })
            .after(startup_ws_server),
        );

        app.add_systems(Update, forward_match_events);
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
