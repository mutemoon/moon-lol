pub struct RolloutBuffer {
    pub states: Vec<Vec<f32>>,
    /// 扁平编码动作向量：Discrete=[idx]，Continuous=[v0..]，Hybrid=[v0, v1, attack_idx]。
    pub actions: Vec<Vec<f32>>,
    pub log_probs: Vec<f32>,
    pub rewards: Vec<f32>,
    pub values: Vec<f32>,
    pub dones: Vec<bool>,
    /// 是否为超时截断 (truncated)：true 表示时间步耗尽
    pub truncateds: Vec<bool>,
    /// 当该步发生超时截断时，超时瞬间真实残局状态 s_T 对应的无偏价值 V(s_T)
    pub truncated_next_values: Vec<Option<f32>>,
    /// 动作掩码（若环境提供）：true = 有效，false = 非法/屏蔽
    pub action_masks: Vec<Option<Vec<bool>>>,
}

impl RolloutBuffer {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            actions: Vec::new(),
            log_probs: Vec::new(),
            rewards: Vec::new(),
            values: Vec::new(),
            dones: Vec::new(),
            truncateds: Vec::new(),
            truncated_next_values: Vec::new(),
            action_masks: Vec::new(),
        }
    }

    pub fn push(
        &mut self,
        state: Vec<f32>,
        action: Vec<f32>,
        log_prob: f32,
        reward: f32,
        value: f32,
        done: bool,
        action_mask: Option<Vec<bool>>,
    ) {
        self.states.push(state);
        self.actions.push(action);
        self.log_probs.push(log_prob);
        self.rewards.push(reward);
        self.values.push(value);
        self.dones.push(done);
        self.truncateds.push(false);
        self.truncated_next_values.push(None);
        self.action_masks.push(action_mask);
    }

    pub fn push_full(
        &mut self,
        state: Vec<f32>,
        action: Vec<f32>,
        log_prob: f32,
        reward: f32,
        value: f32,
        terminated: bool,
        truncated: bool,
        truncated_next_value: Option<f32>,
        action_mask: Option<Vec<bool>>,
    ) {
        self.states.push(state);
        self.actions.push(action);
        self.log_probs.push(log_prob);
        self.rewards.push(reward);
        self.values.push(value);
        self.dones.push(terminated || truncated);
        self.truncateds.push(truncated);
        self.truncated_next_values.push(truncated_next_value);
        self.action_masks.push(action_mask);
    }

    pub fn push_unmasked(
        &mut self,
        state: Vec<f32>,
        action: Vec<f32>,
        log_prob: f32,
        reward: f32,
        value: f32,
        done: bool,
    ) {
        self.push(state, action, log_prob, reward, value, done, None);
    }

    pub fn clear(&mut self) {
        self.states.clear();
        self.actions.clear();
        self.log_probs.clear();
        self.rewards.clear();
        self.values.clear();
        self.dones.clear();
        self.truncateds.clear();
        self.truncated_next_values.clear();
        self.action_masks.clear();
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

impl Default for RolloutBuffer {
    fn default() -> Self {
        Self::new()
    }
}
