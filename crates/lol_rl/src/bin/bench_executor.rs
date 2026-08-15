/// bench_executor: 专用于对比 SingleThreadedExecutor 开关对并发 SPS 的影响
/// 只运行 AutoTuner 快速基准探测，不执行实际训练，目标 1 分钟内完成
use lol_env::fiora_v1::FioraVsRivenRealObs;
use lol_env::{FioraVsRivenRealEnv, RlEnvironment};
use lol_rl::autotune::AutoTuner;
use lol_rl::device::select_device;
use tracing::info;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("warn,lol_rl=info,bench_executor=info")
            }),
        )
        .init();

    let device = select_device().unwrap_or(candle_core::Device::Cpu);
    let state_dim = FioraVsRivenRealObs::dim();
    let action_space = FioraVsRivenRealEnv::action_space();
    let hidden_dim = 64;
    let horizon = 64;
    let ppo_epochs = 4;

    info!("⚡ [bench_executor] 快速基准探测开始 (SingleThreadedExecutor 开/关 对比)");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let t0 = std::time::Instant::now();
    let profile =
        AutoTuner::profile::<FioraVsRivenRealEnv>(state_dim, hidden_dim, &action_space, &device)?;
    let elapsed = t0.elapsed();

    let tuned = AutoTuner::solve(&profile, horizon, ppo_epochs);

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!(
        "🏁 [bench_executor] 探测完成，耗时: {:.1}s",
        elapsed.as_secs_f64()
    );
    info!(
        "  峰值并发 SPS: {:.1}",
        profile
            .parallel_env_us
            .iter()
            .map(|&(n, step_us)| (n as f64 * 10.0) / (step_us * 10.0 / 1_000_000.0))
            .fold(0.0f64, f64::max)
    );
    info!(
        "  AutoTuner 最优: N={}, InferBatch={}, TrainBatch={}, 预估 SPS={:.1}",
        tuned.num_parallel_envs,
        tuned.infer_batch_size,
        tuned.train_batch_size,
        tuned.estimated_sps,
    );

    Ok(())
}
