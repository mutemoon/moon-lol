use candle_core::{DType, Result, Tensor};
use candle_nn::Optimizer;
use lol_rl_protocol::PolicyBackbone;
use rand::seq::SliceRandom;

use crate::algo::buffer::RolloutBuffer;
use crate::algo::grpo::agent::GRPOAgent;
use crate::algo::grpo::config::GRPOStats;

impl GRPOAgent {
    /// 多环境 GRPO 相对优势更新（无 Critic 网络，纯 Actor + 策略熵/KL 正则化）
    pub fn update_multi_buffer(
        &mut self,
        buffers: &[RolloutBuffer],
        mini_batch_size: usize,
    ) -> Result<GRPOStats> {
        let total_n: usize = buffers.iter().map(|b| b.len()).sum();
        if total_n == 0 {
            return Ok(GRPOStats {
                policy_loss: 0.0,
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
                group_reward_mean: 0.0,
                group_reward_std: 0.0,
            });
        }

        let (group_advantages, grp_mean, grp_std) =
            self.compute_group_advantages(buffers, self.config.group_size);

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

        let is_mamba = self.policy.backbone().backbone_type() == PolicyBackbone::Mamba;

        let mut rng = rand::rng();

        let mut last_stats = GRPOStats {
            policy_loss: 0.0,
            entropy: 0.0,
            total_loss: 0.0,
            kl: 0.0,
            clip_frac: 0.0,
            group_reward_mean: grp_mean,
            group_reward_std: grp_std,
        };

        // ════════════════════════════════════════════════════════════════
        // 路径 A：Mamba 时序状态空间模型（Chunk-based GRPO 时序切片训练）
        // ════════════════════════════════════════════════════════════════
        if is_mamba {
            let chunk_len = 16.min(total_n).max(1);
            struct TrajChunk {
                states: Vec<f32>,
                actions: Vec<f32>,
                old_log_probs: Vec<f32>,
                advantages: Vec<f32>,
                masks: Option<Vec<f32>>,
            }

            let mut chunks = Vec::new();
            for (i, buffer) in buffers.iter().enumerate() {
                if buffer.is_empty() {
                    continue;
                }
                let advantages = &group_advantages[i];
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

                    // 尾部不足 chunk_len 时做填充
                    if cl < chunk_len {
                        let pad_count = chunk_len - cl;
                        let last_state = &buffer.states[end - 1];
                        for _ in 0..pad_count {
                            c_states.extend_from_slice(last_state);
                            c_actions.extend_from_slice(&buffer.actions[end - 1]);
                            c_log_probs.push(buffer.log_probs[end - 1]);
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

            for _epoch in 0..self.config.grpo_epochs {
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

                    let (new_log_probs, entropy) = self.policy.evaluate_actions(
                        &mb_states_3d,
                        &mb_actions,
                        mb_masks.as_ref(),
                    )?;

                    let log_ratio = (&new_log_probs - &mb_old_log_probs)?;
                    let ratio = log_ratio.exp()?;

                    let surr1 = (&ratio * &mb_advantages)?;
                    let clamped_ratio =
                        ratio.clamp(1.0 - self.config.clip_eps, 1.0 + self.config.clip_eps)?;
                    let surr2 = (&clamped_ratio * &mb_advantages)?;

                    // GRPO 策略损失（Clipped Surrogate）
                    let policy_loss = surr1.minimum(&surr2)?.neg()?.mean_all()?;

                    // KL 散度与指标监控
                    let kl_loss = (&ratio - 1.0 - &log_ratio)?.mean_all()?;
                    let clip_frac =
                        (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                            + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
                        .mean_all()?;

                    let p_loss_val: f32 = policy_loss.to_scalar()?;
                    let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
                    let kl_val: f32 = kl_loss.to_scalar()?;
                    let clip_frac_val: f32 = clip_frac.to_scalar()?;

                    let mut grads = policy_loss.backward()?;
                    self.clip_grad_norm(&mut grads)?;
                    self.optimizer.step(&grads)?;

                    last_stats = GRPOStats {
                        policy_loss: p_loss_val,
                        entropy: entropy_val,
                        total_loss: p_loss_val,
                        kl: kl_val,
                        clip_frac: clip_frac_val,
                        group_reward_mean: grp_mean,
                        group_reward_std: grp_std,
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
        let mut all_advs = Vec::with_capacity(total_n);
        let mut all_masks: Option<Vec<f32>> = if has_masks {
            Some(Vec::with_capacity(total_n * mask_dim))
        } else {
            None
        };

        for (i, buffer) in buffers.iter().enumerate() {
            if buffer.is_empty() {
                continue;
            }
            let advantages = &group_advantages[i];
            for t in 0..buffer.len() {
                all_states.extend_from_slice(&buffer.states[t]);
                all_actions.extend_from_slice(&buffer.actions[t]);
                all_old_log_probs.push(buffer.log_probs[t]);
                all_advs.push(advantages[t]);
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

        let mb_size = mini_batch_size.min(total_n).max(1);

        for _epoch in 0..self.config.grpo_epochs {
            let mut indices: Vec<usize> = (0..total_n).collect();
            indices.shuffle(&mut rng);

            let mut start_idx = 0;
            while start_idx < total_n {
                let end_idx = (start_idx + mb_size).min(total_n);
                let cur_batch_size = end_idx - start_idx;
                let mb_indices = &indices[start_idx..end_idx];

                let mut mb_states_vec = Vec::with_capacity(cur_batch_size * state_dim);
                let mut mb_actions_vec = Vec::with_capacity(cur_batch_size * enc_dim);
                let mut mb_old_log_probs_vec = Vec::with_capacity(cur_batch_size);
                let mut mb_advantages_vec = Vec::with_capacity(cur_batch_size);
                let mut mb_masks_vec = if has_masks {
                    Some(Vec::with_capacity(cur_batch_size * mask_dim))
                } else {
                    None
                };

                for &idx in mb_indices {
                    mb_states_vec
                        .extend_from_slice(&all_states[idx * state_dim..(idx + 1) * state_dim]);
                    mb_actions_vec
                        .extend_from_slice(&all_actions[idx * enc_dim..(idx + 1) * enc_dim]);
                    mb_old_log_probs_vec.push(all_old_log_probs[idx]);
                    mb_advantages_vec.push(all_advs[idx]);
                    if let (Some(mbm), Some(am)) = (&mut mb_masks_vec, &all_masks) {
                        mbm.extend_from_slice(&am[idx * mask_dim..(idx + 1) * mask_dim]);
                    }
                }

                let mb_states =
                    Tensor::from_vec(mb_states_vec, (cur_batch_size, state_dim), &self.device)?;
                let mb_actions =
                    Tensor::from_vec(mb_actions_vec, (cur_batch_size, enc_dim), &self.device)?;
                let mb_old_log_probs =
                    Tensor::from_vec(mb_old_log_probs_vec, (cur_batch_size,), &self.device)?;
                let mb_advantages =
                    Tensor::from_vec(mb_advantages_vec, (cur_batch_size,), &self.device)?;
                let mb_masks = if let Some(mbm) = mb_masks_vec {
                    Some(Tensor::from_vec(
                        mbm,
                        (cur_batch_size, mask_dim),
                        &self.device,
                    )?)
                } else {
                    None
                };

                let (new_log_probs, entropy) =
                    self.policy
                        .evaluate_actions(&mb_states, &mb_actions, mb_masks.as_ref())?;

                let log_ratio = (&new_log_probs - &mb_old_log_probs)?;
                let ratio = log_ratio.exp()?;

                let surr1 = (&ratio * &mb_advantages)?;
                let clamped_ratio =
                    ratio.clamp(1.0 - self.config.clip_eps, 1.0 + self.config.clip_eps)?;
                let surr2 = (&clamped_ratio * &mb_advantages)?;

                let policy_loss = surr1.minimum(&surr2)?.neg()?.mean_all()?;

                let kl_loss = (&ratio - 1.0 - &log_ratio)?.mean_all()?;
                let clip_frac = (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                    + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
                .mean_all()?;

                let p_loss_val: f32 = policy_loss.to_scalar()?;
                let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
                let kl_val: f32 = kl_loss.to_scalar()?;
                let clip_frac_val: f32 = clip_frac.to_scalar()?;

                let mut grads = policy_loss.backward()?;
                self.clip_grad_norm(&mut grads)?;
                self.optimizer.step(&grads)?;

                last_stats = GRPOStats {
                    policy_loss: p_loss_val,
                    entropy: entropy_val,
                    total_loss: p_loss_val,
                    kl: kl_val,
                    clip_frac: clip_frac_val,
                    group_reward_mean: grp_mean,
                    group_reward_std: grp_std,
                };

                start_idx += mb_size;
            }
        }

        Ok(last_stats)
    }
}
