//! Service 层入口：声明所有 service trait。
//! service 编排 repo + cache + domain，是 handler 的直接调用对象。

pub mod admin_service;
pub mod agent_orchestrator;
pub mod agent_service;
pub mod agent_snapshot_service;
pub mod community_service;
pub mod config_service;
pub mod essence_service;
pub mod local_game_service;
pub mod log_service;
pub mod match_service;
pub mod match_supervisor;
pub mod rank_service;
pub mod room_service;
pub mod scenario_service;
pub mod spawn_preset_service;
pub mod user_service;

pub use admin_service::{AdminMetrics, AdminService, AdminServiceImpl};
pub use agent_orchestrator::{SceneAgentConfig, run_agent_orchestrator};
pub use agent_service::{AgentLimitProvider, AgentService, AgentServiceImpl};
pub use agent_snapshot_service::{
    AgentSnapshotService, AgentSnapshotServiceImpl, build_config_freeze,
};
pub use community_service::{CommunityService, CommunityServiceImpl};
pub use config_service::{ConfigService, ConfigServiceImpl};
pub use essence_service::{
    CheckInResult, EssenceService, EssenceServiceImpl, SubscriptionService, SubscriptionServiceImpl,
};
pub use local_game_service::{
    CommandProcessLauncher, LocalGameService, LocalGameServiceImpl, LocalGameState,
    LocalStartInput, ProcessLauncher,
};
pub use log_service::{
    LogReader, LogService, LogServiceImpl, QueryLogsParams, QueryLogsResult, SqliteLogReader,
};
pub use match_service::{MatchService, MatchServiceImpl};
pub use rank_service::{RankMatchCreator, RankService, RankServiceImpl};
pub use room_service::{RoomService, RoomServiceImpl};
pub use scenario_service::{ScenarioService, ScenarioServiceImpl};
pub use spawn_preset_service::{SpawnPresetService, SpawnPresetServiceImpl};
pub use user_service::{AuthResult, UserService, UserServiceImpl};
