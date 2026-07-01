//! Rig Agent 决策环共享运行时：桌面端与云端共用的纯编排逻辑。
//!
//! 桌面端（`apps/client/src-tauri/src/agent.rs`）与云端
//! （`crates/lol_web_server/src/service/agent_orchestrator.rs`）各自实现
//! [`CredentialResolver`] 与 [`OrchestratorSink`]，复用 [`run_orchestrator`]。

pub mod credentials;
pub mod orchestrator;
pub mod resolver;
pub mod sink;
pub mod testing;

pub use credentials::{
    AgentConfig, PlatformEnv, ProviderCredentials, ResolvedCredentials, resolve_credentials,
    ModelConfig,
};
pub use orchestrator::run_orchestrator;
pub use resolver::CredentialResolver;
pub use sink::{AgentRunResult, NoopSink, OrchestratorSink};
pub use testing::test_model_connection;
