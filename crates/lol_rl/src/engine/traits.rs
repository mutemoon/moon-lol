use std::collections::HashMap;

use lol_rl_protocol::ObsFeaturePayload;

use crate::algo::agent::RlAgent;
use crate::algo::ppo::PPOStats;

/// 推理服务单轮耗时统计（异步引擎专有）
#[derive(Debug, Clone, Default)]
pub struct InferenceTimingStats {
    /// 本轮总推理批次数
    pub batch_count: usize,
    /// 本轮总处理请求数
    pub request_count: usize,
    /// 平均批大小 (request_count / batch_count)
    pub avg_batch_size: f64,
    /// 纯模型前向计算耗时 (毫秒)
    pub forward_ms: f64,
    /// Dynamic Batching 收集等待耗时 (毫秒)
    pub wait_ms: f64,
}

/// 异步轨迹环形缓冲队列监控健康指标
#[derive(Debug, Clone, Default)]
pub struct QueueHealthStats {
    /// 丢弃率 (total_dropped / total_pushed * 100.0)
    pub drop_ratio: f64,
    /// 消费时的平均策略版本差 (current_version - traj_version)
    pub avg_policy_gap: f64,
    /// 队列当前长度
    pub queue_len: usize,
    /// 队列总容量
    pub queue_capacity: usize,
    /// 累计推入总数
    pub total_pushed: usize,
    /// 累计丢弃总数 (满淘汰 + 过期淘汰)
    pub total_dropped: usize,
}

/// 训练单轮耗时统计（统一性能剖析）
#[derive(Debug, Clone, Default)]
pub struct StepTiming {
    /// 样本采样/轨迹收集耗时 (毫秒)
    pub collect_ms: f64,
    /// 梯度更新与优化器步进耗时 (毫秒)
    pub train_ms: f64,
    /// 单轮总耗时 (毫秒)
    pub total_ms: f64,
    /// 异步推理服务器统计指标 (若为异步模式)
    pub infer_stats: Option<InferenceTimingStats>,
    /// 异步缓冲队列监控指标 (若为异步模式)
    pub queue_stats: Option<QueueHealthStats>,
}

/// 一次训练迭代的产出（与 UI / 数据库 / 日志遥测指标完全同口径）。
pub struct StepOutcome<O = ()> {
    /// 本轮产出的训练样本总数（自博弈时每 env 每步产出 num_agents 个样本）。
    pub num_samples: usize,
    /// 与 UI 同口径的吞吐量：num_samples / 本轮墙钟耗时。
    pub sps: f64,
    pub stats: PPOStats,
    pub mean_value: f32,
    /// 本轮耗时性能指标剖析
    pub timing: StepTiming,
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
            timing: self.timing,
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
