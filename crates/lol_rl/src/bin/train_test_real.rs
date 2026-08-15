use std::time::Instant;

use candle_core::Tensor;
use lol_env::fiora_vs_riven_real::{FioraVsRivenRealAction, FioraVsRivenRealObs};
use lol_env::parallel::ParallelFioraVsRivenRealEnvs;
use lol_env::{FioraVsRivenRealEnv, RlEnvironment};
use lol_rl::ppo::{PPOAgent, PPOConfig, RolloutBuffer};
use tracing::info;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting local RL test experiment (hybrid continuous move)...");

    let env_max_steps = 40;
    let state_dim = FioraVsRivenRealObs::dim();
    let action_space = FioraVsRivenRealEnv::action_space();
    let enc_dim = action_space.encoding_dim();
    let num_parallel_envs = 4;
    let total_iterations = 1000;
    let hidden_dim = 64;

    let config = PPOConfig {
        lr: 3e-4, // standard PPO learning rate
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        c1: 0.5,
        c2: 0.05,
        ppo_epochs: 4,
    };

    let device = candle_core::Device::Cpu;

    let mut agent = PPOAgent::new(state_dim, hidden_dim, action_space, config, device.clone())?;

    let par_envs = ParallelFioraVsRivenRealEnvs::new(num_parallel_envs, env_max_steps);
    let mut env_buffers = Vec::with_capacity(num_parallel_envs);
    for _ in 0..num_parallel_envs {
        env_buffers.push(RolloutBuffer::new());
    }
    let mut current_obss = par_envs.reset_all();
    let mut env_returns = vec![0.0f32; num_parallel_envs];
    let mut recent_ep_returns: std::collections::VecDeque<f32> =
        std::collections::VecDeque::with_capacity(50);

    let iter_start_global = Instant::now();

    for iter in 1..=total_iterations {
        let progress = if total_iterations > 1 {
            (iter - 1) as f32 / (total_iterations - 1) as f32
        } else {
            1.0
        };
        let current_c2 = (0.05 * (1.0 - progress) + 0.001 * progress).max(0.001);
        agent.set_entropy_coef(current_c2);

        for b in &mut env_buffers {
            b.clear();
        }

        let mut completed_envs = vec![false; num_parallel_envs];
        let mut iter_steps_count = 0usize;

        while !completed_envs.iter().all(|&c| c) {
            let mut actions = Vec::with_capacity(num_parallel_envs);
            let mut action_encodings = Vec::with_capacity(num_parallel_envs);
            let mut log_probs = Vec::with_capacity(num_parallel_envs);
            let mut values = Vec::with_capacity(num_parallel_envs);

            for i in 0..num_parallel_envs {
                if completed_envs[i] {
                    actions.push(FioraVsRivenRealAction::from_encoding(&vec![0.0; enc_dim]));
                    action_encodings.push(vec![0.0; enc_dim]);
                    log_probs.push(0.0);
                    values.push(0.0);
                    continue;
                }

                let obs = &current_obss[i];
                let state_vec = obs.to_vector();
                let state_tensor = Tensor::from_vec(state_vec.clone(), (1, state_dim), &device)?;

                if let Ok((encoded, log_prob, val)) =
                    agent.actor_critic.sample_action(&state_tensor, &state_vec)
                {
                    actions.push(FioraVsRivenRealAction::from_encoding(&encoded));
                    action_encodings.push(encoded);
                    log_probs.push(log_prob);
                    values.push(val);
                } else {
                    actions.push(FioraVsRivenRealAction::from_encoding(&vec![0.0; enc_dim]));
                    action_encodings.push(vec![0.0; enc_dim]);
                    log_probs.push(0.0);
                    values.push(0.0);
                }
            }

            let step_results = par_envs.step_all(&actions);

            for i in 0..num_parallel_envs {
                if completed_envs[i] {
                    continue;
                }

                let res = &step_results[i];
                env_returns[i] += res.reward;
                iter_steps_count += 1;

                env_buffers[i].push(
                    current_obss[i].to_vector(),
                    action_encodings[i].clone(),
                    log_probs[i],
                    res.reward,
                    values[i],
                    res.terminated || res.truncated,
                );

                let is_kill = res
                    .reward_breakdown
                    .iter()
                    .any(|r| r.name == "kill_reward" && r.value > 0.0);
                if is_kill {
                    info!(
                        ">>> KILLED RIVEN IN ENV {} AT STEP {} <<<",
                        i, iter_steps_count
                    );
                }

                if res.terminated || res.truncated {
                    let ep_ret = env_returns[i];
                    env_returns[i] = 0.0;
                    if recent_ep_returns.len() >= 50 {
                        recent_ep_returns.pop_front();
                    }
                    recent_ep_returns.push_back(ep_ret);
                    current_obss[i] = par_envs.reset_one(i);
                    completed_envs[i] = true;
                } else {
                    current_obss[i] = res.obs.clone();
                }
            }
        }

        let mut combined_buffer = RolloutBuffer::new();
        for b in &env_buffers {
            for t in 0..b.len() {
                combined_buffer.push(
                    b.states[t].clone(),
                    b.actions[t].clone(),
                    b.log_probs[t],
                    b.rewards[t],
                    b.values[t],
                    b.dones[t],
                );
            }
        }

        let last_val_scalar = 0.0f32;
        if let Ok(stats) = agent.update(&combined_buffer, last_val_scalar) {
            let ep_return = if !recent_ep_returns.is_empty() {
                let sum: f32 = recent_ep_returns.iter().sum();
                sum / recent_ep_returns.len() as f32
            } else {
                let sum: f32 = env_returns.iter().sum();
                sum / num_parallel_envs.max(1) as f32
            };

            if iter % 10 == 0 || iter == 1 {
                info!(
                    "Iter: {:4} | EpRet: {:7.2} | PLoss: {:6.3} | VLoss: {:6.3} | Ent: {:5.3} | KL: {:5.3}",
                    iter, ep_return, stats.policy_loss, stats.value_loss, stats.entropy, stats.kl
                );
            }
        }
    }

    info!(
        "Finished local RL test in {:.2}s",
        iter_start_global.elapsed().as_secs_f64()
    );
    Ok(())
}
