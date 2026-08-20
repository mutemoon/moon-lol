use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use candle_core::{Device, Tensor};
use crossbeam_channel::{Sender, unbounded};
use tracing::{debug, error, info};

use crate::policy::ActorCritic;

pub struct InferenceRequest {
    pub worker_id: usize,
    pub obs_vec: Vec<f32>,
    pub action_mask: Option<Vec<bool>>,
    /// 策略槽位：0 = 当前训练权重，>=1 = 历史对手快照（由 opponents 表提供）。
    pub policy_slot: usize,
    pub reply_tx: Sender<InferenceResponse>,
}

#[derive(Debug, Clone)]
pub struct InferenceResponse {
    pub encoded_action: Vec<f32>,
    pub log_prob: f32,
    pub value: f32,
}

pub struct InferenceServer {
    pub req_tx: Sender<InferenceRequest>,
    /// 更新策略槽位：(slot, 新权重)。slot 0 为当前主策略，slot>0 为历史对手。
    pub model_tx: Sender<(usize, ActorCritic)>,
    is_running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl InferenceServer {
    pub fn new(
        initial_ac: ActorCritic,
        state_dim: usize,
        max_batch_size: usize,
        timeout_us: u64,
        device: Device,
    ) -> Self {
        let (req_tx, req_rx) = unbounded::<InferenceRequest>();
        let (model_tx, model_rx) = unbounded::<(usize, ActorCritic)>();
        let is_running = Arc::new(AtomicBool::new(true));
        let running_clone = is_running.clone();

        let handle = thread::spawn(move || {
            let mut current_ac = initial_ac;
            let mut opponents: HashMap<usize, ActorCritic> = HashMap::new();
            let timeout = Duration::from_micros(timeout_us);
            let mut batch_reqs = Vec::with_capacity(max_batch_size);

            info!(
                "🚀 [InferenceServer] 启动 CUDA/CPU 动态批处理推理引擎 (MaxBatch: {}, Timeout: {}µs)",
                max_batch_size, timeout_us
            );

            while running_clone.load(Ordering::Relaxed) {
                // 1. 检查是否有策略权重更新（slot 0 主策略 / slot>0 历史对手）
                while let Ok((slot, new_ac)) = model_rx.try_recv() {
                    if slot == 0 {
                        current_ac = new_ac;
                        debug!("🔄 [InferenceServer] 已同步最新模型权重 (slot 0)");
                    } else {
                        opponents.insert(slot, new_ac);
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
                            // 轻微让步或微自旋
                            std::hint::spin_loop();
                        }
                        Err(crossbeam_channel::TryRecvError::Disconnected) => break,
                    }
                }

                let batch_len = batch_reqs.len();
                if batch_len == 0 {
                    continue;
                }

                // 4. 按策略槽位分组：同一槽位的一批请求用同一策略一次前向
                //    slot 0 用当前主策略，slot>0 用对应的历史对手策略。
                let mut by_slot: HashMap<usize, Vec<usize>> = HashMap::new();
                for (i, req) in batch_reqs.iter().enumerate() {
                    by_slot.entry(req.policy_slot).or_default().push(i);
                }

                // 取走请求（后面按 slot 回填结果）
                let taken: Vec<_> = batch_reqs.drain(..).collect();

                let mut results: Vec<Option<InferenceResponse>> = vec![None; taken.len()];
                let mut any_error = false;

                for (slot, idxs) in &by_slot {
                    // 选定本组策略
                    let policy = if *slot == 0 {
                        &current_ac
                    } else {
                        match opponents.get(slot) {
                            Some(p) => p,
                            None => {
                                // 对手槽位尚未注册：退化为当前主策略
                                error!("策略槽位 {slot} 未注册，退化为当前主策略");
                                &current_ac
                            }
                        }
                    };

                    // 组装本组输入
                    let mut flat_states = Vec::with_capacity(idxs.len() * state_dim);
                    let mut masks = Vec::with_capacity(idxs.len());
                    for &i in idxs {
                        flat_states.extend_from_slice(&taken[i].obs_vec);
                        masks.push(taken[i].action_mask.clone());
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

                    match policy.sample_batch(&state_tensor, masks_ref) {
                        Ok(samples) => {
                            for (&i, (encoded_action, log_prob, value)) in
                                idxs.iter().zip(samples.into_iter())
                            {
                                results[i] = Some(InferenceResponse {
                                    encoded_action,
                                    log_prob,
                                    value,
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
            is_running,
            handle: Some(handle),
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
