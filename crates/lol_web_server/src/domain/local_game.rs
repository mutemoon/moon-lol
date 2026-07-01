//! LocalGame 子系统的领域层：端口池与进程状态。
//!
//! 纯逻辑已迁至共享 crate `lol_game_process_manager`，本模块仅做 re-export，
//! 供 web_server 内部沿用既有路径（`crate::domain::local_game::...`）。
//! 端口池范围 [9100, 9200)，最多 100 个并发对局。

pub use lol_game_process_manager::{
    ManagedProcess, PORT_POOL_END, PORT_POOL_START, ProcessStatus, allocate_port, is_valid_port,
};
