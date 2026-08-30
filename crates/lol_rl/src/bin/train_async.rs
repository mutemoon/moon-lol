use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use lol_env::fiora_v1::FioraVsRivenRealObs;
use lol_env::{FioraVsRivenRealEnv, RlEnvironment};
use lol_rl::algo::ppo::{PPOAgent, PPOConfig};
use lol_rl::autotune::AutoTuner;
use lol_rl::device::select_device;
use lol_rl::engine::r#async::{ActorPool, AsyncLearner, InferenceServer, TrajectoryRingBuffer};
use tracing::info;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(
                    "warn,lol_rl=info,lol_rl_protocol=info,train_async=info",
                )
            }),
        )
        .init();

    info!("🚀 启动基于 CUDA 加速与自动调优的异步强化学习训练系统 (Async Actor-Learner)...");

    // 1. 选择加速设备（优先选择 CUDA 设备）
    let device = select_device().unwrap_or(candle_core::Device::Cpu);
    info!("💻 运算设备: {:?}", device);

    let state_dim = FioraVsRivenRealObs::dim();
    let action_space = FioraVsRivenRealEnv::action_space();
    let hidden_dim = 64;
    let horizon = 64;
    let total_iterations = 500;

    let ppo_config = PPOConfig {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        c1: 0.5,
        ppo_epochs: 4,
        clip_vloss: true,
        max_grad_norm: 0.5,
    };

    // 2. 自动吞吐探测与最优配置求解
    let profile =
        AutoTuner::profile::<FioraVsRivenRealEnv>(state_dim, hidden_dim, &action_space, &device)?;
    let tuned = AutoTuner::solve(&profile, horizon, ppo_config.ppo_epochs);

    // 3. 初始化 PPO Agent
    let agent = PPOAgent::new(
        state_dim,
        hidden_dim,
        action_space.clone(),
        ppo_config,
        device.clone(),
    )?;

    // 4. 创建异步通信环形缓冲队列
    let queue_capacity = (tuned.num_parallel_envs * 4).clamp(32, 2048);
    let traj_queue = TrajectoryRingBuffer::<<FioraVsRivenRealEnv as RlEnvironment>::Obs>::new(queue_capacity);

    // 5. 启动 GPU/CPU 动态批处理推理引擎
    let mut infer_server = InferenceServer::new(
        agent.actor_critic.clone().into(),
        state_dim,
        tuned.infer_batch_size,
        tuned.dynamic_batch_timeout_us,
        device,
    );

    // 6. 启动多环境 Actors 并行池
    let mut actor_pool = ActorPool::spawn::<FioraVsRivenRealEnv>(
        tuned.num_parallel_envs,
        infer_server.req_tx.clone(),
        traj_queue.clone(),
        horizon,
        vec![(0, 0); tuned.num_parallel_envs], // 单智能体：无对手池
    );

    // 7. 构建异步训练器
    let target_rollout_steps = tuned.num_parallel_envs * horizon;
    let mut learner = AsyncLearner::new(
        agent,
        tuned.train_batch_size,
        target_rollout_steps,
        traj_queue,
        infer_server.model_tx.clone(),
    );

    let is_running = Arc::new(AtomicBool::new(true));
    let r_clone = is_running.clone();
    ctrlc::set_handler(move || {
        info!("🛑 收到终止信号，正在优雅关闭 Actor 与 Learner...");
        r_clone.store(false, Ordering::Relaxed);
    })
    .ok();

    // 8. 执行异步训练循环
    learner.run_loop(total_iterations, is_running.clone(), |metrics, _agent| {
        if metrics.iteration % 5 == 0 || metrics.iteration == 1 {
            info!(
                "📈 [Iter {:03}/{}] SPS: {:6.1} | Loss: {:6.3} (P: {:6.3}, V: {:6.3}, Ent: {:5.2}) | KL: {:6.4} | ClipFrac: {:4.1}% | Samples: {}",
                metrics.iteration,
                total_iterations,
                metrics.sps,
                metrics.stats.total_loss,
                metrics.stats.policy_loss,
                metrics.stats.value_loss,
                metrics.stats.entropy,
                metrics.stats.kl,
                metrics.stats.clip_frac * 100.0,
                metrics.total_samples
            );
        }
        Ok(())
    })?;

    // 9. 资源回收与停止
    actor_pool.stop();
    infer_server.stop();

    info!("🎉 异步训练完成！");
    Ok(())
}

