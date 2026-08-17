use lol_env::fiora_riven_selfplay::FioraRivenSelfPlayEnv;
use lol_rl::service::{OutFrame, run_direct_training};
use lol_rl_protocol::{ENV_FIORA_RIVEN_SELFPLAY, TaskConfigPayload};
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(Level::ERROR.into())
        .parse("error,quick_train=info,lol_rl=info")
        .unwrap();
    tracing_subscriber::fmt().with_env_filter(filter).init();

    info!("🚀 [QuickTrain] 启动内部自博弈深度训练（目标突破 10 万步）...");

    let mut config = TaskConfigPayload::default_for_env(ENV_FIORA_RIVEN_SELFPLAY);
    config.name = "QuickTrain-SelfPlay-100k".to_string();
    config.total_iterations = 180; // 180 轮 (预估 120,000 ~ 180,000 步)

    info!(
        "🎯 任务配置: 环境={}, 迭代轮数={}, 并行环境数={}, 单环境Horizon={}, 学习率={}, Epochs={}",
        config.env_name,
        config.total_iterations,
        config.parallel_envs,
        config.rollout_steps_per_env,
        config.lr,
        config.ppo_epochs
    );

    let (mut rx, handle) = run_direct_training::<FioraRivenSelfPlayEnv>(config);

    while let Ok(msg) = rx.blocking_recv() {
        match msg {
            OutFrame::Metrics {
                step,
                ep_return,
                value,
                loss,
                policy_loss,
                value_loss,
                entropy,
                fps,
                ep_steps_avg,
                ep_steps_max,
                ep_steps_min,
                reward_breakdown,
                ..
            } => {
                let damage_dealt = reward_breakdown
                    .iter()
                    .find(|r| r.name == "造成伤害收益")
                    .map(|r| r.value)
                    .unwrap_or(0.0);
                let damage_taken = reward_breakdown
                    .iter()
                    .find(|r| r.name == "承受伤害惩罚")
                    .map(|r| r.value)
                    .unwrap_or(0.0);
                let vital = reward_breakdown
                    .iter()
                    .find(|r| r.name == "破绽攻防转移")
                    .map(|r| r.value)
                    .unwrap_or(0.0);
                let kill = reward_breakdown
                    .iter()
                    .find(|r| r.name == "击杀胜负判定")
                    .map(|r| r.value)
                    .unwrap_or(0.0);

                info!(
                    "[Step {:6}] Return: {:+6.2} | Value: {:+5.2} | Loss: {:6.4} (PLoss:{:+6.4} VLoss:{:6.4}) | Ent: {:.3} | SPS: {:4} | Steps(Avg/Max/Min): {:5.1}/{:3}/{:3} | [Dmg:{:+5.2} Taken:{:+5.2} Vital:{:+5.2} Kill:{:+5.2}]",
                    step,
                    ep_return,
                    value,
                    loss,
                    policy_loss,
                    value_loss,
                    entropy,
                    fps,
                    ep_steps_avg,
                    ep_steps_max,
                    ep_steps_min,
                    damage_dealt,
                    damage_taken,
                    vital,
                    kill
                );
            }
            OutFrame::Status { status, .. } => {
                info!("🔔 训练任务状态变更: {}", status);
                if status == "finished" || status == "failed" {
                    break;
                }
            }
            OutFrame::Log { message, .. } => {
                info!("📝 [RL Log] {}", message);
            }
            _ => {}
        }
    }

    let _ = handle.join();
    info!("🏁 [QuickTrain] 训练验证结束！");
    Ok(())
}
