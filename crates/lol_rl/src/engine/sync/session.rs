use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use candle_core::{Device, Result};
use crossbeam_channel::{Receiver, Sender, unbounded};
use lol_env::RlEnvironment;
use rand::seq::IndexedRandom;
use tracing::{error, info};

use crate::algo::agent::RlAgent;
use crate::engine::traits::{StepOutcome, TrainingEngine};
use crate::engine::trajectory::{WorkerCommand, WorkerTrajectory};
use crate::engine::worker::RolloutWorker;
use crate::policy::{PolicyNetwork, ValueHead};

/// 同步训练会话：持有 RlAgent + N 个持久化 Rollout Worker。
pub type TrainingSession<E> = SyncTrainingSession<E>;

/// 同步训练会话：持有 RlAgent + N 个持久化 Rollout Worker。
pub struct SyncTrainingSession<E: RlEnvironment + 'static> {
    pub agent: RlAgent,
    num_parallel_envs: usize,
    sampler_device: Device,
    cmd_senders: Vec<Sender<WorkerCommand>>,
    resp_receivers: Vec<Receiver<WorkerTrajectory<E::Obs>>>,
    thread_handles: Vec<JoinHandle<()>>,
    opponent_pool: VecDeque<(Arc<PolicyNetwork>, Option<Arc<ValueHead>>)>,
    /// 累计训练样本总数（跨迭代累加，与 UI step 计数同口径）。
    pub total_steps: usize,
}

impl<E: RlEnvironment + 'static> SyncTrainingSession<E> {
    /// 初始化训练会话并启动 N 个持久化 Rollout Worker（环境只在此时初始化一次）。
    ///
    /// `sampler_device` 指定采样前向运行的设备：`Device::Cpu`（默认，机制 A 的 CPU 推理路径）
    /// 或一个 GPU device（把每步策略前向放到 GPU，需权衡 kernel 启动/同步开销）。
    pub fn new(
        agent: impl Into<RlAgent>,
        num_parallel_envs: usize,
        state_dim: usize,
        horizon: usize,
        sampler_device: Device,
    ) -> Self {
        let agent: RlAgent = agent.into();
        agent.print_parameter_summary();

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
                            main_critic,
                            opponent_policy,
                            opponent_critic,
                            main_agent_idx,
                        } => {
                            let traj = worker.rollout(
                                &main_policy,
                                main_critic.as_deref(),
                                opponent_policy.as_deref(),
                                opponent_critic.as_deref(),
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
                        WorkerCommand::UpdateCurriculum {
                            hp_scale,
                            cs_reward,
                            attack_no_cs_penalty,
                            harass_coef,
                        } => {
                            worker.update_curriculum(
                                hp_scale,
                                cs_reward,
                                attack_no_cs_penalty,
                                harass_coef,
                            );
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
    /// 学习率调度 → 克隆采样策略 → 分发 Rollout → 聚合轨迹 → 真实 PPO Mini-Batch 更新。
    ///
    /// 返回与 UI `fps` 同口径的 SPS（不含 DB/事件广播，与 UI 上报时机一致）。
    pub fn step_once(
        &mut self,
        iter: usize,
        lr: f64,
        train_batch_size: usize,
    ) -> Result<StepOutcome<E::Obs>> {
        let is_multi_agent = E::num_agents() > 1;
        let iter_start = Instant::now();

        let _ = self.agent.set_lr(lr);

        // 1. 克隆采样策略（设备与 sampler_device 一致，默认 CPU）
        let sampler_policy = Arc::new(self.agent.policy().to_device(&self.sampler_device)?);
        let sampler_critic = self
            .agent
            .critic()
            .map(|c| c.to_device(&self.sampler_device))
            .transpose()?
            .map(Arc::new);

        // 2. 定期将策略快照存入历史对手池（仅多智能体自博弈环境启用，每 5 轮或第二轮开始）
        if is_multi_agent && (iter % 5 == 0 || (iter == 2 && self.opponent_pool.is_empty())) {
            if self.opponent_pool.len() >= 8 {
                self.opponent_pool.pop_front();
            }
            self.opponent_pool
                .push_back((sampler_policy.clone(), sampler_critic.clone()));
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
            let (opp_policy, opp_critic, main_agent_idx) = if worker_idx < opp_count {
                let opp = pool_vec.choose(&mut rng).cloned();
                let (op_p, op_c) = match opp {
                    Some((p, c)) => (Some(p), c),
                    None => (None, None),
                };
                // 双角色轮换：偶数 Worker 主策略扮演 Fiora (0)，奇数 Worker 主策略扮演 Riven (1)
                let role = if worker_idx % 2 == 0 { 0 } else { 1 };
                (op_p, op_c, role)
            } else {
                (None, None, 0)
            };
            let _ = tx.send(WorkerCommand::Rollout {
                main_policy: sampler_policy.clone(),
                main_critic: sampler_critic.clone(),
                opponent_policy: opp_policy,
                opponent_critic: opp_critic,
                main_agent_idx,
            });
        }

        // 4. 聚合轨迹
        let mut env_buffers = Vec::with_capacity(self.num_parallel_envs * 2);
        let mut last_values = Vec::with_capacity(self.num_parallel_envs * 2);
        let mut ep_returns_all = Vec::new();
        let mut ep_cs_all = Vec::new();
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
            for cs in traj.ep_cs {
                ep_cs_all.push(cs);
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

        // 5. GPU Mini-Batch PPO/GRPO 更新
        let stats = self
            .agent
            .update_multi_buffer(&env_buffers, &last_values, train_batch_size)?;

        let elapsed_sec = iter_start.elapsed().as_secs_f64();
        let sps = (num_samples as f64) / elapsed_sec.max(0.0001);

        let obs_payload = sample_obs.as_ref().and_then(|o| E::obs_to_payload(o));

        Ok(StepOutcome {
            num_samples,
            sps,
            stats,
            mean_value,
            ep_returns: ep_returns_all,
            ep_cs: ep_cs_all,
            ep_steps: ep_steps_all,
            reward_breakdown: iter_reward_breakdown,
            last_reward_variables,
            last_obs: sample_obs,
            obs_payload,
        })
    }

    /// 向所有持久化 Worker 广播课程学习参数（小兵血量缩放与奖励配置）。
    pub fn update_curriculum(
        &self,
        hp_scale: f32,
        cs_reward: f32,
        attack_no_cs_penalty: f32,
        harass_coef: f32,
    ) {
        for tx in &self.cmd_senders {
            let _ = tx.send(WorkerCommand::UpdateCurriculum {
                hp_scale,
                cs_reward,
                attack_no_cs_penalty,
                harass_coef,
            });
        }
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

impl<E: RlEnvironment + 'static> TrainingEngine for SyncTrainingSession<E> {
    fn step_once(
        &mut self,
        iter: usize,
        lr: f64,
        train_batch_size: usize,
    ) -> anyhow::Result<StepOutcome<()>> {
        let outcome = SyncTrainingSession::step_once(self, iter, lr, train_batch_size)?;
        Ok(outcome.erase_obs())
    }

    fn update_curriculum(
        &mut self,
        hp_scale: f32,
        cs_reward: f32,
        attack_no_cs_penalty: f32,
        harass_coef: f32,
    ) {
        SyncTrainingSession::update_curriculum(
            self,
            hp_scale,
            cs_reward,
            attack_no_cs_penalty,
            harass_coef,
        );
    }

    fn agent(&self) -> &RlAgent {
        &self.agent
    }

    fn total_steps(&self) -> usize {
        self.total_steps
    }

    fn stop(&mut self) {
        SyncTrainingSession::stop(self);
    }
}

impl<E: RlEnvironment + 'static> Drop for SyncTrainingSession<E> {
    fn drop(&mut self) {
        self.stop();
    }
}
