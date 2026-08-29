use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, unbounded};
use lol_env::RlEnvironment;
use lol_rl_protocol::ActionMasks;
use tracing::{error, info};

use super::inference::{InferenceRequest, InferenceResponse};
use crate::engine::evaluator::PolicyEvaluator;
use crate::engine::trajectory::WorkerTrajectory;
use crate::engine::worker::RolloutWorker;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurriculumParams {
    pub hp_scale: f32,
    pub cs_reward: f32,
    pub attack_no_cs_penalty: f32,
    pub harass_coef: f32,
}

/// 异步跨线程策略评估器：将单步推理请求打包发送至 InferenceServer，阻塞等待批量推理响应
pub struct ChannelPolicyEvaluator {
    pub worker_id: usize,
    pub infer_tx: Sender<InferenceRequest>,
    pub reply_tx: Sender<InferenceResponse>,
    pub reply_rx: Receiver<InferenceResponse>,
}

impl PolicyEvaluator for ChannelPolicyEvaluator {
    fn evaluate_step(
        &mut self,
        policy_slot: usize,
        state_vec: &[f32],
        action_mask: Option<&[bool]>,
        structured_mask: Option<&ActionMasks>,
        _mamba_state: &mut Option<crate::policy::MambaState>,
    ) -> candle_core::Result<(Vec<f32>, f32, f32)> {
        let req = InferenceRequest {
            worker_id: self.worker_id,
            obs_vec: state_vec.to_vec(),
            action_mask: action_mask.map(|m| m.to_vec()),
            structured_mask: structured_mask.cloned(),
            policy_slot,
            reply_tx: self.reply_tx.clone(),
        };

        if self.infer_tx.send(req).is_err() {
            candle_core::bail!("Inference server channel disconnected");
        }

        match self.reply_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(resp) => Ok((resp.encoded_action, resp.log_prob, resp.value)),
            Err(e) => candle_core::bail!("Inference response timeout: {e}"),
        }
    }
}

pub struct ActorPool {
    is_running: Arc<AtomicBool>,
    /// 每 worker 的策略分派 `(opponent_slot, main_agent_idx)`，可热更新（下轮采样即生效）。
    dispatch: Arc<Mutex<Vec<(usize, usize)>>>,
    /// 课程学习参数（可动态广播至所有环境 Worker）
    curriculum: Arc<Mutex<Option<CurriculumParams>>>,
    handles: Vec<JoinHandle<()>>,
}

impl ActorPool {
    /// 启动 Actor 池。`dispatch` 为每个 worker 提供 `(opponent_slot, main_agent_idx)`（长度 = num_actors）：
    /// - `opponent_slot == 0`：纯自博弈（双方都用当前主策略 slot 0 推理）；
    /// - `opponent_slot > 0`：对抗该历史对手，`main_agent_idx` 指定主策略扮演的角色，
    ///   其余角色用 `opponent_slot` 对应的历史对手策略推理（双角色轮换由调用方安排）。
    pub fn spawn<E: RlEnvironment + 'static>(
        num_actors: usize,
        infer_tx: Sender<InferenceRequest>,
        traj_tx: Sender<WorkerTrajectory<E::Obs>>,
        horizon: usize,
        dispatch: Vec<(usize, usize)>,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let dispatch_shared = Arc::new(Mutex::new(dispatch));
        let curriculum_shared = Arc::new(Mutex::new(None));
        let mut handles = Vec::with_capacity(num_actors);

        info!(
            "🎮 [ActorPool] 启动 {} 个并行无头环境 Actor 线程...",
            num_actors
        );

        for worker_id in 0..num_actors {
            let running = is_running.clone();
            let infer_tx = infer_tx.clone();
            let traj_tx = traj_tx.clone();
            let dispatch = dispatch_shared.clone();
            let curriculum = curriculum_shared.clone();

            let handle = thread::spawn(move || {
                let mut worker = RolloutWorker::<E>::new();
                let (reply_tx, reply_rx) = unbounded::<InferenceResponse>();
                let mut evaluator = ChannelPolicyEvaluator {
                    worker_id,
                    infer_tx,
                    reply_tx,
                    reply_rx,
                };
                let mut applied_curriculum: Option<CurriculumParams> = None;

                while running.load(Ordering::Relaxed) {
                    // 检查并更新课程学习超参数
                    let cur_params = *curriculum.lock().unwrap();
                    if cur_params != applied_curriculum {
                        if let Some(c) = cur_params {
                            worker.update_curriculum(
                                c.hp_scale,
                                c.cs_reward,
                                c.attack_no_cs_penalty,
                                c.harass_coef,
                            );
                        }
                        applied_curriculum = cur_params;
                    }

                    // 每轮读取自身分派（支持外部热更新对手槽位/角色）
                    let (opponent_slot, main_agent_idx) = {
                        let d = dispatch.lock().unwrap();
                        d.get(worker_id).copied().unwrap_or((0, 0))
                    };

                    match worker.rollout_with_evaluator(
                        &mut evaluator,
                        horizon,
                        opponent_slot,
                        main_agent_idx,
                    ) {
                        Ok(traj) => {
                            if traj_tx.send(traj).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            if !running.load(Ordering::Relaxed) {
                                break;
                            }
                            error!("Actor {worker_id} Rollout 失败: {e}");
                        }
                    }
                }
            });

            handles.push(handle);
        }

        Self {
            is_running,
            dispatch: dispatch_shared,
            curriculum: curriculum_shared,
            handles,
        }
    }

    /// 动态更新课程学习参数
    pub fn update_curriculum(
        &self,
        hp_scale: f32,
        cs_reward: f32,
        attack_no_cs_penalty: f32,
        harass_coef: f32,
    ) {
        let mut c = self.curriculum.lock().unwrap();
        *c = Some(CurriculumParams {
            hp_scale,
            cs_reward,
            attack_no_cs_penalty,
            harass_coef,
        });
    }

    /// 热更新每个 worker 的策略分派（`(opponent_slot, main_agent_idx)`），下轮采样即生效。
    pub fn update_dispatch(&self, dispatch: Vec<(usize, usize)>) {
        let mut d = self.dispatch.lock().unwrap();
        *d = dispatch;
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
