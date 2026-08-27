use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use candle_core::Tensor;
use crossbeam_channel::{Receiver, Sender};
use lol_rl_protocol::{ObsFeaturePayload, RewardItem};
use tracing::info;

use super::actor::{EpisodeInfo, SampleTransition};
use crate::policy::ActorCritic;
use crate::ppo::{PPOAgent, PPOStats, RolloutBuffer};

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

pub struct AsyncLearner {
    pub agent: PPOAgent,
    pub train_batch_size: usize,
    pub target_rollout_steps: usize,
    pub sample_rx: Receiver<SampleTransition>,
    /// 模型推送通道：`(slot, 权重)`，slot 0 为主策略，slot>0 为历史对手。
    pub model_update_tx: Sender<(usize, ActorCritic)>,
}

impl AsyncLearner {
    pub fn new(
        agent: PPOAgent,
        train_batch_size: usize,
        target_rollout_steps: usize,
        sample_rx: Receiver<SampleTransition>,
        model_update_tx: Sender<(usize, ActorCritic)>,
    ) -> Self {
        agent.print_parameter_summary();
        Self {
            agent,
            train_batch_size,
            target_rollout_steps,
            sample_rx,
            model_update_tx,
        }
    }

    /// 运行训练迭代循环
    pub fn run_loop<F>(
        &mut self,
        total_iterations: usize,
        is_running: Arc<AtomicBool>,
        mut on_iteration_done: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(&LearnerMetrics, &PPOAgent) -> anyhow::Result<()>,
    {
        // 按 worker（env）分 buffer：GAE 需要在单条连续轨迹内计算，混流 buffer 会破坏时序
        let mut worker_buffers: HashMap<usize, RolloutBuffer> = HashMap::new();
        let mut total_samples_collected = 0usize;
        let mut recent_ep_returns: VecDeque<f32> = VecDeque::with_capacity(50);
        // 记录初始学习率（调度基准），避免被 set_lr 修改后的值污染
        let initial_lr = self.agent.lr();

        info!(
            "🏋️ [AsyncLearner] 启动 CUDA/CPU 异步训练引擎 (目标每轮样本数: {}, 训练 Mini-Batch: {})",
            self.target_rollout_steps, self.train_batch_size
        );

        for iter in 1..=total_iterations {
            if !is_running.load(Ordering::Relaxed) {
                break;
            }

            worker_buffers.clear();
            let iter_start = Instant::now();
            let mut iter_reward_breakdown: HashMap<String, f32> = HashMap::new();
            let mut last_reward_variables: HashMap<String, f32> = HashMap::new();
            let mut last_obs_payload: Option<ObsFeaturePayload> = None;
            let mut completed_episodes: Vec<EpisodeInfo> = Vec::new();

            // 1. 持续从采样队列收集目标数量的样本（各 worker 轨迹进各自 buffer）
            while {
                let n: usize = worker_buffers.values().map(|b| b.len()).sum();
                n < self.target_rollout_steps
            } {
                if !is_running.load(Ordering::Relaxed) {
                    break;
                }
                match self.sample_rx.recv() {
                    Ok(t) => {
                        let buf = worker_buffers
                            .entry(t.worker_id)
                            .or_insert_with(RolloutBuffer::new);
                        buf.push_full(
                            t.state,
                            t.action,
                            t.log_prob,
                            t.reward,
                            t.value,
                            t.terminated,
                            t.truncated,
                            t.truncated_next_value,
                            t.action_mask,
                        );
                        if let Some(ep) = t.episode_info {
                            if recent_ep_returns.len() >= 50 {
                                recent_ep_returns.pop_front();
                            }
                            recent_ep_returns.push_back(ep.ep_return);
                            completed_episodes.push(ep);
                        }
                        for item in t.reward_breakdown {
                            *iter_reward_breakdown.entry(item.name).or_insert(0.0) += item.value;
                        }
                        if !t.reward_variables.is_empty() {
                            last_reward_variables = t.reward_variables;
                        }
                        if t.obs_payload.is_some() {
                            last_obs_payload = t.obs_payload;
                        }
                    }
                    Err(_) => break,
                }
            }

            let num_collected: usize = worker_buffers.values().map(|b| b.len()).sum();
            if num_collected == 0 {
                break;
            }
            total_samples_collected += num_collected;

            // 2. 动态调节策略熵与学习率（对齐机制 A 的 Cosine Schedule with Safe Entropy Floor）
            let progress = if total_iterations > 1 {
                (iter - 1) as f32 / (total_iterations - 1) as f32
            } else {
                1.0
            };
            let cos_progress = (1.0 + (std::f32::consts::PI * progress).cos()) * 0.5;
            let current_c2 = (0.015 + (0.05 - 0.015) * cos_progress).max(0.015);
            self.agent.set_entropy_coef(current_c2);
            let current_lr = (initial_lr * 0.1
                + (initial_lr - initial_lr * 0.1) * (cos_progress as f64))
                .max(initial_lr * 0.05);
            let _ = self.agent.set_lr(current_lr);

            // 3. 训练：每 worker 独立 GAE + Mini-Batch PPO（对齐机制 A 的 update_multi_buffer）
            let mut buffers: Vec<RolloutBuffer> = worker_buffers.drain().map(|(_, b)| b).collect();

            // 3.1 用最新 agent 重算所有样本的 value（消除异步采样滞后权重造成的价值偏差）。
            //     log_prob 保留 actor 采样时计算值（PPO importance ratio 需要），只更新 critic value。
            for b in buffers.iter_mut() {
                if b.is_empty() {
                    continue;
                }
                let n = b.len();
                let state_dim = b.states[0].len();
                let flat: Vec<f32> = b.states.iter().flatten().copied().collect();
                let t = Tensor::from_vec(flat, (n, state_dim), self.agent.device())?;
                b.values = self.agent.actor_critic.get_values(&t)?;
            }

            let mut last_vals = Vec::with_capacity(buffers.len());
            for b in &buffers {
                last_vals.push(bootstrap_last_val(&self.agent, b)?);
            }
            let mean_value = buffers
                .iter()
                .flat_map(|b| b.values.iter().copied())
                .sum::<f32>()
                / (num_collected as f32).max(1.0);
            let stats =
                self.agent
                    .update_multi_buffer(&buffers, &last_vals, self.train_batch_size)?;

            // 4. 将最新模型分发给推理引擎（零锁异步通道），slot 0 = 当前主策略
            let _ = self
                .model_update_tx
                .send((0, self.agent.actor_critic.clone()));

            let elapsed_sec = iter_start.elapsed().as_secs_f64();
            let sps = (num_collected as f64) / elapsed_sec.max(0.0001);

            let ep_return = if !recent_ep_returns.is_empty() {
                recent_ep_returns.iter().sum::<f32>() / recent_ep_returns.len() as f32
            } else {
                0.0
            };

            let (ep_steps_max, ep_steps_min, ep_steps_avg) = if !completed_episodes.is_empty() {
                let max = completed_episodes
                    .iter()
                    .map(|e| e.ep_steps)
                    .max()
                    .unwrap_or(0);
                let min = completed_episodes
                    .iter()
                    .map(|e| e.ep_steps)
                    .min()
                    .unwrap_or(0);
                let avg = completed_episodes.iter().map(|e| e.ep_steps).sum::<usize>() as f32
                    / completed_episodes.len() as f32;
                (max, min, avg)
            } else {
                (0, 0, 0.0)
            };

            let reward_breakdown: Vec<RewardItem> = iter_reward_breakdown
                .into_iter()
                .map(|(k, v)| RewardItem {
                    name: k,
                    value: v / (num_collected as f32).max(1.0),
                })
                .collect();

            let metrics = LearnerMetrics {
                iteration: iter,
                sps,
                total_samples: total_samples_collected,
                stats,
                ep_return,
                mean_value,
                ep_steps_max,
                ep_steps_min,
                ep_steps_avg,
                reward_breakdown,
                reward_variables: last_reward_variables,
                obs_payload: last_obs_payload,
            };

            on_iteration_done(&metrics, &self.agent)?;
        }

        info!("🛑 [AsyncLearner] 训练引擎已完成全部迭代.");
        Ok(())
    }
}

/// 计算一个 worker 轨迹 buffer 末尾未完成 episode 的 bootstrap 价值 V(s_T)。
///
/// 用主策略从 buffer 最后一个 state 前向推断真实残局价值（对齐机制 A 的 last_values 语义）；
/// 若 buffer 末尾恰为 episode 结束（done），GAE 会在 done 处截断、不使用 last_val，返回 0 即可。
fn bootstrap_last_val(agent: &PPOAgent, buffer: &RolloutBuffer) -> candle_core::Result<f32> {
    if buffer.is_empty() {
        return Ok(0.0);
    }
    let last_done = buffer.dones.last().copied().unwrap_or(true);
    if last_done {
        return Ok(0.0);
    }
    let last_state = &buffer.states[buffer.len() - 1];
    let state_dim = last_state.len();
    let tensor = Tensor::from_vec(last_state.clone(), (1, state_dim), agent.device())?;
    agent
        .actor_critic
        .get_values(&tensor)
        .map(|v| v.first().copied().unwrap_or(0.0))
}
