#[derive(Clone, Debug)]
pub struct PPOConfig {
    pub lr: f64,
    pub gamma: f32,
    pub gae_lambda: f32,
    pub clip_eps: f32,
    pub c1: f32, // Value loss coefficient
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
    /// 平均策略熵（正值，上报/展示用）
    pub entropy: f32,
    pub total_loss: f32,
    pub kl: f32,
    /// 本 epoch 被 clip 的比例（ratio 超出 [1-eps, 1+eps] 的占比）
    pub clip_frac: f32,
}
