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
    /// 课程学习配置（可选，None 表示不使用课程学习，使用默认奖励函数）
    #[serde(default)]
    pub curriculum: Option<CurriculumConfig>,
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
            backbone: Some(PolicyBackbone::Mlp),
            curriculum: None,
        }
    }

    /// 解析当前任务的主干网络架构（优先与 agent_type 一致，其次使用 backbone 字段）
    pub fn backbone(&self) -> PolicyBackbone {
        if self.agent_type.to_lowercase().contains("mlp") {
            PolicyBackbone::Mlp
        } else if self.agent_type.to_lowercase().contains("mamba") {
            PolicyBackbone::Mamba
        } else if let Some(bb) = self.backbone {
            bb
        } else {
            PolicyBackbone::Mlp
        }
    }

    /// 转换为 lol_rl_cli 接收的命令行参数列表
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec![
            "--name".to_string(),
            self.name.clone(),
            "--env".to_string(),
            self.env_name.clone(),
            "--agent".to_string(),
            self.agent_type.clone(),
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
        format!("cargo run -p lol_rl --bin lol_rl_cli -- {}", escaped_args.join(" "))
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

