use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Sender, unbounded};
use lol_env::RlEnvironment;
use lol_rl_protocol::{ObsFeaturePayload, RewardItem};
use tracing::info;

use super::inference::{InferenceRequest, InferenceResponse};

#[derive(Debug, Clone)]
pub struct EpisodeInfo {
    pub ep_return: f32,
    pub ep_steps: usize,
}

#[derive(Debug, Clone)]
pub struct SampleTransition {
    /// 来源 worker（env）标识：用于 learner 按 env 分 buffer，保证 GAE 时序正确。
    pub worker_id: usize,
    pub state: Vec<f32>,
    pub action: Vec<f32>,
    pub log_prob: f32,
    pub reward: f32,
    pub value: f32,
    /// 是否真正终止（胜负/阵亡），GAE 中不再 bootstrap 未来价值。
    pub terminated: bool,
    /// 是否超时截断（truncated），GAE 中需 bootstrap 残局真实价值 V(s_T)。
    pub truncated: bool,
    /// 超时截断时真实残局状态对应的无偏价值 V(s_T)（None 表示非截断）。
    pub truncated_next_value: Option<f32>,
    pub done: bool,
    pub episode_info: Option<EpisodeInfo>,
    pub reward_breakdown: Vec<RewardItem>,
    pub reward_variables: HashMap<String, f32>,
    pub obs_payload: Option<ObsFeaturePayload>,
    pub action_mask: Option<Vec<bool>>,
}

pub struct ActorPool {
    is_running: Arc<AtomicBool>,
    /// 每 worker 的策略分派 `(opponent_slot, main_agent_idx)`，可热更新（下轮采样即生效）。
    dispatch: Arc<Mutex<Vec<(usize, usize)>>>,
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
        sample_tx: Sender<SampleTransition>,
        dispatch: Vec<(usize, usize)>,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let dispatch_shared = Arc::new(Mutex::new(dispatch));
        let mut handles = Vec::with_capacity(num_actors);

        info!(
            "🎮 [ActorPool] 启动 {} 个并行无头环境 Actor 线程...",
            num_actors
        );

        for worker_id in 0..num_actors {
            let running = is_running.clone();
            let infer_tx = infer_tx.clone();
            let sample_tx = sample_tx.clone();
            let dispatch = dispatch_shared.clone();

            let handle = thread::spawn(move || {
                let mut env = E::new();
                let mut current_obs = env.reset();
                let (reply_tx, reply_rx) = unbounded::<InferenceResponse>();
                let mut current_ep_return = 0.0f32;
                let mut current_ep_steps = 0usize;

                while running.load(Ordering::Relaxed) {
                    // 每步读取自身分派（支持外部热更新对手槽位/角色）
                    let (opponent_slot, main_agent_idx) = {
                        let d = dispatch.lock().unwrap();
                        d.get(worker_id).copied().unwrap_or((0, 0))
                    };
                    let mut actions = Vec::with_capacity(current_obs.len());
                    let mut pending_transitions = Vec::with_capacity(current_obs.len());

                    for (agent_idx, obs) in current_obs.iter().enumerate() {
                        let obs_vec = E::obs_to_vector(obs);
                        let action_mask = E::action_mask(obs);
                        // 角色感知策略槽位：main 角色用 slot 0（当前主策略），其余用历史对手槽位
                        let policy_slot = if agent_idx == main_agent_idx {
                            0
                        } else {
                            opponent_slot
                        };

                        // 1. 发送推理请求
                        if infer_tx
                            .send(InferenceRequest {
                                worker_id,
                                obs_vec: obs_vec.clone(),
                                action_mask: action_mask.clone(),
                                policy_slot,
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

                        let action = E::action_from_encoding(&resp.encoded_action);
                        actions.push(action);
                        pending_transitions.push((obs_vec, resp, action_mask));
                    }

                    if actions.len() != current_obs.len() {
                        break;
                    }

                    // 3. 执行环境 step
                    let step_results = env.step(&actions);
                    let done = step_results.iter().any(|r| r.terminated || r.truncated);

                    if let Some(r0) = step_results.first() {
                        current_ep_return += r0.reward;
                        current_ep_steps += 1;
                    }

                    let episode_info = if done {
                        let info = EpisodeInfo {
                            ep_return: current_ep_return,
                            ep_steps: current_ep_steps,
                        };
                        current_ep_return = 0.0;
                        current_ep_steps = 0;
                        Some(info)
                    } else {
                        None
                    };

                    // 3.5 超时截断时，为各截断 agent 推断真实残局价值 V(s_T)（无偏 bootstrap 用）。
                    //     仅在 res.truncated 时发一次额外推理请求取 value，其余返回 None。
                    let trunc_vals: Vec<Option<f32>> = step_results
                        .iter()
                        .enumerate()
                        .map(|(agent_idx, res)| {
                            if !res.truncated {
                                return None;
                            }
                            let sv = E::obs_to_vector(&res.obs);
                            let policy_slot = if agent_idx == main_agent_idx {
                                0
                            } else {
                                opponent_slot
                            };
                            if infer_tx
                                .send(InferenceRequest {
                                    worker_id,
                                    obs_vec: sv,
                                    action_mask: None,
                                    policy_slot,
                                    reply_tx: reply_tx.clone(),
                                })
                                .is_err()
                            {
                                return None;
                            }
                            match reply_rx.recv_timeout(Duration::from_secs(5)) {
                                Ok(r) => Some(r.value),
                                Err(_) => None,
                            }
                        })
                        .collect();

                    // 4. 将采样结果推送到训练样本队列
                    for (i, ((obs_vec, resp, action_mask), res)) in pending_transitions
                        .into_iter()
                        .zip(step_results.iter())
                        .enumerate()
                    {
                        let reward_breakdown = res
                            .reward_breakdown
                            .iter()
                            .map(|item| RewardItem {
                                name: item.name.clone(),
                                value: item.value,
                            })
                            .collect();

                        let obs_payload = E::obs_to_payload(&res.obs);

                        let transition = SampleTransition {
                            worker_id,
                            state: obs_vec,
                            action: resp.encoded_action,
                            log_prob: resp.log_prob,
                            reward: res.reward,
                            value: resp.value,
                            terminated: res.terminated,
                            truncated: res.truncated,
                            truncated_next_value: trunc_vals.get(i).copied().flatten(),
                            done,
                            episode_info: episode_info.clone(),
                            reward_breakdown,
                            reward_variables: res.reward_variables.clone(),
                            obs_payload,
                            action_mask,
                        };

                        if sample_tx.send(transition).is_err() {
                            break;
                        }
                    }

                    // 5. 更新环境
                    if done {
                        current_obs = env.reset();
                    } else {
                        current_obs = step_results.into_iter().map(|r| r.obs).collect();
                    }
                }
            });

            handles.push(handle);
        }

        Self {
            is_running,
            dispatch: dispatch_shared,
            handles,
        }
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
