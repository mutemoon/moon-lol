//! 强化学习训练/可视化协议：
//! 动作与观测 AST、奖励公式、任务/环境规格、WS 协议帧与可视化子进程帧。
//!
//! 所有公开类型均在 crate 根路径重导出，外部一律 `use lol_rl_protocol::X`。

mod action;
mod action_space;
mod env_spec;
mod frames;
mod obs;
mod reward;
mod task;
mod visual;

pub mod dsl;

pub const DEFAULT_RL_SERVER_ADDR: &str = "127.0.0.1:8765";

pub use action::{ActionMasks, ActionNode, ActionSchema, ActionValueNode};
pub use action_space::ActionSpace;
pub use dsl::{EnvDslSpec, parse_env_dsl};
pub use env_spec::{
    AVAILABLE_ENVS, ENV_FIORA_V0, ENV_FIORA_V0_SPEC, ENV_FIORA_V1, ENV_FIORA_V1_SPEC, ENV_FIORA_V2,
    ENV_FIORA_V2_SPEC, ENV_SOLO_V0, ENV_SOLO_V0_SPEC, EnvSpec, EnvTrainingParams, get_env_spec,
    get_env_training_params,
};
pub use frames::{
    CheckpointItem, CurriculumTelemetry, InFrame, MetricsRow, ObsFeaturePayload, OutFrame,
};
pub use obs::{EntityEncoderSpec, ObsContext, ObsExpr, ObsNode, ObsSchema, ObsValueNode, PoolType};
pub use reward::{RewardExpr, RewardFormulaSpec, RewardItem, RewardTermSpec};
pub use task::{
    AGENT_PPO_MAMBA, AGENT_PPO_MLP, CurriculumConfig, PolicyBackbone, TaskConfigPayload,
    TaskOverviewItem,
};
pub use visual::{
    ActionBranchDisplay, PolicyDisplay, PolicyItem, VisualInFrame, VisualObsFrame, VisualOutFrame,
};
