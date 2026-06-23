//! Repository 层入口：声明所有 repo trait。
//! 每个 repo trait 对应一组数据访问语义，impl 翻译为 SQL。

pub mod agent_config_repo;
pub mod agent_repo;
pub mod agent_snapshot_repo;
pub mod config_repo;
pub mod essence_repo;
pub mod match_repo;
pub mod rank_repo;
pub mod room_repo;
pub mod scenario_repo;
pub mod spawn_preset_repo;
pub mod user_repo;

pub use agent_config_repo::{AgentConfigRepo, PgAgentConfigRepo};
pub use agent_repo::{AgentRepo, PgAgentRepo};
pub use agent_snapshot_repo::{AgentSnapshotRepo, PgAgentSnapshotRepo};
pub use config_repo::{ConfigRepo, PgConfigRepo};
pub use essence_repo::{
    EssenceRepo, PgEssenceRepo, PgSubscriptionRepo, Subscription, SubscriptionRepo,
};
pub use match_repo::{
    MatchEventInput, MatchEventRepo, MatchInput, MatchParticipantRepo, MatchRepo, ParticipantInput,
    PgMatchEventRepo, PgMatchParticipantRepo, PgMatchRepo,
};
pub use rank_repo::{EloRepo, PgEloRepo, PgRankQueueRepo, PgSeasonRepo, RankQueueRepo, SeasonRepo};
pub use room_repo::{PgRoomRepo, RoomRepo};
pub use scenario_repo::{PgScenarioRepo, ScenarioRepo};
pub use spawn_preset_repo::{PgSpawnPresetRepo, SpawnPresetRepo};
pub use user_repo::{PgUserRepo, UserRepo};
