use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::env_spec::{ENV_FIORA_V2, get_env_training_params};

/// 强化学习训练算法类型
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RlAlgorithm {
    Ppo,
    Grpo,
}

/// 强化学习训练引擎模式（异步 Actor-Learner 流水线 vs 同步 Rollout Worker 池）
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EngineMode {
    Async,
    #[default]
    Sync,
}

impl EngineMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Async => "async",
            Self::Sync => "sync",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Async => "异步引擎 (Async)",
            Self::Sync => "同步引擎 (Sync)",
        }
    }
}

impl Display for EngineMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for EngineMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "async" => Ok(Self::Async),
            "sync" => Ok(Self::Sync),
            other => Err(format!("未知引擎模式 '{other}'，可选: async, sync")),
        }
    }
}

impl RlAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ppo => "ppo",
            Self::Grpo => "grpo",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Ppo => "PPO",
            Self::Grpo => "GRPO",
        }
    }
}

impl Display for RlAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for RlAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "ppo" => Ok(Self::Ppo),
            "grpo" => Ok(Self::Grpo),
            other => Err(format!("未知算法 '{other}'，可选: ppo, grpo")),
        }
    }
}

/// 策略网络的主干网络架构类型
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyBackbone {
    /// 经典多层感知机（无状态前馈网络，计算速度极快）
    Mlp,
    /// Selective State Space Model（带时序记忆与门控状态空间）
    Mamba,
}

impl PolicyBackbone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mlp => "mlp",
            Self::Mamba => "mamba",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Mlp => "MLP",
            Self::Mamba => "Mamba",
        }
    }
}

impl Display for PolicyBackbone {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for PolicyBackbone {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "mlp" => Ok(Self::Mlp),
            "mamba" => Ok(Self::Mamba),
            other => Err(format!("未知主干架构 '{other}'，可选: mlp, mamba")),
        }
    }
}

/// 课程学习配置（可选，不使用时保持 None）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CurriculumConfig {
    /// 小兵血量缩放起始值（第一课开始时小兵只有此比例的满血）
    pub hp_scale_start: f32,
    /// 小兵血量缩放终止值
    pub hp_scale_end: f32,
    /// 第一课持续的迭代数（hp_scale 在此期间线性增长）
    pub phase1_iterations: usize,
    /// 进入第二课的平均 CS 阈值
    pub phase2_cs_threshold: f32,
    /// 第二课中英雄伤害奖励系数（远低于补刀奖励 1.0）
    pub harass_coef: f32,
    /// 每次补刀的奖励值
    pub cs_reward: f32,
    /// 攻击小兵但未补到刀的惩罚值（正数，实际扣减）
    pub attack_no_cs_penalty: f32,
}

impl Default for CurriculumConfig {
    fn default() -> Self {
        Self {
            hp_scale_start: 0.05,
            hp_scale_end: 1.0,
            phase1_iterations: 200,
            phase2_cs_threshold: 2.0,
            harass_coef: 0.3,
            cs_reward: 1.0,
            attack_no_cs_penalty: 0.1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskConfigPayload {
    pub name: String,
    pub env_name: String,
    pub algorithm: RlAlgorithm,
    pub backbone: PolicyBackbone,
    #[serde(default)]
    pub engine_mode: EngineMode,
    pub lr: f32,
    pub gamma: f32,
    pub gae_lambda: f32,
    pub clip_eps: f32,
    pub ppo_epochs: usize,
    pub hidden_dim: usize,
    pub parallel_envs: usize,
    pub rollout_steps_per_env: usize,
    pub total_iterations: usize,
    /// 课程学习配置（可选，None 表示不使用课程学习，使用默认奖励函数）
    #[serde(default)]
    pub curriculum: Option<CurriculumConfig>,
    /// GRPO 算法每组环境/轨迹大小（默认 4 或 8）
    #[serde(default)]
    pub grpo_group_size: Option<usize>,
}

impl TaskConfigPayload {
    pub fn default_for_env(env_name: &str) -> Self {
        let params = get_env_training_params(env_name);
        Self {
            name: "RL 对战训练任务".to_string(),
            env_name: env_name.to_string(),
            algorithm: RlAlgorithm::Ppo,
            backbone: PolicyBackbone::Mlp,
            engine_mode: EngineMode::Sync,
            lr: params.lr,
            gamma: params.gamma,
            gae_lambda: params.gae_lambda,
            clip_eps: params.clip_eps,
            ppo_epochs: params.ppo_epochs,
            hidden_dim: params.hidden_dim,
            parallel_envs: 0,
            rollout_steps_per_env: params.rollout_steps_per_env,
            total_iterations: params.total_iterations,
            curriculum: None,
            grpo_group_size: None,
        }
    }

    pub fn is_grpo(&self) -> bool {
        self.algorithm == RlAlgorithm::Grpo
    }

    pub fn backbone(&self) -> PolicyBackbone {
        self.backbone
    }

    /// 格式化用于 UI 展示的算法模型组合名称（如 "PPO (MLP)", "GRPO (Mamba)"）
    pub fn display_agent_name(&self) -> String {
        format!(
            "{} ({})",
            self.algorithm.display_name(),
            self.backbone.display_name()
        )
    }

    /// 转换为 lol_rl_cli 接收的命令行参数列表
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec![
            "--name".to_string(),
            self.name.clone(),
            "--env".to_string(),
            self.env_name.clone(),
            "--algo".to_string(),
            self.algorithm.as_str().to_string(),
            "--backbone".to_string(),
            self.backbone.as_str().to_string(),
            "--engine".to_string(),
            self.engine_mode.as_str().to_string(),
            "--lr".to_string(),
            self.lr.to_string(),
            "--gamma".to_string(),
            self.gamma.to_string(),
            "--gae-lambda".to_string(),
            self.gae_lambda.to_string(),
            "--clip-eps".to_string(),
            self.clip_eps.to_string(),
            "--ppo-epochs".to_string(),
            self.ppo_epochs.to_string(),
            "--hidden-dim".to_string(),
            self.hidden_dim.to_string(),
            "--parallel-envs".to_string(),
            self.parallel_envs.to_string(),
            "--rollout-steps-per-env".to_string(),
            self.rollout_steps_per_env.to_string(),
            "--total-iterations".to_string(),
            self.total_iterations.to_string(),
        ];
        if let Some(group_size) = self.grpo_group_size {
            args.push("--grpo-group-size".to_string());
            args.push(group_size.to_string());
        }
        if let Some(curriculum) = &self.curriculum {
            if let Ok(json) = serde_json::to_string(curriculum) {
                args.push("--curriculum-json".to_string());
                args.push(json);
            }
        }
        args
    }

    /// 转换为可直接在终端/实验 Agent 中执行的完整 cargo run 启动命令
    pub fn to_cargo_run_command(&self) -> String {
        let args = self.to_cli_args();
        let mut escaped_args = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let key = &args[i];
            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                let val = &args[i + 1];
                if val.contains(' ') || val.contains('"') || val.contains('{') {
                    escaped_args.push(format!("{} \"{}\"", key, val.replace('"', "\\\"")));
                } else {
                    escaped_args.push(format!("{} {}", key, val));
                }
                i += 2;
            } else {
                escaped_args.push(key.clone());
                i += 1;
            }
        }
        format!(
            "cargo run -p lol_rl --bin lol_rl_cli -- {}",
            escaped_args.join(" ")
        )
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
    pub algorithm: RlAlgorithm,
    pub backbone: PolicyBackbone,
    #[serde(default)]
    pub engine_mode: EngineMode,
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

