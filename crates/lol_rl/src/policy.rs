use candle_core::{D, DType, Result, Tensor};
use candle_nn::{Linear, Module, VarBuilder, linear};
use lol_env::{ATTACK_MASK_DISTANCE, OBS_DISTANCE_IDX, OBS_DISTANCE_SCALE};
use lol_rl_protocol::{ActionSpace, PolicyDisplay, PolicyItem};
use rand::Rng;

/// 0.5·ln(2π)，用于高斯策略的 log_prob / 熵。
const HALF_LN_2PI: f32 = 0.9189385;

#[derive(Clone)]
pub struct ActorCritic {
    fc1: Linear,
    fc2: Linear,
    /// 离散：分类 logits；连续/混合：连续维的均值头。
    actor_head: Linear,
    /// 连续/混合：可训练 log_std，形状 (continuous_dims,)。
    log_std: Option<Tensor>,
    /// 混合：离散分类头（攻击）。
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
        let fc1 = linear(state_dim, hidden_dim, vb.pp("fc1"))?;
        let fc2 = linear(hidden_dim, hidden_dim, vb.pp("fc2"))?;
        let critic_head = linear(hidden_dim, 1, vb.pp("critic_head"))?;

        let (actor_out_dim, log_std, attack_head) = match action_space {
            ActionSpace::Discrete(n) => (n, None, None),
            ActionSpace::Continuous(d) => (
                d,
                Some(vb.get_with_hints((d,), "log_std", Default::default())?),
                None,
            ),
            ActionSpace::Hybrid {
                continuous_dims,
                discrete_classes,
            } => (
                continuous_dims,
                Some(vb.get_with_hints((continuous_dims,), "log_std", Default::default())?),
                Some(linear(hidden_dim, discrete_classes, vb.pp("attack_head"))?),
            ),
        };
        let actor_head = linear(hidden_dim, actor_out_dim, vb.pp("actor_head"))?;

        Ok(Self {
            fc1,
            fc2,
            actor_head,
            log_std,
            attack_head,
            critic_head,
            action_space,
        })
    }

    pub fn action_space(&self) -> &ActionSpace {
        &self.action_space
    }

    /// 共享隐藏层输出。
    fn hidden(&self, state: &Tensor) -> Result<Tensor> {
        let h1 = self.fc1.forward(state)?.tanh()?;
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

    /// 从策略采样一个动作。返回 (编码动作向量, log_prob, value)。
    pub fn sample_action(&self, state: &Tensor, obs_vec: &[f32]) -> Result<(Vec<f32>, f32, f32)> {
        let h2 = self.hidden(state)?;
        let values = self.critic_head.forward(&h2)?;
        let val_scalar: f32 = values.squeeze(0)?.squeeze(0)?.to_scalar()?;

        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits = self.actor_head.forward(&h2)?;
                let masked = masked_logits(&logits, obs_vec)?;
                let probs = candle_nn::ops::softmax(&masked, D::Minus1)?;
                let log_probs = candle_nn::ops::log_softmax(&masked, D::Minus1)?;
                let probs_vec: Vec<f32> = probs.squeeze(0)?.to_vec1()?;
                let log_probs_vec: Vec<f32> = log_probs.squeeze(0)?.to_vec1()?;
                let idx = sample_from_probs(&probs_vec);
                Ok((vec![idx as f32], log_probs_vec[idx], val_scalar))
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
                let attack_logits = self.attack_head.as_ref().unwrap().forward(&h2)?;
                let masked = mask_hybrid_attack_single(&attack_logits, obs_vec)?;
                let probs = candle_nn::ops::softmax(&masked, D::Minus1)?;
                let log_probs = candle_nn::ops::log_softmax(&masked, D::Minus1)?;
                let probs_vec: Vec<f32> = probs.squeeze(0)?.to_vec1()?;
                let log_probs_vec: Vec<f32> = log_probs.squeeze(0)?.to_vec1()?;
                let idx = sample_from_probs(&probs_vec);
                encoded.push(idx as f32);
                log_prob += log_probs_vec[idx];
                Ok((encoded, log_prob, val_scalar))
            }
        }
    }

    /// 确定性贪心动作（连续取均值、攻击取 argmax），用于可视化。
    pub fn select_greedy_action(&self, state: &Tensor, obs_vec: &[f32]) -> Result<Vec<f32>> {
        let h2 = self.hidden(state)?;
        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits = self.actor_head.forward(&h2)?;
                let masked = masked_logits(&logits, obs_vec)?;
                let logits_vec: Vec<f32> = masked.squeeze(0)?.to_vec1()?;
                Ok(vec![argmax(&logits_vec) as f32])
            }
            ActionSpace::Continuous(d) => {
                let means: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                Ok(means[..d].to_vec())
            }
            ActionSpace::Hybrid {
                continuous_dims, ..
            } => {
                let means: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                let attack_logits = self.attack_head.as_ref().unwrap().forward(&h2)?;
                let masked = mask_hybrid_attack_single(&attack_logits, obs_vec)?;
                let logits_vec: Vec<f32> = masked.squeeze(0)?.to_vec1()?;
                let mut encoded = means[..continuous_dims].to_vec();
                encoded.push(argmax(&logits_vec) as f32);
                Ok(encoded)
            }
        }
    }

    /// 真实动作空间的策略展示（可视化用）：离散返回逐类概率，混合返回连续均值 + 攻击概率。
    pub fn policy_display_real(
        &self,
        state: &Tensor,
        obs_vec: &[f32],
        labels: &[&str],
    ) -> Result<PolicyDisplay> {
        let h2 = self.hidden(state)?;
        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits = self.actor_head.forward(&h2)?;
                let masked = masked_logits(&logits, obs_vec)?;
                let probs = candle_nn::ops::softmax(&masked, D::Minus1)?;
                let probs_vec: Vec<f32> = probs.squeeze(0)?.to_vec1()?;
                Ok(PolicyDisplay::Discrete(
                    probs_vec
                        .into_iter()
                        .enumerate()
                        .map(|(i, p)| PolicyItem {
                            action_id: i,
                            action: labels
                                .get(i)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("Action {}", i)),
                            prob: p,
                        })
                        .collect(),
                ))
            }
            ActionSpace::Continuous(_) => {
                // 当前无纯连续环境，保守返回空分布（客户端展示「无概率数据」）。
                Ok(PolicyDisplay::Discrete(Vec::new()))
            }
            ActionSpace::Hybrid { .. } => {
                let means: Vec<f32> = self.actor_head.forward(&h2)?.squeeze(0)?.to_vec1()?;
                let attack_logits = self.attack_head.as_ref().unwrap().forward(&h2)?;
                let masked = mask_hybrid_attack_single(&attack_logits, obs_vec)?;
                let attack_probs: Vec<f32> = candle_nn::ops::softmax(&masked, D::Minus1)?
                    .squeeze(0)?
                    .to_vec1()?;
                let p_attack = attack_probs.last().copied().unwrap_or(0.0);
                let move_x = means.first().copied().unwrap_or(0.0).clamp(-1.0, 1.0);
                let move_z = means.get(1).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
                Ok(PolicyDisplay::Hybrid {
                    move_x,
                    move_z,
                    attack_prob: p_attack,
                })
            }
        }
    }

    /// PPO update：给定 (state, actions) 计算 (log_probs, values, entropy)。
    /// actions 形状 (n, encoding_dim)，Discrete=1 / Continuous=d / Hybrid=d+1。
    pub fn evaluate_actions(
        &self,
        state: &Tensor,
        actions: &Tensor,
    ) -> Result<(Tensor, Tensor, Tensor)> {
        let n = state.dim(0)?;
        let h2 = self.hidden(state)?;
        let values = self.critic_head.forward(&h2)?;

        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits = self.actor_head.forward(&h2)?;
                let masked_logits = batch_masked_logits(&logits, state)?;
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
                let masked = mask_hybrid_attack_batch(&attack_logits, state)?;
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

    /// Sample an action for a single state during environment rollout（纯离散，legacy 兼容）。
    pub fn select_action(&self, state: &Tensor) -> Result<(usize, f32, f32)> {
        let (logits, value) = self.forward(state)?;
        let probs = candle_nn::ops::softmax(&logits, D::Minus1)?;
        let log_probs = candle_nn::ops::log_softmax(&logits, D::Minus1)?;

        let probs_vec: Vec<f32> = probs.squeeze(0)?.to_vec1()?;
        let log_probs_vec: Vec<f32> = log_probs.squeeze(0)?.to_vec1()?;
        let val_scalar: f32 = value.squeeze(0)?.squeeze(0)?.to_scalar()?;

        let chosen_action = sample_from_probs(&probs_vec);
        let chosen_log_prob = log_probs_vec[chosen_action];

        Ok((chosen_action, chosen_log_prob, val_scalar))
    }

    /// Sample an action with action masking（纯离散，legacy 兼容）。
    pub fn select_action_masked(
        &self,
        state: &Tensor,
        obs_vec: &[f32],
    ) -> Result<(usize, f32, f32)> {
        let (logits, value) = self.forward(state)?;
        let masked_logits = masked_logits(&logits, obs_vec)?;
        let probs = candle_nn::ops::softmax(&masked_logits, D::Minus1)?;
        let log_probs = candle_nn::ops::log_softmax(&masked_logits, D::Minus1)?;

        let probs_vec: Vec<f32> = probs.squeeze(0)?.to_vec1()?;
        let log_probs_vec: Vec<f32> = log_probs.squeeze(0)?.to_vec1()?;
        let val_scalar: f32 = value.squeeze(0)?.squeeze(0)?.to_scalar()?;

        let chosen_action = sample_from_probs(&probs_vec);
        let chosen_log_prob = log_probs_vec[chosen_action];
        Ok((chosen_action, chosen_log_prob, val_scalar))
    }

    /// Return action probabilities for the given state（纯离散，legacy 兼容）。
    pub fn policy_probs(&self, state: &Tensor, obs_vec: &[f32]) -> Result<Vec<f32>> {
        let (logits, _) = self.forward(state)?;
        let masked = masked_logits(&logits, obs_vec)?;
        let probs = candle_nn::ops::softmax(&masked, D::Minus1)?;
        probs.squeeze(0)?.to_vec1()
    }

    /// Select the greedy action with action masking（纯离散，legacy 兼容）。
    pub fn select_greedy_action_masked(&self, state: &Tensor, obs_vec: &[f32]) -> Result<usize> {
        let (logits, _) = self.forward(state)?;
        let masked = masked_logits(&logits, obs_vec)?;
        let logits_vec: Vec<f32> = masked.squeeze(0)?.to_vec1()?;
        Ok(argmax(&logits_vec))
    }
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

fn distance_from_obs(obs_vec: &[f32]) -> f32 {
    obs_vec.get(OBS_DISTANCE_IDX).copied().unwrap_or(0.0) * OBS_DISTANCE_SCALE
}

/// 单样本攻击头掩码：距离超过阈值时掩掉「攻击」离散类（最后一个类）。
fn mask_hybrid_attack_single(logits: &Tensor, obs_vec: &[f32]) -> Result<Tensor> {
    let k = logits.dim(1)?;
    if k == 0 || distance_from_obs(obs_vec) <= ATTACK_MASK_DISTANCE {
        return Ok(logits.clone());
    }
    let mut mask = vec![0.0f32; k];
    mask[k - 1] = -1e9;
    let mask_tensor = Tensor::from_vec(mask, (1, k), logits.device())?;
    logits.broadcast_add(&mask_tensor)
}

/// 批量攻击头掩码：依据 state 的距离列掩掉「攻击」离散类。
fn mask_hybrid_attack_batch(logits: &Tensor, state: &Tensor) -> Result<Tensor> {
    let (n, k) = (logits.dim(0)?, logits.dim(1)?);
    if k == 0 || state.dim(1)? <= OBS_DISTANCE_IDX {
        return Ok(logits.clone());
    }
    let dist_vec: Vec<f32> = state
        .narrow(1, OBS_DISTANCE_IDX, 1)?
        .squeeze(1)?
        .to_vec1()?;
    let threshold = ATTACK_MASK_DISTANCE / OBS_DISTANCE_SCALE;
    let mut mask_vec = vec![0.0f32; n * k];
    let mut modified = false;
    for (i, &d) in dist_vec.iter().enumerate() {
        if d > threshold {
            mask_vec[i * k + k - 1] = -1e9;
            modified = true;
        }
    }
    if modified {
        let mask_tensor = Tensor::from_vec(mask_vec, (n, k), logits.device())?;
        logits.broadcast_add(&mask_tensor)
    } else {
        Ok(logits.clone())
    }
}

pub fn masked_logits(logits: &Tensor, obs_vec: &[f32]) -> Result<Tensor> {
    let action_dim = logits.dim(1)?;
    if action_dim == 0 || distance_from_obs(obs_vec) <= ATTACK_MASK_DISTANCE {
        return Ok(logits.clone());
    }
    // 「攻击」是最后一个离散动作：距离超限时掩掉它。
    let mut mask_vec = vec![0.0f32; action_dim];
    mask_vec[action_dim - 1] = -1e9;
    let mask_tensor = Tensor::from_vec(mask_vec, (1, action_dim), logits.device())?;
    logits.broadcast_add(&mask_tensor)
}

pub fn batch_masked_logits(logits: &Tensor, state: &Tensor) -> Result<Tensor> {
    let (n, action_dim) = (logits.dim(0)?, logits.dim(1)?);
    if action_dim == 0 || state.dim(1)? <= OBS_DISTANCE_IDX {
        return Ok(logits.clone());
    }
    let dist_vec: Vec<f32> = state
        .narrow(1, OBS_DISTANCE_IDX, 1)?
        .squeeze(1)?
        .to_vec1()?;
    let threshold = ATTACK_MASK_DISTANCE / OBS_DISTANCE_SCALE;
    let mut mask_vec = vec![0.0f32; n * action_dim];
    let mut modified = false;
    for (i, &d) in dist_vec.iter().enumerate() {
        if d > threshold {
            mask_vec[i * action_dim + action_dim - 1] = -1e9;
            modified = true;
        }
    }
    if modified {
        let mask_tensor = Tensor::from_vec(mask_vec, (n, action_dim), logits.device())?;
        logits.broadcast_add(&mask_tensor)
    } else {
        Ok(logits.clone())
    }
}
