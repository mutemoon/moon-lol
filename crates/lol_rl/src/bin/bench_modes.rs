//! 对照基准：同一 selfplay 环境、同一台设备、同一网络尺寸，
//! 扫描不同并发环境数，测量 机制B（异步 GPU 推理 → GPU 训练）的真实 SPS。
//!
//! 用法: cargo run -p lol_rl --bin bench_modes -- <iters> <env_list...>
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crossbeam_channel::unbounded;
use lol_env::RlEnvironment;
use lol_env::solo_v0::SoloV0Env;
use lol_rl::async_engine::actor::SampleTransition;
use lol_rl::async_engine::{ActorPool, AsyncLearner, InferenceServer};
use lol_rl::device::select_device;
use lol_rl::ppo::{PPOAgent, PPOConfig};
use tracing::info;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .parse("warn,bench_modes=info")
                .unwrap(),
        )
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let iters: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(8);
    let envs_list: Vec<usize> = if args.len() > 1 {
        args[1..].iter().filter_map(|s| s.parse().ok()).collect()
    } else {
        vec![8]
    };

    let device = select_device().unwrap_or(candle_core::Device::Cpu);
    let state_dim = <SoloV0Env as RlEnvironment>::state_dim();
    let action_space = SoloV0Env::action_space();
    let hidden_dim = 64;
    let horizon = 64;

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

    info!(
        "💻 设备: {device:?} | selfplay | state_dim={state_dim} | horizon={horizon} | iters={iters} | envs={envs_list:?}"
    );

    for &envs in &envs_list {
        // ── 机制 B（异步 GPU 推理 → GPU 训练）──
        {
            let agent = PPOAgent::new(
                state_dim,
                hidden_dim,
                action_space.clone(),
                ppo_config.clone(),
                device.clone(),
            )?;
            let (sample_tx, sample_rx) = unbounded::<SampleTransition>();
            let target_rollout_steps = envs * horizon * 2;
            let mut infer_server = InferenceServer::new(
                agent.actor_critic.clone(),
                state_dim,
                (envs * 2).max(4),
                200,
                device.clone(),
            );
            let mut actor_pool = ActorPool::spawn::<SoloV0Env>(
                envs,
                infer_server.req_tx.clone(),
                sample_tx,
                vec![(0, 0); envs], // 纯自博弈：无历史对手
            );
            let mut learner = AsyncLearner::new(
                agent,
                512,
                target_rollout_steps,
                sample_rx,
                infer_server.model_tx.clone(),
            );
            let is_running = Arc::new(AtomicBool::new(true));
            let mut sps_sum = 0.0f64;
            let mut samples_total = 0usize;
            learner.run_loop(iters, is_running, |metrics, _a| {
                sps_sum += metrics.sps;
                samples_total += metrics.total_samples;
                Ok(())
            })?;
            info!(
                "[机制B] envs={envs:2}: 平均 SPS = {:.1} | 总样本 = {samples_total}",
                sps_sum / iters as f64
            );
            actor_pool.stop();
            infer_server.stop();
        }
    }

    info!("🏁 对照基准完成.");
    Ok(())
}
