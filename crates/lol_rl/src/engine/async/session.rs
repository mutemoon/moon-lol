use std::collections::VecDeque;
use std::sync::Arc;

use candle_core::Device;
use lol_env::RlEnvironment;
use rand::seq::IndexedRandom;
use tracing::info;

use super::actor::ActorPool;
use super::inference::{InferenceServer, PolicySnapshot};
use super::learner::AsyncLearner;
use super::queue::TrajectoryRingBuffer;
use crate::algo::agent::RlAgent;
use crate::engine::traits::{StepOutcome, TrainingEngine};

/// 异步训练会话：统一编排 B 的三件套 + 自博弈对手池。
pub struct AsyncTrainingSession<E: RlEnvironment + 'static> {
    pub learner: AsyncLearner<E::Obs>,
    pub actor_pool: ActorPool,
    pub infer_server: InferenceServer,
    pub num_parallel_envs: usize,
    /// 历史对手快照池：`(slot, 权重)`，最旧在前（移除最旧时复用其 slot 编号，InferenceServer 表同步更新）。
    opponent_pool: VecDeque<(usize, Arc<PolicySnapshot>)>,
    device: Device,
    _marker: std::marker::PhantomData<fn() -> E>,
}

impl<E: RlEnvironment + 'static> AsyncTrainingSession<E> {
    pub fn new(
        agent: impl Into<RlAgent>,
        num_parallel_envs: usize,
        state_dim: usize,
        horizon: usize,
        train_batch_size: usize,
        infer_batch_size: usize,
        dynamic_batch_timeout_us: u64,
        device: Device,
    ) -> Self {
        let agent: RlAgent = agent.into();
        let num_parallel_envs = num_parallel_envs.max(1);
        let target_rollout_steps = num_parallel_envs * horizon * E::num_agents().max(1);

        info!(
            "🎮 [AsyncTrainingSession] 启动异步 Actor-Learner (envs={}, horizon={}, MBatch={}, 推理Batch={})",
            num_parallel_envs, horizon, train_batch_size, infer_batch_size
        );

        // 环形缓冲容量为并行环境数的 2~4 倍，防止内存爆炸和策略严重过期
        let queue_capacity = (num_parallel_envs * 4).clamp(32, 2048);
        let traj_queue = TrajectoryRingBuffer::<E::Obs>::new(queue_capacity);

        let initial_snapshot = PolicySnapshot::new(agent.policy().clone(), agent.critic().cloned());

        let infer_server = InferenceServer::new(
            initial_snapshot,
            state_dim,
            infer_batch_size.max(4),
            dynamic_batch_timeout_us,
            device.clone(),
        );

        let initial_dispatch = vec![(0usize, 0usize); num_parallel_envs];
        let actor_pool = ActorPool::spawn::<E>(
            num_parallel_envs,
            infer_server.req_tx.clone(),
            traj_queue.clone(),
            horizon,
            initial_dispatch,
        );

        let learner = AsyncLearner::new(
            agent,
            train_batch_size,
            target_rollout_steps,
            traj_queue,
            infer_server.model_tx.clone(),
        );

        Self {
            learner,
            actor_pool,
            infer_server,
            num_parallel_envs,
            opponent_pool: VecDeque::with_capacity(8),
            device,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E: RlEnvironment + 'static> TrainingEngine for AsyncTrainingSession<E> {
    fn step_once(
        &mut self,
        iter: usize,
        lr: f64,
        train_batch_size: usize,
    ) -> anyhow::Result<StepOutcome<()>> {
        let is_multi_agent = E::num_agents() > 1;

        // 1. 自博弈对手池快照（仅多智能体自博弈环境启用，每 5 轮或第二轮开始）
        if is_multi_agent && (iter % 5 == 0 || (iter == 2 && self.opponent_pool.is_empty())) {
            let snapshot = Arc::new(
                PolicySnapshot::new(
                    self.learner.agent.policy().clone(),
                    self.learner.agent.critic().cloned(),
                )
                .to_device(&self.device)?,
            );
            let slot = if self.opponent_pool.len() >= 8 {
                let (old_slot, _) = self.opponent_pool.pop_front().expect("池满必有最旧元素");
                old_slot
            } else {
                1 + self.opponent_pool.len()
            };
            self.opponent_pool.push_back((slot, snapshot.clone()));
            let _ = self.infer_server.model_tx.send((slot, (*snapshot).clone()));
        }

        // 2. 刷新 workers 分派（下轮采样采用随机历史对手 + 双角色轮换）
        if is_multi_agent {
            let opp_count = (self.num_parallel_envs / 4).max(1);
            let mut new_dispatch = vec![(0usize, 0usize); self.num_parallel_envs];
            if !self.opponent_pool.is_empty() && self.num_parallel_envs > 1 {
                let pool_slots: Vec<usize> = self.opponent_pool.iter().map(|(s, _)| *s).collect();
                let mut rng = rand::rng();
                for (i, d) in new_dispatch
                    .iter_mut()
                    .enumerate()
                    .take(opp_count.min(self.num_parallel_envs))
                {
                    let role = if i % 2 == 0 { 0 } else { 1 };
                    let slot = pool_slots.choose(&mut rng).copied().unwrap_or(0);
                    *d = (slot, role);
                }
            }
            self.actor_pool.update_dispatch(new_dispatch);
        }

        let mut outcome = self.learner.step_once(iter, lr, train_batch_size)?;
        let infer_stats = self.infer_server.take_timing_stats();
        outcome.timing.infer_stats = Some(infer_stats);
        let obs_payload = outcome.last_obs.as_ref().and_then(|o| E::obs_to_payload(o));
        Ok(StepOutcome {
            num_samples: outcome.num_samples,
            sps: outcome.sps,
            stats: outcome.stats,
            mean_value: outcome.mean_value,
            timing: outcome.timing,
            ep_returns: outcome.ep_returns,
            ep_cs: outcome.ep_cs,
            ep_steps: outcome.ep_steps,
            reward_breakdown: outcome.reward_breakdown,
            last_reward_variables: outcome.last_reward_variables,
            last_obs: None,
            obs_payload,
        })
    }

    fn update_curriculum(
        &mut self,
        hp_scale: f32,
        cs_reward: f32,
        attack_no_cs_penalty: f32,
        harass_coef: f32,
    ) {
        self.actor_pool
            .update_curriculum(hp_scale, cs_reward, attack_no_cs_penalty, harass_coef);
    }

    fn agent(&self) -> &RlAgent {
        &self.learner.agent
    }

    fn total_steps(&self) -> usize {
        self.learner.total_steps
    }

    fn stop(&mut self) {
        self.actor_pool.stop();
        self.infer_server.stop();
    }
}

impl<E: RlEnvironment + 'static> Drop for AsyncTrainingSession<E> {
    fn drop(&mut self) {
        self.stop();
    }
}
