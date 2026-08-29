use std::path::Path;

use candle_core::{Device, Result};

use crate::algo::buffer::RolloutBuffer;
use crate::algo::grpo::GRPOAgent;
use crate::algo::ppo::{PPOAgent, PPOStats};
use crate::policy::{ModelParamSummary, PolicyNetwork, ValueHead};

/// 强化学习算法后端抽象：支持经典 PPO 与高效 GRPO
pub enum RlAgent {
    Ppo(PPOAgent),
    Grpo(GRPOAgent),
}

impl From<PPOAgent> for RlAgent {
    fn from(a: PPOAgent) -> Self {
        Self::Ppo(a)
    }
}

impl From<GRPOAgent> for RlAgent {
    fn from(a: GRPOAgent) -> Self {
        Self::Grpo(a)
    }
}

impl RlAgent {
    pub fn parameter_summary(&self) -> ModelParamSummary {
        match self {
            Self::Ppo(a) => a.parameter_summary(),
            Self::Grpo(a) => a.parameter_summary(),
        }
    }

    pub fn print_parameter_summary(&self) {
        match self {
            Self::Ppo(a) => a.print_parameter_summary(),
            Self::Grpo(a) => a.print_parameter_summary(),
        }
    }

    pub fn device(&self) -> &Device {
        match self {
            Self::Ppo(a) => a.device(),
            Self::Grpo(a) => a.device(),
        }
    }

    pub fn policy(&self) -> &PolicyNetwork {
        match self {
            Self::Ppo(a) => a.policy(),
            Self::Grpo(a) => &a.policy,
        }
    }

    pub fn critic(&self) -> Option<&ValueHead> {
        match self {
            Self::Ppo(a) => Some(a.critic()),
            Self::Grpo(_) => None,
        }
    }

    pub fn set_lr(&mut self, lr: f64) -> Result<()> {
        match self {
            Self::Ppo(a) => a.set_lr(lr),
            Self::Grpo(a) => a.set_lr(lr),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        match self {
            Self::Ppo(a) => a.save(path),
            Self::Grpo(a) => a.save(path),
        }
    }

    pub fn update_multi_buffer(
        &mut self,
        buffers: &[RolloutBuffer],
        last_vals: &[f32],
        mini_batch_size: usize,
    ) -> Result<PPOStats> {
        match self {
            Self::Ppo(a) => a.update_multi_buffer(buffers, last_vals, mini_batch_size),
            Self::Grpo(a) => {
                let stats = a.update_multi_buffer(buffers, mini_batch_size)?;
                Ok(stats.to_ppo_stats())
            }
        }
    }
}
