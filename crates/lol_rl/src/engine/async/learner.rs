use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crossbeam_channel::Sender;
use lol_rl_protocol::{ObsFeaturePayload, RewardItem};
use tracing::info;

use super::inference::PolicySnapshot;
use super::queue::TrajectoryRingBuffer;
use crate::algo::agent::RlAgent;
use crate::algo::buffer::RolloutBuffer;
use crate::algo::ppo::PPOStats;
use crate::engine::traits::StepOutcome;

#[derive(Debug, Clone)]
pub struct LearnerMetrics {
    pub iteration: usize,
    pub sps: f64,
    pub total_samples: usize,
    pub stats: PPOStats,
    pub ep_return: f32,
    pub mean_value: f32,
    pub ep_steps_max: usize,
    pub ep_steps_min: usize,
    pub ep_steps_avg: f32,
    pub reward_breakdown: Vec<RewardItem>,
    pub reward_variables: HashMap<String, f32>,
    pub obs_payload: Option<ObsFeaturePayload>,
}

pub struct AsyncLearner<O = ()> {
    pub agent: RlAgent,
    pub train_batch_size: usize,
    pub target_rollout_steps: usize,
    pub traj_queue: Arc<TrajectoryRingBuffer<O>>,
    /// 模型推送通道：`(slot, 权重)`，slot 0 为主策略，slot>0 为历史对手。
    pub model_update_tx: Sender<(usize, PolicySnapshot)>,
    pub total_steps: usize,
    pub current_version: usize,
    pub max_staleness: usize,
    recent_ep_returns: VecDeque<f32>,
    recent_ep_steps: VecDeque<usize>,
    initial_lr: f64,
}

impl<O: Send + 'static> AsyncLearner<O> {
    pub fn new(
        agent: impl Into<RlAgent>,
        train_batch_size: usize,
        target_rollout_steps: usize,
        traj_queue: Arc<TrajectoryRingBuffer<O>>,
        model_update_tx: Sender<(usize, PolicySnapshot)>,
    ) -> Self {
        let agent: RlAgent = agent.into();
        agent.print_parameter_summary();
        let initial_lr = match &agent {
            RlAgent::Ppo(a) => a.lr(),
            RlAgent::Grpo(a) => a.lr(),
        };
        Self {
            agent,
            train_batch_size,
            target_rollout_steps,
            traj_queue,
            model_update_tx,
            total_steps: 0,
            current_version: 0,
            max_staleness: 2, // 纯 PPO 建议容忍策略版本差 <= 2
            recent_ep_returns: VecDeque::with_capacity(50),
            recent_ep_steps: VecDeque::with_capacity(50),
            initial_lr,
        }
    }

    /// 执行一次训练迭代（从有界环形队列中消费轨迹，过滤淘汰落后版本）
    pub fn step_once(
        &mut self,
        _iter: usize,
        lr: f64,
        train_batch_size: usize,
    ) -> anyhow::Result<StepOutcome<O>> {
        let iter_start = Instant::now();
        let mut collected_buffers: Vec<RolloutBuffer> = Vec::new();
        let mut collected_last_values: Vec<f32> = Vec::new();
        let mut ep_returns_this_iter = Vec::new();
        let mut ep_cs_this_iter = Vec::new();
        let mut ep_steps_this_iter = Vec::new();
        let mut iter_reward_breakdown: HashMap<String, f32> = HashMap::new();
        let mut last_reward_variables: HashMap<String, f32> = HashMap::new();
        let mut last_obs: Option<O> = None;

        let (trajs, num_collected, queue_stats) = self.traj_queue.recv_rollout_batch(
            self.target_rollout_steps,
            self.current_version,
            self.max_staleness,
            std::time::Duration::from_secs(30),
        )?;

        for traj in trajs {
            for ret in traj.ep_returns {
                if self.recent_ep_returns.len() >= 50 {
                    self.recent_ep_returns.pop_front();
                }
                self.recent_ep_returns.push_back(ret);
                ep_returns_this_iter.push(ret);
            }
            for cs in traj.ep_cs {
                ep_cs_this_iter.push(cs);
            }
            for s in traj.completed_steps {
                if self.recent_ep_steps.len() >= 50 {
                    self.recent_ep_steps.pop_front();
                }
                self.recent_ep_steps.push_back(s);
                ep_steps_this_iter.push(s);
            }
            for (k, v) in traj.reward_breakdown {
                *iter_reward_breakdown.entry(k).or_insert(0.0) += v;
            }
            if !traj.last_reward_variables.is_empty() {
                last_reward_variables = traj.last_reward_variables;
            }
            if last_obs.is_none() {
                last_obs = traj.last_obs;
            }
            for b in traj.buffers {
                collected_buffers.push(b);
            }
            collected_last_values.extend(traj.last_values);
        }

        let collect_elapsed = iter_start.elapsed();

        if collected_buffers.is_empty() {
            anyhow::bail!("异步环形缓冲队列无可用样本（可能全部严重过期或超时）");
        }
        self.total_steps += num_collected;

        // 1. 调度学习率
        let _ = self.agent.set_lr(lr);

        // 2. 平均价值
        let val_sum: f32 = collected_buffers
            .iter()
            .map(|b| b.values.iter().sum::<f32>())
            .sum();
        let val_cnt: usize = collected_buffers.iter().map(|b| b.values.len()).sum();
        let mean_value = val_sum / (val_cnt as f32).max(1.0);

        // 3. GPU/CPU PPO/GRPO Mini-Batch 梯度更新
        let train_start = Instant::now();
        let stats = self.agent.update_multi_buffer(
            &collected_buffers,
            &collected_last_values,
            train_batch_size,
        )?;
        let train_elapsed = train_start.elapsed();

        // 4. 更新主策略版本号，并将最新模型与版本号推送给推理引擎（slot 0 = 当前主策略）
        self.current_version += 1;
        let snapshot = PolicySnapshot::with_version(
            self.agent.policy().clone(),
            self.agent.critic().cloned(),
            self.current_version,
        );
        let _ = self.model_update_tx.send((0, snapshot));

        let elapsed_sec = iter_start.elapsed().as_secs_f64();
        let sps = (num_collected as f64) / elapsed_sec.max(0.0001);

        let timing = crate::engine::traits::StepTiming {
            collect_ms: collect_elapsed.as_secs_f64() * 1000.0,
            train_ms: train_elapsed.as_secs_f64() * 1000.0,
            total_ms: elapsed_sec * 1000.0,
            infer_stats: None,
            queue_stats: Some(queue_stats),
        };

        Ok(StepOutcome {
            num_samples: num_collected,
            sps,
            stats,
            mean_value,
            timing,
            ep_returns: ep_returns_this_iter,
            ep_cs: ep_cs_this_iter,
            ep_steps: ep_steps_this_iter,
            reward_breakdown: iter_reward_breakdown,
            last_reward_variables,
            last_obs,
            obs_payload: None,
        })
    }

    /// 运行训练迭代循环
    pub fn run_loop<F>(
        &mut self,
        total_iterations: usize,
        is_running: Arc<AtomicBool>,
        mut on_iteration_done: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(&LearnerMetrics, &RlAgent) -> anyhow::Result<()>,
    {
        info!(
            "🏋️ [AsyncLearner] 启动 CUDA/CPU 异步训练引擎 (目标每轮样本数: {}, 训练 Mini-Batch: {})",
            self.target_rollout_steps, self.train_batch_size
        );

        for iter in 1..=total_iterations {
            if !is_running.load(Ordering::Relaxed) {
                break;
            }

            // 动态调节学习率（Cosine Schedule）
            let progress = if total_iterations > 1 {
                (iter - 1) as f32 / (total_iterations - 1) as f32
            } else {
                1.0
            };
            let cos_progress = (1.0 + (std::f32::consts::PI * progress).cos()) * 0.5;
            let current_lr = (self.initial_lr * 0.1
                + (self.initial_lr - self.initial_lr * 0.1) * (cos_progress as f64))
                .max(self.initial_lr * 0.05);

            let outcome = self.step_once(iter, current_lr, self.train_batch_size)?;

            let ep_return = if !self.recent_ep_returns.is_empty() {
                self.recent_ep_returns.iter().sum::<f32>() / self.recent_ep_returns.len() as f32
            } else {
                0.0
            };

            let (ep_steps_max, ep_steps_min, ep_steps_avg) = if !self.recent_ep_steps.is_empty() {
                let max = self.recent_ep_steps.iter().copied().max().unwrap_or(0);
                let min = self.recent_ep_steps.iter().copied().min().unwrap_or(0);
                let avg = self.recent_ep_steps.iter().sum::<usize>() as f32
                    / self.recent_ep_steps.len() as f32;
                (max, min, avg)
            } else {
                (0, 0, 0.0)
            };

            let reward_breakdown: Vec<RewardItem> = outcome
                .reward_breakdown
                .into_iter()
                .map(|(k, v)| RewardItem {
                    name: k,
                    value: v / (outcome.num_samples as f32).max(1.0),
                })
                .collect();

            let metrics = LearnerMetrics {
                iteration: iter,
                sps: outcome.sps,
                total_samples: self.total_steps,
                stats: outcome.stats,
                ep_return,
                mean_value: outcome.mean_value,
                ep_steps_max,
                ep_steps_min,
                ep_steps_avg,
                reward_breakdown,
                reward_variables: outcome.last_reward_variables,
                obs_payload: outcome.obs_payload,
            };

            on_iteration_done(&metrics, &self.agent)?;
        }

        info!("🛑 [AsyncLearner] 训练引擎已完成全部迭代.");
        Ok(())
    }
}
