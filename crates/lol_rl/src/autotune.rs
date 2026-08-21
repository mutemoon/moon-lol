use std::sync::Arc;
use std::time::Instant;

use candle_core::{DType, Device, Result, Tensor};
use lol_env::RlEnvironment;
use lol_rl_protocol::ActionSpace;
use tracing::info;

use crate::ppo::{PPOAgent, PPOConfig, RolloutBuffer};
use crate::rollout::RolloutWorker;

/// 硬件基准测试结果画像
#[derive(Debug, Clone)]
pub struct SystemProfile {
    pub cpu_cores: usize,
    pub is_cuda: bool,
    /// 环境智能体数量（自博弈时每 env 每步产出 num_agents 个样本，与 UI SPS 同口径）。
    pub agents_per_env: usize,
    /// 单环境真实单步耗时 (微秒)，含策略推理 + 采样簿记 + env step。
    pub env_step_us: f64,
    /// (并发环境数 N, 多实例并发真实 1 步耗时微秒)，含 CPU 推理 + 采样簿记 + env。
    pub parallel_env_us: Vec<(usize, f64)>,
    /// (batch_size, 推理耗时微秒) —— GPU 批量前向参考曲线（动态批推理引擎用）。
    pub infer_latency_us: Vec<(usize, f64)>,
    /// (batch_size, 真实训练 update 耗时微秒) —— 含反向传播 + AdamW 优化器 + 设备同步。
    pub train_step_us: Vec<(usize, f64)>,
    /// 每迭代固定开销 (微秒)：GPU→CPU 权重克隆 + 熵/LR 调度与轨迹聚合簿记。
    pub fixed_overhead_us: f64,
}

/// 求解出的最优运行参数
#[derive(Debug, Clone)]
pub struct TunedConfig {
    /// 建议并行无头游戏环境数量
    pub num_parallel_envs: usize,
    /// 建议推理批大小（用于 Dynamic Batching 上限）
    pub infer_batch_size: usize,
    /// 建议 PPO 训练 Mini-Batch 大小
    pub train_batch_size: usize,
    /// 动态批处理最大等待微秒（Dynamic batching wait timeout）
    pub dynamic_batch_timeout_us: u64,
    /// 预估综合训练步吞吐量 (Steps Per Second, SPS)
    pub estimated_sps: f64,
}

pub struct AutoTuner;

impl AutoTuner {
    /// 执行深度基准探测。
    ///
    /// 与旧版不同，所有测量都驱动**真实训练组件**：
    /// - 单环境 / 并发压测走 [`RolloutWorker`]（含 CPU 推理 + 采样簿记 + env step）；
    /// - 训练探测调真实 [`PPOAgent::update_multi_buffer`]（含反向 + AdamW + 设备同步）；
    /// - 额外测每迭代固定开销（GPU→CPU 权重克隆）。
    pub fn profile<E: RlEnvironment + 'static>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: &ActionSpace,
        device: &Device,
    ) -> Result<SystemProfile> {
        let cpu_cores = num_cpus::get();
        let is_cuda = !device.is_cpu();
        let agents_per_env = E::num_agents().max(1);
        let enc_dim = action_space.encoding_dim();
        let mask_dim = match action_space {
            ActionSpace::Discrete(n) => *n,
            ActionSpace::Continuous(_) => 0,
            ActionSpace::Hybrid {
                discrete_classes, ..
            } => *discrete_classes,
        };

        info!(
            "🔍 [AutoTuner] 开始硬件算力探测 (Device: {:?}, CPU逻辑核心: {}, 每环境智能体: {})...",
            if is_cuda { "CUDA / GPU" } else { "CPU" },
            cpu_cores,
            agents_per_env
        );

        let warmup_steps = 20;
        let single_steps = 100;
        let steps_per_env = 100;
        let infer_warmup = 30;
        let infer_iters = 200;
        let train_warmup = 5;
        let train_iters = 20;
        let clone_warmup = 5;
        let clone_iters = 20;

        // 1. 创建真实 PPOAgent（含 AdamW 优化器），探测全程复用同一网络。
        //    训练探测只测单 epoch 成本（ppo_epochs=1），solve 按任务配置乘回。
        let ppo_config = PPOConfig {
            lr: 3e-4,
            ppo_epochs: 1,
            ..Default::default()
        };
        let mut agent = PPOAgent::create_for_env::<E>(
            state_dim,
            hidden_dim,
            action_space.clone(),
            ppo_config,
            device.clone(),
        )?;
        let cpu_policy = Arc::new(agent.actor_critic.to_device(&candle_core::Device::Cpu)?);

        // 2. 单环境真实单步耗时（含策略推理 + 采样簿记 + env step）
        let mut single_worker = RolloutWorker::<E>::new();
        let _ = single_worker.rollout(
            &cpu_policy,
            None,
            0,
            warmup_steps,
            state_dim,
            &candle_core::Device::Cpu,
        )?;
        let env_start = Instant::now();
        let _ = single_worker.rollout(
            &cpu_policy,
            None,
            0,
            single_steps,
            state_dim,
            &candle_core::Device::Cpu,
        )?;
        let env_step_us = env_start.elapsed().as_micros() as f64 / (single_steps as f64);

        // 3. 真实多环境实例并发压测（每个实例跑真实策略 Rollout，含 CPU 推理）
        info!("  ⚡ [1/3] 真实多环境实例并发压测 (真实策略 Rollout，含 CPU 推理 + 采样簿记):");
        let mut candidate_n = Vec::new();
        for &n in &[2, 4, 8, 12, 16, 20, 24, 28, 32] {
            if n <= cpu_cores * 2 {
                candidate_n.push(n);
            }
        }
        if !candidate_n.contains(&cpu_cores) && cpu_cores <= 64 {
            candidate_n.push(cpu_cores);
            candidate_n.sort_unstable();
        }

        let mut parallel_env_us = Vec::with_capacity(candidate_n.len());
        for &n in &candidate_n {
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(n + 1));
            let mut handles = Vec::with_capacity(n);

            for _ in 0..n {
                let b = barrier.clone();
                let policy = cpu_policy.clone();
                let h = std::thread::spawn(move || {
                    let mut worker = RolloutWorker::<E>::new();
                    b.wait();
                    let _ = worker.rollout(
                        &policy,
                        None,
                        0,
                        steps_per_env,
                        state_dim,
                        &candle_core::Device::Cpu,
                    );
                });
                handles.push(h);
            }

            barrier.wait();
            let start = Instant::now();
            for h in handles {
                let _ = h.join();
            }
            let total_dur_us = start.elapsed().as_micros() as f64;
            let step_batch_us = total_dur_us / (steps_per_env as f64);
            let real_sps =
                ((n * steps_per_env * agents_per_env) as f64) / (total_dur_us / 1_000_000.0);
            parallel_env_us.push((n, step_batch_us));

            info!(
                "    ├─ 并发实例 {:2}: 并发步耗时 {:7.2} µs | 真实吞吐: {:8.1} SPS",
                n, step_batch_us, real_sps
            );
        }

        // 4. GPU 批量推理参考曲线（动态批推理引擎用，训练循环本身不依赖）
        let ac = &agent.actor_critic;
        let infer_batches = [1, 2, 4, 8, 16, 24, 32, 48, 64, 96, 128];
        let mut infer_latency_us = Vec::with_capacity(infer_batches.len());

        for &b in &infer_batches {
            let dummy_input = Tensor::zeros((b, state_dim), DType::F32, device)?;

            for _ in 0..infer_warmup {
                let _ = ac.sample_batch(&dummy_input, None)?;
            }

            let start = Instant::now();
            for _ in 0..infer_iters {
                let _ = ac.sample_batch(&dummy_input, None)?;
            }
            let lat_us = (start.elapsed().as_micros() as f64) / (infer_iters as f64);
            let throughput = (b as f64) / (lat_us / 1_000_000.0);
            infer_latency_us.push((b, lat_us));
            info!(
                "    ├─ 推理 Batch {:3}: 延迟 {:7.2} µs | 单次吞吐: {:8.1} samples/s",
                b, lat_us, throughput
            );
        }

        // 5. 真实训练 Mini-Batch 探测（update_multi_buffer 含反向 + AdamW + 设备同步）
        info!("  ⚡ [2/3] 真实训练 Mini-Batch 探测 (update_multi_buffer 含反向+优化器+设备同步):");
        let train_batches = [32, 64, 128, 256, 512];
        let mut train_step_us = Vec::with_capacity(train_batches.len());
        for &b in &train_batches {
            let buffers = synthetic_buffers(state_dim, enc_dim, mask_dim, b, agents_per_env);
            let last_vals = vec![0.0f32; buffers.len()];

            for _ in 0..train_warmup {
                let _ = agent.update_multi_buffer(&buffers, &last_vals, b)?;
            }

            let start = Instant::now();
            for _ in 0..train_iters {
                let _ = agent.update_multi_buffer(&buffers, &last_vals, b)?;
            }
            let lat_us = (start.elapsed().as_micros() as f64) / (train_iters as f64);
            let throughput = (b as f64) / (lat_us / 1_000_000.0);
            train_step_us.push((b, lat_us));
            info!(
                "    ├─ 训练 Batch {:3}: 耗时 {:7.2} µs | 梯度吞吐: {:8.1} samples/s",
                b, lat_us, throughput
            );
        }

        // 6. 每迭代固定开销：GPU→CPU 权重克隆（真实循环每轮执行一次）
        for _ in 0..clone_warmup {
            let _ = agent.actor_critic.to_device(&candle_core::Device::Cpu)?;
        }
        let start = Instant::now();
        for _ in 0..clone_iters {
            let _ = agent.actor_critic.to_device(&candle_core::Device::Cpu)?;
        }
        let clone_us = start.elapsed().as_micros() as f64 / (clone_iters as f64);
        // 300µs 预留给熵/LR 调度与轨迹聚合簿记
        let fixed_overhead_us = clone_us + 300.0;

        info!("  ⚡ [3/3] 硬件基准测试完成. GPU→CPU 权重克隆: {clone_us:.1} µs/次");

        Ok(SystemProfile {
            cpu_cores,
            is_cuda,
            agents_per_env,
            env_step_us,
            parallel_env_us,
            infer_latency_us,
            train_step_us,
            fixed_overhead_us,
        })
    }

    /// 数学规划求解最优参数配置。
    ///
    /// 样本口径与 UI 完全一致：`total_samples = N × horizon × agents_per_env`；
    /// Rollout 耗时直接用并发实测每步值（已含 CPU 推理），训练耗时用真实反向更新实测值，
    /// 并计入每迭代固定开销（GPU→CPU 克隆等）。
    pub fn solve(profile: &SystemProfile, horizon: usize, ppo_epochs: usize) -> TunedConfig {
        let mut best_sps = 0.0;
        let mut best_n = 4;
        let mut best_train_b = 64;
        let mut best_infer_b = 4;

        // 根据 CPU 核心数和设备情况确定搜索范围
        let max_n = if profile.is_cuda {
            // CUDA 模式下，GPU 批量吞吐高，可充分压榨 CPU 核心 (例如 1.5x ~ 2x 核心数)
            (profile.cpu_cores * 2).clamp(4, 64)
        } else {
            // CPU 模式下，留出 2 个核心给训练与推理
            profile.cpu_cores.saturating_sub(2).max(2)
        };

        let candidate_n_list: Vec<usize> = if !profile.parallel_env_us.is_empty() {
            profile.parallel_env_us.iter().map(|&(n, _)| n).collect()
        } else {
            (2..=max_n)
                .filter(|&n| n % 2 == 0 || n == profile.cpu_cores)
                .collect()
        };

        let samples_per_step = profile.agents_per_env.max(1) as f64;

        for &n in &candidate_n_list {
            // 真实并发 Rollout 每步耗时（已含 CPU 推理 + 采样簿记 + env），直接用实测值
            let env_batch_us = profile
                .parallel_env_us
                .iter()
                .find(|&&(cand_n, _)| cand_n == n)
                .map(|&(_, us)| us)
                .unwrap_or_else(|| {
                    if n <= profile.cpu_cores {
                        profile.env_step_us * 1.05
                    } else {
                        profile.env_step_us
                            * (1.0
                                + (n as f64 - profile.cpu_cores as f64)
                                    / (profile.cpu_cores as f64)
                                    * 0.8)
                    }
                });

            let rollout_time_us = env_batch_us * (horizon as f64);
            let total_samples = (n as f64) * (horizon as f64) * samples_per_step;

            for &(b_train, train_us) in &profile.train_step_us {
                // 训练约束：单个 batch 不能超过总样本数的 1/2，且至少能做 2 个 mini-batch
                if (b_train as f64) > total_samples / 2.0 {
                    continue;
                }

                let num_batches = (total_samples / (b_train as f64)).ceil();
                let train_total_us = (ppo_epochs as f64) * num_batches * train_us;

                let iter_total_us = rollout_time_us + train_total_us + profile.fixed_overhead_us;
                let iter_sec = iter_total_us / 1_000_000.0;
                let sps = total_samples / iter_sec.max(0.0001);

                if sps > best_sps {
                    best_sps = sps;
                    best_n = n;
                    best_train_b = b_train;
                    best_infer_b = n.next_power_of_two().min(128);
                }
            }
        }

        // 动态批处理超时（微秒）：设定为环境真实单步耗时的 15%~30%
        let dynamic_batch_timeout_us = (profile.env_step_us * 0.25).clamp(20.0, 500.0) as u64;

        let tuned = TunedConfig {
            num_parallel_envs: best_n,
            infer_batch_size: best_infer_b,
            train_batch_size: best_train_b,
            dynamic_batch_timeout_us,
            estimated_sps: best_sps,
        };

        info!("🎯 [AutoTuner] 自适应配置求解完成:");
        info!("  ├─ 并行环境数 (Actors N): {}", tuned.num_parallel_envs);
        info!("  ├─ 推理 Batch 大小: {}", tuned.infer_batch_size);
        info!("  ├─ 训练 Mini-Batch: {}", tuned.train_batch_size);
        info!("  ├─ 动态批聚合超时: {} µs", tuned.dynamic_batch_timeout_us);
        info!(
            "  └─ 预估训练吞吐量: {:.1} SPS (Steps/s)",
            tuned.estimated_sps
        );

        tuned
    }

    /// 真实校准：用候选配置复用真实 [`TrainingSession`]（机制 A）跑 K 轮完整迭代，
    /// 测出与 UI `fps` 完全同口径的真实 SPS（同步 CPU 推理 + PPO 反向更新）。
    ///
    /// 迭代轮数由 `MOON_LOL_CALIBRATE_ITERS` 控制（默认 2），设 0 或 `MOON_LOL_NO_CALIBRATE=1` 跳过。
    pub fn calibrate<E: RlEnvironment + 'static>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: &ActionSpace,
        device: &Device,
        horizon: usize,
        ppo_epochs: usize,
        tuned: &TunedConfig,
    ) -> Result<f64> {
        let iters = calibrate_iters();
        if iters == 0 {
            return Ok(tuned.estimated_sps);
        }

        info!(
            "🎯 [AutoTuner] 真实校准：{} 并发 Workers 跑 {iters} 轮完整真实迭代，测量实测 SPS...",
            tuned.num_parallel_envs
        );
        let ppo_config = PPOConfig {
            lr: 3e-4,
            gamma: 0.99,
            gae_lambda: 0.95,
            clip_eps: 0.2,
            c1: 0.5,
            c2: 0.05,
            ppo_epochs: ppo_epochs.max(1),
            clip_vloss: true,
            max_grad_norm: 0.5,
        };
        let agent = PPOAgent::create_for_env::<E>(
            state_dim,
            hidden_dim,
            action_space.clone(),
            ppo_config,
            device.clone(),
        )?;
        let mut session = crate::training::TrainingSession::<E>::new(
            agent,
            tuned.num_parallel_envs,
            state_dim,
            horizon,
            candle_core::Device::Cpu,
        );

        let mut sps_sum = 0.0f64;
        for i in 1..=iters {
            let outcome = session.step_once(i, 3e-4, 0.05, tuned.train_batch_size)?;
            sps_sum += outcome.sps;
            info!(
                "    └─ 校准 Iter {i}: 真实 SPS {:8.1} | samples {}",
                outcome.sps, outcome.num_samples
            );
        }
        session.stop();

        let measured = sps_sum / (iters as f64);
        info!("🎯 [AutoTuner] 校准完成：实测 SPS {measured:.1}");
        Ok(measured)
    }
}

fn calibrate_iters() -> usize {
    if std::env::var("MOON_LOL_NO_CALIBRATE").is_ok() {
        return 0;
    }
    std::env::var("MOON_LOL_CALIBRATE_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2)
}

/// 构建 n 个正确维度的合成 RolloutBuffer（探测训练真实成本用）。
fn synthetic_buffers(
    state_dim: usize,
    enc_dim: usize,
    mask_dim: usize,
    total: usize,
    num_buffers: usize,
) -> Vec<RolloutBuffer> {
    let nb = num_buffers.max(1);
    let per = total / nb;
    let rem = total % nb;
    let mut out = Vec::with_capacity(nb);
    for i in 0..nb {
        let len = per + if i < rem { 1 } else { 0 };
        out.push(synthetic_buffer(state_dim, enc_dim, mask_dim, len));
    }
    out
}

/// 单个合成 buffer：随机状态 / 合法动作编码 / 全有效掩码。
fn synthetic_buffer(
    state_dim: usize,
    enc_dim: usize,
    mask_dim: usize,
    len: usize,
) -> RolloutBuffer {
    let mut b = RolloutBuffer::new();
    for i in 0..len {
        let state: Vec<f32> = (0..state_dim)
            .map(|j| (j as f32 * 0.05).sin() + 0.001 * (i % 5) as f32)
            .collect();
        let mut action: Vec<f32> = (0..enc_dim).map(|j| (j as f32 * 0.1).cos()).collect();
        // 离散维（纯离散唯一维或混合末位）填合法索引 0
        action[enc_dim - 1] = 0.0;
        let mask = if mask_dim > 0 {
            Some(vec![true; mask_dim])
        } else {
            None
        };
        b.push_full(state, action, -0.5, 0.1, 0.0, false, false, None, mask);
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autotuner_solve() {
        let profile = SystemProfile {
            cpu_cores: 16,
            is_cuda: true,
            agents_per_env: 2,
            env_step_us: 300.0,
            parallel_env_us: vec![
                (2, 320.0),
                (4, 340.0),
                (8, 400.0),
                (16, 700.0),
                (32, 1500.0),
            ],
            infer_latency_us: vec![
                (1, 100.0),
                (4, 120.0),
                (16, 180.0),
                (32, 250.0),
                (64, 400.0),
            ],
            train_step_us: vec![(32, 800.0), (64, 900.0), (128, 1200.0), (256, 1800.0)],
            fixed_overhead_us: 1500.0,
        };

        let tuned = AutoTuner::solve(&profile, 64, 4);
        assert!(tuned.num_parallel_envs >= 2);
        assert!(tuned.train_batch_size >= 32);
        assert!(tuned.estimated_sps > 0.0);
    }
}
