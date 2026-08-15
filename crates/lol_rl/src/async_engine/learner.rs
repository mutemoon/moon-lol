use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};
use tracing::info;

use super::actor::SampleTransition;
use crate::policy::ActorCritic;
use crate::ppo::{PPOAgent, PPOStats, RolloutBuffer};

#[derive(Debug, Clone)]
pub struct LearnerMetrics {
    pub iteration: usize,
    pub sps: f64,
    pub total_samples: usize,
    pub stats: PPOStats,
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
        F: FnMut(&LearnerMetrics),
    {
        let mut buffer = RolloutBuffer::new();
        let mut total_samples_collected = 0usize;

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

            // 1. 持续从采样队列收集目标数量的样本
            while buffer.len() < self.target_rollout_steps {
                if !is_running.load(Ordering::Relaxed) {
                    break;
                }
                match self.sample_rx.recv() {
                    Ok(t) => {
                        buffer.push(t.state, t.action, t.log_prob, t.reward, t.value, t.done);
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
            let stats = self
                .agent
                .update_minibatch(&buffer, last_val, self.train_batch_size)?;

            // 4. 将最新模型分发给推理引擎（零锁异步通道）
            let _ = self.model_update_tx.send(self.agent.actor_critic.clone());

            let elapsed_sec = iter_start.elapsed().as_secs_f64();
            let sps = (num_collected as f64) / elapsed_sec.max(0.0001);

            let metrics = LearnerMetrics {
                iteration: iter,
                sps,
                total_samples: total_samples_collected,
                stats,
            };

            on_iteration_done(&metrics);
        }

        info!("🛑 [AsyncLearner] 训练引擎已完成全部迭代.");
        Ok(())
    }
}
