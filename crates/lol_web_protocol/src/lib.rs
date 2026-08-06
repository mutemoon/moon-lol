//! `lol_web_protocol` — MoonLOL cloud REST / 本地对局的共享 wire DTO。
//!
//! 所有跨进程/跨服务类型（纯 serde，无 IO 依赖）的单一事实源。
//! `lol_web_server`（feature = "axum"）与 `client` 共同依赖本 crate。

pub mod admin;
pub mod agent;
pub mod agent_snapshot;
pub mod auth;
pub mod envelope;
pub mod essence;
pub mod game;
pub mod history;
pub mod match_;
pub mod model_provider;
pub mod rank;
pub mod room;
pub mod scenario;
pub mod spawn_preset;

// ── 常用类型重导出 ──

pub use agent::AgentType;
pub use envelope::{ApiError, ApiResponse, ERROR_INTERNAL, ERROR_NOT_FOUND, ERROR_UNAUTHORIZED};
pub use game::{FrontAgentConfig, GameConfig, RunningGame};
pub use match_::{MatchStatus, Winner};
pub use model_provider::{ApiFormat, ProviderCategory};
pub use spawn_preset::{Team, Visibility};
