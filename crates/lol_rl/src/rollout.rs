//! 持久化环境 Rollout Worker：真实训练循环与 AutoTuner 探测器共用的唯一真实来源。
//!
//! 训练循环与探测器都通过 [`RolloutWorker::rollout`] 驱动环境采样，
//! 保证探测到的每步耗时与真实训练（含 obs→vector、策略采样、采样簿记、env step）完全一致。

use std::collections::HashMap;
use std::sync::Arc;

use candle_core::{Device, Result, Tensor};
use lol_env::RlEnvironment;

use crate::policy::{PolicyNetwork, ValueHead};
use crate::ppo::RolloutBuffer;

/// 一次 Rollout 的完整产出（单个 Worker 一次 horizon 推演）。
pub struct WorkerTrajectory<O> {
    /// 参与训练的轨迹 Buffer（自博弈：每智能体一个；对抗历史对手：仅主角色一个）。
    pub buffers: Vec<RolloutBuffer>,
    /// 与 buffers 一一对齐的末尾价值（GAE bootstrap 用）。
    pub last_values: Vec<f32>,
    pub ep_returns: Vec<f32>,
    pub ep_cs: Vec<f32>,
    pub completed_steps: Vec<usize>,
    pub reward_breakdown: HashMap<String, f32>,
    pub last_reward_variables: HashMap<String, f32>,
    pub last_obs: Option<O>,
}

impl<O> WorkerTrajectory<O> {
    pub fn empty() -> Self {
        Self {
            buffers: Vec::new(),
            last_values: Vec::new(),
            ep_returns: Vec::new(),
            ep_cs: Vec::new(),
            completed_steps: Vec::new(),
            reward_breakdown: HashMap::new(),
            last_reward_variables: HashMap::new(),
            last_obs: None,
        }
    }
}

/// 发给持久化 Worker 的命令。
pub enum WorkerCommand {
    Rollout {
        main_policy: Arc<PolicyNetwork>,
        main_critic: Option<Arc<ValueHead>>,
        opponent_policy: Option<Arc<PolicyNetwork>>,
        opponent_critic: Option<Arc<ValueHead>>,
        main_agent_idx: usize,
    },
    /// 更新课程学习参数（小兵血量缩放 + 奖励配置）
    UpdateCurriculum {
        hp_scale: f32,
        cs_reward: f32,
        attack_no_cs_penalty: f32,
        harass_coef: f32,
    },
    Stop,
}

/// 一个常驻环境 + 回合累计状态的 Rollout Worker。
pub struct RolloutWorker<E: RlEnvironment> {
    env: E,
    current_obs: Vec<E::Obs>,
    cur_return: f32,
    cur_cs: f32,
    cur_steps: usize,
    agent_mamba_states: Vec<Option<crate::policy::MambaState>>,
}

impl<E: RlEnvironment> RolloutWorker<E> {
    /// 创建 Worker 并初始化环境（环境只在启动时初始化一次，全程复用）。
    pub fn new() -> Self {
        let mut env = E::new();
        let current_obs = env.reset();
        let num_agents = current_obs.len().max(1);
        Self {
            env,
            current_obs,
            cur_return: 0.0,
            cur_cs: 0.0,
            cur_steps: 0,
            agent_mamba_states: vec![None; num_agents],
        }
    }

    /// 更新环境中的课程学习参数
    pub fn update_curriculum(
        &mut self,
        hp_scale: f32,
        cs_reward: f32,
        attack_no_cs_penalty: f32,
        harass_coef: f32,
    ) {
        self.env
            .update_curriculum(hp_scale, cs_reward, attack_no_cs_penalty, harass_coef);
    }

    /// 执行一次完整 Rollout。
    ///
    /// - `opponent_policy = None`：纯自博弈，双方同策略推理，所有智能体样本都写入对应 buffer；
    /// - `opponent_policy = Some(_)`：对抗历史对手，仅主策略扮演的角色（`main_agent_idx`）样本进 buffer；
    /// - `sampler_device`：采样前向在哪个设备执行。传 `Device::Cpu` 为原 CPU 推理路径（默认），
    ///   传 GPU device 则把每次策略前向放到 GPU（需 `main_policy`/`opponent_policy` 已迁到该 device）。
    pub fn rollout(
        &mut self,
        main_policy: &PolicyNetwork,
        main_critic: Option<&ValueHead>,
        opponent_policy: Option<&PolicyNetwork>,
        opponent_critic: Option<&ValueHead>,
        main_agent_idx: usize,
        horizon: usize,
        state_dim: usize,
        sampler_device: &Device,
    ) -> Result<WorkerTrajectory<E::Obs>> {
        let has_opp_policy = opponent_policy.is_some();
        let opp_policy = opponent_policy.unwrap_or(main_policy);
        let opp_critic = opponent_critic.or(main_critic);
        let num_agents = self.current_obs.len().max(1);
        let train_agent_count = if has_opp_policy { 1 } else { num_agents };

        if self.agent_mamba_states.len() < num_agents {
            self.agent_mamba_states.resize_with(num_agents, || None);
        }

        let mut buffers: Vec<RolloutBuffer> = (0..train_agent_count)
            .map(|_| RolloutBuffer::new())
            .collect();
        let mut ep_returns = Vec::new();
        let mut ep_cs = Vec::new();
        let mut completed_steps = Vec::new();
        let mut reward_breakdown = HashMap::new();
        let mut last_reward_variables = HashMap::new();

        let is_mlp = main_policy.backbone().backbone_type() == lol_rl_protocol::PolicyBackbone::Mlp;

        for _ in 0..horizon {
            let mut actions = Vec::with_capacity(self.current_obs.len());
            let mut step_samples = Vec::with_capacity(self.current_obs.len());

            if is_mlp && !has_opp_policy && self.current_obs.len() > 1 {
                // MLP 批量推理优化：双方使用同一策略时单次前向完成采样
                let mut batch_flat = Vec::with_capacity(self.current_obs.len() * state_dim);
                let mut masks = Vec::with_capacity(self.current_obs.len());
                let mut structured_masks = Vec::with_capacity(self.current_obs.len());
                let mut state_vecs = Vec::with_capacity(self.current_obs.len());
                for obs in &self.current_obs {
                    let sv = E::obs_to_vector(obs);
                    batch_flat.extend_from_slice(&sv);
                    masks.push(E::action_mask(obs));
                    structured_masks.push(E::action_masks(obs));
                    state_vecs.push(sv);
                }
                let batch_tensor = Tensor::from_vec(
                    batch_flat,
                    (self.current_obs.len(), state_dim),
                    sampler_device,
                )?;
                let batch_samples = main_policy.sample_batch_with_structured_masks(
                    &batch_tensor,
                    Some(&structured_masks),
                    Some(&masks),
                )?;
                let val_vec = if let Some(critic) = main_critic {
                    let feat = main_policy.hidden(&batch_tensor)?;
                    let v = critic.forward(&feat)?;
                    v.squeeze(1)?.to_vec1()?
                } else {
                    vec![0.0; self.current_obs.len()]
                };

                for (((state_vec, mask), (encoded, log_prob)), val) in state_vecs
                    .into_iter()
                    .zip(masks.into_iter())
                    .zip(batch_samples.into_iter())
                    .zip(val_vec.into_iter())
                {
                    let act = E::action_from_encoding(&encoded);
                    actions.push(act);
                    step_samples.push((state_vec, encoded, log_prob, val, mask));
                }
            } else {
                // 逐 Agent 采样（支持 Mamba 状态时序递推及主策略与历史对手分别推理）
                for (agent_idx, obs) in self.current_obs.iter().enumerate() {
                    let state_vec = E::obs_to_vector(obs);
                    let action_mask = E::action_mask(obs);
                    let structured_mask = E::action_masks(obs);
                    let state_tensor =
                        Tensor::from_vec(state_vec.clone(), (1, state_dim), sampler_device)?;

                    let (active_policy, active_critic) =
                        if !has_opp_policy || agent_idx == main_agent_idx {
                            (main_policy, main_critic)
                        } else {
                            (opp_policy, opp_critic)
                        };

                    let (encoded, log_prob) = active_policy.step_with_structured_masks(
                        &state_tensor,
                        &mut self.agent_mamba_states[agent_idx],
                        structured_mask.as_ref(),
                        action_mask.as_deref(),
                    )?;

                    let val = if let Some(critic) = active_critic {
                        let feat = active_policy.hidden(&state_tensor)?;
                        let v = critic.forward(&feat)?;
                        v.squeeze(0)?.squeeze(0)?.to_scalar()?
                    } else {
                        0.0
                    };

                    let act = E::action_from_encoding(&encoded);
                    actions.push(act);
                    step_samples.push((state_vec, encoded, log_prob, val, action_mask));
                }
            }

            if actions.len() != self.current_obs.len() {
                break;
            }

            // 执行环境 step
            let step_results = self.env.step(&actions);
            let done = step_results.iter().any(|r| r.terminated || r.truncated);

            let primary_res = if has_opp_policy {
                step_results.get(main_agent_idx)
            } else {
                step_results.first()
            };
            if let Some(res) = primary_res {
                self.cur_return += res.reward;
                self.cur_steps += 1;
                if let Some(&cs) = res.reward_variables.get("self_cs") {
                    self.cur_cs += cs;
                }
                if !res.reward_variables.is_empty() {
                    last_reward_variables = res.reward_variables.clone();
                }
                for item in &res.reward_breakdown {
                    *reward_breakdown.entry(item.name.clone()).or_insert(0.0) += item.value;
                }
            }

            // 若发生超时截断 (truncated)，在 env.reset() 前基于真实残局观测推断真实价值 V(s_T)
            let trunc_next_vals: Vec<Option<f32>> = step_results
                .iter()
                .enumerate()
                .map(|(idx, res)| {
                    if res.truncated {
                        let sv = E::obs_to_vector(&res.obs);
                        let (active_policy, active_critic) =
                            if !has_opp_policy || idx == main_agent_idx {
                                (main_policy, main_critic)
                            } else {
                                (opp_policy, opp_critic)
                            };
                        if let Some(critic) = active_critic {
                            match Tensor::from_vec(sv, (1, state_dim), sampler_device) {
                                Ok(t) => {
                                    if let Ok(feat) = active_policy.hidden(&t) {
                                        critic
                                            .forward(&feat)
                                            .ok()
                                            .and_then(|v| v.squeeze(0).ok())
                                            .and_then(|v| v.squeeze(0).ok())
                                            .and_then(|v| v.to_scalar().ok())
                                    } else {
                                        None
                                    }
                                }
                                Err(_) => None,
                            }
                        } else {
                            Some(0.0)
                        }
                    } else {
                        None
                    }
                })
                .collect();

            // 将采样结果写入对应轨迹 Buffer
            for (agent_idx, ((state_vec, encoded, log_prob, val, action_mask), res)) in step_samples
                .into_iter()
                .zip(step_results.iter())
                .enumerate()
            {
                let trunc_val = trunc_next_vals.get(agent_idx).copied().flatten();
                if !has_opp_policy {
                    // 纯自博弈：双方样本均写入对应 buffer
                    if agent_idx < buffers.len() {
                        buffers[agent_idx].push_full(
                            state_vec,
                            encoded,
                            log_prob,
                            res.reward,
                            val,
                            res.terminated,
                            res.truncated,
                            trunc_val,
                            action_mask,
                        );
                    }
                } else if agent_idx == main_agent_idx {
                    // 对抗历史对手：仅主策略扮演的角色写入 buffers[0] 用于梯度更新
                    buffers[0].push_full(
                        state_vec,
                        encoded,
                        log_prob,
                        res.reward,
                        val,
                        res.terminated,
                        res.truncated,
                        trunc_val,
                        action_mask,
                    );
                }
            }

            // 更新环境观测
            if done {
                for s in &mut self.agent_mamba_states {
                    *s = None;
                }
                ep_returns.push(self.cur_return);
                ep_cs.push(self.cur_cs);
                completed_steps.push(self.cur_steps);
                self.cur_return = 0.0;
                self.cur_cs = 0.0;
                self.cur_steps = 0;
                self.current_obs = self.env.reset();
            } else {
                self.current_obs = step_results.into_iter().map(|r| r.obs).collect();
            }
        }

        // 独立推断未完成轨迹的末尾价值 last_values
        let mut last_values = Vec::with_capacity(train_agent_count);
        if !has_opp_policy {
            for obs in &self.current_obs {
                let last_state_vec = E::obs_to_vector(obs);
                let last_val = if let Some(critic) = main_critic {
                    match Tensor::from_vec(last_state_vec, (1, state_dim), sampler_device) {
                        Ok(tensor) => main_policy
                            .hidden(&tensor)
                            .and_then(|feat| critic.forward(&feat))
                            .and_then(|v| v.squeeze(0))
                            .and_then(|v| v.squeeze(0))
                            .and_then(|v| v.to_scalar())
                            .unwrap_or(0.0),
                        Err(_) => 0.0,
                    }
                } else {
                    0.0
                };
                last_values.push(last_val);
            }
        } else if let Some(obs) = self.current_obs.get(main_agent_idx) {
            let last_state_vec = E::obs_to_vector(obs);
            let last_val = if let Some(critic) = main_critic {
                match Tensor::from_vec(last_state_vec, (1, state_dim), sampler_device) {
                    Ok(tensor) => main_policy
                        .hidden(&tensor)
                        .and_then(|feat| critic.forward(&feat))
                        .and_then(|v| v.squeeze(0))
                        .and_then(|v| v.squeeze(0))
                        .and_then(|v| v.to_scalar())
                        .unwrap_or(0.0),
                    Err(_) => 0.0,
                }
            } else {
                0.0
            };
            last_values.push(last_val);
        }

        let last_obs_primary = self.current_obs.first().cloned();

        Ok(WorkerTrajectory {
            buffers,
            last_values,
            ep_returns,
            ep_cs,
            completed_steps,
            reward_breakdown,
            last_reward_variables,
            last_obs: last_obs_primary,
        })
    }
}

impl<E: RlEnvironment> Default for RolloutWorker<E> {
    fn default() -> Self {
        Self::new()
    }
}
