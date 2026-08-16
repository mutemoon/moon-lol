use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;

use candle_core::Device;
use crossbeam_channel::unbounded;
use lol_env::RlEnvironment;
use lol_env::fiora_v0::FioraVsRivenEnv;
use lol_rl::device::select_device;
use lol_rl::ppo::{PPOAgent, PPOConfig, RolloutBuffer};
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;

struct WorkerTrajectoryResult {
    buffer: RolloutBuffer,
    last_value: f32,
    ep_returns: Vec<f32>,
    reward_breakdown: HashMap<String, f32>,
}

enum WorkerMsg {
    Rollout(Arc<lol_rl::policy::ActorCritic>),
    Stop,
}

fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .parse("warn,quick_train=info,lol_rl=info")
        .unwrap();
    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!("🚀 [QuickTrain] 启动持久化环境长驻线程池 + GPU 训练测试...");

    let num_parallel_envs = 12;
    let horizon = 128; // 12 * 128 = 1536 样本/轮
    let total_iterations = 20;
    let hidden_dim = 128;
    let train_batch_size = 512;

    let gpu_device = select_device().unwrap_or(Device::Cpu);
    info!("💻 训练反传设备: {:?}", gpu_device);

    let state_dim = FioraVsRivenEnv::state_dim();
    let action_space = FioraVsRivenEnv::action_space();

    let ppo_config = PPOConfig {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        c1: 0.5,
        c2: 0.01,
        ppo_epochs: 4,
        clip_vloss: true,
        max_grad_norm: 0.5,
    };

    let mut agent = PPOAgent::new(
        state_dim,
        hidden_dim,
        action_space.clone(),
        ppo_config,
        gpu_device.clone(),
    )?;

    info!(
        "🎯 初始化持久化环境线程池: 并发环境数={}, Horizon={}, 单轮样本数={}",
        num_parallel_envs,
        horizon,
        num_parallel_envs * horizon
    );

    // 启动 32 个持久化长驻 Worker 线程（只初始化一次 Bevy 环境）
    let mut cmd_senders = Vec::with_capacity(num_parallel_envs);
    let mut resp_receivers = Vec::with_capacity(num_parallel_envs);
    let mut thread_handles = Vec::with_capacity(num_parallel_envs);

    for _ in 0..num_parallel_envs {
        let (cmd_tx, cmd_rx) = unbounded::<WorkerMsg>();
        let (resp_tx, resp_rx) = unbounded::<WorkerTrajectoryResult>();

        let handle = std::thread::spawn(move || {
            let mut env = FioraVsRivenEnv::new();
            let mut current_obs = env.reset();
            let mut cur_return = 0.0f32;

            while let Ok(msg) = cmd_rx.recv() {
                match msg {
                    WorkerMsg::Rollout(policy) => {
                        let mut buffer = RolloutBuffer::new();
                        let mut ep_returns = Vec::new();
                        let mut reward_breakdown = HashMap::new();

                        for _ in 0..horizon {
                            let state_vec = FioraVsRivenEnv::obs_to_vector(&current_obs);
                            let action_mask = FioraVsRivenEnv::action_mask(&current_obs);
                            let state_tensor = candle_core::Tensor::from_vec(
                                state_vec.clone(),
                                (1, state_dim),
                                &Device::Cpu,
                            )
                            .unwrap();

                            let (encoded, log_prob, val) = policy
                                .sample_action(&state_tensor, action_mask.as_deref())
                                .unwrap();

                            let act = FioraVsRivenEnv::action_from_encoding(&encoded);
                            let res = env.step(act);
                            let done = res.terminated || res.truncated;

                            cur_return += res.reward;
                            for item in &res.reward_breakdown {
                                *reward_breakdown.entry(item.name.clone()).or_insert(0.0) +=
                                    item.value;
                            }

                            buffer.push(
                                state_vec,
                                encoded,
                                log_prob,
                                res.reward,
                                val,
                                done,
                                action_mask,
                            );

                            if done {
                                ep_returns.push(cur_return);
                                cur_return = 0.0;
                                current_obs = env.reset();
                            } else {
                                current_obs = res.obs;
                            }
                        }

                        let last_state_vec = FioraVsRivenEnv::obs_to_vector(&current_obs);
                        let last_state_tensor = candle_core::Tensor::from_vec(
                            last_state_vec.clone(),
                            (1, state_dim),
                            &Device::Cpu,
                        )
                        .unwrap();
                        let last_value = policy
                            .get_values(&last_state_tensor)
                            .map(|v| v.first().copied().unwrap_or(0.0))
                            .unwrap_or(0.0);

                        let _ = resp_tx.send(WorkerTrajectoryResult {
                            buffer,
                            last_value,
                            ep_returns,
                            reward_breakdown,
                        });
                    }
                    WorkerMsg::Stop => break,
                }
            }
        });

        cmd_senders.push(cmd_tx);
        resp_receivers.push(resp_rx);
        thread_handles.push(handle);
    }

    let mut recent_ep_returns: VecDeque<f32> = VecDeque::with_capacity(50);
    let mut completed_episodes = 0usize;

    for iter in 1..=total_iterations {
        let iter_start = Instant::now();

        // 1. 获取 CPU 上的推理策略副本
        let cpu_policy = Arc::new(agent.actor_critic.to_device(&Device::Cpu)?);

        // 2. 触发持久化 Worker 线程并行步进 horizon 步
        let sample_t0 = Instant::now();
        for tx in &cmd_senders {
            let _ = tx.send(WorkerMsg::Rollout(cpu_policy.clone()));
        }

        let mut env_buffers = Vec::with_capacity(num_parallel_envs);
        let mut last_values = Vec::with_capacity(num_parallel_envs);
        let mut iter_reward_breakdown: HashMap<String, f32> = HashMap::new();

        for rx in &resp_receivers {
            let res = rx.recv().unwrap();
            for ret in res.ep_returns {
                if recent_ep_returns.len() >= 50 {
                    recent_ep_returns.pop_front();
                }
                recent_ep_returns.push_back(ret);
                completed_episodes += 1;
            }
            for (k, v) in res.reward_breakdown {
                *iter_reward_breakdown.entry(k).or_insert(0.0) += v;
            }
            env_buffers.push(res.buffer);
            last_values.push(res.last_value);
        }
        let sample_time = sample_t0.elapsed().as_secs_f64();

        let num_samples = num_parallel_envs * horizon;

        // 3. GPU Mini-Batch PPO 训练
        let train_t0 = Instant::now();
        let stats = agent.update_multi_buffer(&env_buffers, &last_values, train_batch_size)?;
        let train_time = train_t0.elapsed().as_secs_f64();

        let elapsed = iter_start.elapsed().as_secs_f64();
        let sps = (num_samples as f64) / elapsed;

        let avg_return = if !recent_ep_returns.is_empty() {
            recent_ep_returns.iter().sum::<f32>() / recent_ep_returns.len() as f32
        } else {
            0.0
        };

        info!(
            "📊 [Iter {:02}/{}] SPS: {:6.1} (Sample: {:.1}ms, Train: {:.1}ms) | EpReturn: {:6.2} | Loss: {:6.3} (P: {:6.3}, V: {:6.3}, Ent: {:5.2}) | KL: {:6.4} | Clip: {:4.1}% | CompletedEps: {}",
            iter,
            total_iterations,
            sps,
            sample_time * 1000.0,
            train_time * 1000.0,
            avg_return,
            stats.total_loss,
            stats.policy_loss,
            stats.value_loss,
            stats.entropy,
            stats.kl,
            stats.clip_frac * 100.0,
            completed_episodes
        );

        if iter % 5 == 0 {
            info!(
                "   🔍 奖励分项统计 (Per Step): {:?}",
                iter_reward_breakdown
                    .iter()
                    .map(|(k, v)| (k, v / num_samples as f32))
                    .collect::<Vec<_>>()
            );
        }
    }

    // 优雅关闭 Worker 线程池
    for tx in cmd_senders {
        let _ = tx.send(WorkerMsg::Stop);
    }
    for h in thread_handles {
        let _ = h.join();
    }

    info!("🎉 快速训练诊断完成！");
    Ok(())
}
