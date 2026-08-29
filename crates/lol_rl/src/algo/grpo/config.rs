use crate::algo::ppo::config::PPOStats;

/// GRPO 超参数配置 (Group Relative Policy Optimization)
#[derive(Clone, Debug)]
pub struct GRPOConfig {
    pub lr: f64,
    pub gamma: f32,
    pub clip_eps: f32,
    /// 每轮采样后的更新 Epoch 次数
    pub grpo_epochs: usize,
    /// 相对优势估计的分组大小 G（默认 4）
    pub group_size: usize,
    /// 全局梯度 L2 范数截断上限 (0.0 为不截断，推荐 0.5)
    pub max_grad_norm: f32,
}

impl Default for GRPOConfig {
    fn default() -> Self {
        Self {
            lr: 5e-4,
            gamma: 0.99,
            clip_eps: 0.2,
            grpo_epochs: 4,
            group_size: 4,
            max_grad_norm: 0.5,
        }
    }
}

/// GRPO 训练迭代统计指标
#[derive(Debug, Clone, Copy)]
pub struct GRPOStats {
    pub policy_loss: f32,
    /// 平均策略熵（正值，上报/展示用）
    pub entropy: f32,
    pub total_loss: f32,
    pub kl: f32,
    /// 本 epoch 被 clip 的比例（ratio 超出 [1-eps, 1+eps] 的占比）
    pub clip_frac: f32,
    /// 组内轨迹回报均值
    pub group_reward_mean: f32,
    /// 组内轨迹回报标准差
    pub group_reward_std: f32,
}

impl GRPOStats {
    /// 转换为系统统一上报的 PPOStats 结构（value_loss 恒为 0.0）
    pub fn to_ppo_stats(&self) -> PPOStats {
        PPOStats {
            policy_loss: self.policy_loss,
            value_loss: 0.0,
            entropy: self.entropy,
            total_loss: self.total_loss,
            kl: self.kl,
            clip_frac: self.clip_frac,
        }
    }
}
