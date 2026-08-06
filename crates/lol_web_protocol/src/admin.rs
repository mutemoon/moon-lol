//! Admin wire DTO（运维指标）。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminMetrics {
    pub running_matches: usize,
    pub pending_matches: usize,
    pub queued_agents: usize,
    pub managed_processes: usize,
}
