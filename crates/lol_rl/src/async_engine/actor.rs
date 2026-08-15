use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Sender, unbounded};
use lol_env::RlEnvironment;
use tracing::info;

use super::inference::{InferenceRequest, InferenceResponse};

#[derive(Debug, Clone)]
pub struct SampleTransition {
    pub state: Vec<f32>,
    pub action: Vec<f32>,
    pub log_prob: f32,
    pub reward: f32,
    pub value: f32,
    pub done: bool,
}

pub struct ActorPool {
    is_running: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}

impl ActorPool {
    pub fn spawn<E: RlEnvironment + 'static>(
        num_actors: usize,
        max_steps: usize,
        infer_tx: Sender<InferenceRequest>,
        sample_tx: Sender<SampleTransition>,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let mut handles = Vec::with_capacity(num_actors);

        info!("🎮 [ActorPool] 启动 {} 个并行无头环境 Actor 线程...", num_actors);

        for worker_id in 0..num_actors {
            let running = is_running.clone();
            let infer_tx = infer_tx.clone();
            let sample_tx = sample_tx.clone();

            let handle = thread::spawn(move || {
                let mut env = E::new(max_steps);
                let mut current_obs = env.reset();
                let (reply_tx, reply_rx) = unbounded::<InferenceResponse>();

                while running.load(Ordering::Relaxed) {
                    let obs_vec = E::obs_to_vector(&current_obs);

                    // 1. 发送推理请求
                    if infer_tx
                        .send(InferenceRequest {
                            worker_id,
                            obs_vec: obs_vec.clone(),
                            reply_tx: reply_tx.clone(),
                        })
                        .is_err()
                    {
                        break;
                    }

                    // 2. 等待推理响应
                    let resp = match reply_rx.recv_timeout(Duration::from_secs(5)) {
                        Ok(r) => r,
                        Err(_) => {
                            if !running.load(Ordering::Relaxed) {
                                break;
                            }
                            continue;
                        }
                    };

                    // 3. 执行环境 step
                    let action = E::action_from_encoding(&resp.encoded_action);
                    let res = env.step(action);
                    let done = res.terminated || res.truncated;

                    // 4. 将采样结果推送到训练样本队列
                    let transition = SampleTransition {
                        state: obs_vec,
                        action: resp.encoded_action,
                        log_prob: resp.log_prob,
                        reward: res.reward,
                        value: resp.value,
                        done,
                    };

                    if sample_tx.send(transition).is_err() {
                        break;
                    }

                    // 5. 更新环境
                    if done {
                        current_obs = env.reset();
                    } else {
                        current_obs = res.obs;
                    }
                }
            });

            handles.push(handle);
        }

        Self {
            is_running,
            handles,
        }
    }

    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        for h in self.handles.drain(..) {
            let _ = h.join();
        }
    }
}

impl Drop for ActorPool {
    fn drop(&mut self) {
        self.stop();
    }
}

