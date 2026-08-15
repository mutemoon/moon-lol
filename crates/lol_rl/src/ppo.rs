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
    ) {
        self.states.push(state);
        self.actions.push(action);
        self.log_probs.push(log_prob);
        self.rewards.push(reward);
        self.values.push(value);
        self.dones.push(done);
    }

    pub fn clear(&mut self) {
        self.states.clear();
        self.actions.clear();
        self.log_probs.clear();
        self.rewards.clear();
        self.values.clear();
        self.dones.clear();
    }

    pub fn len(&self) -> usize {
        self.states.len()
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
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let actor_critic = ActorCritic::new(state_dim, hidden_dim, action_space, vb)?;

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

        // 校验 checkpoint 的动作空间结构与请求一致：依据是否存在 log_std（连续/混合）与
        // attack_head.weight（混合）张量判定，不依赖层数/维度，避免网络结构演进时误报。
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

        // Normalize advantages
        let mean = advantages.iter().sum::<f32>() / n as f32;
        let variance = advantages.iter().map(|a| (a - mean).powi(2)).sum::<f32>() / n as f32;
        let std = (variance + 1e-8).sqrt();

        let norm_advantages: Vec<f32> = advantages.iter().map(|a| (a - mean) / std).collect();

        (returns, norm_advantages)
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

        let (returns, advantages) = self.compute_gae(buffer, last_val);

        // Convert buffer to tensors
        let flat_states: Vec<f32> = buffer.states.iter().flatten().copied().collect();
        let state_dim = buffer.states[0].len();

        let states_tensor = Tensor::from_vec(flat_states, (n, state_dim), &self.device)?;
        let enc_dim = buffer.actions[0].len();
        let flat_actions: Vec<f32> = buffer.actions.iter().flatten().copied().collect();
        let actions_tensor = Tensor::from_vec(flat_actions, (n, enc_dim), &self.device)?;
        let old_log_probs_tensor = Tensor::from_vec(buffer.log_probs.clone(), (n,), &self.device)?;
        let returns_tensor = Tensor::from_vec(returns, (n,), &self.device)?;
        let advantages_tensor = Tensor::from_vec(advantages, (n,), &self.device)?;

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
            let (new_log_probs, new_values, entropy) = self
                .actor_critic
                .evaluate_actions(&states_tensor, &actions_tensor)?;

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

            // Value Loss = 0.5 * mean((new_values - returns)^2)
            let val_diff = (&new_values - &returns_tensor)?;
            let value_loss = (&val_diff * &val_diff)?.mean_all()?.affine(0.5, 0.0)?;

            // Entropy Loss = - mean(entropy)
            let entropy_loss = entropy.neg()?.mean_all()?;

            // KL divergence (K1 estimator: ratio - 1 - log(ratio))
            let kl = (&ratio - 1.0 - &log_ratio)?.mean_all()?;

            // clip_frac：ratio 超出 [1-eps, 1+eps] 的元素占比（训练健康度信号）
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

            let grads = total_loss.backward()?;
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

    /// 使用 Mini-Batch 划分更新策略网络（充分发挥 GPU 吞吐量和梯度估计稳定性）
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

        let (returns, advantages) = self.compute_gae(buffer, last_val);

        let flat_states: Vec<f32> = buffer.states.iter().flatten().copied().collect();
        let state_dim = buffer.states[0].len();
        let states_tensor = Tensor::from_vec(flat_states, (n, state_dim), &self.device)?;

        let enc_dim = buffer.actions[0].len();
        let flat_actions: Vec<f32> = buffer.actions.iter().flatten().copied().collect();
        let actions_tensor = Tensor::from_vec(flat_actions, (n, enc_dim), &self.device)?;

        let old_log_probs_tensor = Tensor::from_vec(buffer.log_probs.clone(), (n,), &self.device)?;
        let returns_tensor = Tensor::from_vec(returns, (n,), &self.device)?;
        let advantages_tensor = Tensor::from_vec(advantages, (n,), &self.device)?;

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
            let mut start_idx = 0;
            while start_idx < n {
                let end_idx = (start_idx + mini_batch_size).min(n);
                let mb_len = end_idx - start_idx;

                let mb_states = states_tensor.narrow(0, start_idx, mb_len)?;
                let mb_actions = actions_tensor.narrow(0, start_idx, mb_len)?;
                let mb_old_log_probs = old_log_probs_tensor.narrow(0, start_idx, mb_len)?;
                let mb_returns = returns_tensor.narrow(0, start_idx, mb_len)?;
                let mb_advantages = advantages_tensor.narrow(0, start_idx, mb_len)?;

                let (new_log_probs, new_values, entropy) = self
                    .actor_critic
                    .evaluate_actions(&mb_states, &mb_actions)?;

                let log_ratio = (&new_log_probs - &mb_old_log_probs)?;
                let ratio = log_ratio.exp()?;

                let surr1 = (&ratio * &mb_advantages)?;
                let clamped_ratio =
                    ratio.clamp(1.0 - self.config.clip_eps, 1.0 + self.config.clip_eps)?;
                let surr2 = (&clamped_ratio * &mb_advantages)?;

                let policy_loss = surr1.minimum(&surr2)?.neg()?.mean_all()?;
                let val_diff = (&new_values - &mb_returns)?;
                let value_loss = (&val_diff * &val_diff)?.mean_all()?.affine(0.5, 0.0)?;
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

                let grads = total_loss.backward()?;
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
            let (encoded, log_prob, value) = agent.actor_critic.sample_action(&state, &obs_vec)?;
            assert_eq!(encoded.len(), 3, "hybrid 编码应为 [move_x, move_z, attack]");
            buffer.push(obs_vec, encoded, log_prob, 0.1, value, false);
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
        let (encoded_after, _, _) = loaded.actor_critic.sample_action(&state, &obs_vec)?;
        assert_eq!(encoded_after.len(), 3);

        let _ = std::fs::remove_file(&save_path);
        Ok(())
    }
}
