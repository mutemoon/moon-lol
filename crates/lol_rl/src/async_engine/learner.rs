use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

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
    pub model_update_tx: Sender<ActorCritic>,
}

impl AsyncLearner {
    pub fn new(
        agent: PPOAgent,
        train_batch_size: usize,
        target_rollout_steps: usize,
        sample_rx: Receiver<SampleTransition>,
        model_update_tx: Sender<ActorCritic>,
    ) -> Self {
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
        let mut buffer = RolloutBuffer::new();
        let mut total_samples_collected = 0usize;
        let mut recent_ep_returns: VecDeque<f32> = VecDeque::with_capacity(50);

        info!(
            "🏋️ [AsyncLearner] 启动 CUDA/CPU 异步训练引擎 (目标每轮样本数: {}, 训练 Mini-Batch: {})",
            self.target_rollout_steps, self.train_batch_size
        );

        for iter in 1..=total_iterations {
            if !is_running.load(Ordering::Relaxed) {
                break;
            }

            buffer.clear();
            let iter_start = Instant::now();
            let mut iter_reward_breakdown: HashMap<String, f32> = HashMap::new();
            let mut last_reward_variables: HashMap<String, f32> = HashMap::new();
            let mut last_obs_payload: Option<ObsFeaturePayload> = None;
            let mut completed_episodes: Vec<EpisodeInfo> = Vec::new();

            // 1. 持续从采样队列收集目标数量的样本
            while buffer.len() < self.target_rollout_steps {
                if !is_running.load(Ordering::Relaxed) {
                    break;
                }
                match self.sample_rx.recv() {
                    Ok(t) => {
                        buffer.push(t.state, t.action, t.log_prob, t.reward, t.value, t.done);
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

            let num_collected = buffer.len();
            if num_collected == 0 {
                break;
            }
            total_samples_collected += num_collected;

            // 2. 动态调节策略熵退火
            let progress = if total_iterations > 1 {
                (iter - 1) as f32 / (total_iterations - 1) as f32
            } else {
                1.0
            };
            let current_c2 = (0.05 * (1.0 - progress) + 0.001 * progress).max(0.001);
            self.agent.set_entropy_coef(current_c2);

            // 3. Mini-Batch GPU/CPU PPO 训练更新
            let last_val = buffer.values.last().copied().unwrap_or(0.0);
            let mean_value = buffer.values.iter().sum::<f32>() / (num_collected as f32).max(1.0);
            let stats = self
                .agent
                .update_minibatch(&buffer, last_val, self.train_batch_size)?;

            // 4. 将最新模型分发给推理引擎（零锁异步通道）
            let _ = self.model_update_tx.send(self.agent.actor_critic.clone());

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
