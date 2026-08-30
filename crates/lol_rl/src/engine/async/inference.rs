use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use candle_core::{Device, Result, Tensor};
use crossbeam_channel::{Sender, unbounded};
use lol_rl_protocol::ActionMasks;
use tracing::{debug, error, info};

use crate::algo::agent::RlAgent;
use crate::engine::traits::InferenceTimingStats;
use crate::policy::{ActorCritic, PolicyNetwork, ValueHead};

/// 供推理引擎使用的策略快照（包含 Actor 策略网络与可选的 Critic 价值头）
#[derive(Clone)]
pub struct PolicySnapshot {
    pub policy: PolicyNetwork,
    pub critic: Option<ValueHead>,
    pub version: usize,
}

impl PolicySnapshot {
    pub fn new(policy: PolicyNetwork, critic: Option<ValueHead>) -> Self {
        Self {
            policy,
            critic,
            version: 0,
        }
    }

    pub fn with_version(policy: PolicyNetwork, critic: Option<ValueHead>, version: usize) -> Self {
        Self {
            policy,
            critic,
            version,
        }
    }

    pub fn from_agent(agent: &RlAgent, device: &Device) -> Result<Self> {
        let policy = agent.policy().to_device(device)?;
        let critic = agent.critic().map(|c| c.to_device(device)).transpose()?;
        Ok(Self {
            policy,
            critic,
            version: 0,
        })
    }

    pub fn to_device(&self, device: &Device) -> Result<Self> {
        let policy = self.policy.to_device(device)?;
        let critic = self
            .critic
            .as_ref()
            .map(|c| c.to_device(device))
            .transpose()?;
        Ok(Self {
            policy,
            critic,
            version: self.version,
        })
    }

    pub fn sample_batch(
        &self,
        states: &Tensor,
        masks: Option<&[Option<Vec<bool>>]>,
    ) -> Result<Vec<(Vec<f32>, f32, f32)>> {
        self.sample_batch_with_structured_masks(states, None, masks)
    }

    pub fn sample_batch_with_structured_masks(
        &self,
        states: &Tensor,
        structured_masks: Option<&[Option<ActionMasks>]>,
        masks: Option<&[Option<Vec<bool>>]>,
    ) -> Result<Vec<(Vec<f32>, f32, f32)>> {
        let act_lps = self
            .policy
            .sample_batch_with_structured_masks(states, structured_masks, masks)?;
        let values = if let Some(ref critic) = self.critic {
            let feat = self.policy.hidden(states)?;
            let v = critic.forward(&feat)?;
            v.squeeze(1)?.to_vec1()?
        } else {
            vec![0.0; act_lps.len()]
        };
        let mut res = Vec::with_capacity(act_lps.len());
        for (i, (act, lp)) in act_lps.into_iter().enumerate() {
            res.push((act, lp, values[i]));
        }
        Ok(res)
    }
}

impl From<ActorCritic> for PolicySnapshot {
    fn from(ac: ActorCritic) -> Self {
        Self {
            policy: ac.policy,
            critic: Some(ac.critic),
            version: 0,
        }
    }
}

pub struct InferenceRequest {
    pub worker_id: usize,
    pub obs_vec: Vec<f32>,
    pub action_mask: Option<Vec<bool>>,
    pub structured_mask: Option<ActionMasks>,
    /// 策略槽位：0 = 当前训练权重，>=1 = 历史对手快照（由 opponents 表提供）。
    pub policy_slot: usize,
    pub reply_tx: Sender<InferenceResponse>,
}

#[derive(Debug, Clone)]
pub struct InferenceResponse {
    pub encoded_action: Vec<f32>,
    pub log_prob: f32,
    pub value: f32,
    pub policy_version: usize,
}

#[derive(Debug, Default)]
pub struct InferenceMetrics {
    pub total_requests: AtomicUsize,
    pub total_batches: AtomicUsize,
    pub forward_micros: AtomicU64,
    pub wait_micros: AtomicU64,
}

pub struct InferenceServer {
    pub req_tx: Sender<InferenceRequest>,
    /// 更新策略槽位：(slot, 新权重)。slot 0 为当前主策略，slot>0 为历史对手。
    pub model_tx: Sender<(usize, PolicySnapshot)>,
    pub metrics: Arc<InferenceMetrics>,
    is_running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl InferenceServer {
    pub fn new(
        initial_model: PolicySnapshot,
        state_dim: usize,
        max_batch_size: usize,
        timeout_us: u64,
        device: Device,
    ) -> Self {
        let (req_tx, req_rx) = unbounded::<InferenceRequest>();
        let (model_tx, model_rx) = unbounded::<(usize, PolicySnapshot)>();
        let is_running = Arc::new(AtomicBool::new(true));
        let running_clone = is_running.clone();
        let metrics = Arc::new(InferenceMetrics::default());
        let metrics_clone = metrics.clone();

        let handle = thread::spawn(move || {
            let mut current_model = initial_model;
            let mut opponents: HashMap<usize, PolicySnapshot> = HashMap::new();
            let timeout = Duration::from_micros(timeout_us);
            let mut batch_reqs = Vec::with_capacity(max_batch_size);

            info!(
                "🚀 [InferenceServer] 启动 CUDA/CPU 动态批处理推理引擎 (MaxBatch: {}, Timeout: {}µs)",
                max_batch_size, timeout_us
            );

            while running_clone.load(Ordering::Relaxed) {
                // 1. 检查是否有策略权重更新（slot 0 主策略 / slot>0 历史对手）
                while let Ok((slot, new_model)) = model_rx.try_recv() {
                    if slot == 0 {
                        current_model = new_model;
                        debug!("🔄 [InferenceServer] 已同步最新模型权重 (slot 0)");
                    } else {
                        opponents.insert(slot, new_model);
                        debug!("🔄 [InferenceServer] 已注册历史对手 (slot {slot})");
                    }
                }

                // 2. 阻塞等待第一个请求
                let first_req = match req_rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(req) => req,
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                };

                batch_reqs.clear();
                batch_reqs.push(first_req);

                // 3. Dynamic Batching：在超时窗口或达到 max_batch_size 前收集更多请求
                let start_batch_wait = Instant::now();
                while batch_reqs.len() < max_batch_size {
                    if start_batch_wait.elapsed() >= timeout {
                        break;
                    }
                    match req_rx.try_recv() {
                        Ok(req) => batch_reqs.push(req),
                        Err(crossbeam_channel::TryRecvError::Empty) => {
                            std::hint::spin_loop();
                        }
                        Err(crossbeam_channel::TryRecvError::Disconnected) => break,
                    }
                }
                let wait_time = start_batch_wait.elapsed();

                let batch_len = batch_reqs.len();
                if batch_len == 0 {
                    continue;
                }

                metrics_clone
                    .wait_micros
                    .fetch_add(wait_time.as_micros() as u64, Ordering::Relaxed);
                metrics_clone
                    .total_requests
                    .fetch_add(batch_len, Ordering::Relaxed);
                metrics_clone.total_batches.fetch_add(1, Ordering::Relaxed);

                let forward_start = Instant::now();

                // 4. 按策略槽位分组：同一槽位的一批请求用同一策略一次前向
                let mut by_slot: HashMap<usize, Vec<usize>> = HashMap::new();
                for (i, req) in batch_reqs.iter().enumerate() {
                    by_slot.entry(req.policy_slot).or_default().push(i);
                }

                let taken: Vec<_> = batch_reqs.drain(..).collect();
                let mut results: Vec<Option<InferenceResponse>> = vec![None; taken.len()];
                let mut any_error = false;

                for (slot, idxs) in &by_slot {
                    let model = if *slot == 0 {
                        &current_model
                    } else {
                        match opponents.get(slot) {
                            Some(p) => p,
                            None => {
                                error!("策略槽位 {slot} 未注册，退化为当前主策略");
                                &current_model
                            }
                        }
                    };

                    let mut flat_states = Vec::with_capacity(idxs.len() * state_dim);
                    let mut masks = Vec::with_capacity(idxs.len());
                    let mut structured_masks = Vec::with_capacity(idxs.len());
                    for &i in idxs {
                        flat_states.extend_from_slice(&taken[i].obs_vec);
                        masks.push(taken[i].action_mask.clone());
                        structured_masks.push(taken[i].structured_mask.clone());
                    }

                    let state_tensor =
                        match Tensor::from_vec(flat_states, (idxs.len(), state_dim), &device) {
                            Ok(t) => t,
                            Err(e) => {
                                error!("创建推理 Tensor 失败: {e}");
                                any_error = true;
                                continue;
                            }
                        };

                    let has_any_mask = masks.iter().any(|m| m.is_some());
                    let masks_ref = if has_any_mask {
                        Some(masks.as_slice())
                    } else {
                        None
                    };
                    let has_any_struct_mask = structured_masks.iter().any(|m| m.is_some());
                    let struct_masks_ref = if has_any_struct_mask {
                        Some(structured_masks.as_slice())
                    } else {
                        None
                    };

                    match model.sample_batch_with_structured_masks(&state_tensor, struct_masks_ref, masks_ref) {
                        Ok(samples) => {
                            for (&i, (encoded_action, log_prob, value)) in
                                idxs.iter().zip(samples.into_iter())
                            {
                                results[i] = Some(InferenceResponse {
                                    encoded_action,
                                    log_prob,
                                    value,
                                    policy_version: model.version,
                                });
                            }
                        }
                        Err(e) => {
                            error!("批量推理失败: {e}");
                            any_error = true;
                        }
                    }
                }

                if any_error {
                    continue;
                }

                let forward_time = forward_start.elapsed();
                metrics_clone
                    .forward_micros
                    .fetch_add(forward_time.as_micros() as u64, Ordering::Relaxed);

                // 5. 回填响应
                for (req, res) in taken.into_iter().zip(results.into_iter()) {
                    if let Some(res) = res {
                        let _ = req.reply_tx.send(res);
                    }
                }
            }

            info!("🛑 [InferenceServer] 推理引擎已停止.");
        });

        Self {
            req_tx,
            model_tx,
            metrics,
            is_running,
            handle: Some(handle),
        }
    }

    /// 原子读取并重置推理服务器的单轮统计指标
    pub fn take_timing_stats(&self) -> InferenceTimingStats {
        let reqs = self.metrics.total_requests.swap(0, Ordering::Relaxed);
        let batches = self.metrics.total_batches.swap(0, Ordering::Relaxed);
        let fwd_us = self.metrics.forward_micros.swap(0, Ordering::Relaxed);
        let wait_us = self.metrics.wait_micros.swap(0, Ordering::Relaxed);
        let avg_batch_size = if batches > 0 {
            reqs as f64 / batches as f64
        } else {
            0.0
        };
        InferenceTimingStats {
            batch_count: batches,
            request_count: reqs,
            avg_batch_size,
            forward_ms: fwd_us as f64 / 1000.0,
            wait_ms: wait_us as f64 / 1000.0,
        }
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for InferenceServer {
    fn drop(&mut self) {
        self.stop();
    }
}
