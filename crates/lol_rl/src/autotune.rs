use std::sync::Arc;
use std::time::Instant;

use candle_core::{DType, Device, Result, Tensor};
use lol_env::RlEnvironment;
use lol_rl_protocol::ActionSpace;
use tracing::info;

use crate::algo::agent::RlAgent;
use crate::algo::buffer::RolloutBuffer;
use crate::algo::grpo::{GRPOAgent, GRPOConfig};
use crate::algo::ppo::{PPOAgent, PPOConfig};
use crate::engine::sync::TrainingSession;
use crate::engine::worker::RolloutWorker;

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
    /// 建议 PPO 训练 Epoch 次数
    pub ppo_epochs: usize,
    /// 动态批处理最大等待微秒（Dynamic batching wait timeout）
    pub dynamic_batch_timeout_us: u64,
    /// 预估综合训练步吞吐量 (Steps Per Second, SPS)
    pub estimated_sps: f64,
    /// 动态微调后的学习率
    pub adjusted_lr: f64,
}

/// 动态自适应超参数推导结果 (根据显存、CPU核数、episode长度与并发规模推导)
#[derive(Debug, Clone)]
pub struct DynamicHyperparameters {
    pub num_parallel_envs: usize,
    pub ppo_epochs: usize,
    pub train_batch_size: usize,
    pub infer_batch_size: usize,
    pub num_minibatches: usize,
    pub target_updates_r: usize,
    pub adjusted_lr: f64,
    pub total_iteration_samples: usize,
}

pub struct AutoTuner;

impl AutoTuner {
    /// 依据环境规模、硬件设备与引擎模式，按工业级标准执行 Step 1 ~ Step 8 动态求解
    pub fn compute_dynamic_hyperparameters(
        user_envs: usize,
        horizon: usize,
        agents_per_env: usize,
        engine_mode: lol_rl_protocol::EngineMode,
        user_ppo_epochs: Option<usize>,
        base_lr: f64,
        device: &Device,
    ) -> DynamicHyperparameters {
        // Step 1: n_envs
        let cpu_cores = num_cpus::get();
        let num_parallel_envs = if user_envs > 0 {
            user_envs
        } else if !device.is_cpu() {
            (cpu_cores * 4).clamp(16, 256)
        } else {
            cpu_cores.saturating_sub(2).clamp(4, 32)
        };

        // Step 2: n_steps
        let n_steps = horizon.max(16);

        // Step 3: iteration_batch = n_envs × n_steps (考虑 agents_per_env)
        let iteration_batch = num_parallel_envs * n_steps * agents_per_env.max(1);

        // Step 4: 定 ppo_epochs（同步训练 3~4；异步+staleness 明显时 1~2）
        let ppo_epochs = match user_ppo_epochs {
            Some(e) if e > 0 && e != 8 => e,
            _ => match engine_mode {
                lol_rl_protocol::EngineMode::Async => 2,
                lol_rl_protocol::EngineMode::Sync => 4,
            },
        };

        // Step 5: 目标更新次数 R ∈ [2, 8]
        // num_minibatches = round(R / ppo_epochs)，取 iteration_batch 的因子
        let target_r = 4.0;
        let ideal_minibatches = ((target_r / (ppo_epochs as f64)).round() as usize).clamp(1, 16);

        // 从标准 2 的幂次中选取最适合的 MiniBatch 大小
        let candidate_sizes = [4096, 2048, 1024, 512, 256, 128, 64];
        let mut best_mb_size = 256;
        let mut best_num_mb = (iteration_batch / 256).max(1);
        let mut min_diff = usize::MAX;

        for &sz in &candidate_sizes {
            if sz <= iteration_batch {
                let nb = (iteration_batch / sz).max(1);
                let diff = (nb as isize - ideal_minibatches as isize).unsigned_abs();
                if diff < min_diff {
                    min_diff = diff;
                    best_mb_size = sz;
                    best_num_mb = nb;
                }
            }
        }

        // Step 7: 显存安全检验（超标则减半 minibatch）
        while best_mb_size > 4096 && best_num_mb < 64 {
            best_mb_size /= 2;
            best_num_mb *= 2;
        }

        // Step 8: lr 按 minibatch_size 相对基准（256）做 √ 缩放微调
        let lr_scale = (best_mb_size as f64 / 256.0).sqrt().clamp(0.5, 4.0);
        let adjusted_lr = base_lr * lr_scale;

        let infer_batch_size = num_parallel_envs.next_power_of_two().min(256);

        DynamicHyperparameters {
            num_parallel_envs,
            ppo_epochs,
            train_batch_size: best_mb_size,
            infer_batch_size,
            num_minibatches: best_num_mb,
            target_updates_r: best_num_mb * ppo_epochs,
            adjusted_lr,
            total_iteration_samples: iteration_batch,
        }
    }
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
        Self::profile_with_backbone::<E>(
            state_dim,
            hidden_dim,
            action_space,
            device,
            lol_rl_protocol::PolicyBackbone::Mamba,
        )
    }

    pub fn profile_with_backbone<E: RlEnvironment + 'static>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: &ActionSpace,
        device: &Device,
        backbone_type: lol_rl_protocol::PolicyBackbone,
    ) -> Result<SystemProfile> {
        Self::profile_with_algo_and_backbone::<E>(
            state_dim,
            hidden_dim,
            action_space,
            device,
            lol_rl_protocol::RlAlgorithm::Ppo,
            backbone_type,
        )
    }

    pub fn profile_with_algo_and_backbone<E: RlEnvironment + 'static>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: &ActionSpace,
        device: &Device,
        algorithm: lol_rl_protocol::RlAlgorithm,
        backbone_type: lol_rl_protocol::PolicyBackbone,
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
            "🔍 [AutoTuner] 开始硬件算力探测 (Algorithm: {:?}, Backbone: {:?}, Device: {:?}, CPU逻辑核心: {}, 每环境智能体: {})...",
            algorithm,
            backbone_type,
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

        // 1. 创建真实 Agent（含 AdamW 优化器），探测全程复用同一网络。
        let (cpu_policy, cpu_critic, mut rl_agent) = match algorithm {
            lol_rl_protocol::RlAlgorithm::Grpo => {
                let grpo_config = GRPOConfig {
                    lr: 3e-4,
                    grpo_epochs: 1,
                    ..Default::default()
                };
                let agent = GRPOAgent::create_for_env_with_backbone::<E>(
                    state_dim,
                    hidden_dim,
                    action_space.clone(),
                    grpo_config,
                    device.clone(),
                    backbone_type,
                )?;
                let cpu_policy = Arc::new(agent.policy.to_device(&candle_core::Device::Cpu)?);
                (cpu_policy, None, RlAgent::Grpo(agent))
            }
            lol_rl_protocol::RlAlgorithm::Ppo => {
                let ppo_config = PPOConfig {
                    lr: 3e-4,
                    ppo_epochs: 1,
                    ..Default::default()
                };
                let agent = PPOAgent::create_for_env_with_backbone::<E>(
                    state_dim,
                    hidden_dim,
                    action_space.clone(),
                    ppo_config,
                    device.clone(),
                    backbone_type,
                )?;
                let cpu_policy = Arc::new(
                    agent
                        .actor_critic
                        .policy
                        .to_device(&candle_core::Device::Cpu)?,
                );
                let cpu_critic = Arc::new(
                    agent
                        .actor_critic
                        .critic
                        .to_device(&candle_core::Device::Cpu)?,
                );
                (
                    cpu_policy,
                    Some(cpu_critic),
                    RlAgent::Ppo(agent),
                )
            }
        };

        // 2. 单环境真实单步耗时（含策略推理 + 采样簿记 + env step）
        let mut single_worker = RolloutWorker::<E>::new();
        let _ = single_worker.rollout(
            &cpu_policy,
            cpu_critic.as_deref(),
            None,
            None,
            0,
            warmup_steps,
            state_dim,
            &candle_core::Device::Cpu,
        )?;
        let env_start = Instant::now();
        let _ = single_worker.rollout(
            &cpu_policy,
            cpu_critic.as_deref(),
            None,
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
        }
        candidate_n.sort_unstable();

        let mut parallel_env_us = Vec::with_capacity(candidate_n.len());
        for &n in &candidate_n {
            let barrier = std::sync::Arc::new(std::sync::Barrier::new(n + 1));
            let mut handles = Vec::with_capacity(n);

            for _ in 0..n {
                let b = barrier.clone();
                let policy = cpu_policy.clone();
                let critic = cpu_critic.clone();
                let h = std::thread::spawn(move || {
                    let mut worker = RolloutWorker::<E>::new();
                    b.wait();
                    let _ = worker.rollout(
                        &policy,
                        critic.as_deref(),
                        None,
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
        let policy_ref = rl_agent.policy();
        let infer_batches = [1, 2, 4, 8, 16, 24, 32, 48, 64, 96, 128];
        let mut infer_latency_us = Vec::with_capacity(infer_batches.len());

        for &b in &infer_batches {
            let dummy_input = Tensor::zeros((b, state_dim), DType::F32, device)?;

            for _ in 0..infer_warmup {
                let _ = policy_ref.sample_batch(&dummy_input, None)?;
            }

            let start = Instant::now();
            for _ in 0..infer_iters {
                let _ = policy_ref.sample_batch(&dummy_input, None)?;
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

            match &mut rl_agent {
                RlAgent::Ppo(agent) => {
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
                RlAgent::Grpo(agent) => {
                    for _ in 0..train_warmup {
                        let _ = agent.update_multi_buffer(&buffers, b)?;
                    }

                    let start = Instant::now();
                    for _ in 0..train_iters {
                        let _ = agent.update_multi_buffer(&buffers, b)?;
                    }
                    let lat_us = (start.elapsed().as_micros() as f64) / (train_iters as f64);
                    let throughput = (b as f64) / (lat_us / 1_000_000.0);
                    train_step_us.push((b, lat_us));
                    info!(
                        "    ├─ 训练 Batch {:3}: 耗时 {:7.2} µs | 梯度吞吐: {:8.1} samples/s",
                        b, lat_us, throughput
                    );
                }
            }
        }

        // 6. 每迭代固定开销：GPU→CPU 权重克隆（真实循环每轮执行一次）
        for _ in 0..clone_warmup {
            let _ = rl_agent.policy().to_device(&candle_core::Device::Cpu)?;
        }
        let start = Instant::now();
        for _ in 0..clone_iters {
            let _ = rl_agent.policy().to_device(&candle_core::Device::Cpu)?;
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

    /// 数学规划求解最优参数配置（默认自动寻找最优并发数）。
    ///
    /// 样本口径与 UI 完全一致：`total_samples = N × horizon × agents_per_env`；
    /// Rollout 耗时直接用并发实测每步值（已含 CPU 推理），训练耗时用真实反向更新实测值，
    /// 并计入每迭代固定开销（GPU→CPU 克隆等）。
    pub fn solve(profile: &SystemProfile, horizon: usize, ppo_epochs: usize) -> TunedConfig {
        Self::solve_with_forced_n(profile, horizon, ppo_epochs, None)
    }

    /// 数学规划求解参数配置（支持指定强制固定并行环境数 `forced_n`，或为 `None` 自动求解）。
    pub fn solve_with_forced_n(
        profile: &SystemProfile,
        horizon: usize,
        ppo_epochs: usize,
        forced_n: Option<usize>,
    ) -> TunedConfig {
        let mut best_sps = 0.0;
        let mut best_n = forced_n.unwrap_or(4).max(1);
        let mut best_train_b = 64;
        let mut best_infer_b = best_n.next_power_of_two().min(128);

        // 如果用户指定了强制并行环境数，则仅针对该特定 N 求解最优批处理参数
        let candidate_n_list: Vec<usize> = if let Some(fn_val) = forced_n.filter(|&n| n > 0) {
            vec![fn_val]
        } else if !profile.parallel_env_us.is_empty() {
            profile.parallel_env_us.iter().map(|&(n, _)| n).collect()
        } else {
            // 根据 CPU 核心数和设备情况确定搜索范围
            let max_n = if profile.is_cuda {
                (profile.cpu_cores * 2).clamp(4, 64)
            } else {
                profile.cpu_cores.saturating_sub(2).max(2)
            };
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
                // 训练约束：单个 batch 优先不超过总样本数的 1/2，且至少能做 2 个 mini-batch
                if (b_train as f64) > total_samples / 2.0 && total_samples > (b_train as f64) {
                    continue;
                }

                let num_batches = (total_samples / (b_train as f64)).max(1.0).ceil();
                let train_total_us = (ppo_epochs as f64) * num_batches * train_us;

                let iter_total_us = rollout_time_us + train_total_us + profile.fixed_overhead_us;
                let iter_sec = iter_total_us / 1_000_000.0;
                let sps = total_samples / iter_sec.max(0.0001);

                if sps > best_sps || best_sps == 0.0 {
                    best_sps = sps;
                    best_n = n;
                    best_train_b = b_train.min(total_samples as usize).max(16);
                    best_infer_b = n.next_power_of_two().min(128);
                }
            }

            // 保底 fallback（在样本总数极小导致所有预设 batch 均过大时生效）
            if best_sps == 0.0 {
                best_n = n;
                best_train_b = (total_samples as usize / 2).clamp(16, 64);
                best_infer_b = n.next_power_of_two().min(128);
                best_sps = total_samples
                    / ((rollout_time_us + profile.fixed_overhead_us) / 1_000_000.0).max(0.0001);
            }
        }

        // 动态批处理超时（微秒）：设定为环境真实单步耗时的 15%~30%
        let dynamic_batch_timeout_us = (profile.env_step_us * 0.25).clamp(20.0, 500.0) as u64;

        let lr_scale = (best_train_b as f64 / 256.0).sqrt().clamp(0.5, 4.0);
        let adjusted_lr = 3e-4 * lr_scale;

        let tuned = TunedConfig {
            num_parallel_envs: best_n,
            infer_batch_size: best_infer_b,
            train_batch_size: best_train_b,
            ppo_epochs,
            dynamic_batch_timeout_us,
            estimated_sps: best_sps,
            adjusted_lr,
        };

        if forced_n.is_some() {
            info!("🎯 [AutoTuner] 固定自定义并行环境数 ({best_n}) 求解完成:");
        } else {
            info!("🎯 [AutoTuner] 自适应配置求解完成:");
        }
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
    /// 真实校准：用实际生效配置复用真实 TrainingSession 跑 K 轮完整迭代，
    /// 测量真实的完整迭代墙钟耗时与实测 SPS，覆盖组件级预估。
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
        Self::calibrate_with_backbone::<E>(
            state_dim,
            hidden_dim,
            action_space,
            device,
            horizon,
            ppo_epochs,
            tuned,
            lol_rl_protocol::PolicyBackbone::Mamba,
        )
    }

    pub fn calibrate_with_backbone<E: RlEnvironment + 'static>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: &ActionSpace,
        device: &Device,
        horizon: usize,
        ppo_epochs: usize,
        tuned: &TunedConfig,
        backbone_type: lol_rl_protocol::PolicyBackbone,
    ) -> Result<f64> {
        Self::calibrate_with_algo_and_backbone::<E>(
            state_dim,
            hidden_dim,
            action_space,
            device,
            horizon,
            ppo_epochs,
            tuned,
            lol_rl_protocol::RlAlgorithm::Ppo,
            backbone_type,
        )
    }

    pub fn calibrate_with_algo_and_backbone<E: RlEnvironment + 'static>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: &ActionSpace,
        device: &Device,
        horizon: usize,
        ppo_epochs: usize,
        tuned: &TunedConfig,
        algorithm: lol_rl_protocol::RlAlgorithm,
        backbone_type: lol_rl_protocol::PolicyBackbone,
    ) -> Result<f64> {
        let iters = calibrate_iters();
        if iters == 0 {
            return Ok(tuned.estimated_sps);
        }

        info!(
            "🎯 [AutoTuner] 真实校准 (算法: {:?}, 主干: {:?})：{} 并发 Workers 跑 {iters} 轮完整真实迭代，测量实测 SPS...",
            algorithm, backbone_type, tuned.num_parallel_envs
        );

        let rl_agent: RlAgent = match algorithm {
            lol_rl_protocol::RlAlgorithm::Grpo => {
                let grpo_config = GRPOConfig {
                    lr: 3e-4,
                    gamma: 0.99,
                    clip_eps: 0.2,
                    grpo_epochs: ppo_epochs.max(1),
                    group_size: 4,
                    ..Default::default()
                };
                let agent = GRPOAgent::create_for_env_with_backbone::<E>(
                    state_dim,
                    hidden_dim,
                    action_space.clone(),
                    grpo_config,
                    device.clone(),
                    backbone_type,
                )?;
                RlAgent::Grpo(agent)
            }
            lol_rl_protocol::RlAlgorithm::Ppo => {
                let ppo_config = PPOConfig {
                    lr: 3e-4,
                    gamma: 0.99,
                    gae_lambda: 0.95,
                    clip_eps: 0.2,
                    c1: 0.5,
                    ppo_epochs: ppo_epochs.max(1),
                    clip_vloss: true,
                    max_grad_norm: 0.5,
                };
                let agent = PPOAgent::create_for_env_with_backbone::<E>(
                    state_dim,
                    hidden_dim,
                    action_space.clone(),
                    ppo_config,
                    device.clone(),
                    backbone_type,
                )?;
                RlAgent::Ppo(agent)
            }
        };

        let mut session = TrainingSession::<E>::new(
            rl_agent,
            tuned.num_parallel_envs,
            state_dim,
            horizon,
            candle_core::Device::Cpu,
        );

        let mut sps_sum = 0.0f64;
        for i in 1..=iters {
            let outcome = session.step_once(i, 3e-4, tuned.train_batch_size)?;
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

        // 测试指定强制固定并发数
        let forced_tuned = AutoTuner::solve_with_forced_n(&profile, 64, 4, Some(8));
        assert_eq!(forced_tuned.num_parallel_envs, 8);
        assert!(forced_tuned.train_batch_size >= 32);
        assert!(forced_tuned.estimated_sps > 0.0);

        let small_forced = AutoTuner::solve_with_forced_n(&profile, 64, 4, Some(1));
        assert_eq!(small_forced.num_parallel_envs, 1);
        assert!(small_forced.train_batch_size >= 16);
    }

    #[test]
    fn test_autotuner_grpo_profile_and_calibrate() {
        use lol_env::FioraV2Env;
        let device = Device::Cpu;
        let action_space = FioraV2Env::action_space();
        let state_dim = FioraV2Env::state_dim();

        let profile = AutoTuner::profile_with_algo_and_backbone::<FioraV2Env>(
            state_dim,
            64,
            &action_space,
            &device,
            lol_rl_protocol::RlAlgorithm::Grpo,
            lol_rl_protocol::PolicyBackbone::Mlp,
        )
        .unwrap();
        assert!(profile.env_step_us > 0.0);

        let tuned = AutoTuner::solve(&profile, 32, 1);
        let measured = AutoTuner::calibrate_with_algo_and_backbone::<FioraV2Env>(
            state_dim,
            64,
            &action_space,
            &device,
            32,
            1,
            &tuned,
            lol_rl_protocol::RlAlgorithm::Grpo,
            lol_rl_protocol::PolicyBackbone::Mlp,
        )
        .unwrap();
        assert!(measured > 0.0);
    }

    #[test]
    fn test_compute_dynamic_hyperparameters() {
        let dev = Device::Cpu;
        // 测试 256 环境异步模式下的动态推导
        let dyn_hp = AutoTuner::compute_dynamic_hyperparameters(
            256,
            128,
            2,
            lol_rl_protocol::EngineMode::Async,
            None,
            3e-4,
            &dev,
        );
        assert_eq!(dyn_hp.num_parallel_envs, 256);
        assert_eq!(dyn_hp.total_iteration_samples, 256 * 128 * 2); // 65,536
        assert_eq!(dyn_hp.ppo_epochs, 2);
        assert!(dyn_hp.train_batch_size >= 1024, "MiniBatch 大小应自适应扩大至 >= 1024");
        assert!(dyn_hp.target_updates_r <= 128, "反向传播总次数应被显著削减");
        assert!(dyn_hp.adjusted_lr > 3e-4, "学习率应随 MiniBatch 增大而自适应放大");
    }
}
