use candle_core::{Device, Result, Tensor};
use lol_rl_protocol::ActionMasks;

use crate::policy::{PolicyNetwork, ValueHead};

/// 策略推断/采样器接口：解耦游戏环境推演与策略前向计算（支持本地直接推演与异步通道批量推演）。
pub trait PolicyEvaluator {
    /// 对单个 Agent 的观测进行策略采样，返回 `(encoded_action, log_prob, value)`
    fn evaluate_step(
        &mut self,
        policy_slot: usize,
        state_vec: &[f32],
        action_mask: Option<&[bool]>,
        structured_mask: Option<&ActionMasks>,
        mamba_state: &mut Option<crate::policy::MambaState>,
    ) -> Result<(Vec<f32>, f32, f32)>;

    /// （可选优化）对多个同策略 Agent 批量前向采样，返回每项的 `(encoded_action, log_prob, value)`
    fn evaluate_batch(
        &mut self,
        policy_slot: usize,
        states: &[Vec<f32>],
        action_masks: &[Option<Vec<bool>>],
        structured_masks: &[Option<ActionMasks>],
    ) -> Result<Vec<(Vec<f32>, f32, f32)>> {
        let mut results = Vec::with_capacity(states.len());
        for i in 0..states.len() {
            let mut dummy_mamba = None;
            let act_lp_val = self.evaluate_step(
                policy_slot,
                &states[i],
                action_masks.get(i).and_then(|m| m.as_deref()),
                structured_masks.get(i).and_then(|m| m.as_ref()),
                &mut dummy_mamba,
            )?;
            results.push(act_lp_val);
        }
        Ok(results)
    }

    /// 单独推断一个状态的价值（用于残局 Bootstrap V(s_T)）
    fn evaluate_value(&mut self, policy_slot: usize, state_vec: &[f32]) -> Result<f32> {
        let mut dummy_mamba = None;
        let (_, _, val) = self.evaluate_step(policy_slot, state_vec, None, None, &mut dummy_mamba)?;
        Ok(val)
    }
}

/// 本地直接前向评估器（同步模式使用，直接在当前线程使用持有策略与价值网络）
pub struct DirectPolicyEvaluator<'a> {
    pub main_policy: &'a PolicyNetwork,
    pub main_critic: Option<&'a ValueHead>,
    pub opponent_policy: Option<&'a PolicyNetwork>,
    pub opponent_critic: Option<&'a ValueHead>,
    pub main_agent_idx: usize,
    pub state_dim: usize,
    pub device: &'a Device,
}

impl<'a> PolicyEvaluator for DirectPolicyEvaluator<'a> {
    fn evaluate_step(
        &mut self,
        policy_slot: usize,
        state_vec: &[f32],
        action_mask: Option<&[bool]>,
        structured_mask: Option<&ActionMasks>,
        mamba_state: &mut Option<crate::policy::MambaState>,
    ) -> Result<(Vec<f32>, f32, f32)> {
        let is_main = policy_slot == 0;
        let policy = if is_main {
            self.main_policy
        } else {
            self.opponent_policy.unwrap_or(self.main_policy)
        };
        let critic = if is_main {
            self.main_critic
        } else {
            self.opponent_critic.or(self.main_critic)
        };

        let state_tensor = Tensor::from_vec(state_vec.to_vec(), (1, self.state_dim), self.device)?;
        let mask_vec = action_mask.map(|m| m.to_vec());

        let (encoded, log_prob) = policy.step_with_structured_masks(
            &state_tensor,
            mamba_state,
            structured_mask,
            mask_vec.as_deref(),
        )?;

        let val = if let Some(c) = critic {
            let feat = policy.hidden(&state_tensor)?;
            let v = c.forward(&feat)?;
            v.squeeze(0)?.squeeze(0)?.to_scalar()?
        } else {
            0.0
        };

        Ok((encoded, log_prob, val))
    }

    fn evaluate_batch(
        &mut self,
        policy_slot: usize,
        states: &[Vec<f32>],
        action_masks: &[Option<Vec<bool>>],
        structured_masks: &[Option<ActionMasks>],
    ) -> Result<Vec<(Vec<f32>, f32, f32)>> {
        let is_main = policy_slot == 0;
        let policy = if is_main {
            self.main_policy
        } else {
            self.opponent_policy.unwrap_or(self.main_policy)
        };
        let critic = if is_main {
            self.main_critic
        } else {
            self.opponent_critic.or(self.main_critic)
        };

        let is_mlp = policy.backbone().backbone_type() == lol_rl_protocol::PolicyBackbone::Mlp;
        if is_mlp && states.len() > 1 {
            let mut flat = Vec::with_capacity(states.len() * self.state_dim);
            for s in states {
                flat.extend_from_slice(s);
            }
            let tensor = Tensor::from_vec(flat, (states.len(), self.state_dim), self.device)?;
            let samples = policy.sample_batch_with_structured_masks(
                &tensor,
                Some(structured_masks),
                Some(action_masks),
            )?;
            let val_vec = if let Some(c) = critic {
                let feat = policy.hidden(&tensor)?;
                let v = c.forward(&feat)?;
                v.squeeze(1)?.to_vec1()?
            } else {
                vec![0.0; states.len()]
            };
            let mut res = Vec::with_capacity(states.len());
            for ((enc, lp), v) in samples.into_iter().zip(val_vec.into_iter()) {
                res.push((enc, lp, v));
            }
            Ok(res)
        } else {
            let mut results = Vec::with_capacity(states.len());
            for i in 0..states.len() {
                let mut dummy_mamba = None;
                results.push(self.evaluate_step(
                    policy_slot,
                    &states[i],
                    action_masks.get(i).and_then(|m| m.as_deref()),
                    structured_masks.get(i).and_then(|m| m.as_ref()),
                    &mut dummy_mamba,
                )?);
            }
            Ok(results)
        }
    }
}
