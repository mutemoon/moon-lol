use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::env_spec::{ENV_FIORA_V2, get_env_training_params};

pub const AGENT_PPO_MAMBA: &str = "PPO (Mamba)";
pub const AGENT_PPO_MLP: &str = "PPO (MLP)";

/// 策略网络的主干网络架构类型
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyBackbone {
    /// 经典多层感知机（无状态前馈网络，计算速度极快）
    Mlp,
    /// Selective State Space Model（带时序记忆与门控状态空间）
    Mamba,
}

impl PolicyBackbone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mlp => "MLP",
            Self::Mamba => "Mamba",
        }
    }
}

impl Display for PolicyBackbone {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskConfigPayload {
    pub name: String,
    pub agent_type: String,
    pub env_name: String,
    pub lr: f32,
    pub gamma: f32,
    pub gae_lambda: f32,
    pub clip_eps: f32,
    pub ppo_epochs: usize,
    pub hidden_dim: usize,
    pub parallel_envs: usize,
    pub rollout_steps_per_env: usize,
    pub total_iterations: usize,
    #[serde(default)]
    pub backbone: Option<PolicyBackbone>,
}

impl TaskConfigPayload {
    pub fn default_for_env(env_name: &str) -> Self {
        let params = get_env_training_params(env_name);
        Self {
            name: "RL 对战训练任务".to_string(),
            agent_type: AGENT_PPO_MLP.to_string(),
            env_name: env_name.to_string(),
            lr: params.lr,
            gamma: params.gamma,
            gae_lambda: params.gae_lambda,
            clip_eps: params.clip_eps,
            ppo_epochs: params.ppo_epochs,
            hidden_dim: params.hidden_dim,
            parallel_envs: 0,
            rollout_steps_per_env: params.rollout_steps_per_env,
            total_iterations: params.total_iterations,
            backbone: Some(PolicyBackbone::Mamba),
        }
    }

    /// 解析当前任务的主干网络架构（优先使用 backbone 字段，其次解析 agent_type）
    pub fn backbone(&self) -> PolicyBackbone {
        if let Some(bb) = self.backbone {
            return bb;
        }
        if self.agent_type.to_lowercase().contains("mlp") {
            PolicyBackbone::Mlp
        } else {
            PolicyBackbone::Mamba
        }
    }
}

impl Default for TaskConfigPayload {
    fn default() -> Self {
        Self::default_for_env(ENV_FIORA_V2)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskOverviewItem {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub env_name: String,
    pub status: String,
    pub current_step: usize,
    pub ep_return: f32,
    pub checkpoints_count: usize,
    pub hidden_dim: usize,
    pub parallel_envs: usize,
    pub lr: f32,
    pub total_iterations: usize,
    pub rollout_steps_per_env: usize,
    pub created_at: String,
}
