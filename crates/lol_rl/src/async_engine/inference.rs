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
    pub model_tx: Sender<ActorCritic>,
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
        let (model_tx, model_rx) = unbounded::<ActorCritic>();
        let is_running = Arc::new(AtomicBool::new(true));
        let running_clone = is_running.clone();

        let handle = thread::spawn(move || {
            let mut current_ac = initial_ac;
            let timeout = Duration::from_micros(timeout_us);
            let mut batch_reqs = Vec::with_capacity(max_batch_size);

            info!("🚀 [InferenceServer] 启动 CUDA/CPU 动态批处理推理引擎 (MaxBatch: {}, Timeout: {}µs)", max_batch_size, timeout_us);

            while running_clone.load(Ordering::Relaxed) {
                // 1. 检查是否有模型权重更新
                while let Ok(new_ac) = model_rx.try_recv() {
                    current_ac = new_ac;
                    debug!("🔄 [InferenceServer] 已同步最新模型权重");
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

                // 4. 组装输入 Tensor (batch_len, state_dim)
                let mut flat_states = Vec::with_capacity(batch_len * state_dim);
                let mut obs_refs = Vec::with_capacity(batch_len);
                for req in &batch_reqs {
                    flat_states.extend_from_slice(&req.obs_vec);
                    obs_refs.push(req.obs_vec.as_slice());
                }

                let state_tensor = match Tensor::from_vec(flat_states, (batch_len, state_dim), &device) {
                    Ok(t) => t,
                    Err(e) => {
                        error!("创建推理 Tensor 失败: {e}");
                        continue;
                    }
                };

                // 5. 批量前向推理
                let sample_res = current_ac.sample_batch(&state_tensor, &obs_refs);
                match sample_res {
                    Ok(results) => {
                        for (i, req) in batch_reqs.drain(..).enumerate() {
                            let (encoded_action, log_prob, value) = &results[i];
                            let _ = req.reply_tx.send(InferenceResponse {
                                encoded_action: encoded_action.clone(),
                                log_prob: *log_prob,
                                value: *value,
                            });
                        }
                    }
                    Err(e) => {
                        error!("批量推理失败: {e}");
                        batch_reqs.clear();
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
