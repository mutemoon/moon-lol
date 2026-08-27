//! SoloV0 全速吞吐量全景基准测试工具 (Headless Benchmark)
//!
//! 包含三大测试维度：
//! 1. 纯无头环境 step 性能 (Raw Env Steps & Sample SPS，排除神经网络干扰)
//! 2. 机制 A：同步 Rollout + CPU 推理 + GPU 训练闭环 SPS (TrainingSession)
//! 3. 机制 B：异步 GPU 推理 + Actor 并发池 + GPU 训练闭环 SPS (InferenceServer + ActorPool + AsyncLearner)

use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

use lol_env::solo_v0::{SoloV0Action, SoloV0DiscreteAction, SoloV0Env};
use lol_env::traits::{EnvConfig, RenderMode, RlEnvironment};
use lol_rl::device::select_device;

struct BenchResult {
    threads: usize,
    avg_step_us: f64,
    env_sps: f64,
    agent_sps: f64,
    speedup: f64,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .parse("error")
                .unwrap(),
        )
        .init();

    let num_cpus = num_cpus::get();
    let device = select_device().unwrap_or(candle_core::Device::Cpu);

    // ─────────────────────────────────────────────────────────────────────────────
    // 维度 1: 纯无头环境 Step 吞吐量 (Raw Env TPS & Agent SPS)
    // ─────────────────────────────────────────────────────────────────────────────
    let thread_candidates = vec![1, 2, 4, 8];
    let steps_per_worker = 300;
    let warmup_steps = 30;
    let mut baseline_env_sps = 0.0f64;
    let mut results = Vec::new();

    for &threads in &thread_candidates {
        let barrier = Arc::new(Barrier::new(threads + 1));
        let mut handles = Vec::with_capacity(threads);

        for _ in 0..threads {
            let b = barrier.clone();
            let h = thread::spawn(move || {
                let mut env = SoloV0Env::with_config(EnvConfig {
                    max_steps: 1000,
                    render_mode: RenderMode::Headless,
                });
                env.reset();

                let act_f = SoloV0Action::new(0.5, 0.0, SoloV0DiscreteAction::CastQ);
                let act_r = SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::Attack);

                // Warmup
                for _ in 0..warmup_steps {
                    let _ = env.step_both(act_f, act_r);
                }

                b.wait();

                for _ in 0..steps_per_worker {
                    let _ = env.step_both(act_f, act_r);
                }
            });
            handles.push(h);
        }

        barrier.wait();
        let start = Instant::now();
        for h in handles {
            let _ = h.join();
        }
        let elapsed = start.elapsed().as_secs_f64();

        let total_env_steps = (threads * steps_per_worker) as f64;
        let env_sps = total_env_steps / elapsed;
        let agent_sps = env_sps * 2.0; // SoloV0 双智能体
        let avg_step_us = (elapsed / steps_per_worker as f64) * 1_000_000.0;

        if threads == 1 {
            baseline_env_sps = env_sps;
        }
        let speedup = env_sps / baseline_env_sps.max(1.0);

        results.push(BenchResult {
            threads,
            avg_step_us,
            env_sps,
            agent_sps,
            speedup,
        });
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // 所有测试完成后一次性全部打印
    // ─────────────────────────────────────────────────────────────────────────────
    println!();
    println!("================================================================================");
    println!("🚀 [SoloV0 全速基准性能评测 (Headless)]");
    println!("💻 CPU 逻辑核心数: {} | 计算设备: {:?}", num_cpus, device);
    println!("🎮 环境: SoloV0 (剑姬 vs 瑞雯 上路对线自博弈，双智能体 2 agents/env)");
    println!("================================================================================");
    println!();

    println!("📊 [1/3] 纯无头环境 Step 吞吐量压测 (无神经网络推理/训练开销)");
    println!("--------------------------------------------------------------------------------");
    println!(
        "{:<10} {:<15} {:<18} {:<18} {:<15}",
        "并发线程", "单实例单步 (µs)", "Env 吞吐 (Steps/s)", "Agent 吞吐 (SPS)", "加速比"
    );
    println!("--------------------------------------------------------------------------------");

    for res in &results {
        let speedup_str = format!("{:.2}x", res.speedup);
        println!(
            "{:<10} {:<15.1} {:<18.1} {:<18.1} {:<15}",
            res.threads, res.avg_step_us, res.env_sps, res.agent_sps, speedup_str
        );
    }
    println!("--------------------------------------------------------------------------------\n");

    Ok(())
}
