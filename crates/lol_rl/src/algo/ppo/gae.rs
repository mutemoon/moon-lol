use crate::algo::buffer::RolloutBuffer;
use crate::algo::ppo::agent::PPOAgent;

impl PPOAgent {
    /// Compute GAE advantages and returns with True Truncation Bootstrapping
    pub fn compute_gae(&self, buffer: &RolloutBuffer, last_val: f32) -> (Vec<f32>, Vec<f32>) {
        let n = buffer.len();
        let mut returns = vec![0.0; n];
        let mut advantages = vec![0.0; n];

        let mut gae = 0.0;
        for t in (0..n).rev() {
            let truncated = buffer.truncateds.get(t).copied().unwrap_or(false);
            let done = buffer.dones.get(t).copied().unwrap_or(false);
            let terminated = done && !truncated;

            // 超时截断时，优先使用真实残局状态 s_T 的价值 V(s_T)，避免被新回合重置后的开局价值污染
            let next_val = if truncated {
                buffer
                    .truncated_next_values
                    .get(t)
                    .and_then(|v| *v)
                    .unwrap_or_else(|| {
                        if t + 1 < n {
                            buffer.values[t + 1]
                        } else {
                            last_val
                        }
                    })
            } else if t + 1 < n {
                buffer.values[t + 1]
            } else {
                last_val
            };

            // 真正的胜负/阵亡终止(terminated)没有未来价值(0.0)；
            // 超时截断(truncated)或正常推进保留未来期望价值 bootstrap (1.0)
            let next_non_terminal = if terminated { 0.0 } else { 1.0 };

            let delta = buffer.rewards[t] + self.config.gamma * next_val * next_non_terminal
                - buffer.values[t];

            // 回合结束（无论是 terminated 还是 truncated），GAE 优势递归在此步截断，不跨 episode 传递
            let gae_discount = if done { 0.0 } else { 1.0 };
            gae = delta + self.config.gamma * self.config.gae_lambda * gae_discount * gae;

            advantages[t] = gae;
            returns[t] = gae + buffer.values[t];
        }

        (returns, advantages)
    }
}
