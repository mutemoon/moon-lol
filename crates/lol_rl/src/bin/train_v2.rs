//! 临时复现：v2 环境走机制 B 主路径训练，观察回报收敛。
use lol_env::fiora_v2::FioraV2Env;
use lol_rl::service::{OutFrame, run_direct_training};
use lol_rl_protocol::{ENV_FIORA_V2, TaskConfigPayload};
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(Level::ERROR.into())
        .parse(
            &std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "error,train_v2=info,lol_rl=info".to_string()),
        )
        .unwrap();
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let iters: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(80);
    let force_envs: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut config = TaskConfigPayload::default_for_env(ENV_FIORA_V2);
    config.name = "TrainV2-B".to_string();
    config.total_iterations = iters;
    if force_envs > 0 {
        config.parallel_envs = force_envs;
    }
    info!(
        "🎯 v2 配置: envs={} horizon={} lr={} epochs={}",
        config.parallel_envs, config.rollout_steps_per_env, config.lr, config.ppo_epochs
    );

    let (mut rx, handle) = run_direct_training::<FioraV2Env>(config);

    while let Ok(msg) = rx.blocking_recv() {
        match msg {
            OutFrame::Metrics {
                step,
                ep_return,
                value,
                loss,
                fps: _,
                reward_breakdown,
                ep_steps_avg,
                ..
            } => {
                let breakdown: Vec<String> = reward_breakdown
                    .iter()
                    .filter(|r| r.value.abs() > 0.0001)
                    .map(|r| format!("{}={:.2}", r.name, r.value))
                    .collect();
                info!(
                    "[Step {:6}] R:{:+6.2} V:{:+5.2} L:{:6.4} | epSteps:{:.1} | {}",
                    step,
                    ep_return,
                    value,
                    loss,
                    ep_steps_avg,
                    breakdown.join(" ")
                );
            }
            OutFrame::Status { status, .. } => {
                info!("🔔 状态: {status}");
                if status == "finished" || status == "failed" {
                    break;
                }
            }
            OutFrame::Log { message, .. } => {
                info!("📝 {message}");
            }
            _ => {}
        }
    }
    let _ = handle.join();
    info!("🏁 v2 训练结束");
    Ok(())
}
