use std::time::Instant;

use candle_core::{DType, Device, Result, Tensor};
use candle_nn::{VarBuilder, VarMap};
use lol_env::RlEnvironment;
use lol_rl_protocol::ActionSpace;
use tracing::info;

use crate::policy::ActorCritic;

/// 硬件基准测试结果画像
#[derive(Debug, Clone)]
pub struct SystemProfile {
    pub cpu_cores: usize,
    pub is_cuda: bool,
    /// 单环境单步耗时 (微秒)
    pub env_step_us: f64,
    /// (batch_size, 推理耗时微秒)
    pub infer_latency_us: Vec<(usize, f64)>,
    /// (batch_size, 训练 step 耗时微秒)
    pub train_step_us: Vec<(usize, f64)>,
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
    /// 预估综合吞吐量 (Samples/sec 或 FPS)
    pub estimated_fps: f64,
}

pub struct AutoTuner;

impl AutoTuner {
    /// 执行深度基准探测（耗时约 2.0 ~ 3.0 秒，充分预热并实测多线程争用拐点）
    pub fn profile<E: RlEnvironment + 'static>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: &ActionSpace,
        device: &Device,
    ) -> Result<SystemProfile> {
        let cpu_cores = num_cpus::get();
        let is_cuda = !device.is_cpu();

        info!(
            "🔍 [AutoTuner] 开始高精度硬件算力探测 (Device: {:?}, CPU逻辑核心: {})...",
            if is_cuda { "CUDA / GPU" } else { "CPU" },
            cpu_cores
        );

        // 1. 深度探针：无头游戏环境单步与稳态耗时 (充分预热 200 步，实测 1000 步)
        let mut env = E::new(100);
        let _ = env.reset();
        let act = E::action_from_index(0);
        // 预热：让 CPU 提升调度频率并填满 CPU L1/L2 缓存
        for _ in 0..200 {
            let res = env.step(act.clone());
            if res.terminated || res.truncated {
                let _ = env.reset();
            }
        }
        let env_start = Instant::now();
        let test_steps = 1000;
        for _ in 0..test_steps {
            let res = env.step(act.clone());
            if res.terminated || res.truncated {
                let _ = env.reset();
            }
        }
        let env_step_us = (env_start.elapsed().as_micros() as f64) / (test_steps as f64);
        info!(
            "  ⚡ [1/3] 环境稳态单步耗时: {:.2} µs (单环境稳态上限: {:.0} FPS)",
            env_step_us,
            1_000_000.0 / env_step_us.max(1.0)
        );

        // 创建临时 Policy 用于测试计算性能
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, device);
        let ac = ActorCritic::new(state_dim, hidden_dim, action_space.clone(), vb)?;

        // 2. 深度探针：神经网络批量推理耗时曲线 (更充分采样与稳态计时)
        let infer_batches = [1, 2, 4, 8, 16, 24, 32, 48, 64, 96, 128];
        let mut infer_latency_us = Vec::with_capacity(infer_batches.len());

        let dummy_obs = vec![0.0f32; state_dim];
        for &b in &infer_batches {
            let dummy_input = Tensor::zeros((b, state_dim), DType::F32, device)?;
            let obs_refs: Vec<&[f32]> = vec![dummy_obs.as_slice(); b];

            // 预热 GPU / SIMD kernel
            for _ in 0..30 {
                let _ = ac.sample_batch(&dummy_input, &obs_refs)?;
            }

            let start = Instant::now();
            let iters = 200;
            for _ in 0..iters {
                let _ = ac.sample_batch(&dummy_input, &obs_refs)?;
            }
            let lat_us = (start.elapsed().as_micros() as f64) / (iters as f64);
            let throughput = (b as f64) / (lat_us / 1_000_000.0);
            infer_latency_us.push((b, lat_us));
            info!(
                "    ├─ 推理 Batch {:3}: 延迟 {:7.2} µs | 单次吞吐: {:8.1} samples/s",
                b, lat_us, throughput
            );
        }

        // 3. 深度探针：神经网络训练 Mini-Batch 评估与反向传播耗时 (更充分样本)
        let train_batches = [32, 64, 128, 256, 512];
        let mut train_step_us = Vec::with_capacity(train_batches.len());
        let enc_dim = action_space.encoding_dim();

        info!("  ⚡ [2/3] 训练 Mini-Batch 梯度计算耗时探测:");
        for &b in &train_batches {
            let dummy_states = Tensor::zeros((b, state_dim), DType::F32, device)?;
            let dummy_actions = Tensor::zeros((b, enc_dim), DType::F32, device)?;

            // 预热
            for _ in 0..10 {
                let _ = ac.evaluate_actions(&dummy_states, &dummy_actions)?;
            }

            let start = Instant::now();
            let iters = 50;
            for _ in 0..iters {
                let _ = ac.evaluate_actions(&dummy_states, &dummy_actions)?;
            }
            let lat_us = (start.elapsed().as_micros() as f64) / (iters as f64);
            let throughput = (b as f64) / (lat_us / 1_000_000.0);
            train_step_us.push((b, lat_us));
            info!(
                "    ├─ 训练 Batch {:3}: 耗时 {:7.2} µs | 梯度吞吐: {:8.1} samples/s",
                b, lat_us, throughput
            );
        }

        info!("  ⚡ [3/3] 硬件基准测试完成.");
        Ok(SystemProfile {
            cpu_cores,
            is_cuda,
            env_step_us,
            infer_latency_us,
            train_step_us,
        })
    }

    /// 数学规划求解最优参数配置
    pub fn solve(profile: &SystemProfile, horizon: usize, ppo_epochs: usize) -> TunedConfig {
        let mut best_fps = 0.0;
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

        let candidate_n_list: Vec<usize> = (2..=max_n)
            .filter(|&n| n % 2 == 0 || n == profile.cpu_cores)
            .collect();

        for &n in &candidate_n_list {
            // 插值或匹配最接近的推理延迟
            let infer_lat_us = profile
                .infer_latency_us
                .iter()
                .find(|&&(b, _)| b >= n)
                .map(|&(_, lat)| lat)
                .unwrap_or_else(|| {
                    profile
                        .infer_latency_us
                        .last()
                        .map(|&(_, lat)| lat)
                        .unwrap_or(500.0)
                });

            // 多线程环境 Step 耗时（考虑核心饱和度）
            let parallel_scale = if n <= profile.cpu_cores {
                1.05 // 线程切换开销极小
            } else {
                1.0 + (n as f64 - profile.cpu_cores as f64) / (profile.cpu_cores as f64) * 0.8
            };
            let env_batch_us = profile.env_step_us * parallel_scale;

            let rollout_time_us = (env_batch_us + infer_lat_us) * (horizon as f64);
            let total_samples = n * horizon;

            for &(b_train, train_us) in &profile.train_step_us {
                // 训练约束：单个 batch 不能超过总样本数的 1/2，且至少能做 2 个 mini-batch
                if b_train > total_samples / 2 {
                    continue;
                }

                let num_batches = ((total_samples as f64) / (b_train as f64)).ceil();
                let train_total_us = (ppo_epochs as f64) * num_batches * train_us;

                let iter_total_us = rollout_time_us + train_total_us;
                let iter_sec = iter_total_us / 1_000_000.0;
                let fps = (total_samples as f64) / iter_sec.max(0.0001);

                if fps > best_fps {
                    best_fps = fps;
                    best_n = n;
                    best_train_b = b_train;
                    best_infer_b = n.next_power_of_two().min(128);
                }
            }
        }

        // 动态批处理超时（微秒）：设定为环境单步耗时的 15%~30%，兼顾极速反馈与组批收益
        let dynamic_batch_timeout_us = (profile.env_step_us * 0.25).clamp(20.0, 500.0) as u64;

        let tuned = TunedConfig {
            num_parallel_envs: best_n,
            infer_batch_size: best_infer_b,
            train_batch_size: best_train_b,
            dynamic_batch_timeout_us,
            estimated_fps: best_fps,
        };

        info!("🎯 [AutoTuner] 自适应配置求解完成:");
        info!("  ├─ 并行环境数 (Actors N): {}", tuned.num_parallel_envs);
        info!("  ├─ 推理 Batch 大小: {}", tuned.infer_batch_size);
        info!("  ├─ 训练 Mini-Batch: {}", tuned.train_batch_size);
        info!(
            "  ├─ 动态批聚合超时: {} µs",
            tuned.dynamic_batch_timeout_us
        );
        info!("  └─ 预估训练吞吐量: {:.1} Steps/s (FPS)", tuned.estimated_fps);

        tuned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autotuner_solve() {
        let profile = SystemProfile {
            cpu_cores: 16,
            is_cuda: true,
            env_step_us: 50.0,
            infer_latency_us: vec![(1, 100.0), (4, 120.0), (16, 180.0), (32, 250.0), (64, 400.0)],
            train_step_us: vec![(32, 800.0), (64, 900.0), (128, 1200.0), (256, 1800.0)],
        };

        let tuned = AutoTuner::solve(&profile, 64, 4);
        assert!(tuned.num_parallel_envs >= 2);
        assert!(tuned.train_batch_size >= 32);
        assert!(tuned.estimated_fps > 0.0);
    }
}

