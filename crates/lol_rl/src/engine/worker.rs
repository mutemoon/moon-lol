use std::collections::HashMap;

use candle_core::{Device, Result};
use lol_env::RlEnvironment;

use crate::algo::buffer::RolloutBuffer;
use crate::engine::evaluator::{DirectPolicyEvaluator, PolicyEvaluator};
use crate::engine::trajectory::WorkerTrajectory;
use crate::policy::{MambaState, PolicyNetwork, ValueHead};

/// 一个常驻环境 + 回合累计状态的 Rollout Worker。
pub struct RolloutWorker<E: RlEnvironment> {
    pub env: E,
    pub current_obs: Vec<E::Obs>,
    pub cur_return: f32,
    pub cur_cs: f32,
    pub cur_steps: usize,
    pub agent_mamba_states: Vec<Option<MambaState>>,
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

    /// 使用给定的 `PolicyEvaluator` 执行一段完整的 horizon 推演。
    ///
    /// 无论同步（DirectPolicyEvaluator）还是异步（ChannelPolicyEvaluator / InferenceServer），
    /// 所有环境交互、因式分解掩码提取、多智能体 Buffer 隔离、对手数据过滤、真实残局 Bootstrap 判定均完全共用本逻辑。
    pub fn rollout_with_evaluator<Eval: PolicyEvaluator>(
        &mut self,
        evaluator: &mut Eval,
        horizon: usize,
        opponent_slot: usize,
        main_agent_idx: usize,
    ) -> Result<WorkerTrajectory<E::Obs>> {
        let has_opponent = opponent_slot > 0;
        let num_agents = self.current_obs.len().max(1);
        let train_agent_count = if has_opponent { 1 } else { num_agents };

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

        for _ in 0..horizon {
            let mut actions = Vec::with_capacity(self.current_obs.len());
            let mut step_samples = Vec::with_capacity(self.current_obs.len());

            if !has_opponent && self.current_obs.len() > 1 {
                // 纯自博弈（双方使用相同策略 slot 0）：支持批量前向
                let mut state_vecs = Vec::with_capacity(self.current_obs.len());
                let mut masks = Vec::with_capacity(self.current_obs.len());
                let mut structured_masks = Vec::with_capacity(self.current_obs.len());
                for obs in &self.current_obs {
                    state_vecs.push(E::obs_to_vector(obs));
                    masks.push(E::action_mask(obs));
                    structured_masks.push(E::action_masks(obs));
                }

                let batch_samples = evaluator.evaluate_batch(
                    0,
                    &state_vecs,
                    &masks,
                    &structured_masks,
                )?;

                for ((state_vec, mask), (encoded, log_prob, val)) in state_vecs
                    .into_iter()
                    .zip(masks.into_iter())
                    .zip(batch_samples.into_iter())
                {
                    let act = E::action_from_encoding(&encoded);
                    actions.push(act);
                    step_samples.push((state_vec, encoded, log_prob, val, mask));
                }
            } else {
                // 逐 Agent 采样（支持主策略与历史对手分别推理）
                for (agent_idx, obs) in self.current_obs.iter().enumerate() {
                    let state_vec = E::obs_to_vector(obs);
                    let action_mask = E::action_mask(obs);
                    let structured_mask = E::action_masks(obs);
                    let policy_slot = if !has_opponent || agent_idx == main_agent_idx {
                        0
                    } else {
                        opponent_slot
                    };

                    let mamba_state = if agent_idx < self.agent_mamba_states.len() {
                        &mut self.agent_mamba_states[agent_idx]
                    } else {
                        &mut None
                    };

                    let (encoded, log_prob, val) = evaluator.evaluate_step(
                        policy_slot,
                        &state_vec,
                        action_mask.as_deref(),
                        structured_mask.as_ref(),
                        mamba_state,
                    )?;

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

            let primary_res = if has_opponent {
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
                        let policy_slot = if !has_opponent || idx == main_agent_idx {
                            0
                        } else {
                            opponent_slot
                        };
                        evaluator.evaluate_value(policy_slot, &sv).ok()
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
                if !has_opponent {
                    // 纯自博弈：双方样本均写入各自独立的 buffer
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

        // 独立推断未完成轨迹的末尾价值 last_values（用于 GAE Bootstrap）
        let mut last_values = Vec::with_capacity(train_agent_count);
        if !has_opponent {
            for obs in &self.current_obs {
                let last_state_vec = E::obs_to_vector(obs);
                let last_val = evaluator.evaluate_value(0, &last_state_vec).unwrap_or(0.0);
                last_values.push(last_val);
            }
        } else if let Some(obs) = self.current_obs.get(main_agent_idx) {
            let last_state_vec = E::obs_to_vector(obs);
            let last_val = evaluator.evaluate_value(0, &last_state_vec).unwrap_or(0.0);
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

    /// 执行一次完整 Rollout（同步模式便利封装）。
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
        let opponent_slot = if opponent_policy.is_some() { 1 } else { 0 };
        let mut evaluator = DirectPolicyEvaluator {
            main_policy,
            main_critic,
            opponent_policy,
            opponent_critic,
            main_agent_idx,
            state_dim,
            device: sampler_device,
        };
        self.rollout_with_evaluator(&mut evaluator, horizon, opponent_slot, main_agent_idx)
    }
}

impl<E: RlEnvironment> Default for RolloutWorker<E> {
    fn default() -> Self {
        Self::new()
    }
}
