use candle_core::{DType, Result, Tensor};
use candle_nn::Optimizer;
use lol_rl_protocol::PolicyBackbone;
use rand::seq::SliceRandom;

use crate::algo::buffer::RolloutBuffer;
use crate::algo::ppo::agent::PPOAgent;
use crate::algo::ppo::config::PPOStats;

impl PPOAgent {
    /// Update policy using buffer data
    pub fn update(&mut self, buffer: &RolloutBuffer, last_val: f32) -> Result<PPOStats> {
        let n = buffer.len();
        if n == 0 {
            return Ok(PPOStats {
                policy_loss: 0.0,
                value_loss: 0.0,
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
            });
        }
        self.update_multi_buffer(std::slice::from_ref(buffer), &[last_val], n.min(64))
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
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
            });
        }
        self.update_multi_buffer(std::slice::from_ref(buffer), &[last_val], mini_batch_size)
    }

    /// 多环境独立 GAE 计算 + 全样本 GPU Mini-Batch PPO 更新（支持 MLP 无状态打乱与 Mamba 时序切片）
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
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
            });
        }

        // 取第一个非空 buffer 推断维度（首个 buffer 可能因 Worker 异常为空）
        let first_non_empty = buffers
            .iter()
            .find(|b| !b.is_empty())
            .expect("total_n>0 必有非空 buffer");
        let state_dim = first_non_empty.states[0].len();
        let enc_dim = first_non_empty.actions[0].len();

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

        let is_mamba = self.actor_critic.backbone().backbone_type() == PolicyBackbone::Mamba;

        let mut rng = rand::rng();

        let mut last_stats = PPOStats {
            policy_loss: 0.0,
            value_loss: 0.0,
            entropy: 0.0,
            total_loss: 0.0,
            kl: 0.0,
            clip_frac: 0.0,
        };

        // ════════════════════════════════════════════════════════════════
        // 路径 A：Mamba 时序状态空间模型（Chunk-based Recurrent PPO 时序切片训练）
        // ════════════════════════════════════════════════════════════════
        if is_mamba {
            let chunk_len = 16.min(total_n).max(1);
            struct TrajChunk {
                states: Vec<f32>,
                actions: Vec<f32>,
                old_log_probs: Vec<f32>,
                old_values: Vec<f32>,
                returns: Vec<f32>,
                advantages: Vec<f32>,
                masks: Option<Vec<f32>>,
            }

            let mut chunks = Vec::new();
            for (i, buffer) in buffers.iter().enumerate() {
                if buffer.is_empty() {
                    continue;
                }
                let last_val = last_vals.get(i).copied().unwrap_or(0.0);
                let (returns, advantages) = self.compute_gae(buffer, last_val);
                let b_len = buffer.len();
                let mut start = 0;
                while start < b_len {
                    let end = (start + chunk_len).min(b_len);
                    let cl = end - start;
                    if cl == 0 {
                        break;
                    }
                    let mut c_states = Vec::with_capacity(chunk_len * state_dim);
                    let mut c_actions = Vec::with_capacity(chunk_len * enc_dim);
                    let mut c_log_probs = Vec::with_capacity(chunk_len);
                    let mut c_old_values = Vec::with_capacity(chunk_len);
                    let mut c_returns = Vec::with_capacity(chunk_len);
                    let mut c_advantages = Vec::with_capacity(chunk_len);
                    let mut c_masks = if has_masks {
                        Some(Vec::with_capacity(chunk_len * mask_dim))
                    } else {
                        None
                    };

                    for t in start..end {
                        c_states.extend_from_slice(&buffer.states[t]);
                        c_actions.extend_from_slice(&buffer.actions[t]);
                        c_log_probs.push(buffer.log_probs[t]);
                        c_old_values.push(buffer.values[t]);
                        c_returns.push(returns[t]);
                        c_advantages.push(advantages[t]);
                        if let Some(ref mut cm) = c_masks {
                            if let Some(ref m) = buffer.action_masks[t] {
                                for &valid in m {
                                    cm.push(if valid { 1.0f32 } else { 0.0f32 });
                                }
                            } else {
                                cm.extend(std::iter::repeat_n(1.0f32, mask_dim));
                            }
                        }
                    }

                    // 尾部不足 chunk_len 时做同状态填充，保证 3D Tensor 规整，advantages 设 0 避免梯度干扰
                    if cl < chunk_len {
                        let pad_count = chunk_len - cl;
                        let last_state = &buffer.states[end - 1];
                        for _ in 0..pad_count {
                            c_states.extend_from_slice(last_state);
                            c_actions.extend_from_slice(&buffer.actions[end - 1]);
                            c_log_probs.push(buffer.log_probs[end - 1]);
                            c_old_values.push(buffer.values[end - 1]);
                            c_returns.push(returns[end - 1]);
                            c_advantages.push(0.0);
                            if let Some(ref mut cm) = c_masks {
                                cm.extend(std::iter::repeat_n(1.0f32, mask_dim));
                            }
                        }
                    }

                    chunks.push(TrajChunk {
                        states: c_states,
                        actions: c_actions,
                        old_log_probs: c_log_probs,
                        old_values: c_old_values,
                        returns: c_returns,
                        advantages: c_advantages,
                        masks: c_masks,
                    });
                    start += chunk_len;
                }
            }

            let num_chunks = chunks.len();
            if num_chunks == 0 {
                return Ok(last_stats);
            }

            let chunks_per_mb = (mini_batch_size / chunk_len).max(1).min(num_chunks);

            for _epoch in 0..self.config.ppo_epochs {
                let mut chunk_indices: Vec<usize> = (0..num_chunks).collect();
                chunk_indices.shuffle(&mut rng);

                let mut start_c = 0;
                while start_c < num_chunks {
                    let end_c = (start_c + chunks_per_mb).min(num_chunks);
                    let m = end_c - start_c;
                    let total_steps_mb = m * chunk_len;

                    let mut mb_states_vec = Vec::with_capacity(total_steps_mb * state_dim);
                    let mut mb_actions_vec = Vec::with_capacity(total_steps_mb * enc_dim);
                    let mut mb_old_log_probs_vec = Vec::with_capacity(total_steps_mb);
                    let mut mb_old_values_vec = Vec::with_capacity(total_steps_mb);
                    let mut mb_returns_vec = Vec::with_capacity(total_steps_mb);
                    let mut mb_advantages_vec = Vec::with_capacity(total_steps_mb);
                    let mut mb_masks_vec = if has_masks {
                        Some(Vec::with_capacity(total_steps_mb * mask_dim))
                    } else {
                        None
                    };

                    for &ci in &chunk_indices[start_c..end_c] {
                        let c = &chunks[ci];
                        mb_states_vec.extend_from_slice(&c.states);
                        mb_actions_vec.extend_from_slice(&c.actions);
                        mb_old_log_probs_vec.extend_from_slice(&c.old_log_probs);
                        mb_old_values_vec.extend_from_slice(&c.old_values);
                        mb_returns_vec.extend_from_slice(&c.returns);
                        mb_advantages_vec.extend_from_slice(&c.advantages);
                        if let (Some(mbm), Some(cm)) = (&mut mb_masks_vec, &c.masks) {
                            mbm.extend_from_slice(cm);
                        }
                    }

                    let mb_states_3d =
                        Tensor::from_vec(mb_states_vec, (m, chunk_len, state_dim), &self.device)?;
                    let mb_actions =
                        Tensor::from_vec(mb_actions_vec, (total_steps_mb, enc_dim), &self.device)?;
                    let mb_old_log_probs =
                        Tensor::from_vec(mb_old_log_probs_vec, (total_steps_mb,), &self.device)?;
                    let mb_old_values =
                        Tensor::from_vec(mb_old_values_vec, (total_steps_mb,), &self.device)?;
                    let mb_returns =
                        Tensor::from_vec(mb_returns_vec, (total_steps_mb,), &self.device)?;
                    let mb_advantages =
                        Tensor::from_vec(mb_advantages_vec, (total_steps_mb,), &self.device)?;
                    let mb_masks = if let Some(mbm) = mb_masks_vec {
                        Some(Tensor::from_vec(
                            mbm,
                            (total_steps_mb, mask_dim),
                            &self.device,
                        )?)
                    } else {
                        None
                    };

                    let mb_advantages_norm = if total_steps_mb > 1 {
                        let mean = mb_advantages.mean_all()?;
                        let diff = mb_advantages.broadcast_sub(&mean)?;
                        let var = (&diff * &diff)?.mean_all()?;
                        let std = (var + 1e-8)?.sqrt()?;
                        diff.broadcast_div(&std)?
                    } else {
                        mb_advantages
                    };

                    let (new_log_probs, new_values, entropy) = self.actor_critic.evaluate_actions(
                        &mb_states_3d,
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

                    let value_loss = if self.config.clip_vloss {
                        let v_diff = (&new_values - &mb_old_values)?;
                        let v_clamped_diff =
                            v_diff.clamp(-self.config.clip_eps, self.config.clip_eps)?;
                        let v_clipped = (&mb_old_values + &v_clamped_diff)?;
                        let v_loss_unclipped = (&new_values - &mb_returns)?.powf(2.0)?;
                        let v_loss_clipped = (&v_clipped - &mb_returns)?.powf(2.0)?;
                        v_loss_unclipped
                            .maximum(&v_loss_clipped)?
                            .mean_all()?
                            .affine(0.5, 0.0)?
                    } else {
                        let val_diff = (&new_values - &mb_returns)?;
                        (&val_diff * &val_diff)?.mean_all()?.affine(0.5, 0.0)?
                    };

                    let kl = (&ratio - 1.0 - &log_ratio)?.mean_all()?;
                    let clip_frac =
                        (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                            + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
                        .mean_all()?;

                    let p_loss_val: f32 = policy_loss.to_scalar()?;
                    let v_loss_val: f32 = value_loss.to_scalar()?;
                    let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
                    let kl_val: f32 = kl.to_scalar()?;
                    let clip_frac_val: f32 = clip_frac.to_scalar()?;

                    let total_loss =
                        (&policy_loss + (value_loss.affine(self.config.c1 as f64, 0.0)?))?;
                    let tot_loss_val: f32 = total_loss.to_scalar()?;

                    let mut grads = total_loss.backward()?;
                    self.clip_grad_norm(&mut grads)?;
                    self.optimizer.step(&grads)?;

                    last_stats = PPOStats {
                        policy_loss: p_loss_val,
                        value_loss: v_loss_val,
                        entropy: entropy_val,
                        total_loss: tot_loss_val,
                        kl: kl_val,
                        clip_frac: clip_frac_val,
                    };

                    start_c += chunks_per_mb;
                }
            }

            return Ok(last_stats);
        }

        // ════════════════════════════════════════════════════════════════
        // 路径 B：MLP 无状态纯前馈网络（Transition-level 全局打乱与 2D Mini-Batch 极速更新）
        // ════════════════════════════════════════════════════════════════
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
            entropy: 0.0,
            total_loss: 0.0,
            kl: 0.0,
            clip_frac: 0.0,
        };

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
                    let v_clamped_diff =
                        v_diff.clamp(-self.config.clip_eps, self.config.clip_eps)?;
                    let v_clipped = (&mb_old_values + &v_clamped_diff)?;
                    let v_loss_unclipped = (&new_values - &mb_returns)?.powf(2.0)?;
                    let v_loss_clipped = (&v_clipped - &mb_returns)?.powf(2.0)?;
                    v_loss_unclipped
                        .maximum(&v_loss_clipped)?
                        .mean_all()?
                        .affine(0.5, 0.0)?
                } else {
                    let val_diff = (&new_values - &mb_returns)?;
                    (&val_diff * &val_diff)?.mean_all()?.affine(0.5, 0.0)?
                };

                let kl = (&ratio - 1.0 - &log_ratio)?.mean_all()?;
                let clip_frac = (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                    + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
                .mean_all()?;

                let p_loss_val: f32 = policy_loss.to_scalar()?;
                let v_loss_val: f32 = value_loss.to_scalar()?;
                let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
                let kl_val: f32 = kl.to_scalar()?;
                let clip_frac_val: f32 = clip_frac.to_scalar()?;

                let total_loss =
                    (&policy_loss + (value_loss.affine(self.config.c1 as f64, 0.0)?))?;
                let tot_loss_val: f32 = total_loss.to_scalar()?;

                let mut grads = total_loss.backward()?;
                self.clip_grad_norm(&mut grads)?;
                self.optimizer.step(&grads)?;

                last_stats = PPOStats {
                    policy_loss: p_loss_val,
                    value_loss: v_loss_val,
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
