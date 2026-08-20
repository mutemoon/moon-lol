//! 异步训练会话（机制 B 的统一定义）：InferenceServer + ActorPool + AsyncLearner + 自博弈对手池。
//!
//! 取代旧的同步 `TrainingSession`（机制 A）作为产品主路径。采样推理走 GPU 动态批处理，
//! 训练用异步 Actor-Learner 流水线重叠；自博弈对手池（历史对手 + 双角色轮换）在此协调。

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use candle_core::Device;
use crossbeam_channel::unbounded;
use lol_env::RlEnvironment;
use rand::seq::IndexedRandom;
use tracing::info;

use crate::async_engine::InferenceServer;
use crate::async_engine::actor::{ActorPool, SampleTransition};
use crate::async_engine::learner::AsyncLearner;
use crate::policy::ActorCritic;
use crate::ppo::{PPOAgent, PPOStats};

/// 异步训练会话的并发环境安全上限。
///
/// 实测：并发环境数超过 8 时，异步采样滞后（actor 用旧权重采样）+ 每 worker 轨迹变短
/// 显著拉低每样本学习质量（v2: envs=8 回报 0.79 / envs=16 0.54 / envs=32 0.43）。
/// 吞吐最优不等于学习最优，这里以学习质量为优先。
pub const MAX_PARALLEL_ENVS: usize = 8;

/// 一次异步训练迭代的簿记产出（对齐 UI/DB 所需的一切）。
pub struct AsyncIteration {
    pub iter: usize,
    pub total_steps: usize,
    pub num_samples: usize,
    pub sps: f64,
    pub stats: PPOStats,
    pub ep_return: f32,
    pub mean_value: f32,
    pub ep_steps_max: usize,
    pub ep_steps_min: usize,
    pub ep_steps_avg: f32,
    pub reward_breakdown: Vec<lol_rl_protocol::RewardItem>,
    pub reward_variables: HashMap<String, f32>,
    pub obs_payload: Option<lol_rl_protocol::ObsFeaturePayload>,
}

/// 异步训练会话：统一编排 B 的三件套 + 自博弈对手池。
pub struct AsyncTrainingSession<E: RlEnvironment + 'static> {
    pub agent: PPOAgent,
    state_dim: usize,
    pub num_parallel_envs: usize,
    horizon: usize,
    train_batch_size: usize,
    pub infer_batch_size: usize,
    dynamic_batch_timeout_us: u64,
    device: Device,
    /// 历史对手快照池：`(slot, 权重)`，最旧在前（移除最旧时复用其 slot 编号，InferenceServer 表同步更新）。
    opponent_pool: VecDeque<(usize, Arc<ActorCritic>)>,
    _marker: std::marker::PhantomData<E>,
}

impl<E: RlEnvironment + 'static> AsyncTrainingSession<E> {
    pub fn new(
        agent: PPOAgent,
        num_parallel_envs: usize,
        state_dim: usize,
        horizon: usize,
        train_batch_size: usize,
        infer_batch_size: usize,
        dynamic_batch_timeout_us: u64,
        device: Device,
    ) -> Self {
        // 限制并发环境到安全上限（见 MAX_PARALLEL_ENVS 注释）
        let num_parallel_envs = num_parallel_envs.min(MAX_PARALLEL_ENVS);
        info!(
            "🎮 [AsyncTrainingSession] 异步 Actor-Learner (envs={}, horizon={}, MBatch={}, 推理Batch={})",
            num_parallel_envs, horizon, train_batch_size, infer_batch_size
        );
        Self {
            agent,
            state_dim,
            num_parallel_envs,
            horizon,
            train_batch_size,
            infer_batch_size,
            dynamic_batch_timeout_us,
            device,
            opponent_pool: VecDeque::with_capacity(8),
            _marker: std::marker::PhantomData,
        }
    }

    /// 运行训练循环。
    ///
    /// `on_iteration` 在每个迭代完成后回调，携带簿记所需数据（DB/事件/日志在外部实现）与最新 agent（checkpoint 用）。
    pub fn run<F>(
        mut self,
        total_iterations: usize,
        is_running: Arc<AtomicBool>,
        mut on_iteration: F,
    ) -> anyhow::Result<PPOAgent>
    where
        F: FnMut(&AsyncIteration, &PPOAgent) -> anyhow::Result<()>,
    {
        let is_multi_agent = E::num_agents() > 1;
        let target_rollout_steps = self.num_parallel_envs * self.horizon * E::num_agents().max(1);

        // 解构字段：self.agent 将 move 进 learner，其余字段以局部变量供闭包使用
        let state_dim = self.state_dim;
        let mut opponent_pool = std::mem::take(&mut self.opponent_pool);
        let device = self.device.clone();

        let (sample_tx, sample_rx) = unbounded::<SampleTransition>();

        let mut infer_server = InferenceServer::new(
            self.agent.actor_critic.clone(),
            state_dim,
            self.infer_batch_size.max(4),
            self.dynamic_batch_timeout_us,
            device.clone(),
        );

        // 初始无历史对手，全部纯自博弈（dual 自博弈同样用 slot 0 双方同策略）
        let initial_dispatch = vec![(0usize, 0usize); self.num_parallel_envs];
        let mut actor_pool = ActorPool::spawn::<E>(
            self.num_parallel_envs,
            infer_server.req_tx.clone(),
            sample_tx,
            initial_dispatch,
        );

        let mut learner = AsyncLearner::new(
            self.agent,
            self.train_batch_size,
            target_rollout_steps,
            sample_rx,
            infer_server.model_tx.clone(),
        );

        let mut total_steps = 0usize;
        learner.run_loop(total_iterations, is_running, |metrics, agent| {
            let num_samples = metrics.total_samples.saturating_sub(total_steps);
            total_steps = metrics.total_samples;
            let iter = metrics.iteration;

            // 1. 自博弈对手池：每 5 轮把当前权重快照为历史对手，注册到推理引擎
            if is_multi_agent && (iter % 5 == 0 || (iter == 2 && opponent_pool.is_empty())) {
                let snapshot = Arc::new(agent.actor_critic.to_device(&device)?);
                let slot = if opponent_pool.len() >= 8 {
                    // 池满：移除最旧对手，复用其 slot 编号（InferenceServer 表随之覆盖更新）
                    let (old_slot, _) = opponent_pool.pop_front().expect("池满必有最旧元素");
                    old_slot
                } else {
                    1 + opponent_pool.len()
                };
                opponent_pool.push_back((slot, snapshot.clone()));
                let _ = infer_server.model_tx.send((slot, (*snapshot).clone()));
            }

            // 2. 刷新 workers 分派（下轮采样采用随机历史对手 + 双角色轮换）
            if is_multi_agent {
                let opp_count = (self.num_parallel_envs / 4).max(1);
                let mut new_dispatch = vec![(0usize, 0usize); self.num_parallel_envs];
                if !opponent_pool.is_empty() && self.num_parallel_envs > 1 {
                    let pool_slots: Vec<usize> = opponent_pool.iter().map(|(s, _)| *s).collect();
                    let mut rng = rand::rng();
                    for i in 0..opp_count.min(self.num_parallel_envs) {
                        // 每个对抗 worker 独立随机选一个历史对手，双角色轮换（偶数 main=0，奇数 main=1）
                        let role = if i % 2 == 0 { 0 } else { 1 };
                        let slot = pool_slots.choose(&mut rng).copied().unwrap_or(0);
                        new_dispatch[i] = (slot, role);
                    }
                }
                actor_pool.update_dispatch(new_dispatch);
            }

            let result = AsyncIteration {
                iter,
                total_steps,
                num_samples,
                sps: metrics.sps,
                stats: metrics.stats,
                ep_return: metrics.ep_return,
                mean_value: metrics.mean_value,
                ep_steps_max: metrics.ep_steps_max,
                ep_steps_min: metrics.ep_steps_min,
                ep_steps_avg: metrics.ep_steps_avg,
                reward_breakdown: metrics.reward_breakdown.clone(),
                reward_variables: metrics.reward_variables.clone(),
                obs_payload: metrics.obs_payload.clone(),
            };
            on_iteration(&result, agent)?;
            Ok(())
        })?;

        // 回收
        actor_pool.stop();
        infer_server.stop();
        // 返回训练结束后的 agent（供最终 checkpoint）
        Ok(learner.agent)
    }
}
