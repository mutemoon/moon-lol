//! SoloV0 单步与单帧性能消融实验 (Ablation Benchmark)
//!
//! 用于深度定量分析各系统/Plugin（寻路、日志分发、小兵兵线等）
//! 对无头模式下单 step (10帧) 及单帧耗时的贡献与性能损失。

use std::time::Instant;

use bevy::prelude::*;
use lol_env::fiora_riven_common::FioraRivenBaseEnv;
use lol_env::flash_plugin::register_flash_plugin;
use lol_env::solo_v0::{
    SoloV0Action, SoloV0DiscreteAction, SoloV0Env, dispatch_single_action, get_ego_obs_from_world,
    setup_solo_v0_env_world,
};
use lol_env::traits::{EnvConfig, RenderMode};

struct AblationResult {
    name: &'static str,
    step_us: f64,
    frame_us: f64,
    step_tps: f64,
    agent_sps: f64,
    delta_vs_baseline_pct: f64,
    savings_us: f64,
}

fn create_env(
    enable_barrack: bool,
    enable_log: bool,
    enable_navigation: bool,
) -> SoloV0Env {
    let base = FioraRivenBaseEnv::builder(
        EnvConfig {
            max_steps: 1000,
            render_mode: RenderMode::Headless,
        },
        SoloV0Env::DEFAULT_MAX_STEPS,
    )
    .window_title("Solo 1v1 V0 (Ablation)")
    .map_name("solo")
    .enable_barrack(enable_barrack)
    .enable_log(enable_log)
    .enable_navigation(enable_navigation)
    .initial_positions(
        Vec3::new(2200.0, 0.0, 12650.0),
        Vec3::new(2500.0, 0.0, 12910.0),
    )
    .initial_skill_levels([1, 0, 0, 0])
    .warmup_secs(if enable_barrack { 30.0 } else { 0.0 })
    .with_plugin(register_flash_plugin)
    .on_ready(setup_solo_v0_env_world)
    .on_reset(setup_solo_v0_env_world)
    .build();

    SoloV0Env { base }
}

fn run_benchmark_for_env(
    mut env: SoloV0Env,
    test_steps: usize,
    warmup_steps: usize,
) -> (f64, f64, f64, f64) {
    let act_f = SoloV0Action::new(0.5, 0.0, SoloV0DiscreteAction::CastQ);
    let act_r = SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::Attack);

    for _ in 0..warmup_steps {
        let _ = env.step_both(act_f, act_r);
    }

    let start = Instant::now();
    for _ in 0..test_steps {
        let _ = env.step_both(act_f, act_r);
    }
    let elapsed = start.elapsed().as_secs_f64();

    let step_us = (elapsed / test_steps as f64) * 1_000_000.0;
    let frame_us = step_us / 10.0;
    let step_tps = (test_steps as f64) / elapsed;
    let agent_sps = step_tps * 2.0;

    (step_us, frame_us, step_tps, agent_sps)
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .parse("error")
                .unwrap(),
        )
        .init();

    let test_steps = 400;
    let warmup_steps = 30;

    println!();
    println!("========================================================================================================");
    println!("🧪 [SoloV0 性能消融实验 (Ablation Study)]");
    println!("🎯 目标: 拆解单步 10 帧 (1000µs / 100µs 帧) 耗时来源，精准定位寻路与日志损失");
    println!("⚙️ 测试步数: {} steps/配置 (每 step = 10 ticks 固定时间步)", test_steps);
    println!("========================================================================================================");
    println!();

    // 1. 基线配置
    println!("⏳ 正在压测 [Baseline] 完整默认环境 (兵线/Log/寻路全开)...");
    let baseline_env = create_env(true, true, true);
    let (base_step_us, base_frame_us, base_tps, base_sps) =
        run_benchmark_for_env(baseline_env, test_steps, warmup_steps);

    let mut results = Vec::new();
    results.push(AblationResult {
        name: "Baseline (默认全开)",
        step_us: base_step_us,
        frame_us: base_frame_us,
        step_tps: base_tps,
        agent_sps: base_sps,
        delta_vs_baseline_pct: 0.0,
        savings_us: 0.0,
    });

    let configurations: Vec<(&'static str, bool, bool, bool)> = vec![
        (
            "Ablation 1: No-Log (禁用日志系统)",
            true,
            false,
            true,
        ),
        (
            "Ablation 2: No-Nav (禁用寻路系统)",
            true,
            true,
            false,
        ),
        (
            "Ablation 3: No-Log & No-Nav (禁用日志+寻路)",
            true,
            false,
            false,
        ),
        (
            "Ablation 4: No-Barrack (纯1v1无小兵对战)",
            false,
            true,
            true,
        ),
        (
            "Ablation 5: No-Barrack + No-Log-Nav (纯1v1+关日志寻路)",
            false,
            false,
            false,
        ),
    ];

    for (name, barrack, log, nav) in configurations {
        println!("⏳ 正在压测 [{}]...", name);
        let env = create_env(barrack, log, nav);
        let (step_us, frame_us, tps, sps) = run_benchmark_for_env(env, test_steps, warmup_steps);
        let savings_us = base_step_us - step_us;
        let delta_pct = (savings_us / base_step_us) * 100.0;

        results.push(AblationResult {
            name,
            step_us,
            frame_us,
            step_tps: tps,
            agent_sps: sps,
            delta_vs_baseline_pct: delta_pct,
            savings_us,
        });
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // 维度 2: 单 Step 内部各项开销精细微观拆解 (Micro Profiling)
    // ─────────────────────────────────────────────────────────────────────────────
    let mut env = create_env(true, true, true);
    let act_f = SoloV0Action::new(0.5, 0.0, SoloV0DiscreteAction::CastQ);
    let act_r = SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::Attack);

    let mut t_dispatch_sum = 0.0;
    let mut t_update_sum = 0.0;
    let mut t_obs_sum = 0.0;
    let mut t_minion_query_sum = 0.0;

    let micro_steps = 300;
    let fiora = env.base.fiora;
    let riven = env.base.riven;

    for _ in 0..micro_steps {
        let t0 = Instant::now();
        let _prev_f = get_ego_obs_from_world(env.base.world(), fiora, riven, 0.0);
        let _prev_r = get_ego_obs_from_world(env.base.world(), riven, fiora, 1.0);
        let t_obs_pre = t0.elapsed().as_secs_f64();

        let t_m0 = Instant::now();
        let mut prev_minion_hps = std::collections::HashMap::new();
        {
            let mut q_minions = env.base.world_mut().query_filtered::<(Entity, &lol_core::team::Team, &lol_core::life::Health), With<lol_core::entities::minion::Minion>>();
            for (e, team, hp) in q_minions.iter(env.base.world()) {
                prev_minion_hps.insert(e, (*team, hp.value));
            }
        }
        let t_minion_pre = t_m0.elapsed().as_secs_f64();

        let t1 = Instant::now();
        dispatch_single_action(env.base.world_mut(), fiora, riven, act_f);
        dispatch_single_action(env.base.world_mut(), riven, fiora, act_r);
        let t_dispatch = t1.elapsed().as_secs_f64();

        let t2 = Instant::now();
        for _ in 0..10 {
            env.base.app.update();
        }
        let t_update = t2.elapsed().as_secs_f64();

        let t3 = Instant::now();
        let _curr_f = get_ego_obs_from_world(env.base.world(), fiora, riven, 0.0);
        let _curr_r = get_ego_obs_from_world(env.base.world(), riven, fiora, 1.0);
        let t_obs_post = t3.elapsed().as_secs_f64();

        let t_m1 = Instant::now();
        {
            let mut q_minions = env.base.world_mut().query_filtered::<(Entity, &lol_core::team::Team, &lol_core::life::Health, &Transform), With<lol_core::entities::minion::Minion>>();
            for (e, _, hp, _) in q_minions.iter(env.base.world()) {
                if let Some(&(_, prev_hp)) = prev_minion_hps.get(&e) {
                    let _ = prev_hp - hp.value;
                }
            }
        }
        let t_minion_post = t_m1.elapsed().as_secs_f64();

        t_dispatch_sum += t_dispatch;
        t_update_sum += t_update;
        t_obs_sum += t_obs_pre + t_obs_post;
        t_minion_query_sum += t_minion_pre + t_minion_post;
    }

    let avg_dispatch_us = (t_dispatch_sum / micro_steps as f64) * 1_000_000.0;
    let avg_update_us = (t_update_sum / micro_steps as f64) * 1_000_000.0;
    let avg_obs_us = (t_obs_sum / micro_steps as f64) * 1_000_000.0;
    let avg_minion_us = (t_minion_query_sum / micro_steps as f64) * 1_000_000.0;
    let avg_total_us = avg_dispatch_us + avg_update_us + avg_obs_us + avg_minion_us;

    // ─────────────────────────────────────────────────────────────────────────────
    // 在最后一次性全部打印所有消融结果和微观剖析
    // ─────────────────────────────────────────────────────────────────────────────
    println!();
    println!("========================================================================================================");
    println!("📊 [SoloV0 模块性能消融实验全景表]");
    println!("--------------------------------------------------------------------------------------------------------");
    println!(
        "{:<38} {:<13} {:<13} {:<15} {:<15} {:<18}",
        "测试配置 (Configuration)", "单步耗时(µs)", "单帧耗时(µs)", "Env 吞吐(TPS)", "Agent SPS", "性能提升 / 优化收益"
    );
    println!("--------------------------------------------------------------------------------------------------------");

    for r in &results {
        let speedup_str = if r.savings_us >= 0.0 {
            format!("+{:.1}% (-{:.0}µs)", r.delta_vs_baseline_pct, r.savings_us)
        } else {
            format!("-{:.1}% (+{:.0}µs)", -r.delta_vs_baseline_pct, -r.savings_us)
        };
        println!(
            "{:<38} {:<13.1} {:<13.1} {:<15.1} {:<15.1} {:<18}",
            r.name, r.step_us, r.frame_us, r.step_tps, r.agent_sps, speedup_str
        );
    }
    println!("--------------------------------------------------------------------------------------------------------");
    println!();
    println!("🔬 [微观剖析] Baseline 单 Step (10 帧) 内部耗时拆解:");
    println!("--------------------------------------------------------------------------------");
    println!("  1. 动作触发与分发 (dispatch_single_action):     {:>7.1} µs ({:>4.1}%)", avg_dispatch_us, (avg_dispatch_us / avg_total_us) * 100.0);
    println!("  2. ECS 10 帧主调度推进 (10x app.update()):      {:>7.1} µs ({:>4.1}%)  --> 单帧约为 {:.1} µs", avg_update_us, (avg_update_us / avg_total_us) * 100.0, avg_update_us / 10.0);
    println!("  3. 前后观测抽取 (2x get_ego_obs_from_world):  {:>7.1} µs ({:>4.1}%)", avg_obs_us, (avg_obs_us / avg_total_us) * 100.0);
    println!("  4. 小兵血量追踪/奖励辅助计算:                 {:>7.1} µs ({:>4.1}%)", avg_minion_us, (avg_minion_us / avg_total_us) * 100.0);
    println!("  ----------------------------------------------------------------");
    println!("  单 Step 汇总总计:                               {:>7.1} µs (100.0%)", avg_total_us);
    println!("========================================================================================================\n");

    Ok(())
}
