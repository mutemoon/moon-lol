use candle_core::{D, DType, Result, Tensor};
use candle_nn::{Embedding, Linear, Module, VarBuilder};
use lol_rl_protocol::{ActionSpace, PolicyDisplay, PolicyItem};
use rand::Rng;

/// 0.5·ln(2π)，用于高斯策略的 log_prob / 熵。
const HALF_LN_2PI: f32 = 0.9189385;

/// 生成标准正交权重矩阵（Modified Gram-Schmidt），用于工业级深度网络初始化。
pub fn orthogonal_weight(out_dim: usize, in_dim: usize, gain: f32) -> Vec<f32> {
    let rows = out_dim.max(in_dim);
    let cols = out_dim.min(in_dim);
    let mut rng = rand::rng();

    // 生成 rows x cols 的标准正态分布随机矩阵（Box-Muller 变换）
    let mut mat = Vec::with_capacity(rows * cols);
    while mat.len() < rows * cols {
        let u1: f32 = rng.random_range(1e-7..1.0);
        let u2: f32 = rng.random_range(0.0..std::f32::consts::TAU);
        let r = (-2.0 * u1.ln()).sqrt();
        let z0 = r * u2.cos();
        let z1 = r * u2.sin();
        mat.push(z0);
        if mat.len() < rows * cols {
            mat.push(z1);
        }
    }

    // Modified Gram-Schmidt QR 分解正交化每一列
    for j in 0..cols {
        for k in 0..j {
            let mut dot = 0.0f32;
            for r in 0..rows {
                dot += mat[r * cols + j] * mat[r * cols + k];
            }
            for r in 0..rows {
                mat[r * cols + j] -= dot * mat[r * cols + k];
            }
        }
        let mut norm_sq = 0.0f32;
        for r in 0..rows {
            norm_sq += mat[r * cols + j] * mat[r * cols + j];
        }
        let inv_norm = if norm_sq > 1e-12 {
            1.0 / norm_sq.sqrt()
        } else {
            0.0
        };
        for r in 0..rows {
            mat[r * cols + j] *= inv_norm;
        }
    }

    let mut result = vec![0.0f32; out_dim * in_dim];
    if out_dim >= in_dim {
        for r in 0..out_dim {
            for c in 0..in_dim {
                result[r * in_dim + c] = mat[r * cols + c] * gain;
            }
        }
    } else {
        for r in 0..out_dim {
            for c in 0..in_dim {
                result[r * in_dim + c] = mat[c * cols + r] * gain;
            }
        }
    }
    result
}

/// Configuration for hero-id embedding (OpenAI Five style conditional input).
/// The first element of the state vector (obs[0]) is treated as an integer hero index,
/// looked up in an embedding table, and concatenated with the remaining state features.
#[derive(Clone, Debug, PartialEq)]
pub struct HeroEmbedConfig {
    pub num_heroes: usize,
    pub embed_dim: usize,
}

impl Default for HeroEmbedConfig {
    fn default() -> Self {
        Self {
            num_heroes: 4, // 默认支持最多 4 个英雄 (0: Fiora, 1: Riven, 2..3 扩展)
            embed_dim: 16,
        }
    }
}

#[derive(Clone)]
pub struct ActorCritic {
    /// Hero-id embedding (OpenAI Five style).
    /// obs[0] is treated as hero index → embedding, replacing the raw float.
    hero_embed: Embedding,
    hero_embed_config: HeroEmbedConfig,
    fc1: Linear,
    fc2: Linear,
    /// 离散：分类 logits；连续/混合：连续维的均值头。
    actor_head: Linear,
    /// 连续/混合：可训练 log_std，形状 (continuous_dims,)。
    log_std: Option<Tensor>,
    /// 混合：离散分类头。
    attack_head: Option<Linear>,
    critic_head: Linear,
    action_space: ActionSpace,
}

impl ActorCritic {
    pub fn new(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        vb: VarBuilder,
    ) -> Result<Self> {
        Self::with_hero_embed(state_dim, hidden_dim, action_space, HeroEmbedConfig::default(), vb)
    }

    pub fn with_hero_embed(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        hero_embed_config: HeroEmbedConfig,
        vb: VarBuilder,
    ) -> Result<Self> {
        let emb = candle_nn::embedding(
            hero_embed_config.num_heroes,
            hero_embed_config.embed_dim,
            vb.pp("hero_embed"),
        )?;
        let fc1_input_dim = hero_embed_config.embed_dim + state_dim - 1;
        let fc1 = candle_nn::linear(fc1_input_dim, hidden_dim, vb.pp("fc1"))?;
        let fc2 = candle_nn::linear(hidden_dim, hidden_dim, vb.pp("fc2"))?;
        let critic_head = candle_nn::linear(hidden_dim, 1, vb.pp("critic_head"))?;

        let (actor_out_dim, log_std, attack_head) = match action_space {
            ActionSpace::Discrete(n) => (n, None, None),
            ActionSpace::Continuous(d) => (
                d,
                Some(vb.get_with_hints((d,), "log_std", candle_nn::Init::Const(0.0))?),
                None,
            ),
            ActionSpace::Hybrid {
                continuous_dims,
                discrete_classes,
            } => (
                continuous_dims,
                Some(vb.get_with_hints(
                    (continuous_dims,),
                    "log_std",
                    candle_nn::Init::Const(0.0),
                )?),
                Some(candle_nn::linear(
                    hidden_dim,
                    discrete_classes,
                    vb.pp("attack_head"),
                )?),
            ),
        };
        let actor_head = candle_nn::linear(hidden_dim, actor_out_dim, vb.pp("actor_head"))?;

        Ok(Self {
            hero_embed: emb,
            hero_embed_config,
            fc1,
            fc2,
            actor_head,
            log_std,
            attack_head,
            critic_head,
            action_space,
        })
    }

    /// 将策略网络权重复制并迁移到指定计算设备（例如将 GPU 权重克隆至 CPU）
    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        let fc1_w = self.fc1.weight().to_device(device)?;
        let fc1_b = self.fc1.bias().map(|b| b.to_device(device)).transpose()?;
        let fc1 = Linear::new(fc1_w, fc1_b);

        let fc2_w = self.fc2.weight().to_device(device)?;
        let fc2_b = self.fc2.bias().map(|b| b.to_device(device)).transpose()?;
        let fc2 = Linear::new(fc2_w, fc2_b);

        let actor_w = self.actor_head.weight().to_device(device)?;
        let actor_b = self
            .actor_head
            .bias()
            .map(|b| b.to_device(device))
            .transpose()?;
        let actor_head = Linear::new(actor_w, actor_b);

        let critic_w = self.critic_head.weight().to_device(device)?;
        let critic_b = self
            .critic_head
            .bias()
            .map(|b| b.to_device(device))
            .transpose()?;
        let critic_head = Linear::new(critic_w, critic_b);

        let log_std = self
            .log_std
            .as_ref()
            .map(|s| s.to_device(device))
            .transpose()?;

        let attack_head = self
            .attack_head
            .as_ref()
            .map(|a| -> Result<Linear> {
                let w = a.weight().to_device(device)?;
                let b = a.bias().map(|b| b.to_device(device)).transpose()?;
                Ok(Linear::new(w, b))
            })
            .transpose()?;

        let w = self.hero_embed.embeddings().to_device(device)?;
        let hero_embed = Embedding::new(w, self.hero_embed.hidden_size());

        Ok(Self {
            hero_embed,
            hero_embed_config: self.hero_embed_config.clone(),
            fc1,
            fc2,
            actor_head,
            log_std,
            attack_head,
            critic_head,
            action_space: self.action_space.clone(),
        })
    }

    pub fn action_space(&self) -> &ActionSpace {
        &self.action_space
    }

    pub fn hero_embed_config(&self) -> &HeroEmbedConfig {
        &self.hero_embed_config
    }

    pub fn has_hero_embed(&self) -> bool {
        true
    }

    /// Prepare the input by replacing hero_id float with embedding.
    fn prepare_input(&self, state: &Tensor) -> Result<Tensor> {
        // state shape: (batch, state_dim)
        // obs[0] = hero_id (float 0.0 or 1.0) → cast to u32 index
        let hero_ids = state.narrow(1, 0, 1)?.squeeze(1)?.to_dtype(DType::U32)?;
        let hero_vecs = self.hero_embed.forward(&hero_ids)?; // (batch, embed_dim)
        let rest = state.narrow(1, 1, state.dim(1)? - 1)?; // (batch, state_dim - 1)
        Tensor::cat(&[&hero_vecs, &rest], 1) // (batch, embed_dim + state_dim - 1)
    }

    /// 共享隐藏层输出。
    fn hidden(&self, state: &Tensor) -> Result<Tensor> {
        let input = self.prepare_input(state)?;
        let h1 = self.fc1.forward(&input)?.tanh()?;
        self.fc2.forward(&h1)?.tanh()
    }

    /// Forward pass 返回 (actor_head 原始输出, values)。
    /// 离散：logits (batch, n)；连续/混合：连续均值 (batch, continuous_dims)。
    pub fn forward(&self, state: &Tensor) -> Result<(Tensor, Tensor)> {
        let h2 = self.hidden(state)?;
        let out = self.actor_head.forward(&h2)?;
        let values = self.critic_head.forward(&h2)?;
        Ok((out, values))
    }

    /// 批量获取 Critic 状态价值估值
    pub fn get_values(&self, state: &Tensor) -> Result<Vec<f32>> {
        let h2 = self.hidden(state)?;
        let values = self.critic_head.forward(&h2)?;
        values.squeeze(1)?.to_vec1()
    }

    /// 从策略采样一个动作。返回 (编码动作向量, log_prob, value)。
    pub fn sample_action(
        &self,
        state: &Tensor,
        mask: Option<&[bool]>,
    ) -> Result<(Vec<f32>, f32, f32)> {
        let h2 = self.hidden(state)?;
        let values = self.critic_head.forward(&h2)?;
        let val_scalar: f32 = values.squeeze(0)?.squeeze(0)?.to_scalar()?;

        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                let masked = mask_logits_slice(&logits, mask);
                let (idx, log_prob) = sample_categorical(&masked);
                Ok((vec![idx as f32], log_prob, val_scalar))
            }
            ActionSpace::Continuous(d) => {
                let means: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                let log_std: Vec<f32> = self.log_std.as_ref().unwrap().to_vec1()?;
                let mut rng = rand::rng();
                let mut encoded = Vec::with_capacity(d);
                let mut log_prob = 0.0;
                for i in 0..d {
                    let std = log_std[i].exp();
                    let a = means[i] + std * sample_gaussian(&mut rng);
                    encoded.push(a);
                    log_prob += gaussian_log_prob(means[i], std, a);
                }
                Ok((encoded, log_prob, val_scalar))
            }
            ActionSpace::Hybrid {
                continuous_dims, ..
            } => {
                let means: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                let log_std: Vec<f32> = self.log_std.as_ref().unwrap().to_vec1()?;
                let mut rng = rand::rng();
                let mut encoded = Vec::with_capacity(continuous_dims + 1);
                let mut log_prob = 0.0;
                for i in 0..continuous_dims {
                    let std = log_std[i].exp();
                    let a = means[i] + std * sample_gaussian(&mut rng);
                    encoded.push(a);
                    log_prob += gaussian_log_prob(means[i], std, a);
                }
                let attack_logits: Vec<f32> = self
                    .attack_head
                    .as_ref()
                    .unwrap()
                    .forward(&h2)?
                    .squeeze(0)?
                    .to_vec1()?;
                let masked = mask_logits_slice(&attack_logits, mask);
                let (idx, cat_log_prob) = sample_categorical(&masked);
                encoded.push(idx as f32);
                log_prob += cat_log_prob;
                Ok((encoded, log_prob, val_scalar))
            }
        }
    }

    /// 批量从策略采样动作（一次 GPU/CPU 前向计算），返回每个样本的 (encoded_action, log_prob, value)。
    pub fn sample_batch(
        &self,
        states: &Tensor,
        masks: Option<&[Option<Vec<bool>>]>,
    ) -> Result<Vec<(Vec<f32>, f32, f32)>> {
        let b = states.dim(0)?;
        if b == 0 {
            return Ok(Vec::new());
        }
        let h2 = self.hidden(states)?;
        let values = self.critic_head.forward(&h2)?;
        let val_vec: Vec<f32> = values.squeeze(1)?.to_vec1()?;

        let mut results = Vec::with_capacity(b);

        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits = self.actor_head.forward(&h2)?;
                let logits_mat: Vec<Vec<f32>> = logits.to_vec2()?;
                for i in 0..b {
                    let mask_i = masks.and_then(|ms| ms.get(i)).and_then(|m| m.as_deref());
                    let masked = mask_logits_slice(&logits_mat[i], mask_i);
                    let (idx, log_prob) = sample_categorical(&masked);
                    results.push((vec![idx as f32], log_prob, val_vec[i]));
                }
            }
            ActionSpace::Continuous(d) => {
                let means_mat: Vec<Vec<f32>> = self.actor_head.forward(&h2)?.to_vec2()?;
                let log_std: Vec<f32> = self.log_std.as_ref().unwrap().to_vec1()?;
                let mut rng = rand::rng();
                for i in 0..b {
                    let means = &means_mat[i];
                    let mut encoded = Vec::with_capacity(d);
                    let mut log_prob = 0.0;
                    for j in 0..d {
                        let std = log_std[j].exp();
                        let a = means[j] + std * sample_gaussian(&mut rng);
                        encoded.push(a);
                        log_prob += gaussian_log_prob(means[j], std, a);
                    }
                    results.push((encoded, log_prob, val_vec[i]));
                }
            }
            ActionSpace::Hybrid {
                continuous_dims, ..
            } => {
                let means_mat: Vec<Vec<f32>> = self.actor_head.forward(&h2)?.to_vec2()?;
                let log_std: Vec<f32> = self.log_std.as_ref().unwrap().to_vec1()?;
                let attack_logits_mat: Vec<Vec<f32>> =
                    self.attack_head.as_ref().unwrap().forward(&h2)?.to_vec2()?;
                let mut rng = rand::rng();

                for i in 0..b {
                    let means = &means_mat[i];
                    let mut encoded = Vec::with_capacity(continuous_dims + 1);
                    let mut log_prob = 0.0;
                    for j in 0..continuous_dims {
                        let std = log_std[j].exp();
                        let a = means[j] + std * sample_gaussian(&mut rng);
                        encoded.push(a);
                        log_prob += gaussian_log_prob(means[j], std, a);
                    }

                    let mask_i = masks.and_then(|ms| ms.get(i)).and_then(|m| m.as_deref());
                    let masked = mask_logits_slice(&attack_logits_mat[i], mask_i);
                    let (idx, cat_log_prob) = sample_categorical(&masked);
                    encoded.push(idx as f32);
                    log_prob += cat_log_prob;
                    results.push((encoded, log_prob, val_vec[i]));
                }
            }
        }

        Ok(results)
    }

    /// 确定性贪心动作（连续取均值、离散取 argmax），用于可视化与评估。
    pub fn select_greedy_action(&self, state: &Tensor, mask: Option<&[bool]>) -> Result<Vec<f32>> {
        let h2 = self.hidden(state)?;
        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                let masked = mask_logits_slice(&logits, mask);
                Ok(vec![argmax(&masked) as f32])
            }
            ActionSpace::Continuous(d) => {
                let means: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                Ok(means[..d].to_vec())
            }
            ActionSpace::Hybrid {
                continuous_dims, ..
            } => {
                let means: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                let attack_logits: Vec<f32> = self
                    .attack_head
                    .as_ref()
                    .unwrap()
                    .forward(&h2)?
                    .squeeze(0)?
                    .to_vec1()?;
                let masked = mask_logits_slice(&attack_logits, mask);
                let mut encoded = means[..continuous_dims].to_vec();
                encoded.push(argmax(&masked) as f32);
                Ok(encoded)
            }
        }
    }

    /// 真实动作空间的策略展示（可视化用）：离散返回逐类概率，混合返回连续均值 + 离散各动作概率。
    pub fn policy_display_real(
        &self,
        state: &Tensor,
        mask: Option<&[bool]>,
        labels: &[&str],
    ) -> Result<PolicyDisplay> {
        let h2 = self.hidden(state)?;
        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                let raw_probs_vec = softmax_slice(&logits);
                let masked = mask_logits_slice(&logits, mask);
                let probs_vec = softmax_slice(&masked);
                Ok(PolicyDisplay::Discrete(
                    probs_vec
                        .into_iter()
                        .enumerate()
                        .map(|(i, p)| {
                            let is_masked = mask
                                .map(|m| !m.get(i).copied().unwrap_or(true))
                                .unwrap_or(false);
                            PolicyItem {
                                action_id: i,
                                action: labels
                                    .get(i)
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| format!("Action {}", i)),
                                prob: p,
                                raw_prob: raw_probs_vec.get(i).copied().unwrap_or(0.0),
                                is_masked,
                            }
                        })
                        .collect(),
                ))
            }
            ActionSpace::Continuous(_) => Ok(PolicyDisplay::Discrete(Vec::new())),
            ActionSpace::Hybrid {
                continuous_dims,
                discrete_classes,
            } => {
                let means: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                let attack_logits: Vec<f32> = self
                    .attack_head
                    .as_ref()
                    .unwrap()
                    .forward(&h2)?
                    .squeeze(0)?
                    .to_vec1()?;
                let raw_discrete_probs_vec = softmax_slice(&attack_logits);
                let masked = mask_logits_slice(&attack_logits, mask);
                let discrete_probs_vec = softmax_slice(&masked);

                if discrete_classes == 2 {
                    let p_attack = discrete_probs_vec.last().copied().unwrap_or(0.0);
                    let raw_p_attack = raw_discrete_probs_vec.last().copied().unwrap_or(0.0);
                    let is_attack_masked = mask
                        .map(|m| !m.get(1).copied().unwrap_or(true))
                        .unwrap_or(false);
                    let move_x = means.first().copied().unwrap_or(0.0).clamp(-1.0, 1.0);
                    let move_z = means.get(1).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
                    Ok(PolicyDisplay::Hybrid {
                        move_x,
                        move_z,
                        attack_prob: p_attack,
                        raw_attack_prob: raw_p_attack,
                        is_attack_masked,
                    })
                } else {
                    let discrete_probs = discrete_probs_vec
                        .iter()
                        .enumerate()
                        .map(|(i, &prob)| {
                            let action_label = labels
                                .get(i)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("Act_{}", i));
                            let is_masked = mask
                                .map(|m| !m.get(i).copied().unwrap_or(true))
                                .unwrap_or(false);
                            PolicyItem {
                                action_id: i,
                                action: action_label,
                                prob,
                                raw_prob: raw_discrete_probs_vec.get(i).copied().unwrap_or(0.0),
                                is_masked,
                            }
                        })
                        .collect();
                    let continuous_means = means[..continuous_dims.min(means.len())].to_vec();
                    Ok(PolicyDisplay::HybridMulti {
                        continuous_means,
                        discrete_probs,
                    })
                }
            }
        }
    }

    /// PPO update：给定 (state, actions, masks) 计算 (log_probs, values, entropy)。
    /// actions 形状 (n, encoding_dim)，Discrete=1 / Continuous=d / Hybrid=d+1。
    /// masks 形状 (n, num_classes)，用于屏蔽非法离散动作（1.0 = 有效，0.0 = 屏蔽）。
    pub fn evaluate_actions(
        &self,
        state: &Tensor,
        actions: &Tensor,
        masks: Option<&Tensor>,
    ) -> Result<(Tensor, Tensor, Tensor)> {
        let n = state.dim(0)?;
        let h2 = self.hidden(state)?;
        let values = self.critic_head.forward(&h2)?;

        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits = self.actor_head.forward(&h2)?;
                let masked_logits = mask_logits_tensor(&logits, masks)?;
                let log_probs_all = candle_nn::ops::log_softmax(&masked_logits, D::Minus1)?;
                let probs_all = candle_nn::ops::softmax(&masked_logits, D::Minus1)?;
                let act = actions.squeeze(1)?.to_dtype(DType::U32)?;
                let selected_log_probs = log_probs_all.gather(&act.unsqueeze(1)?, 1)?.squeeze(1)?;
                let entropy = (probs_all * log_probs_all)?
                    .neg()?
                    .sum_keepdim(D::Minus1)?
                    .squeeze(1)?;
                Ok((selected_log_probs, values.squeeze(1)?, entropy))
            }
            ActionSpace::Continuous(d) => {
                let means = self.actor_head.forward(&h2)?;
                let log_std = self.log_std.as_ref().unwrap();
                let log_std_b = log_std.broadcast_as((n, d))?;
                let std_b = log_std_b.exp()?;
                let cont = actions.narrow(1, 0, d)?;
                let z = {
                    let diff = (&cont - &means)?;
                    (&diff / &std_b)?
                };
                let log_prob = z
                    .powf(2.0)?
                    .neg()?
                    .affine(0.5, 0.0)?
                    .sub(&log_std_b)?
                    .affine(1.0, -HALF_LN_2PI as f64)?
                    .sum(D::Minus1)?;
                let entropy = log_std_b
                    .affine(1.0, (0.5 + HALF_LN_2PI) as f64)?
                    .sum(D::Minus1)?;
                Ok((log_prob, values.squeeze(1)?, entropy))
            }
            ActionSpace::Hybrid {
                continuous_dims, ..
            } => {
                let means = self.actor_head.forward(&h2)?;
                let log_std = self.log_std.as_ref().unwrap();
                let log_std_b = log_std.broadcast_as((n, continuous_dims))?;
                let std_b = log_std_b.exp()?;
                let cont = actions.narrow(1, 0, continuous_dims)?;
                let z = {
                    let diff = (&cont - &means)?;
                    (&diff / &std_b)?
                };
                let gauss_log_prob = z
                    .powf(2.0)?
                    .neg()?
                    .affine(0.5, 0.0)?
                    .sub(&log_std_b)?
                    .affine(1.0, -HALF_LN_2PI as f64)?
                    .sum(D::Minus1)?;
                let gauss_entropy = log_std_b
                    .affine(1.0, (0.5 + HALF_LN_2PI) as f64)?
                    .sum(D::Minus1)?;

                let attack_logits = self.attack_head.as_ref().unwrap().forward(&h2)?;
                let masked = mask_logits_tensor(&attack_logits, masks)?;
                let log_probs_all = candle_nn::ops::log_softmax(&masked, D::Minus1)?;
                let probs_all = candle_nn::ops::softmax(&masked, D::Minus1)?;
                let act = actions
                    .narrow(1, continuous_dims, 1)?
                    .squeeze(1)?
                    .to_dtype(DType::U32)?;
                let cat_log_prob = log_probs_all.gather(&act.unsqueeze(1)?, 1)?.squeeze(1)?;
                let cat_entropy = (probs_all * log_probs_all)?
                    .neg()?
                    .sum_keepdim(D::Minus1)?
                    .squeeze(1)?;

                let log_prob = (&gauss_log_prob + &cat_log_prob)?;
                let entropy = (&gauss_entropy + &cat_entropy)?;
                Ok((log_prob, values.squeeze(1)?, entropy))
            }
        }
    }
}

/// 对一维切片应用布尔掩码（valid=true 保留，invalid=false 置为 -1e9）
fn mask_logits_slice(logits: &[f32], mask: Option<&[bool]>) -> Vec<f32> {
    match mask {
        Some(m) => logits
            .iter()
            .zip(m.iter())
            .map(|(&l, &valid)| if valid { l } else { -1e9 })
            .collect(),
        None => logits.to_vec(),
    }
}

/// 对 Tensor 应用掩码（mask 形状 (batch, classes)，1.0=有效，0.0=无效置 -1e9）
fn mask_logits_tensor(logits: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
    match mask {
        Some(m) => {
            let m_cast = if m.dtype() != logits.dtype() {
                m.to_dtype(logits.dtype())?
            } else {
                m.clone()
            };
            let penalty = m_cast.affine(1e9, -1e9)?; // valid(1.0)->0.0, invalid(0.0)->-1e9
            logits.broadcast_add(&penalty)
        }
        None => Ok(logits.clone()),
    }
}

fn softmax_slice(logits: &[f32]) -> Vec<f32> {
    let max_l = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits.iter().map(|&x| (x - max_l).exp()).collect();
    let sum_exp: f32 = exps.iter().sum();
    if sum_exp > 0.0 {
        exps.iter().map(|&e| e / sum_exp).collect()
    } else {
        vec![1.0 / logits.len() as f32; logits.len()]
    }
}

fn sample_categorical(logits: &[f32]) -> (usize, f32) {
    let probs = softmax_slice(logits);
    let idx = sample_from_probs(&probs);
    let log_prob = if probs[idx] > 1e-12 {
        probs[idx].ln()
    } else {
        -20.0
    };
    (idx, log_prob)
}

fn sample_from_probs(probs: &[f32]) -> usize {
    let mut rng = rand::rng();
    let r: f32 = rng.random();
    let mut cum_prob = 0.0;
    for (idx, &prob) in probs.iter().enumerate() {
        cum_prob += prob;
        if r <= cum_prob {
            return idx;
        }
    }
    probs.len() - 1
}

fn argmax(values: &[f32]) -> usize {
    let mut max_idx = 0;
    let mut max_val = f32::NEG_INFINITY;
    for (idx, &val) in values.iter().enumerate() {
        if val > max_val {
            max_val = val;
            max_idx = idx;
        }
    }
    max_idx
}

/// Box-Muller 采样标准正态 N(0,1)。
fn sample_gaussian(rng: &mut impl rand::Rng) -> f32 {
    let u1: f64 = rng.random::<f64>().max(f64::EPSILON);
    let u2: f64 = rng.random::<f64>();
    let r = (-2.0 * u1.ln()).sqrt();
    let theta = 2.0 * std::f64::consts::PI * u2;
    (r * theta.cos()) as f32
}

fn gaussian_log_prob(mean: f32, std: f32, action: f32) -> f32 {
    let z = (action - mean) / std;
    -0.5 * z * z - std.ln() - HALF_LN_2PI
}
