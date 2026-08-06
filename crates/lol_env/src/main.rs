use std::time::Instant;

use lol_env::fiora_vs_riven::{FioraVsRivenAction, FioraVsRivenEnv};
use lol_env::parallel::ParallelFioraVsRivenEnvs;

fn main() -> anyhow::Result<()> {
    println!("=== lol_env: Level 6 Fiora vs Riven Fast Kill RL Env ===");

    println!("\n[1] Testing Single Environment Step (Single-Threaded Bevy App)");
    let mut env = FioraVsRivenEnv::new(300);
    let initial_obs = env.reset();

    println!("Initial Observation:");
    println!("  Fiora Pos: {:?}", initial_obs.fiora_pos);
    println!(
        "  Fiora HP : {:.1}/{:.1}",
        initial_obs.fiora_hp, initial_obs.fiora_max_hp
    );
    println!("  Riven Pos: {:?}", initial_obs.riven_pos);
    println!(
        "  Riven HP : {:.1}/{:.1}",
        initial_obs.riven_hp, initial_obs.riven_max_hp
    );
    println!("  Distance : {:.1}", initial_obs.distance);
    println!(
        "  Skills   : Q={}, W={}, E={}, R={}",
        initial_obs.q_ready, initial_obs.w_ready, initial_obs.e_ready, initial_obs.r_ready
    );

    let actions = vec![
        FioraVsRivenAction::MoveEast50,
        FioraVsRivenAction::CastQ,
        FioraVsRivenAction::AttackRiven,
        FioraVsRivenAction::CastE,
        FioraVsRivenAction::CastW,
        FioraVsRivenAction::CastR,
    ];

    println!("\nExecuting Action Sequence:");
    for (idx, &action) in actions.iter().enumerate() {
        let res = env.step(action);
        println!(
            "Step {:02} | Action: {:<12} | Riven HP: {:6.1} | Reward: {:6.2} | Terminated: {}",
            idx + 1,
            format!("{:?}", action),
            res.obs.riven_hp,
            res.reward,
            res.terminated
        );
    }

    println!("\n[2] Testing Parallel Environments (Multi-Threaded Bevy Envs per bevy.md)");
    let num_parallel_envs = 4;
    let total_steps = 100;

    let start_time = Instant::now();
    let par_envs = ParallelFioraVsRivenEnvs::new(num_parallel_envs, 200);
    par_envs.reset_all();

    let par_actions = vec![FioraVsRivenAction::CastQ; num_parallel_envs];

    for _ in 0..total_steps {
        par_envs.step_all(&par_actions);
    }

    let elapsed = start_time.elapsed();
    let total_env_steps = num_parallel_envs * total_steps;

    println!(
        "Stepped {} total environment steps across {} parallel Bevy App instances in {:.2?}",
        total_env_steps, num_parallel_envs, elapsed
    );
    println!(
        "SPS (Steps Per Second): {:.0}",
        total_env_steps as f64 / elapsed.as_secs_f64()
    );

    println!("\n[Success] lol_env verification completed successfully!");
    Ok(())
}
