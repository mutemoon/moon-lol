use std::collections::HashMap;

use lol_rl_protocol::ObsFeaturePayload;

use crate::algo::agent::RlAgent;
use crate::algo::ppo::PPOStats;

/// 一次训练迭代的产出（与 UI / 数据库 / 日志遥测指标完全同口径）。
pub struct StepOutcome<O = ()> {
    /// 本轮产出的训练样本总数（自博弈时每 env 每步产出 num_agents 个样本）。
    pub num_samples: usize,
    /// 与 UI 同口径的吞吐量：num_samples / 本轮墙钟耗时。
    pub sps: f64,
    pub stats: PPOStats,
    pub mean_value: f32,
    /// 本轮结束的所有回合累计回报。
    pub ep_returns: Vec<f32>,
    /// 本轮结束的所有回合补刀数。
    pub ep_cs: Vec<f32>,
    /// 本轮结束的所有回合步数。
    pub ep_steps: Vec<usize>,
    pub reward_breakdown: HashMap<String, f32>,
    pub last_reward_variables: HashMap<String, f32>,
    pub last_obs: Option<O>,
    pub obs_payload: Option<ObsFeaturePayload>,
}

impl<O> StepOutcome<O> {
    /// 转换为去泛型的 StepOutcome<()>
    pub fn erase_obs(self) -> StepOutcome<()> {
        StepOutcome {
            num_samples: self.num_samples,
            sps: self.sps,
            stats: self.stats,
            mean_value: self.mean_value,
            ep_returns: self.ep_returns,
            ep_cs: self.ep_cs,
            ep_steps: self.ep_steps,
            reward_breakdown: self.reward_breakdown,
            last_reward_variables: self.last_reward_variables,
            last_obs: None,
            obs_payload: self.obs_payload,
        }
    }
}

/// 强化学习训练引擎统一抽象 Trait
pub trait TrainingEngine: Send {
    /// 执行一次训练迭代
    fn step_once(
        &mut self,
        iter: usize,
        lr: f64,
        train_batch_size: usize,
    ) -> anyhow::Result<StepOutcome<()>>;

    /// 向所有并发环境广播更新课程学习参数
    fn update_curriculum(
        &mut self,
        hp_scale: f32,
        cs_reward: f32,
        attack_no_cs_penalty: f32,
        harass_coef: f32,
    );

    /// 获取底层智能体引用（用于保存权重、参数量查询等）
    fn agent(&self) -> &RlAgent;

    /// 累计消费/训练的真实样本步数
    fn total_steps(&self) -> usize;

    /// 停止并回收后台线程与资源
    fn stop(&mut self);
}
