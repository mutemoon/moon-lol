use candle_core::{D, Result, Tensor};
use candle_nn::{Linear, Module, VarBuilder, linear};
use rand::Rng;

#[derive(Clone)]
pub struct ActorCritic {
    fc1: Linear,
    fc2: Linear,
    actor_head: Linear,
    critic_head: Linear,
}

impl ActorCritic {
    pub fn new(
        state_dim: usize,
        hidden_dim: usize,
        action_dim: usize,
        vb: VarBuilder,
    ) -> Result<Self> {
        let fc1 = linear(state_dim, hidden_dim, vb.pp("fc1"))?;
        let fc2 = linear(hidden_dim, hidden_dim, vb.pp("fc2"))?;
        let actor_head = linear(hidden_dim, action_dim, vb.pp("actor_head"))?;
        let critic_head = linear(hidden_dim, 1, vb.pp("critic_head"))?;

        Ok(Self {
            fc1,
            fc2,
            actor_head,
            critic_head,
        })
    }

    /// Forward pass returning (logits, values)
    /// state shape: (batch_size, state_dim)
    /// logits shape: (batch_size, action_dim)
    /// values shape: (batch_size, 1)
    pub fn forward(&self, state: &Tensor) -> Result<(Tensor, Tensor)> {
        let h1 = self.fc1.forward(state)?.tanh()?;
        let h2 = self.fc2.forward(&h1)?.tanh()?;

        let logits = self.actor_head.forward(&h2)?;
        let values = self.critic_head.forward(&h2)?;

        Ok((logits, values))
    }

    /// Evaluate actions for PPO update
    /// state: (N, state_dim)
    /// actions: (N,) with u32 or i64
    pub fn evaluate_actions(
        &self,
        state: &Tensor,
        actions: &Tensor,
    ) -> Result<(Tensor, Tensor, Tensor)> {
        let (logits, values) = self.forward(state)?;
        let masked_logits = batch_masked_logits(&logits, state)?;

        let log_probs_all = candle_nn::ops::log_softmax(&masked_logits, D::Minus1)?;
        let probs_all = candle_nn::ops::softmax(&masked_logits, D::Minus1)?;

        let actions_dim = actions.unsqueeze(1)?;
        let selected_log_probs = log_probs_all.gather(&actions_dim, 1)?.squeeze(1)?;

        let entropy = (probs_all * log_probs_all)?
            .neg()?
            .sum_keepdim(D::Minus1)?
            .squeeze(1)?;

        Ok((selected_log_probs, values.squeeze(1)?, entropy))
    }

    /// Sample an action for a single state during environment rollout
    pub fn select_action(&self, state: &Tensor) -> Result<(usize, f32, f32)> {
        let (logits, value) = self.forward(state)?;
        let probs = candle_nn::ops::softmax(&logits, D::Minus1)?;
        let log_probs = candle_nn::ops::log_softmax(&logits, D::Minus1)?;

        let probs_vec: Vec<f32> = probs.squeeze(0)?.to_vec1()?;
        let log_probs_vec: Vec<f32> = log_probs.squeeze(0)?.to_vec1()?;
        let val_scalar: f32 = value.squeeze(0)?.squeeze(0)?.to_scalar()?;

        let mut rng = rand::rng();
        let r: f32 = rng.random();
        let mut cum_prob = 0.0;
        let mut chosen_action = probs_vec.len() - 1;

        for (idx, &prob) in probs_vec.iter().enumerate() {
            cum_prob += prob;
            if r <= cum_prob {
                chosen_action = idx;
                break;
            }
        }

        let chosen_log_prob = log_probs_vec[chosen_action];

        Ok((chosen_action, chosen_log_prob, val_scalar))
    }

    /// Sample an action with action masking (masks out movement actions 0..=3 when distance <= 60.0 and skills on cooldown)
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

        let mut rng = rand::rng();
        let r: f32 = rng.random();
        let mut cum_prob = 0.0;
        let mut chosen_action = probs_vec.len() - 1;

        for (idx, &prob) in probs_vec.iter().enumerate() {
            cum_prob += prob;
            if r <= cum_prob {
                chosen_action = idx;
                break;
            }
        }

        let chosen_log_prob = log_probs_vec[chosen_action];
        Ok((chosen_action, chosen_log_prob, val_scalar))
    }

    /// Return action probabilities for the given state (masked, softmax distribution)
    pub fn policy_probs(&self, state: &Tensor, obs_vec: &[f32]) -> Result<Vec<f32>> {
        let (logits, _) = self.forward(state)?;
        let masked = masked_logits(&logits, obs_vec)?;
        let probs = candle_nn::ops::softmax(&masked, D::Minus1)?;
        probs.squeeze(0)?.to_vec1()
    }

    /// Select the greedy action with action masking for deterministic evaluation
    pub fn select_greedy_action_masked(&self, state: &Tensor, obs_vec: &[f32]) -> Result<usize> {
        let (logits, _) = self.forward(state)?;
        let masked = masked_logits(&logits, obs_vec)?;
        let logits_vec: Vec<f32> = masked.squeeze(0)?.to_vec1()?;

        let mut max_idx = 0;
        let mut max_val = f32::NEG_INFINITY;
        for (idx, &val) in logits_vec.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = idx;
            }
        }
        Ok(max_idx)
    }
}

/// Apply action masking to logits based on observation state (single instance).
pub fn masked_logits(logits: &Tensor, obs_vec: &[f32]) -> Result<Tensor> {
    let mut logits_vec: Vec<f32> = logits.squeeze(0)?.to_vec1()?;
    let action_dim = logits.dim(1)?;

    let distance = if obs_vec.len() > 8 {
        obs_vec[8] * 100.0
    } else if obs_vec.len() > 6 {
        obs_vec[6] * 100.0
    } else {
        250.0
    };

    // 如果距离太远（> 220u），无法进行普通攻击，则屏蔽 Attack 动作
    if distance > 220.0 && action_dim > 4 {
        logits_vec[4] = -1e9;
    }

    Tensor::from_vec(logits_vec, (1, action_dim), logits.device())
}

/// Apply action masking to batch logits based on batch state tensor.
pub fn batch_masked_logits(logits: &Tensor, state: &Tensor) -> Result<Tensor> {
    let (n, action_dim) = (logits.dim(0)?, logits.dim(1)?);
    let state_dim = state.dim(1)?;
    if action_dim <= 4 || state_dim <= 8 {
        return Ok(logits.clone());
    }

    // 距离特征在第 8 列 (distance / 100.0)
    let dist_col = state.narrow(1, 8, 1)?;
    let dist_vec: Vec<f32> = dist_col.squeeze(1)?.to_vec1()?;
    let mut logits_vec: Vec<f32> = logits.to_vec2::<f32>()?.into_iter().flatten().collect();

    let mut modified = false;
    for (i, &d) in dist_vec.iter().enumerate() {
        if d > 2.2 {
            logits_vec[i * action_dim + 4] = -1e9;
            modified = true;
        }
    }

    if modified {
        Tensor::from_vec(logits_vec, (n, action_dim), logits.device())
    } else {
        Ok(logits.clone())
    }
}
