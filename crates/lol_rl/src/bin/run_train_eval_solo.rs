//! 实测 SoloV0 真实训练迭代与补刀数据变化
use lol_env::solo_v0::SoloV0Env;
use lol_env::traits::RlEnvironment;
use lol_rl::ppo::{PPOAgent, PPOConfig};
use lol_rl::training::TrainingSession;
use candle_core::Device;

fn main() -> anyhow::Result<()> {
    println!("🏋️ [SoloV0 Real Training Test] 启动 SoloV0 真实训练环境实测...\n");

    let ppo_config = PPOConfig {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        c1: 0.5,
        c2: 0.05,
        ppo_epochs: 4,
        clip_vloss: true,
        max_grad_norm: 0.5,
    };

    let state_dim = SoloV0Env::state_dim();
    let hidden_dim = 256;
    let action_space = SoloV0Env::action_space();
    let device = Device::Cpu;

    let agent = PPOAgent::create_for_env_with_backbone::<SoloV0Env>(
        state_dim,
        hidden_dim,
        action_space,
        ppo_config,
        device.clone(),
        lol_rl_protocol::PolicyBackbone::Mlp,
    )?;

    let num_parallel_envs = 4;
    let horizon = 160;

    let mut session = TrainingSession::<SoloV0Env>::new(
        agent,
        num_parallel_envs,
        state_dim,
        horizon,
        device,
    );

    // 初始设置非对称课程：0.10 小兵血量（残血小兵约 43 HP，一击必杀）
    session.update_curriculum(0.10, 1.0, 0.0, 0.0);

    println!("--------------------------------------------------------------------------------------");
    println!(" Iter | Total Steps | Avg Ep Return | Avg Ep CS | Avg Steps | SPS    | Policy Loss ");
    println!("--------------------------------------------------------------------------------------");

    for iter in 1..=30 {
        let outcome = session.step_once(iter, 3e-4, 0.05, 64)?;

        let avg_return = if !outcome.ep_returns.is_empty() {
            outcome.ep_returns.iter().sum::<f32>() / outcome.ep_returns.len() as f32
        } else {
            0.0
        };

        let avg_cs = if !outcome.ep_cs.is_empty() {
            outcome.ep_cs.iter().sum::<f32>() / outcome.ep_cs.len() as f32
        } else {
            0.0
        };

        let avg_steps = if !outcome.ep_steps.is_empty() {
            outcome.ep_steps.iter().sum::<usize>() as f32 / outcome.ep_steps.len() as f32
        } else {
            0.0
        };

        println!(
            " {:4} | {:11} | {:13.2} | {:9.2} | {:9.1} | {:6.0} | {:11.4} ",
            iter, session.total_steps, avg_return, avg_cs, avg_steps, outcome.sps, outcome.stats.policy_loss
        );
    }

    println!("--------------------------------------------------------------------------------------\n");
    println!("✅ 实测结束。");
    Ok(())
}
