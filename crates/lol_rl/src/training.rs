//! 同步训练会话（机制 A）：持久化 Worker 池 + 一次真实迭代（调度 → 克隆 CPU 策略 → Rollout → 聚合 → PPO 更新）。
//!
//! 生产 RL 服务当前以本会话为主路径（同步、学习效果稳定）；异步机制 B 见 `crate::async_session`。
//! 训练循环（`crate::service`）与 AutoTuner 校准（`crate::autotune`）复用同一会话，
//! 保证校准测出的 SPS 与 UI 上报的 SPS 完全同口径。

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use candle_core::{Device, Result};
use crossbeam_channel::{Receiver, Sender, unbounded};
use lol_env::RlEnvironment;
use rand::seq::IndexedRandom;
use tracing::{error, info};

use crate::policy::ActorCritic;
use crate::ppo::{PPOAgent, PPOStats};
use crate::rollout::{RolloutWorker, WorkerCommand, WorkerTrajectory};

/// 一次训练迭代的产出（训练机制部分；DB/事件/日志等簿记由调用方完成）。
pub struct StepOutcome<O> {
    /// 本轮产出的训练样本总数（自博弈时每 env 每步产出 num_agents 个样本）。
    pub num_samples: usize,
    /// 与 UI 同口径的吞吐量：num_samples / 本轮墙钟耗时。
    pub sps: f64,
    pub stats: PPOStats,
    pub mean_value: f32,
    /// 本轮结束的所有回合累计回报。
    pub ep_returns: Vec<f32>,
    /// 本轮结束的所有回合步数。
    pub ep_steps: Vec<usize>,
    pub reward_breakdown: HashMap<String, f32>,
    pub last_reward_variables: HashMap<String, f32>,
    pub last_obs: Option<O>,
}

/// 同步训练会话：持有 PPOAgent + N 个持久化 Rollout Worker。
pub struct TrainingSession<E: RlEnvironment + 'static> {
    pub agent: PPOAgent,
    num_parallel_envs: usize,
    sampler_device: Device,
    cmd_senders: Vec<Sender<WorkerCommand>>,
    resp_receivers: Vec<Receiver<WorkerTrajectory<E::Obs>>>,
    thread_handles: Vec<JoinHandle<()>>,
    opponent_pool: VecDeque<Arc<ActorCritic>>,
    /// 累计训练样本总数（跨迭代累加，与 UI step 计数同口径）。
    pub total_steps: usize,
}

impl<E: RlEnvironment + 'static> TrainingSession<E> {
    /// 初始化 PPO Agent 并启动 N 个持久化 Rollout Worker（环境只在此时初始化一次）。
    ///
    /// `sampler_device` 指定采样前向运行的设备：`Device::Cpu`（默认，机制 A 的 CPU 推理路径）
    /// 或一个 GPU device（把每步策略前向放到 GPU，需权衡 kernel 启动/同步开销）。
    pub fn new(
        agent: PPOAgent,
        num_parallel_envs: usize,
        state_dim: usize,
        horizon: usize,
        sampler_device: Device,
    ) -> Self {
        info!(
            "🎮 [TrainingSession] 启动 {} 个并行无头环境 Rollout Worker (horizon={}, 采样设备 {:?})...",
            num_parallel_envs, horizon, sampler_device
        );

        let mut cmd_senders = Vec::with_capacity(num_parallel_envs);
        let mut resp_receivers = Vec::with_capacity(num_parallel_envs);
        let mut thread_handles = Vec::with_capacity(num_parallel_envs);

        for _ in 0..num_parallel_envs {
            let (cmd_tx, cmd_rx) = unbounded::<WorkerCommand>();
            let (resp_tx, resp_rx) = unbounded::<WorkerTrajectory<E::Obs>>();
            let sampler_device = sampler_device.clone();

            let handle = thread::spawn(move || {
                let mut worker = RolloutWorker::<E>::new();
                while let Ok(cmd) = cmd_rx.recv() {
                    match cmd {
                        WorkerCommand::Rollout {
                            main_policy,
                            opponent_policy,
                            main_agent_idx,
                        } => {
                            let traj = worker.rollout(
                                &main_policy,
                                opponent_policy.as_deref(),
                                main_agent_idx,
                                horizon,
                                state_dim,
                                &sampler_device,
                            );
                            let _ = resp_tx.send(traj.unwrap_or_else(|e| {
                                error!("Worker Rollout 失败: {e}");
                                WorkerTrajectory::empty()
                            }));
                        }
                        WorkerCommand::Stop => break,
                    }
                }
            });

            cmd_senders.push(cmd_tx);
            resp_receivers.push(resp_rx);
            thread_handles.push(handle);
        }

        Self {
            agent,
            num_parallel_envs,
            sampler_device,
            cmd_senders,
            resp_receivers,
            thread_handles,
            opponent_pool: VecDeque::with_capacity(8),
            total_steps: 0,
        }
    }

    /// 一次真实训练迭代：
    /// 熵/学习率调度 → 克隆采样策略 → 分发 Rollout → 聚合轨迹 → 真实 PPO Mini-Batch 更新。
    ///
    /// 返回与 UI `fps` 同口径的 SPS（不含 DB/事件广播，与 UI 上报时机一致）。
    pub fn step_once(
        &mut self,
        iter: usize,
        lr: f64,
        c2: f32,
        train_batch_size: usize,
    ) -> Result<StepOutcome<E::Obs>> {
        let is_multi_agent = E::num_agents() > 1;
        let iter_start = Instant::now();

        self.agent.set_entropy_coef(c2);
        let _ = self.agent.set_lr(lr);

        // 1. 克隆采样策略（设备与 sampler_device 一致，默认 CPU）
        let sampler_policy = Arc::new(self.agent.actor_critic.to_device(&self.sampler_device)?);

        // 2. 定期将策略快照存入历史对手池（仅多智能体自博弈环境启用，每 5 轮或第二轮开始）
        if is_multi_agent && (iter % 5 == 0 || (iter == 2 && self.opponent_pool.is_empty())) {
            if self.opponent_pool.len() >= 8 {
                self.opponent_pool.pop_front();
            }
            self.opponent_pool.push_back(sampler_policy.clone());
        }

        // 3. 触发持久化 Worker 并行采样
        //    多智能体自博弈: 75% 最新对抗最新，25% 历史对手双角色轮换；单智能体: 100% 最新主策略推演
        let opp_count =
            if is_multi_agent && !self.opponent_pool.is_empty() && self.num_parallel_envs > 1 {
                (self.num_parallel_envs / 4).max(1)
            } else {
                0
            };

        let mut rng = rand::rng();
        let pool_vec: Vec<_> = self.opponent_pool.iter().cloned().collect();

        for (worker_idx, tx) in self.cmd_senders.iter().enumerate() {
            let (opp_policy, main_agent_idx) = if worker_idx < opp_count {
                let opp = pool_vec.choose(&mut rng).cloned();
                // 双角色轮换：偶数 Worker 主策略扮演 Fiora (0)，奇数 Worker 主策略扮演 Riven (1)
                let role = if worker_idx % 2 == 0 { 0 } else { 1 };
                (opp, role)
            } else {
                (None, 0)
            };
            let _ = tx.send(WorkerCommand::Rollout {
                main_policy: sampler_policy.clone(),
                opponent_policy: opp_policy,
                main_agent_idx,
            });
        }

        // 4. 聚合轨迹
        let mut env_buffers = Vec::with_capacity(self.num_parallel_envs * 2);
        let mut last_values = Vec::with_capacity(self.num_parallel_envs * 2);
        let mut ep_returns_all = Vec::new();
        let mut ep_steps_all = Vec::new();
        let mut iter_reward_breakdown: HashMap<String, f32> = HashMap::new();
        let mut last_reward_variables = HashMap::new();
        let mut sample_obs: Option<E::Obs> = None;

        for rx in &self.resp_receivers {
            let traj = match rx.recv() {
                Ok(t) => t,
                Err(_) => break,
            };
            if traj.buffers.is_empty() {
                continue;
            }
            for ret in traj.ep_returns {
                ep_returns_all.push(ret);
            }
            for s in traj.completed_steps {
                ep_steps_all.push(s);
            }
            for (k, v) in traj.reward_breakdown {
                *iter_reward_breakdown.entry(k).or_insert(0.0) += v;
            }
            if !traj.last_reward_variables.is_empty() {
                last_reward_variables = traj.last_reward_variables;
            }
            if sample_obs.is_none() {
                sample_obs = traj.last_obs;
            }
            env_buffers.extend(traj.buffers);
            last_values.extend(traj.last_values);
        }

        let num_samples: usize = env_buffers.iter().map(|b| b.len()).sum();
        self.total_steps += num_samples;

        // 平均价值（在 buffer 被训练消费前计算）
        let val_sum: f32 = env_buffers
            .iter()
            .map(|b| b.values.iter().sum::<f32>())
            .sum();
        let val_cnt: usize = env_buffers.iter().map(|b| b.values.len()).sum();
        let mean_value = val_sum / (val_cnt as f32).max(1.0);

        // 5. GPU Mini-Batch PPO 更新
        let stats = self
            .agent
            .update_multi_buffer(&env_buffers, &last_values, train_batch_size)?;

        let elapsed_sec = iter_start.elapsed().as_secs_f64();
        let sps = (num_samples as f64) / elapsed_sec.max(0.0001);

        Ok(StepOutcome {
            num_samples,
            sps,
            stats,
            mean_value,
            ep_returns: ep_returns_all,
            ep_steps: ep_steps_all,
            reward_breakdown: iter_reward_breakdown,
            last_reward_variables,
            last_obs: sample_obs,
        })
    }

    /// 停止并回收所有 Worker 线程。
    pub fn stop(&mut self) {
        for tx in &self.cmd_senders {
            let _ = tx.send(WorkerCommand::Stop);
        }
        for h in self.thread_handles.drain(..) {
            let _ = h.join();
        }
    }
}

impl<E: RlEnvironment + 'static> Drop for TrainingSession<E> {
    fn drop(&mut self) {
        self.stop();
    }
}
