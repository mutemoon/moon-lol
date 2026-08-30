use std::sync::Arc;
use std::time::Instant;

use candle_core::{DType, Device, Tensor};
use lol_env::fiora_v2::FioraV2Env;
use lol_env::traits::RlEnvironment;
use lol_rl::algo::ppo::agent::PPOAgent;
use lol_rl::algo::ppo::config::PPOConfig;
use lol_rl::policy::PolicyNetwork;
use lol_rl_protocol::PolicyBackbone;

fn format_num(n: f64) -> String {
    let int_part = n.floor() as usize;
    let s = int_part.to_string();
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

fn create_agent(
    hidden_dim: usize,
    backbone: PolicyBackbone,
    device: &Device,
) -> anyhow::Result<(PPOAgent, usize)> {
    let state_dim = FioraV2Env::state_dim();
    let action_space = FioraV2Env::action_space();
    let config = PPOConfig::default();
    let agent = PPOAgent::create_for_env_with_backbone::<FioraV2Env>(
        state_dim,
        hidden_dim,
        action_space,
        config,
        device.clone(),
        backbone,
    )?;
    let total_params = agent.parameter_summary().total_params;
    Ok((agent, total_params))
}

fn bench_batch_scaling(
    name: &str,
    policy: &PolicyNetwork,
    state_dim: usize,
    device: &Device,
    batch_sizes: &[usize],
) -> anyhow::Result<()> {
    println!("\n📊 【{}】不同 Batch Size 下的 GPU 推理延迟与吞吐量", name);
    println!("--------------------------------------------------------------------------------------------------");
    println!(
        "{:<10} | {:<18} | {:<18} | {:<20} | {:<16}",
        "Batch Size", "单批耗时 (µs)", "单样本耗时 (µs)", "GPU 吞吐量 (SPS)", "相对 Batch=1 提速"
    );
    println!("--------------------------------------------------------------------------------------------------");

    let mut base_per_sample_us = 1.0;

    for (idx, &b) in batch_sizes.iter().enumerate() {
        let input = Tensor::zeros((b, state_dim), DType::F32, device)?;

        // GPU 预热
        for _ in 0..10 {
            let _ = policy.forward_actor(&input)?;
        }
        if let Device::Cuda(_) = device {
            device.synchronize()?;
        }

        let iters = match b {
            1..=8 => 150,
            16..=64 => 100,
            128..=512 => 80,
            _ => 50,
        };

        let start = Instant::now();
        for _ in 0..iters {
            let _ = policy.forward_actor(&input)?;
        }
        if let Device::Cuda(_) = device {
            device.synchronize()?;
        }
        let dur = start.elapsed();

        let batch_us = dur.as_secs_f64() * 1_000_000.0 / iters as f64;
        let per_sample_us = batch_us / b as f64;
        let sps = (iters * b) as f64 / dur.as_secs_f64();

        if idx == 0 {
            base_per_sample_us = per_sample_us;
        }
        let speedup = base_per_sample_us / per_sample_us;

        println!(
            "{:<10} | {:<18.2} | {:<18.3} | \x1b[1;36m{:<20}\x1b[0m | \x1b[1;32m{:<16.1}x\x1b[0m",
            b,
            batch_us,
            per_sample_us,
            format!("{} SPS", format_num(sps)),
            speedup
        );
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    println!("================================================================================");
    println!("🚀 [LOL_RL] GPU (CUDA) 纯推理性能与吞吐量基准评测");
    println!("================================================================================");

    let gpu_device = match Device::new_cuda(0) {
        Ok(dev) => dev,
        Err(e) => {
            eprintln!("❌ 无法初始化 CUDA 设备: {e}");
            return Ok(());
        }
    };
    let cpu_device = Device::Cpu;
    let state_dim = FioraV2Env::state_dim();

    println!("✅ 成功检测并绑定 CUDA 设备: GPU 0\n");

    // ─────────────────────────────────────────────────────────────────────────────
    // 1. 模型定义与构建
    // ─────────────────────────────────────────────────────────────────────────────
    println!("📦 正在构建 2 种策略网络拓扑:");

    // 模型 1: 小型 MLP (hidden_dim = 256, ~91K 参数)
    let (agent_mlp_256, p1) = create_agent(256, PolicyBackbone::Mlp, &gpu_device)?;
    let policy_mlp_256 = Arc::new(agent_mlp_256.policy().clone());
    println!("  ├─ 1. 【小型网络】MLP-256 (当前基线):   {:>8} 参数 ({:.2} K)", format_num(p1 as f64), p1 as f64 / 1000.0);

    // 模型 2: 中型 MLP (hidden_dim = 512, ~350K 参数)
    let (agent_mlp_512, p2) = create_agent(512, PolicyBackbone::Mlp, &gpu_device)?;
    let policy_mlp_512 = Arc::new(agent_mlp_512.policy().clone());
    println!("  └─ 2. 【中型网络】MLP-512 (加宽前馈):   {:>8} 参数 ({:.2} K)", format_num(p2 as f64), p2 as f64 / 1000.0);

    let batch_sizes = [1, 4, 16, 64, 128, 256, 512, 1024, 2048, 4096];

    // ─────────────────────────────────────────────────────────────────────────────
    // 2. 压测 GPU Batch 缩放吞吐
    // ─────────────────────────────────────────────────────────────────────────────
    bench_batch_scaling("小型网络 (MLP-256, 91K)", &policy_mlp_256, state_dim, &gpu_device, &batch_sizes)?;
    bench_batch_scaling("中型网络 (MLP-512, 350K)", &policy_mlp_512, state_dim, &gpu_device, &batch_sizes)?;

    // ─────────────────────────────────────────────────────────────────────────────
    // 3. GPU 端到端细粒度耗时拆解 (PCIe H2D + CUDA Kernel + D2H)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("\n--------------------------------------------------------------------------------------------------");
    println!("🔬 深入剖析: GPU 单步端到端推理中的【各阶段耗时分布拆解】(Batch = 256)");
    println!("--------------------------------------------------------------------------------------------------");

    let host_raw_data = vec![0.0f32; state_dim * 256];
    let iters = 300;

    // A. 测量 CPU -> GPU 数据拷贝 (Host to Device)
    let start = Instant::now();
    for _ in 0..iters {
        let _ = Tensor::from_vec(host_raw_data.clone(), (256, state_dim), &gpu_device)?;
    }
    gpu_device.synchronize()?;
    let h2d_us = start.elapsed().as_secs_f64() * 1_000_000.0 / iters as f64;

    // B. 测量纯 GPU 计算 (Kernel Execution on VRAM)
    let gpu_input = Tensor::zeros((256, state_dim), DType::F32, &gpu_device)?;
    for _ in 0..20 {
        let _ = policy_mlp_256.forward_actor(&gpu_input)?;
    }
    gpu_device.synchronize()?;

    let start = Instant::now();
    for _ in 0..iters {
        let _ = policy_mlp_256.forward_actor(&gpu_input)?;
    }
    gpu_device.synchronize()?;
    let kernel_us = start.elapsed().as_secs_f64() * 1_000_000.0 / iters as f64;

    // C. 测量 GPU -> CPU 动作结果拷贝 (Device to Host)
    let gpu_output = policy_mlp_256.forward_actor(&gpu_input)?;
    let start = Instant::now();
    for _ in 0..iters {
        let _: Vec<Vec<f32>> = gpu_output.to_vec2()?;
    }
    let d2h_us = start.elapsed().as_secs_f64() * 1_000_000.0 / iters as f64;

    let total_e2e_us = h2d_us + kernel_us + d2h_us;
    println!("  ├─ 1. [CPU ➔ GPU] Host-to-Device 拷贝: {:>8.2} µs (占 {:>4.1}%)", h2d_us, h2d_us / total_e2e_us * 100.0);
    println!("  ├─ 2. [GPU 核心]  CUDA 算子前向计算:     {:>8.2} µs (占 {:>4.1}%)", kernel_us, kernel_us / total_e2e_us * 100.0);
    println!("  ├─ 3. [GPU ➔ CPU] Device-to-Host 拷贝: {:>8.2} µs (占 {:>4.1}%)", d2h_us, d2h_us / total_e2e_us * 100.0);
    println!("  └─ 🎯 [端到端总计] 单批 256 样本总耗时:   {:>8.2} µs (单样本折合 \x1b[1;32m{:.3} µs\x1b[0m)", total_e2e_us, total_e2e_us / 256.0);

    // ─────────────────────────────────────────────────────────────────────────────
    // 4. CPU vs GPU 在不同模型规模下的性能拐点 (Cross-over Analysis)
    // ─────────────────────────────────────────────────────────────────────────────
    println!("\n--------------------------------------------------------------------------------------------------");
    println!("⚖️ CPU vs GPU 跨模型规模【吞吐量与性能拐点对比】(Batch = 256)");
    println!("--------------------------------------------------------------------------------------------------");
    println!("{:<24} | {:<16} | {:<18} | {:<18} | {:<14}", "模型规格 (架构 / 参数量)", "CPU 吞吐量 (SPS)", "GPU 吞吐量 (SPS)", "GPU 相对 CPU 加速比", "推荐计算设备");
    println!("--------------------------------------------------------------------------------------------------");

    let models = [
        ("MLP-256 (91K)", 256, PolicyBackbone::Mlp),
        ("MLP-512 (350K)", 512, PolicyBackbone::Mlp),
    ];

    for &(m_name, h_dim, b_bone) in &models {
        // CPU 测试
        let (cpu_agent, _) = create_agent(h_dim, b_bone, &cpu_device)?;
        let cpu_pol = cpu_agent.policy();
        let cpu_in = Tensor::zeros((256, state_dim), DType::F32, &cpu_device)?;
        for _ in 0..10 { let _ = cpu_pol.forward_actor(&cpu_in)?; }
        let c_start = Instant::now();
        for _ in 0..100 { let _ = cpu_pol.forward_actor(&cpu_in)?; }
        let c_dur = c_start.elapsed();
        let cpu_sps = (100 * 256) as f64 / c_dur.as_secs_f64();

        // GPU 测试
        let (gpu_agent, _) = create_agent(h_dim, b_bone, &gpu_device)?;
        let gpu_pol = gpu_agent.policy();
        let gpu_in = Tensor::zeros((256, state_dim), DType::F32, &gpu_device)?;
        for _ in 0..10 { let _ = gpu_pol.forward_actor(&gpu_in)?; }
        gpu_device.synchronize()?;
        let g_start = Instant::now();
        for _ in 0..200 { let _ = gpu_pol.forward_actor(&gpu_in)?; }
        gpu_device.synchronize()?;
        let g_dur = g_start.elapsed();
        let gpu_sps = (200 * 256) as f64 / g_dur.as_secs_f64();

        let ratio = gpu_sps / cpu_sps;
        let recommendation = if ratio > 1.5 { "🟢 GPU 显著领先" } else if ratio > 0.8 { "🟡 CPU/GPU 相当" } else { "🔵 CPU 优势" };

        println!(
            "{:<24} | {:<16} | {:<18} | \x1b[1;32m{:<18.2}x\x1b[0m | {}",
            m_name,
            format!("{} SPS", format_num(cpu_sps)),
            format!("{} SPS", format_num(gpu_sps)),
            ratio,
            recommendation
        );
    }

    println!("\n================================================================================");
    println!("✅ GPU 深度基准压测完成！");
    println!("================================================================================");
    Ok(())
}
