use std::collections::HashMap;
use std::sync::Arc;

use crate::algo::buffer::RolloutBuffer;
use crate::policy::{PolicyNetwork, ValueHead};

/// 一次 Rollout 的完整产出（单个 Worker 一次 horizon 推演）。
pub struct WorkerTrajectory<O> {
    /// 参与训练的轨迹 Buffer（自博弈：每智能体一个；对抗历史对手：仅主角色一个）。
    pub buffers: Vec<RolloutBuffer>,
    /// 与 buffers 一一对齐的末尾价值（GAE bootstrap 用）。
    pub last_values: Vec<f32>,
    pub ep_returns: Vec<f32>,
    pub ep_cs: Vec<f32>,
    pub completed_steps: Vec<usize>,
    pub reward_breakdown: HashMap<String, f32>,
    pub last_reward_variables: HashMap<String, f32>,
    pub last_obs: Option<O>,
    /// 该轨迹产生时所依据的主策略版本号（用于异步 Staleness 检测与版本淘汰）。
    pub policy_version: usize,
}

impl<O> WorkerTrajectory<O> {
    pub fn empty() -> Self {
        Self {
            buffers: Vec::new(),
            last_values: Vec::new(),
            ep_returns: Vec::new(),
            ep_cs: Vec::new(),
            completed_steps: Vec::new(),
            reward_breakdown: HashMap::new(),
            last_reward_variables: HashMap::new(),
            last_obs: None,
            policy_version: 0,
        }
    }
}

/// 发给持久化 Worker 的命令。
pub enum WorkerCommand {
    Rollout {
        main_policy: Arc<PolicyNetwork>,
        main_critic: Option<Arc<ValueHead>>,
        opponent_policy: Option<Arc<PolicyNetwork>>,
        opponent_critic: Option<Arc<ValueHead>>,
        main_agent_idx: usize,
    },
    /// 更新课程学习参数（小兵血量缩放 + 奖励配置）
    UpdateCurriculum {
        hp_scale: f32,
        cs_reward: f32,
        attack_no_cs_penalty: f32,
        harass_coef: f32,
    },
    Stop,
}
