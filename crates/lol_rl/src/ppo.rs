use std::path::Path;

use candle_core::{DType, Device, Result, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW, VarBuilder, VarMap};
use lol_rl_protocol::ActionSpace;

use crate::policy::ActorCritic;

pub struct RolloutBuffer {
    pub states: Vec<Vec<f32>>,
    /// 扁平编码动作向量：Discrete=[idx]，Continuous=[v0..]，Hybrid=[v0, v1, attack_idx]。
    pub actions: Vec<Vec<f32>>,
    pub log_probs: Vec<f32>,
    pub rewards: Vec<f32>,
    pub values: Vec<f32>,
    pub dones: Vec<bool>,
    /// 动作掩码（若环境提供）：true = 有效，false = 非法/屏蔽
    pub action_masks: Vec<Option<Vec<bool>>>,
}

impl RolloutBuffer {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            actions: Vec::new(),
            log_probs: Vec::new(),
            rewards: Vec::new(),
            values: Vec::new(),
            dones: Vec::new(),
            action_masks: Vec::new(),
        }
    }

    pub fn push(
        &mut self,
        state: Vec<f32>,
        action: Vec<f32>,
        log_prob: f32,
        reward: f32,
        value: f32,
        done: bool,
        action_mask: Option<Vec<bool>>,
    ) {
        self.states.push(state);
        self.actions.push(action);
        self.log_probs.push(log_prob);
        self.rewards.push(reward);
        self.values.push(value);
        self.dones.push(done);
        self.action_masks.push(action_mask);
    }

    pub fn push_unmasked(
        &mut self,
        state: Vec<f32>,
        action: Vec<f32>,
        log_prob: f32,
        reward: f32,
        value: f32,
        done: bool,
    ) {
        self.push(state, action, log_prob, reward, value, done, None);
    }

    pub fn clear(&mut self) {
        self.states.clear();
        self.actions.clear();
        self.log_probs.clear();
        self.rewards.clear();
        self.values.clear();
        self.dones.clear();
        self.action_masks.clear();
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

impl Default for RolloutBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct PPOConfig {
    pub lr: f64,
    pub gamma: f32,
    pub gae_lambda: f32,
    pub clip_eps: f32,
    pub c1: f32, // Value loss coefficient
    pub c2: f32, // Entropy coefficient
    pub ppo_epochs: usize,
    /// 价值函数损失截断 (Value Loss Clipping, PPO2 工业级标准)
    pub clip_vloss: bool,
    /// 全局梯度 L2 范数截断上限 (0.0 为不截断，推荐 0.5)
    pub max_grad_norm: f32,
}

impl Default for PPOConfig {
    fn default() -> Self {
        Self {
            lr: 5e-4,
            gamma: 0.99,
            gae_lambda: 0.95,
            clip_eps: 0.2,
            c1: 0.5,
            c2: 0.05,
            ppo_epochs: 4,
            clip_vloss: true,
            max_grad_norm: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PPOStats {
    pub policy_loss: f32,
    pub value_loss: f32,
    pub entropy_loss: f32,
    /// 平均策略熵（正值，上报/展示用），与 entropy_loss（负值，参与总损失）区分。
    pub entropy: f32,
    pub total_loss: f32,
    pub kl: f32,
    /// 本 epoch 被 clip 的比例（ratio 超出 [1-eps, 1+eps] 的占比）
    pub clip_frac: f32,
}

pub struct PPOAgent {
    pub actor_critic: ActorCritic,
    varmap: VarMap,
    optimizer: AdamW,
    config: PPOConfig,
    device: Device,
}

impl PPOAgent {
    pub fn new(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: PPOConfig,
        device: Device,
    ) -> Result<Self> {
        let mut varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let actor_critic = ActorCritic::new(state_dim, hidden_dim, action_space, vb)?;

        // 对网络权重应用工业级正交初始化（正交矩阵 + 分层增益 Gain）
        let hidden_gain = std::f32::consts::SQRT_2;
        let fc1_w = Tensor::from_vec(
            crate::policy::orthogonal_weight(hidden_dim, state_dim, hidden_gain),
            (hidden_dim, state_dim),
            &device,
        )?;
        let fc2_w = Tensor::from_vec(
            crate::policy::orthogonal_weight(hidden_dim, hidden_dim, hidden_gain),
            (hidden_dim, hidden_dim),
            &device,
        )?;
        let actor_w = Tensor::from_vec(
            crate::policy::orthogonal_weight(actor_critic.action_space().actor_head_dim(), hidden_dim, 0.01),
            (actor_critic.action_space().actor_head_dim(), hidden_dim),
            &device,
        )?;
        let critic_w = Tensor::from_vec(
            crate::policy::orthogonal_weight(1, hidden_dim, 1.0),
            (1, hidden_dim),
            &device,
        )?;

        let _ = varmap.set_one("fc1.weight", fc1_w);
        let _ = varmap.set_one("fc2.weight", fc2_w);
        let _ = varmap.set_one("actor_head.weight", actor_w);
        let _ = varmap.set_one("critic_head.weight", critic_w);

        let params = ParamsAdamW {
            lr: config.lr,
            ..Default::default()
        };
        let optimizer = AdamW::new(varmap.all_vars(), params)?;

        Ok(Self {
            actor_critic,
            varmap,
            optimizer,
            config,
            device,
        })
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn set_entropy_coef(&mut self, c2: f32) {
        self.config.c2 = c2;
    }

    pub fn set_lr(&mut self, lr: f64) -> Result<()> {
        self.config.lr = lr;
        let params = ParamsAdamW {
            lr,
            ..Default::default()
        };
        self.optimizer = AdamW::new(self.varmap.all_vars(), params)?;
        Ok(())
    }

    /// 全局梯度 L2 范数裁剪（Industrial PPO Standard: max_grad_norm = 0.5）
    pub fn clip_grad_norm(&self, grads: &mut candle_core::backprop::GradStore) -> Result<f32> {
        if self.config.max_grad_norm <= 0.0 {
            return Ok(0.0);
        }
        let vars = self.varmap.all_vars();
        let mut total_norm_sq = 0.0f32;
        for var in &vars {
            if let Some(grad) = grads.get(var) {
                let norm_sq: f32 = (grad * grad)?.sum_all()?.to_scalar()?;
                total_norm_sq += norm_sq;
            }
        }
        let total_norm = total_norm_sq.sqrt();
        let max_norm = self.config.max_grad_norm;
        if total_norm > max_norm {
            let scale = (max_norm / (total_norm + 1e-6)) as f64;
            for var in &vars {
                if let Some(grad) = grads.get(var) {
                    let scaled_grad = grad.affine(scale, 0.0)?;
                    grads.insert(var, scaled_grad);
                }
            }
        }
        Ok(total_norm)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    candle_core::Error::Msg(format!("创建 checkpoint 目录失败: {e}"))
                })?;
            }
        }
        self.varmap.save(path)
    }

    pub fn load(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: PPOConfig,
        device: Device,
        path: &Path,
    ) -> Result<Self> {
        let meta = std::fs::metadata(path)
            .map_err(|e| candle_core::Error::Msg(format!("checkpoint 文件不存在: {e}")))?;
        if meta.len() == 0 {
            return Err(candle_core::Error::Msg("checkpoint 文件为空".to_string()));
        }
        let tensors = candle_core::safetensors::load(path, &device)?;

        // 从 fc2.bias（或 fc1.bias）的形状自动推断隐藏层维度，兼容不同 hidden_dim 的 checkpoint。
        let hidden_dim = tensors
            .get("fc2.bias")
            .or_else(|| tensors.get("fc1.bias"))
            .and_then(|t| t.shape().dims().first().copied())
            .unwrap_or(hidden_dim);

        // 校验 checkpoint 的动作空间结构与请求一致
        let has_log_std = tensors.contains_key("log_std");
        let has_attack_head = tensors.contains_key("attack_head.weight");
        let want_log_std = !matches!(action_space, ActionSpace::Discrete(_));
        let want_attack_head = matches!(action_space, ActionSpace::Hybrid { .. });
        if has_log_std != want_log_std || has_attack_head != want_attack_head {
            return Err(candle_core::Error::Msg(format!(
                "checkpoint 动作空间不匹配: 期望 log_std={want_log_std} attack_head={want_attack_head}, \
                 实际 log_std={has_log_std} attack_head={has_attack_head}"
            )));
        }

        let mut varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let actor_critic = ActorCritic::new(state_dim, hidden_dim, action_space, vb)?;
        varmap.load(path)?;
        let params = ParamsAdamW {
            lr: config.lr,
            ..Default::default()
        };
        let optimizer = AdamW::new(varmap.all_vars(), params)?;
        Ok(Self {
            actor_critic,
            varmap,
            optimizer,
            config,
            device,
        })
    }

    /// Compute GAE advantages and returns
    pub fn compute_gae(&self, buffer: &RolloutBuffer, last_val: f32) -> (Vec<f32>, Vec<f32>) {
        let n = buffer.len();
        let mut returns = vec![0.0; n];
        let mut advantages = vec![0.0; n];

        let mut gae = 0.0;
        for t in (0..n).rev() {
            let next_val = if t + 1 < n {
                buffer.values[t + 1]
            } else {
                last_val
            };
            let next_non_terminal = if buffer.dones[t] { 0.0 } else { 1.0 };

            let delta = buffer.rewards[t] + self.config.gamma * next_val * next_non_terminal
                - buffer.values[t];
            gae = delta + self.config.gamma * self.config.gae_lambda * next_non_terminal * gae;

            advantages[t] = gae;
            returns[t] = gae + buffer.values[t];
        }

        (returns, advantages)
    }

    /// Update policy using buffer data
    pub fn update(&mut self, buffer: &RolloutBuffer, last_val: f32) -> Result<PPOStats> {
        let n = buffer.len();
        if n == 0 {
            return Ok(PPOStats {
                policy_loss: 0.0,
                value_loss: 0.0,
                entropy_loss: 0.0,
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
            });
        }

        let (returns, mut advantages) = self.compute_gae(buffer, last_val);

        // Normalize advantages globally
        let mean = advantages.iter().sum::<f32>() / n as f32;
        let variance = advantages.iter().map(|a| (a - mean).powi(2)).sum::<f32>() / n as f32;
        let std = (variance + 1e-8).sqrt();
        for a in advantages.iter_mut() {
            *a = (*a - mean) / std;
        }

        // Convert buffer to tensors
        let flat_states: Vec<f32> = buffer.states.iter().flatten().copied().collect();
        let state_dim = buffer.states[0].len();

        let states_tensor = Tensor::from_vec(flat_states, (n, state_dim), &self.device)?;
        let enc_dim = buffer.actions[0].len();
        let flat_actions: Vec<f32> = buffer.actions.iter().flatten().copied().collect();
        let actions_tensor = Tensor::from_vec(flat_actions, (n, enc_dim), &self.device)?;
        let old_log_probs_tensor = Tensor::from_vec(buffer.log_probs.clone(), (n,), &self.device)?;
        let old_values_tensor = Tensor::from_vec(buffer.values.clone(), (n,), &self.device)?;
        let returns_tensor = Tensor::from_vec(returns, (n,), &self.device)?;
        let advantages_tensor = Tensor::from_vec(advantages, (n,), &self.device)?;
        let masks_tensor = build_masks_tensor(&buffer.action_masks, &self.device)?;

        let mut last_stats = PPOStats {
            policy_loss: 0.0,
            value_loss: 0.0,
            entropy_loss: 0.0,
            entropy: 0.0,
            total_loss: 0.0,
            kl: 0.0,
            clip_frac: 0.0,
        };

        for _epoch in 0..self.config.ppo_epochs {
            let (new_log_probs, new_values, entropy) = self.actor_critic.evaluate_actions(
                &states_tensor,
                &actions_tensor,
                masks_tensor.as_ref(),
            )?;

            // Ratio r(theta) = exp(new_log_probs - old_log_probs)
            let log_ratio = (&new_log_probs - &old_log_probs_tensor)?;
            let ratio = log_ratio.exp()?;

            // Clipped Surrogate Objective
            let surr1 = (&ratio * &advantages_tensor)?;
            let clamped_ratio =
                ratio.clamp(1.0 - self.config.clip_eps, 1.0 + self.config.clip_eps)?;
            let surr2 = (&clamped_ratio * &advantages_tensor)?;

            // Policy Loss = - mean(min(surr1, surr2))
            let policy_loss = surr1.minimum(&surr2)?.neg()?.mean_all()?;

            // Value Loss: PPO2 Clipped Value Loss
            let value_loss = if self.config.clip_vloss {
                let v_diff = (&new_values - &old_values_tensor)?;
                let v_clamped_diff = v_diff.clamp(-self.config.clip_eps, self.config.clip_eps)?;
                let v_clipped = (&old_values_tensor + &v_clamped_diff)?;
                let v_loss_unclipped = (&new_values - &returns_tensor)?.powf(2.0)?;
                let v_loss_clipped = (&v_clipped - &returns_tensor)?.powf(2.0)?;
                v_loss_unclipped.maximum(&v_loss_clipped)?.mean_all()?.affine(0.5, 0.0)?
            } else {
                let val_diff = (&new_values - &returns_tensor)?;
                (&val_diff * &val_diff)?.mean_all()?.affine(0.5, 0.0)?
            };

            // Entropy Loss = - mean(entropy)
            let entropy_loss = entropy.neg()?.mean_all()?;

            // KL divergence (K1 estimator: ratio - 1 - log(ratio))
            let kl = (&ratio - 1.0 - &log_ratio)?.mean_all()?;

            // clip_frac：ratio 超出 [1-eps, 1+eps] 的元素占比
            let clip_frac = (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
            .mean_all()?;

            let p_loss_val: f32 = policy_loss.to_scalar()?;
            let v_loss_val: f32 = value_loss.to_scalar()?;
            let e_loss_val: f32 = entropy_loss.to_scalar()?;
            let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
            let kl_val: f32 = kl.to_scalar()?;
            let clip_frac_val: f32 = clip_frac.to_scalar()?;

            // Total Loss = Policy Loss + c1 * Value Loss + c2 * Entropy Loss
            let c1_val = (&policy_loss + (value_loss.affine(self.config.c1 as f64, 0.0)?))?;
            let total_loss = (c1_val + (entropy_loss.affine(self.config.c2 as f64, 0.0)?))?;
            let tot_loss_val: f32 = total_loss.to_scalar()?;

            let mut grads = total_loss.backward()?;
            self.clip_grad_norm(&mut grads)?;
            self.optimizer.step(&grads)?;

            last_stats = PPOStats {
                policy_loss: p_loss_val,
                value_loss: v_loss_val,
                entropy_loss: e_loss_val,
                entropy: entropy_val,
                total_loss: tot_loss_val,
                kl: kl_val,
                clip_frac: clip_frac_val,
            };
        }

        Ok(last_stats)
    }

    /// 使用 Mini-Batch 划分更新策略网络
    pub fn update_minibatch(
        &mut self,
        buffer: &RolloutBuffer,
        last_val: f32,
        mini_batch_size: usize,
    ) -> Result<PPOStats> {
        let n = buffer.len();
        if n == 0 {
            return Ok(PPOStats {
                policy_loss: 0.0,
                value_loss: 0.0,
                entropy_loss: 0.0,
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
            });
        }

        if mini_batch_size >= n {
            return self.update(buffer, last_val);
        }

        let (returns, mut advantages) = self.compute_gae(buffer, last_val);

        // Normalize advantages globally
        let mean = advantages.iter().sum::<f32>() / n as f32;
        let variance = advantages.iter().map(|a| (a - mean).powi(2)).sum::<f32>() / n as f32;
        let std = (variance + 1e-8).sqrt();
        for a in advantages.iter_mut() {
            *a = (*a - mean) / std;
        }

        let flat_states: Vec<f32> = buffer.states.iter().flatten().copied().collect();
        let state_dim = buffer.states[0].len();
        let enc_dim = buffer.actions[0].len();
        let flat_actions: Vec<f32> = buffer.actions.iter().flatten().copied().collect();

        let mask_dim = buffer
            .action_masks
            .iter()
            .find_map(|m| m.as_ref().map(|v| v.len()))
            .unwrap_or(0);
        let has_masks = mask_dim > 0 && buffer.action_masks.iter().any(|m| m.is_some());
        let flat_masks: Option<Vec<f32>> = if has_masks {
            let mut fm = Vec::with_capacity(n * mask_dim);
            for m_opt in &buffer.action_masks {
                if let Some(m) = m_opt {
                    for &valid in m {
                        fm.push(if valid { 1.0f32 } else { 0.0f32 });
                    }
                } else {
                    fm.extend(std::iter::repeat_n(1.0f32, mask_dim));
                }
            }
            Some(fm)
        } else {
            None
        };

        let mut last_stats = PPOStats {
            policy_loss: 0.0,
            value_loss: 0.0,
            entropy_loss: 0.0,
            entropy: 0.0,
            total_loss: 0.0,
            kl: 0.0,
            clip_frac: 0.0,
        };

        use rand::seq::SliceRandom;
        let mut rng = rand::rng();

        for _epoch in 0..self.config.ppo_epochs {
            let mut indices: Vec<usize> = (0..n).collect();
            indices.shuffle(&mut rng);

            let mut shuffled_states = Vec::with_capacity(n * state_dim);
            let mut shuffled_actions = Vec::with_capacity(n * enc_dim);
            let mut shuffled_log_probs = Vec::with_capacity(n);
            let mut shuffled_old_values = Vec::with_capacity(n);
            let mut shuffled_returns = Vec::with_capacity(n);
            let mut shuffled_advantages = Vec::with_capacity(n);
            let mut shuffled_masks = if has_masks {
                Some(Vec::with_capacity(n * mask_dim))
            } else {
                None
            };

            for &idx in &indices {
                shuffled_states
                    .extend_from_slice(&flat_states[idx * state_dim..(idx + 1) * state_dim]);
                shuffled_actions
                    .extend_from_slice(&flat_actions[idx * enc_dim..(idx + 1) * enc_dim]);
                shuffled_log_probs.push(buffer.log_probs[idx]);
                shuffled_old_values.push(buffer.values[idx]);
                shuffled_returns.push(returns[idx]);
                shuffled_advantages.push(advantages[idx]);
                if let (Some(sm), Some(fm)) = (&mut shuffled_masks, &flat_masks) {
                    sm.extend_from_slice(&fm[idx * mask_dim..(idx + 1) * mask_dim]);
                }
            }

            let states_tensor = Tensor::from_vec(shuffled_states, (n, state_dim), &self.device)?;
            let actions_tensor = Tensor::from_vec(shuffled_actions, (n, enc_dim), &self.device)?;
            let old_log_probs_tensor = Tensor::from_vec(shuffled_log_probs, (n,), &self.device)?;
            let old_values_tensor = Tensor::from_vec(shuffled_old_values, (n,), &self.device)?;
            let returns_tensor = Tensor::from_vec(shuffled_returns, (n,), &self.device)?;
            let advantages_tensor = Tensor::from_vec(shuffled_advantages, (n,), &self.device)?;
            let masks_tensor = if let Some(sm) = shuffled_masks {
                Some(Tensor::from_vec(sm, (n, mask_dim), &self.device)?)
            } else {
                None
            };

            let mut start_idx = 0;
            while start_idx < n {
                let end_idx = (start_idx + mini_batch_size).min(n);
                let mb_len = end_idx - start_idx;

                let mb_states = states_tensor.narrow(0, start_idx, mb_len)?;
                let mb_actions = actions_tensor.narrow(0, start_idx, mb_len)?;
                let mb_old_log_probs = old_log_probs_tensor.narrow(0, start_idx, mb_len)?;
                let mb_old_values = old_values_tensor.narrow(0, start_idx, mb_len)?;
                let mb_returns = returns_tensor.narrow(0, start_idx, mb_len)?;
                let mb_advantages = advantages_tensor.narrow(0, start_idx, mb_len)?;
                let mb_masks = if let Some(ref mt) = masks_tensor {
                    Some(mt.narrow(0, start_idx, mb_len)?)
                } else {
                    None
                };

                // Mini-Batch 内部优势重归一化 (CleanRL / PPO2 Detail)
                let mb_advantages_norm = if mb_len > 1 {
                    let mean = mb_advantages.mean_all()?;
                    let diff = mb_advantages.broadcast_sub(&mean)?;
                    let var = (&diff * &diff)?.mean_all()?;
                    let std = (var + 1e-8)?.sqrt()?;
                    diff.broadcast_div(&std)?
                } else {
                    mb_advantages
                };

                let (new_log_probs, new_values, entropy) = self.actor_critic.evaluate_actions(
                    &mb_states,
                    &mb_actions,
                    mb_masks.as_ref(),
                )?;

                let log_ratio = (&new_log_probs - &mb_old_log_probs)?;
                let ratio = log_ratio.exp()?;

                let surr1 = (&ratio * &mb_advantages_norm)?;
                let clamped_ratio =
                    ratio.clamp(1.0 - self.config.clip_eps, 1.0 + self.config.clip_eps)?;
                let surr2 = (&clamped_ratio * &mb_advantages_norm)?;

                let policy_loss = surr1.minimum(&surr2)?.neg()?.mean_all()?;

                // Value Loss: PPO2 Clipped Value Loss
                let value_loss = if self.config.clip_vloss {
                    let v_diff = (&new_values - &mb_old_values)?;
                    let v_clamped_diff = v_diff.clamp(-self.config.clip_eps, self.config.clip_eps)?;
                    let v_clipped = (&mb_old_values + &v_clamped_diff)?;
                    let v_loss_unclipped = (&new_values - &mb_returns)?.powf(2.0)?;
                    let v_loss_clipped = (&v_clipped - &mb_returns)?.powf(2.0)?;
                    v_loss_unclipped.maximum(&v_loss_clipped)?.mean_all()?.affine(0.5, 0.0)?
                } else {
                    let val_diff = (&new_values - &mb_returns)?;
                    (&val_diff * &val_diff)?.mean_all()?.affine(0.5, 0.0)?
                };

                let entropy_loss = entropy.neg()?.mean_all()?;

                let kl = (&ratio - 1.0 - &log_ratio)?.mean_all()?;
                let clip_frac = (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                    + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
                .mean_all()?;

                let p_loss_val: f32 = policy_loss.to_scalar()?;
                let v_loss_val: f32 = value_loss.to_scalar()?;
                let e_loss_val: f32 = entropy_loss.to_scalar()?;
                let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
                let kl_val: f32 = kl.to_scalar()?;
                let clip_frac_val: f32 = clip_frac.to_scalar()?;

                let c1_val = (&policy_loss + (value_loss.affine(self.config.c1 as f64, 0.0)?))?;
                let total_loss = (c1_val + (entropy_loss.affine(self.config.c2 as f64, 0.0)?))?;
                let tot_loss_val: f32 = total_loss.to_scalar()?;

                let mut grads = total_loss.backward()?;
                self.clip_grad_norm(&mut grads)?;
                self.optimizer.step(&grads)?;

                last_stats = PPOStats {
                    policy_loss: p_loss_val,
                    value_loss: v_loss_val,
                    entropy_loss: e_loss_val,
                    entropy: entropy_val,
                    total_loss: tot_loss_val,
                    kl: kl_val,
                    clip_frac: clip_frac_val,
                };

                start_idx += mini_batch_size;
            }
        }

        Ok(last_stats)
    }

    /// 多环境独立 GAE 计算 + 全样本 GPU Mini-Batch PPO 更新
    pub fn update_multi_buffer(
        &mut self,
        buffers: &[RolloutBuffer],
        last_vals: &[f32],
        mini_batch_size: usize,
    ) -> Result<PPOStats> {
        let total_n: usize = buffers.iter().map(|b| b.len()).sum();
        if total_n == 0 {
            return Ok(PPOStats {
                policy_loss: 0.0,
                value_loss: 0.0,
                entropy_loss: 0.0,
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
            });
        }

        let state_dim = buffers[0].states[0].len();
        let enc_dim = buffers[0].actions[0].len();

        let mask_dim = buffers
            .iter()
            .find_map(|b| {
                b.action_masks
                    .iter()
                    .find_map(|m| m.as_ref().map(|v| v.len()))
            })
            .unwrap_or(0);
        let has_masks = mask_dim > 0
            && buffers
                .iter()
                .any(|b| b.action_masks.iter().any(|m| m.is_some()));

        let mut all_states = Vec::with_capacity(total_n * state_dim);
        let mut all_actions = Vec::with_capacity(total_n * enc_dim);
        let mut all_old_log_probs = Vec::with_capacity(total_n);
        let mut all_old_values = Vec::with_capacity(total_n);
        let mut all_returns = Vec::with_capacity(total_n);
        let mut all_advantages = Vec::with_capacity(total_n);
        let mut all_masks: Option<Vec<f32>> = if has_masks {
            Some(Vec::with_capacity(total_n * mask_dim))
        } else {
            None
        };

        for (i, buffer) in buffers.iter().enumerate() {
            if buffer.is_empty() {
                continue;
            }
            let last_val = last_vals.get(i).copied().unwrap_or(0.0);
            let (returns, advantages) = self.compute_gae(buffer, last_val);

            for t in 0..buffer.len() {
                all_states.extend_from_slice(&buffer.states[t]);
                all_actions.extend_from_slice(&buffer.actions[t]);
                all_old_log_probs.push(buffer.log_probs[t]);
                all_old_values.push(buffer.values[t]);
                all_returns.push(returns[t]);
                all_advantages.push(advantages[t]);
                if let Some(ref mut am) = all_masks {
                    if let Some(ref m) = buffer.action_masks[t] {
                        for &valid in m {
                            am.push(if valid { 1.0f32 } else { 0.0f32 });
                        }
                    } else {
                        am.extend(std::iter::repeat_n(1.0f32, mask_dim));
                    }
                }
            }
        }

        // Normalize advantages globally across all buffers
        let mean = all_advantages.iter().sum::<f32>() / total_n as f32;
        let variance = all_advantages
            .iter()
            .map(|a| (a - mean).powi(2))
            .sum::<f32>()
            / total_n as f32;
        let std = (variance + 1e-8).sqrt();
        for a in all_advantages.iter_mut() {
            *a = (*a - mean) / std;
        }

        let mut last_stats = PPOStats {
            policy_loss: 0.0,
            value_loss: 0.0,
            entropy_loss: 0.0,
            entropy: 0.0,
            total_loss: 0.0,
            kl: 0.0,
            clip_frac: 0.0,
        };

        use rand::seq::SliceRandom;
        let mut rng = rand::rng();

        for _epoch in 0..self.config.ppo_epochs {
            let mut indices: Vec<usize> = (0..total_n).collect();
            indices.shuffle(&mut rng);

            let mut shuffled_states = Vec::with_capacity(total_n * state_dim);
            let mut shuffled_actions = Vec::with_capacity(total_n * enc_dim);
            let mut shuffled_log_probs = Vec::with_capacity(total_n);
            let mut shuffled_old_values = Vec::with_capacity(total_n);
            let mut shuffled_returns = Vec::with_capacity(total_n);
            let mut shuffled_advantages = Vec::with_capacity(total_n);
            let mut shuffled_masks: Option<Vec<f32>> = if has_masks {
                Some(Vec::with_capacity(total_n * mask_dim))
            } else {
                None
            };

            for &idx in &indices {
                shuffled_states
                    .extend_from_slice(&all_states[idx * state_dim..(idx + 1) * state_dim]);
                shuffled_actions
                    .extend_from_slice(&all_actions[idx * enc_dim..(idx + 1) * enc_dim]);
                shuffled_log_probs.push(all_old_log_probs[idx]);
                shuffled_old_values.push(all_old_values[idx]);
                shuffled_returns.push(all_returns[idx]);
                shuffled_advantages.push(all_advantages[idx]);
                if let (Some(sm), Some(am)) = (&mut shuffled_masks, &all_masks) {
                    sm.extend_from_slice(&am[idx * mask_dim..(idx + 1) * mask_dim]);
                }
            }

            let states_tensor =
                Tensor::from_vec(shuffled_states, (total_n, state_dim), &self.device)?;
            let actions_tensor =
                Tensor::from_vec(shuffled_actions, (total_n, enc_dim), &self.device)?;
            let old_log_probs_tensor =
                Tensor::from_vec(shuffled_log_probs, (total_n,), &self.device)?;
            let old_values_tensor =
                Tensor::from_vec(shuffled_old_values, (total_n,), &self.device)?;
            let returns_tensor = Tensor::from_vec(shuffled_returns, (total_n,), &self.device)?;
            let advantages_tensor =
                Tensor::from_vec(shuffled_advantages, (total_n,), &self.device)?;
            let masks_tensor = if let Some(sm) = shuffled_masks {
                Some(Tensor::from_vec(sm, (total_n, mask_dim), &self.device)?)
            } else {
                None
            };

            let mut start_idx = 0;
            while start_idx < total_n {
                let end_idx = (start_idx + mini_batch_size).min(total_n);
                let mb_len = end_idx - start_idx;

                let mb_states = states_tensor.narrow(0, start_idx, mb_len)?;
                let mb_actions = actions_tensor.narrow(0, start_idx, mb_len)?;
                let mb_old_log_probs = old_log_probs_tensor.narrow(0, start_idx, mb_len)?;
                let mb_old_values = old_values_tensor.narrow(0, start_idx, mb_len)?;
                let mb_returns = returns_tensor.narrow(0, start_idx, mb_len)?;
                let mb_advantages = advantages_tensor.narrow(0, start_idx, mb_len)?;
                let mb_masks = if let Some(ref mt) = masks_tensor {
                    Some(mt.narrow(0, start_idx, mb_len)?)
                } else {
                    None
                };

                // Mini-Batch 内部优势重归一化 (CleanRL / PPO2 Detail)
                let mb_advantages_norm = if mb_len > 1 {
                    let mean = mb_advantages.mean_all()?;
                    let diff = mb_advantages.broadcast_sub(&mean)?;
                    let var = (&diff * &diff)?.mean_all()?;
                    let std = (var + 1e-8)?.sqrt()?;
                    diff.broadcast_div(&std)?
                } else {
                    mb_advantages
                };

                let (new_log_probs, new_values, entropy) = self.actor_critic.evaluate_actions(
                    &mb_states,
                    &mb_actions,
                    mb_masks.as_ref(),
                )?;

                let log_ratio = (&new_log_probs - &mb_old_log_probs)?;
                let ratio = log_ratio.exp()?;

                let surr1 = (&ratio * &mb_advantages_norm)?;
                let clamped_ratio =
                    ratio.clamp(1.0 - self.config.clip_eps, 1.0 + self.config.clip_eps)?;
                let surr2 = (&clamped_ratio * &mb_advantages_norm)?;

                let policy_loss = surr1.minimum(&surr2)?.neg()?.mean_all()?;

                // Value Loss: PPO2 Clipped Value Loss
                let value_loss = if self.config.clip_vloss {
                    let v_diff = (&new_values - &mb_old_values)?;
                    let v_clamped_diff = v_diff.clamp(-self.config.clip_eps, self.config.clip_eps)?;
                    let v_clipped = (&mb_old_values + &v_clamped_diff)?;
                    let v_loss_unclipped = (&new_values - &mb_returns)?.powf(2.0)?;
                    let v_loss_clipped = (&v_clipped - &mb_returns)?.powf(2.0)?;
                    v_loss_unclipped.maximum(&v_loss_clipped)?.mean_all()?.affine(0.5, 0.0)?
                } else {
                    let val_diff = (&new_values - &mb_returns)?;
                    (&val_diff * &val_diff)?.mean_all()?.affine(0.5, 0.0)?
                };

                let entropy_loss = entropy.neg()?.mean_all()?;

                let kl = (&ratio - 1.0 - &log_ratio)?.mean_all()?;
                let clip_frac = (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                    + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
                .mean_all()?;

                let p_loss_val: f32 = policy_loss.to_scalar()?;
                let v_loss_val: f32 = value_loss.to_scalar()?;
                let e_loss_val: f32 = entropy_loss.to_scalar()?;
                let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
                let kl_val: f32 = kl.to_scalar()?;
                let clip_frac_val: f32 = clip_frac.to_scalar()?;

                let c1_val = (&policy_loss + (value_loss.affine(self.config.c1 as f64, 0.0)?))?;
                let total_loss = (c1_val + (entropy_loss.affine(self.config.c2 as f64, 0.0)?))?;
                let tot_loss_val: f32 = total_loss.to_scalar()?;

                let mut grads = total_loss.backward()?;
                self.clip_grad_norm(&mut grads)?;
                self.optimizer.step(&grads)?;

                last_stats = PPOStats {
                    policy_loss: p_loss_val,
                    value_loss: v_loss_val,
                    entropy_loss: e_loss_val,
                    entropy: entropy_val,
                    total_loss: tot_loss_val,
                    kl: kl_val,
                    clip_frac: clip_frac_val,
                };

                start_idx += mini_batch_size;
            }
        }

        Ok(last_stats)
    }
}

fn build_masks_tensor(masks: &[Option<Vec<bool>>], device: &Device) -> Result<Option<Tensor>> {
    if !masks.iter().any(|m| m.is_some()) {
        return Ok(None);
    }
    let n = masks.len();
    let dim = masks
        .iter()
        .find_map(|m| m.as_ref().map(|v| v.len()))
        .unwrap_or(0);
    if dim == 0 {
        return Ok(None);
    }
    let mut flat = Vec::with_capacity(n * dim);
    for m_opt in masks {
        if let Some(m) = m_opt {
            for &valid in m {
                flat.push(if valid { 1.0f32 } else { 0.0f32 });
            }
        } else {
            flat.extend(std::iter::repeat_n(1.0f32, dim));
        }
    }
    Tensor::from_vec(flat, (n, dim), device).map(Some)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn save_load_roundtrip() -> Result<()> {
        let state_dim = 17;
        let hidden_dim = 64;
        let action_dim = 9;
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            ActionSpace::Discrete(action_dim),
            config.clone(),
            device.clone(),
        )?;

        let obs_vec: Vec<f32> = (0..state_dim).map(|i| i as f32 * 0.1).collect();
        let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device)?;
        let (probs_before, _) = agent.actor_critic.forward(&state)?;
        let probs_before_vec: Vec<f32> = probs_before.squeeze(0)?.to_vec1()?;

        let tmp_dir = std::env::temp_dir().join("moon_lol_test");
        std::fs::create_dir_all(&tmp_dir).ok();
        let save_path = tmp_dir.join("test_ckpt.safetensors");
        let _ = std::fs::remove_file(&save_path);
        agent.save(&save_path)?;
        assert!(save_path.exists());
        assert!(save_path.metadata().unwrap().len() > 0);

        let loaded = PPOAgent::load(
            state_dim,
            hidden_dim,
            ActionSpace::Discrete(action_dim),
            config.clone(),
            device.clone(),
            &save_path,
        )?;
        let (probs_after, _) = loaded.actor_critic.forward(&state)?;
        let probs_after_vec: Vec<f32> = probs_after.squeeze(0)?.to_vec1()?;

        for (i, (b, a)) in probs_before_vec
            .iter()
            .zip(probs_after_vec.iter())
            .enumerate()
        {
            assert!(
                (b - a).abs() < 1e-4,
                "action {} prob mismatch: before={}, after={}",
                i,
                b,
                a
            );
        }

        let _ = std::fs::remove_file(&save_path);
        Ok(())
    }

    #[test]
    fn load_empty_file_fails() {
        let tmp_dir = std::env::temp_dir().join("moon_lol_test_empty");
        std::fs::create_dir_all(&tmp_dir).ok();
        let empty_path = tmp_dir.join("empty.safetensors");
        std::fs::write(&empty_path, []).ok();

        let result = PPOAgent::load(
            17,
            64,
            ActionSpace::Discrete(9),
            PPOConfig::default(),
            Device::Cpu,
            &empty_path,
        );
        assert!(result.is_err());

        let _ = std::fs::remove_file(&empty_path);
    }

    #[test]
    fn load_custom_hidden_dim_auto_detect() -> Result<()> {
        let state_dim = 17;
        let hidden_dim = 256;
        let action_dim = 5;
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            ActionSpace::Discrete(action_dim),
            config.clone(),
            device.clone(),
        )?;

        let tmp_dir = std::env::temp_dir().join("moon_lol_test_256");
        std::fs::create_dir_all(&tmp_dir).ok();
        let save_path = tmp_dir.join("test_ckpt_256.safetensors");
        let _ = std::fs::remove_file(&save_path);
        agent.save(&save_path)?;

        // Load with dummy hidden_dim=64, it should auto-detect 256 from safetensors file
        let loaded = PPOAgent::load(
            state_dim,
            64,
            ActionSpace::Discrete(action_dim),
            config,
            device.clone(),
            &save_path,
        )?;
        let state = Tensor::zeros((1, state_dim), DType::F32, &device)?;
        let (probs, val) = loaded.actor_critic.forward(&state)?;
        assert_eq!(probs.dim(1)?, action_dim);
        assert_eq!(val.dim(1)?, 1);

        let _ = std::fs::remove_file(&save_path);
        Ok(())
    }

    #[test]
    fn load_nonexistent_file_fails() {
        let result = PPOAgent::load(
            17,
            64,
            ActionSpace::Discrete(9),
            PPOConfig::default(),
            Device::Cpu,
            &PathBuf::from("/nonexistent/path/checkpoint.safetensors"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn hybrid_ppo_smoke() -> Result<()> {
        let state_dim = 9;
        let hidden_dim = 32;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 2,
        };
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
        )?;

        let mut buffer = RolloutBuffer::new();
        for _ in 0..8 {
            let obs_vec: Vec<f32> = (0..state_dim).map(|i| i as f32 * 0.1).collect();
            let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device)?;
            let (encoded, log_prob, value) = agent.actor_critic.sample_action(&state, None)?;
            assert_eq!(encoded.len(), 3, "hybrid 编码应为 [move_x, move_z, attack]");
            buffer.push_unmasked(obs_vec, encoded, log_prob, 0.1, value, false);
        }

        let stats = agent.update(&buffer, 0.0)?;
        assert!(stats.policy_loss.is_finite(), "policy_loss 应为有限值");
        assert!(stats.value_loss.is_finite(), "value_loss 应为有限值");

        // 保存/加载混合 checkpoint
        let tmp_dir = std::env::temp_dir().join("moon_lol_test_hybrid");
        std::fs::create_dir_all(&tmp_dir).ok();
        let save_path = tmp_dir.join("hybrid_ckpt.safetensors");
        let _ = std::fs::remove_file(&save_path);
        agent.save(&save_path)?;
        let loaded = PPOAgent::load(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            &save_path,
        )?;
        let obs_vec: Vec<f32> = (0..state_dim).map(|i| i as f32 * 0.1).collect();
        let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &loaded.device())?;
        let (encoded_after, _, _) = loaded.actor_critic.sample_action(&state, None)?;
        assert_eq!(encoded_after.len(), 3);

        let _ = std::fs::remove_file(&save_path);
        Ok(())
    }

    #[test]
    fn hybrid_ppo_fiora_v2_smoke() -> Result<()> {
        let state_dim = 33;
        let hidden_dim = 64;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 7,
        };
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
        )?;

        let mut buffer = RolloutBuffer::new();
        for step in 0..16 {
            let mut obs_vec: Vec<f32> = (0..state_dim).map(|i| (i as f32 * 0.05).sin()).collect();
            obs_vec[16] = if step % 2 == 0 { 1.5 } else { 3.0 };
            let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device)?;
            let mask = Some(vec![true, true, step % 2 == 0, true, true, true, true]);
            let (encoded, log_prob, value) =
                agent.actor_critic.sample_action(&state, mask.as_deref())?;
            assert_eq!(
                encoded.len(),
                3,
                "FioraV2 hybrid 动作编码应为 [offset_x, offset_z, discrete_idx]"
            );
            let disc_idx = encoded[2] as usize;
            assert!(disc_idx < 7, "离散动作索引应在 [0, 6] 范围内");
            buffer.push(obs_vec, encoded, log_prob, 0.25, value, false, mask);
        }

        let stats = agent.update(&buffer, 0.0)?;
        assert!(
            stats.policy_loss.is_finite(),
            "V2 policy_loss 应为有效有限值"
        );
        assert!(stats.value_loss.is_finite(), "V2 value_loss 应为有效有限值");
        assert!(
            stats.entropy_loss.is_finite(),
            "V2 entropy_loss 应为有效有限值"
        );

        // 验证批量采样 sample_batch
        let states = Tensor::zeros((4, state_dim), DType::F32, &device)?;
        let batch_samples = agent.actor_critic.sample_batch(&states, None)?;
        assert_eq!(batch_samples.len(), 4);

        // 验证策略可视化显示
        let dummy_state = Tensor::zeros((1, state_dim), DType::F32, &device)?;
        let labels = ["NoOp", "Move", "Attack", "Q", "E", "R", "Flash"];
        let display = agent
            .actor_critic
            .policy_display_real(&dummy_state, None, &labels)?;
        match display {
            lol_rl_protocol::PolicyDisplay::HybridMulti {
                continuous_means,
                discrete_probs,
            } => {
                assert_eq!(continuous_means.len(), 2);
                assert_eq!(discrete_probs.len(), 7);
                let sum_prob: f32 = discrete_probs.iter().map(|p| p.prob).sum();
                assert!((sum_prob - 1.0).abs() < 1e-3, "离散概率之和应为 1.0");
            }
            other => panic!("预期返回 PolicyDisplay::HybridMulti，实际为 {:?}", other),
        }

        // 验证多 Buffer 掩码 Mini-Batch 更新
        let stats_multi = agent.update_multi_buffer(&[buffer], &[0.0], 8)?;
        assert!(
            stats_multi.policy_loss.is_finite(),
            "update_multi_buffer policy_loss 应为有效有限值"
        );
        assert!(
            stats_multi.value_loss.is_finite(),
            "update_multi_buffer value_loss 应为有效有限值"
        );

        Ok(())
    }

    #[test]
    fn test_orthogonal_weight_properties() {
        use crate::policy::orthogonal_weight;
        let out_dim = 16;
        let in_dim = 32;
        let gain = 1.414f32;
        let w = orthogonal_weight(out_dim, in_dim, gain);
        assert_eq!(w.len(), out_dim * in_dim);

        // 验证行向量正交性：W * W^T ≈ gain^2 * I
        for r1 in 0..out_dim {
            for r2 in 0..out_dim {
                let dot: f32 = (0..in_dim)
                    .map(|c| w[r1 * in_dim + c] * w[r2 * in_dim + c])
                    .sum();
                if r1 == r2 {
                    let expected = gain * gain;
                    assert!(
                        (dot - expected).abs() < 1e-3,
                        "对角元素 dot ({dot}) 应接近 gain^2 ({expected})"
                    );
                } else {
                    assert!(
                        dot.abs() < 1e-3,
                        "非对角元素 dot ({dot}) 应接近 0 (正交)"
                    );
                }
            }
        }
    }

    #[test]
    fn test_industrial_ppo_clip_vloss_and_grad_norm() -> Result<()> {
        let state_dim = 8;
        let hidden_dim = 32;
        let action_space = ActionSpace::Discrete(4);
        let mut config = PPOConfig::default();
        config.clip_vloss = true;
        config.max_grad_norm = 0.5;
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(state_dim, hidden_dim, action_space, config, device)?;
        agent.set_lr(1e-4)?;

        let mut buffer = RolloutBuffer::new();
        for _ in 0..10 {
            let obs = vec![0.5f32; state_dim];
            buffer.push_unmasked(obs, vec![0.0], -0.5, 1.0, 0.2, false);
        }

        let stats = agent.update(&buffer, 0.0)?;
        assert!(stats.policy_loss.is_finite());
        assert!(stats.value_loss.is_finite());
        assert!(stats.total_loss.is_finite());
        assert!(stats.kl.is_finite());
        assert!(stats.clip_frac >= 0.0 && stats.clip_frac <= 1.0);

        Ok(())
    }
}
