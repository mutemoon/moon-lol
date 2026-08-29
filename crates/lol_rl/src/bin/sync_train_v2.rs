//! 对照：A 风格（同步 CPU 推理 + 同步批次 + update_multi_buffer）v2 训练。
//! 用 RolloutWorker 复现机制 A 的训练核心，判断 v2 在 A 语义下能否到 1.5。
use std::collections::VecDeque;
use std::sync::Arc;

use crossbeam_channel::unbounded;
use lol_env::RlEnvironment;
use lol_env::fiora_v2::FioraV2Env;
use lol_rl::algo::ppo::{PPOAgent, PPOConfig};
use lol_rl::device::select_device;
use lol_rl::engine::trajectory::{WorkerCommand, WorkerTrajectory};
use lol_rl::engine::worker::RolloutWorker;
use tracing::info;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .parse("error,sync_train_v2=info,lol_rl=info")
                .unwrap(),
        )
        .init();

    let iters: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let envs: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    let horizon = 160;
    let state_dim = <FioraV2Env as RlEnvironment>::state_dim();
    let action_space = FioraV2Env::action_space();
    let device = select_device().unwrap_or(candle_core::Device::Cpu);

    let ppo_config = PPOConfig {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        c1: 0.5,
        ppo_epochs: 8,
        clip_vloss: true,
        max_grad_norm: 0.5,
    };
    let mut agent = PPOAgent::new(
        state_dim,
        64,
        action_space.clone(),
        ppo_config,
        device.clone(),
    )?;

    info!("[A风格] v2 同步训练 envs={envs} horizon={horizon} state_dim={state_dim}");

    // 启动 N 个 RolloutWorker（CPU 推理）
    let mut cmd_senders = Vec::new();
    let mut resp_receivers = Vec::new();
    let mut handles = Vec::new();
    for _ in 0..envs {
        let (cmd_tx, cmd_rx) = unbounded::<WorkerCommand>();
        let (resp_tx, resp_rx) =
            unbounded::<WorkerTrajectory<lol_env::fiora_v2::FioraV2Obs>>();
        let h = std::thread::spawn(move || {
            let mut worker = RolloutWorker::<FioraV2Env>::new();
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
                            &candle_core::Device::Cpu,
                        );
                        let _ = resp_tx.send(
                            traj.unwrap_or_else(|_| WorkerTrajectory::empty()),
                        );
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
        handles.push(h);
    }

    let mut recent_returns: VecDeque<f32> = VecDeque::with_capacity(50);
    let mut total_steps = 0usize;

    for iter in 1..=iters {
        // 学习率 cosine 退火
        let progress = (iter - 1) as f32 / (iters - 1).max(1) as f32;
        let cos_progress = (1.0 + (std::f32::consts::PI * progress).cos()) * 0.5;
        let current_lr = (3e-4 * 0.1 + (3e-4 - 3e-4 * 0.1) * cos_progress as f64).max(3e-4 * 0.05);
        let _ = agent.set_lr(current_lr);

        let cpu_policy = Arc::new(
            agent
                .actor_critic
                .policy
                .to_device(&candle_core::Device::Cpu)?,
        );
        let cpu_critic = Arc::new(
            agent
                .actor_critic
                .critic
                .to_device(&candle_core::Device::Cpu)?,
        );
        for tx in &cmd_senders {
            let _ = tx.send(WorkerCommand::Rollout {
                main_policy: cpu_policy.clone(),
                main_critic: Some(cpu_critic.clone()),
                opponent_policy: None,
                opponent_critic: None,
                main_agent_idx: 0,
            });
        }
        let mut buffers = Vec::new();
        let mut last_values = Vec::new();
        for rx in &resp_receivers {
            let traj = rx
                .recv()
                .unwrap_or_else(|_| WorkerTrajectory::empty());
            for ret in traj.ep_returns {
                if recent_returns.len() >= 50 {
                    recent_returns.pop_front();
                }
                recent_returns.push_back(ret);
            }
            buffers.extend(traj.buffers);
            last_values.extend(traj.last_values);
        }
        let num_samples: usize = buffers.iter().map(|b| b.len()).sum();
        total_steps += num_samples;
        let stats = agent.update_multi_buffer(&buffers, &last_values, 512)?;
        let ep_return = if !recent_returns.is_empty() {
            recent_returns.iter().sum::<f32>() / recent_returns.len() as f32
        } else {
            0.0
        };
        info!(
            "[A风格 Iter {iter:3}] Step {total_steps:7} | Return {ep_return:+6.2} | V-loss {:.4}",
            stats.value_loss
        );
    }

    for tx in cmd_senders {
        let _ = tx.send(WorkerCommand::Stop);
    }
    for h in handles {
        let _ = h.join();
    }
    info!("🏁 [A风格] 同步 v2 训练结束，最终 Return ≈ 由日志曲线");
    Ok(())
}
