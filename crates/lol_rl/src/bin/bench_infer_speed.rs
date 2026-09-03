use std::sync::Arc;
use std::time::Instant;

use candle_core::{Device, Tensor};
use lol_env::fiora_v2::FioraV2Env;
use lol_env::traits::RlEnvironment;
use lol_rl::algo::ppo::agent::PPOAgent;
use lol_rl::algo::ppo::config::PPOConfig;
use lol_rl_protocol::PolicyBackbone;

fn format_int(n: usize) -> String {
    let s = n.to_string();
    let mut res = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            res.push(',');
        }
        res.push(c);
    }
    res
}

fn format_float(n: f64) -> String {
    let int_part = n.floor() as usize;
    let frac_part = ((n - int_part as f64) * 10.0).round() as usize;
    if frac_part == 0 {
        format_int(int_part)
    } else {
        format!("{}.{}", format_int(int_part), frac_part)
    }
}

fn main() -> anyhow::Result<()> {
    println!("================================================================================");
    println!("🧪 [LOL_RL] FioraV2 (MLP-256) 真实推理速度与基准压测");
    println!("================================================================================");

    let state_dim = FioraV2Env::state_dim();
    let action_space = FioraV2Env::action_space();
    let hidden_dim = 256;
    let config = PPOConfig::default();
    let cpu_device = Device::Cpu;

    println!("⚙️ 正在构建 FioraV2 PPO 策略网络 (Backbone: MLP, hidden_dim: 256)...");
    let agent = PPOAgent::create_for_env_with_backbone::<FioraV2Env>(
        state_dim,
        hidden_dim,
        action_space.clone(),
        config.clone(),
        cpu_device.clone(),
        PolicyBackbone::Mlp,
    )?;

    println!("\n📋 模型网络参数量明细:");
    agent.print_parameter_summary();

    let policy = Arc::new(agent.policy().clone());

    // 构造一条合法的虚拟状态观测
    let dummy_obs_vec = vec![0.0f32; state_dim];
    let dummy_obs_tensor = Tensor::from_vec(dummy_obs_vec.clone(), (1, state_dim), &cpu_device)?;

    // ─────────────────────────────────────────────────────────────────────────────
    // 1. CPU 单样本纯前向 (Batch=1 forward) 耗时测量
    // ─────────────────────────────────────────────────────────────────────────────
    println!("\n--------------------------------------------------------------------------------");
    println!("🔬 测试 1: CPU 单步策略前向 (Batch=1, forward only)");
    println!("--------------------------------------------------------------------------------");

    // 预热 10,000 次
    for _ in 0..10_000 {
        let _ = policy.forward_actor(&dummy_obs_tensor)?;
    }

    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = policy.forward_actor(&dummy_obs_tensor)?;
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64;
    let inferences_per_sec = iterations as f64 / elapsed.as_secs_f64();

    println!("  ├─ 循环次数:       {} 次", format_int(iterations));
    println!("  ├─ 总计耗时:       {:.2?}", elapsed);
    println!(
        "  ├─ 单次推理平均耗时: \x1b[1;32m{:.3} µs\x1b[0m (微秒)",
        avg_us
    );
    println!(
        "  └─ 单核纯前向吞吐:   \x1b[1;36m{} ops/sec\x1b[0m",
        format_float(inferences_per_sec)
    );

    // ─────────────────────────────────────────────────────────────────────────────
    // 2. CPU 单步完整动作采样 (Batch=1, sample_action 包含特征提取+前向+采样)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("\n--------------------------------------------------------------------------------");
    println!("🔬 测试 2: CPU 单步端到端动作决策 (Batch=1, sample_action 包含完整采样与掩码)");
    println!("--------------------------------------------------------------------------------");

    for _ in 0..10_000 {
        let _ = policy.sample_action(&dummy_obs_tensor, None)?;
    }

    let sample_iterations = 100_000;
    let start = Instant::now();
    for _ in 0..sample_iterations {
        let _ = policy.sample_action(&dummy_obs_tensor, None)?;
    }
    let elapsed = start.elapsed();
    let avg_sample_us = elapsed.as_secs_f64() * 1_000_000.0 / sample_iterations as f64;
    let samples_per_sec = sample_iterations as f64 / elapsed.as_secs_f64();

    println!("  ├─ 循环次数:       {} 次", format_int(sample_iterations));
    println!("  ├─ 总计耗时:       {:.2?}", elapsed);
    println!(
        "  ├─ 单次决策平均耗时: \x1b[1;32m{:.3} µs\x1b[0m (微秒)",
        avg_sample_us
    );
    println!(
        "  └─ 单核决策采样吞吐: \x1b[1;36m{} steps/sec\x1b[0m",
        format_float(samples_per_sec)
    );

    // ─────────────────────────────────────────────────────────────────────────────
    // 3. CPU 不同 Batch Size 下的推理吞吐量压测
    // ─────────────────────────────────────────────────────────────────────────────
    println!("\n--------------------------------------------------------------------------------");
    println!("🔬 测试 3: CPU 不同 Batch Size 批处理前向耗时与吞吐量对比");
    println!("--------------------------------------------------------------------------------");
    println!(
        "{:<12} | {:<16} | {:<18} | {:<16}",
        "Batch Size", "单批耗时 (µs)", "单样本耗时 (µs)", "吞吐量 (SPS)"
    );
    println!("--------------------------------------------------------------------------------");

    let batch_sizes = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
    for &b in &batch_sizes {
        let batch_vec = vec![0.0f32; state_dim * b];
        let batch_tensor = Tensor::from_vec(batch_vec, (b, state_dim), &cpu_device)?;

        // 预热
        for _ in 0..500 {
            let _ = policy.forward_actor(&batch_tensor)?;
        }

        let iters = (50_000 / b).max(200);
        let start = Instant::now();
        for _ in 0..iters {
            let _ = policy.forward_actor(&batch_tensor)?;
        }
        let dur = start.elapsed();
        let batch_us = dur.as_secs_f64() * 1_000_000.0 / iters as f64;
        let per_sample_us = batch_us / b as f64;
        let sps = (iters * b) as f64 / dur.as_secs_f64();

        println!(
            "{:<12} | {:<16.2} | {:<18.3} | {:<16}",
            b,
            batch_us,
            per_sample_us,
            format_float(sps)
        );
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // 4. 模拟 32 线程并发无锁本地推理 (Local Snapshot Actor Pool)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("\n--------------------------------------------------------------------------------");
    println!("🔬 测试 4: 32 个并发 Actor 线程本地无锁推理 (Sample Factory 架构模拟)");
    println!("--------------------------------------------------------------------------------");

    let num_threads = 32;
    let steps_per_thread = 50_000;
    let mut handles = Vec::new();

    let start = Instant::now();
    for _ in 0..num_threads {
        let policy_clone = policy.clone();
        let obs_vec = dummy_obs_vec.clone();
        let dev = cpu_device.clone();
        handles.push(std::thread::spawn(move || {
            let obs_t = Tensor::from_vec(obs_vec, (1, state_dim), &dev).unwrap();
            for _ in 0..steps_per_thread {
                let _ = policy_clone.sample_action(&obs_t, None).unwrap();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
    let total_elapsed = start.elapsed();
    let total_steps = num_threads * steps_per_thread;
    let multi_thread_sps = total_steps as f64 / total_elapsed.as_secs_f64();

    println!("  ├─ 并发线程数:     {} 线程", num_threads);
    println!(
        "  ├─ 每线程决策数:   {} 次 (总计: {} 次)",
        format_int(steps_per_thread),
        format_int(total_steps)
    );
    println!("  ├─ 并发总耗时:     {:.2?}", total_elapsed);
    println!(
        "  └─ 32 线程并发采样吞吐量: \x1b[1;32m{} SPS\x1b[0m (决策/秒)",
        format_float(multi_thread_sps)
    );

    // ─────────────────────────────────────────────────────────────────────────────
    // 5. 若有 CUDA 设备，测试 GPU 推理
    // ─────────────────────────────────────────────────────────────────────────────
    if let Ok(gpu_dev) = Device::new_cuda(0) {
        println!(
            "\n--------------------------------------------------------------------------------"
        );
        println!("🔬 测试 5: GPU (CUDA) 推理耗时对比");
        println!(
            "--------------------------------------------------------------------------------"
        );
        let gpu_agent = PPOAgent::create_for_env_with_backbone::<FioraV2Env>(
            state_dim,
            hidden_dim,
            action_space,
            config,
            gpu_dev.clone(),
            PolicyBackbone::Mlp,
        )?;
        let gpu_policy = gpu_agent.policy();
        let batch_tensor = Tensor::zeros((256, state_dim), candle_core::DType::F32, &gpu_dev)?;

        for _ in 0..500 {
            let _ = gpu_policy.forward_actor(&batch_tensor)?;
        }

        let start = Instant::now();
        let iters = 2000;
        for _ in 0..iters {
            let _ = gpu_policy.forward_actor(&batch_tensor)?;
        }
        let dur = start.elapsed();
        let batch_us = dur.as_secs_f64() * 1_000_000.0 / iters as f64;
        let sps = (iters * 256) as f64 / dur.as_secs_f64();
        println!("  ├─ GPU Batch=256 单批耗时: {:.2} µs", batch_us);
        println!("  └─ GPU Batch=256 吞吐量:   {} SPS", format_float(sps));
    }

    println!("\n================================================================================");
    println!("✅ 测试完成！");
    println!("================================================================================");
    Ok(())
}
