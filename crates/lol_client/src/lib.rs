//! Bevy-free 的游戏客户端：协议类型、WS 会话、类型化命令面与 MCP 工具层。
//!
//! CLI、MCP、Tauri 后端三处都依赖本 crate，不再各自持有一份协议或会话代码。

pub mod action;
pub mod game_client;
pub mod launch;
pub mod mcp;
pub mod protocol;
pub mod session;

pub use action::Action;
pub use game_client::GameClient;
pub use mcp::{GameToolServer, serve_inprocess};
pub use protocol::{WsEvent, WsRequest, WsResponse};
pub use session::{WsSession, start_ws_client};
