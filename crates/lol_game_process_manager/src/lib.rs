//! 游戏进程托管内核：端口池 + 进程表 + spawn/kill 抽象。
//!
//! 桌面端（本地调试：起多游戏 + WS 交互）与云端（竞技对局 `LocalGameService`）
//! 共用本 crate 做进程托管，对局体系（match 记录、胜负判定、supervisor）由各端
//! 自行叠加。本 crate 不含对局语义、不依赖 Bevy / Postgres / HTTP。
//!
//! spawn 命令构建复用 [`lol_client::launch`]（`BevySpawnRequest` / `build_command`）；
//! AI 决策环由调用方注入的 [`AgentRunner`] 承载（内部走 `lol_agent_runtime`）。

mod error;
mod manager;
mod port_pool;

pub use error::{ManagerError, ManagerResult};
pub use manager::{
    AgentRunner, GameProcessManager, ManagedProcess, ProcessLauncher, ProcessManagerState,
    StartGameInput,
};
pub use port_pool::{PORT_POOL_END, PORT_POOL_START, ProcessStatus, allocate_port, is_valid_port};
