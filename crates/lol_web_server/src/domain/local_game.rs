//! LocalGame 子系统的领域层（本地对局进程托管）。
//!
//! 职责：管理 Bevy 子进程的端口分配、启动/停止生命周期。
//! 端口池范围 [9100, 9200)，最多 100 个并发对局。

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 端口池范围。
pub const PORT_POOL_START: i32 = 9100;
pub const PORT_POOL_END: i32 = 9200;

/// 进程状态。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopped,
    Crashed,
}

impl ProcessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessStatus::Starting => "starting",
            ProcessStatus::Running => "running",
            ProcessStatus::Stopped => "stopped",
            ProcessStatus::Crashed => "crashed",
        }
    }
}

/// 从已用端口集合中分配一个空闲端口。
///
/// 返回 [PORT_POOL_START, PORT_POOL_END) 内第一个不在 used 中的端口。
/// 池满返回 None。
pub fn allocate_port(used: &HashSet<i32>) -> Option<i32> {
    (PORT_POOL_START..PORT_POOL_END).find(|p| !used.contains(p))
}

/// 校验端口是否在合法池范围内。
pub fn is_valid_port(port: i32) -> bool {
    (PORT_POOL_START..PORT_POOL_END).contains(&port)
}

/// 托管进程信息（运行时状态，不持久化——进程退出即失效）。
#[derive(Debug, Clone)]
pub struct ManagedProcess {
    pub match_id: uuid::Uuid,
    pub port: i32,
    pub status: ProcessStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_first_port_when_empty() {
        let used = HashSet::new();
        assert_eq!(allocate_port(&used), Some(9100));
    }

    #[test]
    fn allocate_skips_used_ports() {
        let mut used = HashSet::new();
        used.insert(9100);
        used.insert(9101);
        assert_eq!(allocate_port(&used), Some(9102));
    }

    #[test]
    fn allocate_returns_none_when_full() {
        let used: HashSet<i32> = (PORT_POOL_START..PORT_POOL_END).collect();
        assert_eq!(allocate_port(&used), None);
    }

    #[test]
    fn allocate_finds_gap_in_middle() {
        let mut used = HashSet::new();
        used.insert(9100);
        used.insert(9102);
        // 9100 占了，9101 空闲
        assert_eq!(allocate_port(&used), Some(9101));
    }

    #[test]
    fn is_valid_port_in_range() {
        assert!(is_valid_port(9100));
        assert!(is_valid_port(9199));
        assert!(!is_valid_port(9099));
        assert!(!is_valid_port(9200));
        assert!(!is_valid_port(0));
    }

    #[test]
    fn pool_capacity_is_100() {
        assert_eq!(PORT_POOL_END - PORT_POOL_START, 100);
    }

    #[test]
    fn process_status_roundtrip() {
        assert_eq!(ProcessStatus::Running.as_str(), "running");
        assert_eq!(ProcessStatus::Crashed.as_str(), "crashed");
    }
}
