use std::path::Path;

use candle_core::{DType, Device, Result, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW, VarBuilder, VarMap};
use lol_rl_protocol::{ActionSpace, PolicyBackbone};

use crate::policy::{ActorCritic, HeroEmbedConfig};

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

#[derive(Clone)]
pub struct PPOConfig {
    pub lr: f64,
    pub gamma: f32,
    pub gae_lambda: f32,
    pub clip_eps: f32,
    pub c1: f32, // Value loss coefficient
    pub c2: f32, // Entropy coefficient
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
            c2: 0.05,
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
    pub entropy_loss: f32,
    /// 平均策略熵（正值，上报/展示用），与 entropy_loss（负值，参与总损失）区分。
    pub entropy: f32,
    pub total_loss: f32,
    pub kl: f32,
    /// 本 epoch 被 clip 的比例（ratio 超出 [1-eps, 1+eps] 的占比）
    pub clip_frac: f32,
}

pub struct PPOAgent {
    pub actor_critic: ActorCritic,
    varmap: VarMap,
    optimizer: AdamW,
    config: PPOConfig,
    device: Device,
    hero_embed_config: HeroEmbedConfig,
}

impl PPOAgent {
    pub fn new(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: PPOConfig,
        device: Device,
    ) -> Result<Self> {
        Self::with_hero_embed(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            HeroEmbedConfig::default(),
        )
    }

    /// Create a PPOAgent with custom hero-id embedding config and backbone.
    pub fn with_hero_embed(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: PPOConfig,
        device: Device,
        hero_embed_config: HeroEmbedConfig,
    ) -> Result<Self> {
        Self::with_hero_embed_and_backbone(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            hero_embed_config,
            PolicyBackbone::Mamba,
        )
    }

    /// 创建指定主干架构 (MLP 或 Mamba) 的 PPOAgent
    pub fn with_hero_embed_and_backbone(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: PPOConfig,
        device: Device,
        hero_embed_config: HeroEmbedConfig,
        backbone_type: PolicyBackbone,
    ) -> Result<Self> {
        let mut varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let actor_critic = ActorCritic::with_hero_embed_and_backbone(
            state_dim,
            hidden_dim,
            action_space,
            hero_embed_config.clone(),
            backbone_type,
            None,
            vb,
        )?;

        let in_dim = hero_embed_config.embed_dim + state_dim - 1;
        let hidden_gain = std::f32::consts::SQRT_2;

        match backbone_type {
            PolicyBackbone::Mlp => {
                let fc1_w = Tensor::from_vec(
                    crate::policy::orthogonal_weight(hidden_dim, in_dim, hidden_gain),
                    (hidden_dim, in_dim),
                    &device,
                )?;
                let fc2_w = Tensor::from_vec(
                    crate::policy::orthogonal_weight(hidden_dim, hidden_dim, hidden_gain),
                    (hidden_dim, hidden_dim),
                    &device,
                )?;
                let _ = varmap.set_one("fc1.weight", fc1_w);
                let _ = varmap.set_one("fc2.weight", fc2_w);
            }
            PolicyBackbone::Mamba => {
                let proj_in_w = Tensor::from_vec(
                    crate::policy::orthogonal_weight(hidden_dim, in_dim, hidden_gain),
                    (hidden_dim, in_dim),
                    &device,
                )?;
                let _ = varmap.set_one("proj_in.weight", proj_in_w);

                let d_inner = hidden_dim * 2;
                let d_state = 16;
                let mut a_log_vals = Vec::with_capacity(d_inner * d_state);
                for _ in 0..d_inner {
                    for j in 1..=d_state {
                        a_log_vals.push((j as f32).ln());
                    }
                }
                let a_log_tensor = Tensor::from_vec(a_log_vals, (d_inner, d_state), &device)?;
                let _ = varmap.set_one("mamba.A_log", a_log_tensor);
                let d_tensor = Tensor::ones(d_inner, DType::F32, &device)?;
                let _ = varmap.set_one("mamba.D", d_tensor);

                let dt_bias = Tensor::from_vec(vec![-3.0f32; d_inner], (d_inner,), &device)?;
                let _ = varmap.set_one("mamba.dt_proj.bias", dt_bias);

                let out_proj_w = Tensor::from_vec(
                    crate::policy::orthogonal_weight(hidden_dim, d_inner, 0.1),
                    (hidden_dim, d_inner),
                    &device,
                )?;
                let _ = varmap.set_one("mamba.out_proj.weight", out_proj_w);
            }
        }

        let actor_w = Tensor::from_vec(
            crate::policy::orthogonal_weight(
                actor_critic.action_space().actor_head_dim(),
                hidden_dim,
                0.01,
            ),
            (actor_critic.action_space().actor_head_dim(), hidden_dim),
            &device,
        )?;
        let critic_w = Tensor::from_vec(
            crate::policy::orthogonal_weight(1, hidden_dim, 1.0),
            (1, hidden_dim),
            &device,
        )?;

        let _ = varmap.set_one("actor_head.weight", actor_w);
        let _ = varmap.set_one("critic_head.weight", critic_w);

        let params = ParamsAdamW {
            lr: config.lr,
            ..Default::default()
        };
        let optimizer = AdamW::new(varmap.all_vars(), params)?;

        Ok(Self {
            actor_critic,
            varmap,
            optimizer,
            config,
            device,
            hero_embed_config,
        })
    }

    /// 统一为环境创建 PPOAgent，默认使用 Mamba 主干
    pub fn create_for_env<E: lol_env::RlEnvironment>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: PPOConfig,
        device: Device,
    ) -> Result<Self> {
        Self::create_for_env_with_backbone::<E>(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            PolicyBackbone::Mamba,
        )
    }

    /// 统一为环境创建 PPOAgent，支持指定 PolicyBackbone (MLP 或 Mamba)
    pub fn create_for_env_with_backbone<E: lol_env::RlEnvironment>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: PPOConfig,
        device: Device,
        backbone_type: PolicyBackbone,
    ) -> Result<Self> {
        Self::with_hero_embed_and_backbone(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            HeroEmbedConfig::default(),
            backbone_type,
        )
    }

    pub fn hero_embed_config(&self) -> &HeroEmbedConfig {
        &self.hero_embed_config
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    /// 当前学习率（用于训练循环中的退火调度）。
    pub fn lr(&self) -> f64 {
        self.config.lr
    }

    pub fn set_entropy_coef(&mut self, c2: f32) {
        self.config.c2 = c2;
    }

    pub fn set_lr(&mut self, lr: f64) -> Result<()> {
        self.config.lr = lr;
        let params = ParamsAdamW {
            lr,
            ..Default::default()
        };
        self.optimizer = AdamW::new(self.varmap.all_vars(), params)?;
        Ok(())
    }

    /// 全局梯度 L2 范数裁剪（Industrial PPO Standard: max_grad_norm = 0.5）
    pub fn clip_grad_norm(&self, grads: &mut candle_core::backprop::GradStore) -> Result<f32> {
        if self.config.max_grad_norm <= 0.0 {
            return Ok(0.0);
        }
        let vars = self.varmap.all_vars();
        let mut total_norm_sq = 0.0f32;
        for var in &vars {
            if let Some(grad) = grads.get(var) {
                let norm_sq: f32 = (grad * grad)?.sum_all()?.to_scalar()?;
                total_norm_sq += norm_sq;
            }
        }
        let total_norm = total_norm_sq.sqrt();
        let max_norm = self.config.max_grad_norm;
        if total_norm > max_norm {
            let scale = (max_norm / (total_norm + 1e-6)) as f64;
            for var in &vars {
                if let Some(grad) = grads.get(var) {
                    let scaled_grad = grad.affine(scale, 0.0)?;
                    grads.insert(var, scaled_grad);
                }
            }
        }
        Ok(total_norm)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    candle_core::Error::Msg(format!("创建 checkpoint 目录失败: {e}"))
                })?;
            }
        }
        self.varmap.save(path)
    }

    pub fn load(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: PPOConfig,
        device: Device,
        path: &Path,
    ) -> Result<Self> {
        let meta = std::fs::metadata(path)
            .map_err(|e| candle_core::Error::Msg(format!("checkpoint 文件不存在: {e}")))?;
        if meta.len() == 0 {
            return Err(candle_core::Error::Msg("checkpoint 文件为空".to_string()));
        }
        let tensors = candle_core::safetensors::load(path, &device)?;

        // 自动识别 Checkpoint 属于 MLP 还是 Mamba 架构
        let is_mlp = tensors.contains_key("fc1.weight") || tensors.contains_key("fc1.bias");
        let backbone_type = if is_mlp {
            PolicyBackbone::Mlp
        } else {
            PolicyBackbone::Mamba
        };

        // 从 fc2.bias / fc1.bias / proj_in.bias / proj_in.weight 的形状自动推断隐藏层维度，兼容不同 hidden_dim 的 checkpoint。
        let hidden_dim = tensors
            .get("fc2.bias")
            .or_else(|| tensors.get("fc1.bias"))
            .or_else(|| tensors.get("proj_in.bias"))
            .or_else(|| tensors.get("proj_in.weight"))
            .and_then(|t| t.shape().dims().first().copied())
            .unwrap_or(hidden_dim);

        // 校验 checkpoint 的动作空间结构与请求一致
        let has_log_std = tensors.contains_key("log_std");
        let has_attack_head = tensors.contains_key("attack_head.weight");
        let want_log_std = !matches!(action_space, ActionSpace::Discrete(_));
        let want_attack_head = matches!(action_space, ActionSpace::Hybrid { .. });
        if has_log_std != want_log_std || has_attack_head != want_attack_head {
            return Err(candle_core::Error::Msg(format!(
                "checkpoint 动作空间不匹配: 期望 log_std={want_log_std} attack_head={want_attack_head}, \
                 实际 log_std={has_log_std} attack_head={has_attack_head}"
            )));
        }

        let mut varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        // Detect hero embedding from checkpoint: if "hero_embed.weight" tensor exists, use its dims
        let hero_embed_config = tensors
            .get("hero_embed.weight")
            .map(|t| {
                let dims = t.shape().dims();
                HeroEmbedConfig {
                    num_heroes: dims.first().copied().unwrap_or(4),
                    embed_dim: dims.get(1).copied().unwrap_or(16),
                }
            })
            .unwrap_or_default();
        let actor_critic = ActorCritic::with_hero_embed_and_backbone(
            state_dim,
            hidden_dim,
            action_space,
            hero_embed_config.clone(),
            backbone_type,
            None,
            vb,
        )?;
        varmap.load(path)?;
        let params = ParamsAdamW {
            lr: config.lr,
            ..Default::default()
        };
        let optimizer = AdamW::new(varmap.all_vars(), params)?;
        Ok(Self {
            actor_critic,
            varmap,
            optimizer,
            config,
            device,
            hero_embed_config,
        })
    }

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

    /// Update policy using buffer data
    pub fn update(&mut self, buffer: &RolloutBuffer, last_val: f32) -> Result<PPOStats> {
        let n = buffer.len();
        if n == 0 {
            return Ok(PPOStats {
                policy_loss: 0.0,
                value_loss: 0.0,
                entropy_loss: 0.0,
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
            });
        }
        self.update_multi_buffer(std::slice::from_ref(buffer), &[last_val], n.min(64))
    }

    /// 使用 Mini-Batch 划分更新策略网络
    pub fn update_minibatch(
        &mut self,
        buffer: &RolloutBuffer,
        last_val: f32,
        mini_batch_size: usize,
    ) -> Result<PPOStats> {
        let n = buffer.len();
        if n == 0 {
            return Ok(PPOStats {
                policy_loss: 0.0,
                value_loss: 0.0,
                entropy_loss: 0.0,
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
            });
        }
        self.update_multi_buffer(std::slice::from_ref(buffer), &[last_val], mini_batch_size)
    }

    /// 多环境独立 GAE 计算 + 全样本 GPU Mini-Batch PPO 更新（支持 MLP 无状态打乱与 Mamba 时序切片）
    pub fn update_multi_buffer(
        &mut self,
        buffers: &[RolloutBuffer],
        last_vals: &[f32],
        mini_batch_size: usize,
    ) -> Result<PPOStats> {
        let total_n: usize = buffers.iter().map(|b| b.len()).sum();
        if total_n == 0 {
            return Ok(PPOStats {
                policy_loss: 0.0,
                value_loss: 0.0,
                entropy_loss: 0.0,
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
            });
        }

        // 取第一个非空 buffer 推断维度（首个 buffer 可能因 Worker 异常为空）
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

        let is_mamba = self.actor_critic.backbone().backbone_type() == PolicyBackbone::Mamba;

        use rand::seq::SliceRandom;
        let mut rng = rand::rng();

        let mut last_stats = PPOStats {
            policy_loss: 0.0,
            value_loss: 0.0,
            entropy_loss: 0.0,
            entropy: 0.0,
            total_loss: 0.0,
            kl: 0.0,
            clip_frac: 0.0,
        };

        // ════════════════════════════════════════════════════════════════
        // 路径 A：Mamba 时序状态空间模型（Chunk-based Recurrent PPO 时序切片训练）
        // ════════════════════════════════════════════════════════════════
        if is_mamba {
            let chunk_len = 16.min(total_n).max(1);
            struct TrajChunk {
                states: Vec<f32>,
                actions: Vec<f32>,
                old_log_probs: Vec<f32>,
                old_values: Vec<f32>,
                returns: Vec<f32>,
                advantages: Vec<f32>,
                masks: Option<Vec<f32>>,
            }

            let mut chunks = Vec::new();
            for (i, buffer) in buffers.iter().enumerate() {
                if buffer.is_empty() {
                    continue;
                }
                let last_val = last_vals.get(i).copied().unwrap_or(0.0);
                let (returns, advantages) = self.compute_gae(buffer, last_val);
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
                    let mut c_old_values = Vec::with_capacity(chunk_len);
                    let mut c_returns = Vec::with_capacity(chunk_len);
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
                        c_old_values.push(buffer.values[t]);
                        c_returns.push(returns[t]);
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

                    // 尾部不足 chunk_len 时做同状态填充，保证 3D Tensor 规整，advantages 设 0 避免梯度干扰
                    if cl < chunk_len {
                        let pad_count = chunk_len - cl;
                        let last_state = &buffer.states[end - 1];
                        for _ in 0..pad_count {
                            c_states.extend_from_slice(last_state);
                            c_actions.extend_from_slice(&buffer.actions[end - 1]);
                            c_log_probs.push(buffer.log_probs[end - 1]);
                            c_old_values.push(buffer.values[end - 1]);
                            c_returns.push(returns[end - 1]);
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
                        old_values: c_old_values,
                        returns: c_returns,
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

            for _epoch in 0..self.config.ppo_epochs {
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
                    let mut mb_old_values_vec = Vec::with_capacity(total_steps_mb);
                    let mut mb_returns_vec = Vec::with_capacity(total_steps_mb);
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
                        mb_old_values_vec.extend_from_slice(&c.old_values);
                        mb_returns_vec.extend_from_slice(&c.returns);
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
                    let mb_old_values =
                        Tensor::from_vec(mb_old_values_vec, (total_steps_mb,), &self.device)?;
                    let mb_returns =
                        Tensor::from_vec(mb_returns_vec, (total_steps_mb,), &self.device)?;
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

                    let mb_advantages_norm = if total_steps_mb > 1 {
                        let mean = mb_advantages.mean_all()?;
                        let diff = mb_advantages.broadcast_sub(&mean)?;
                        let var = (&diff * &diff)?.mean_all()?;
                        let std = (var + 1e-8)?.sqrt()?;
                        diff.broadcast_div(&std)?
                    } else {
                        mb_advantages
                    };

                    let (new_log_probs, new_values, entropy) = self.actor_critic.evaluate_actions(
                        &mb_states_3d,
                        &mb_actions,
                        mb_masks.as_ref(),
                    )?;

                    let log_ratio = (&new_log_probs - &mb_old_log_probs)?;
                    let ratio = log_ratio.exp()?;

                    let surr1 = (&ratio * &mb_advantages_norm)?;
                    let clamped_ratio =
                        ratio.clamp(1.0 - self.config.clip_eps, 1.0 + self.config.clip_eps)?;
                    let surr2 = (&clamped_ratio * &mb_advantages_norm)?;

                    let policy_loss = surr1.minimum(&surr2)?.neg()?.mean_all()?;

                    let value_loss = if self.config.clip_vloss {
                        let v_diff = (&new_values - &mb_old_values)?;
                        let v_clamped_diff =
                            v_diff.clamp(-self.config.clip_eps, self.config.clip_eps)?;
                        let v_clipped = (&mb_old_values + &v_clamped_diff)?;
                        let v_loss_unclipped = (&new_values - &mb_returns)?.powf(2.0)?;
                        let v_loss_clipped = (&v_clipped - &mb_returns)?.powf(2.0)?;
                        v_loss_unclipped
                            .maximum(&v_loss_clipped)?
                            .mean_all()?
                            .affine(0.5, 0.0)?
                    } else {
                        let val_diff = (&new_values - &mb_returns)?;
                        (&val_diff * &val_diff)?.mean_all()?.affine(0.5, 0.0)?
                    };

                    let entropy_loss = entropy.neg()?.mean_all()?;

                    let kl = (&ratio - 1.0 - &log_ratio)?.mean_all()?;
                    let clip_frac =
                        (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                            + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
                        .mean_all()?;

                    let p_loss_val: f32 = policy_loss.to_scalar()?;
                    let v_loss_val: f32 = value_loss.to_scalar()?;
                    let e_loss_val: f32 = entropy_loss.to_scalar()?;
                    let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
                    let kl_val: f32 = kl.to_scalar()?;
                    let clip_frac_val: f32 = clip_frac.to_scalar()?;

                    let c1_val =
                        (&policy_loss + (value_loss.affine(self.config.c1 as f64, 0.0)?))?;
                    let total_loss =
                        (c1_val + (entropy_loss.affine(self.config.c2 as f64, 0.0)?))?;
                    let tot_loss_val: f32 = total_loss.to_scalar()?;

                    let mut grads = total_loss.backward()?;
                    self.clip_grad_norm(&mut grads)?;
                    self.optimizer.step(&grads)?;

                    last_stats = PPOStats {
                        policy_loss: p_loss_val,
                        value_loss: v_loss_val,
                        entropy_loss: e_loss_val,
                        entropy: entropy_val,
                        total_loss: tot_loss_val,
                        kl: kl_val,
                        clip_frac: clip_frac_val,
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
        let mut all_old_values = Vec::with_capacity(total_n);
        let mut all_returns = Vec::with_capacity(total_n);
        let mut all_advantages = Vec::with_capacity(total_n);
        let mut all_masks: Option<Vec<f32>> = if has_masks {
            Some(Vec::with_capacity(total_n * mask_dim))
        } else {
            None
        };

        for (i, buffer) in buffers.iter().enumerate() {
            if buffer.is_empty() {
                continue;
            }
            let last_val = last_vals.get(i).copied().unwrap_or(0.0);
            let (returns, advantages) = self.compute_gae(buffer, last_val);

            for t in 0..buffer.len() {
                all_states.extend_from_slice(&buffer.states[t]);
                all_actions.extend_from_slice(&buffer.actions[t]);
                all_old_log_probs.push(buffer.log_probs[t]);
                all_old_values.push(buffer.values[t]);
                all_returns.push(returns[t]);
                all_advantages.push(advantages[t]);
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

        // Normalize advantages globally across all buffers
        let mean = all_advantages.iter().sum::<f32>() / total_n as f32;
        let variance = all_advantages
            .iter()
            .map(|a| (a - mean).powi(2))
            .sum::<f32>()
            / total_n as f32;
        let std = (variance + 1e-8).sqrt();
        for a in all_advantages.iter_mut() {
            *a = (*a - mean) / std;
        }

        let mut last_stats = PPOStats {
            policy_loss: 0.0,
            value_loss: 0.0,
            entropy_loss: 0.0,
            entropy: 0.0,
            total_loss: 0.0,
            kl: 0.0,
            clip_frac: 0.0,
        };

        let mut rng = rand::rng();

        for _epoch in 0..self.config.ppo_epochs {
            let mut indices: Vec<usize> = (0..total_n).collect();
            indices.shuffle(&mut rng);

            let mut shuffled_states = Vec::with_capacity(total_n * state_dim);
            let mut shuffled_actions = Vec::with_capacity(total_n * enc_dim);
            let mut shuffled_log_probs = Vec::with_capacity(total_n);
            let mut shuffled_old_values = Vec::with_capacity(total_n);
            let mut shuffled_returns = Vec::with_capacity(total_n);
            let mut shuffled_advantages = Vec::with_capacity(total_n);
            let mut shuffled_masks: Option<Vec<f32>> = if has_masks {
                Some(Vec::with_capacity(total_n * mask_dim))
            } else {
                None
            };

            for &idx in &indices {
                shuffled_states
                    .extend_from_slice(&all_states[idx * state_dim..(idx + 1) * state_dim]);
                shuffled_actions
                    .extend_from_slice(&all_actions[idx * enc_dim..(idx + 1) * enc_dim]);
                shuffled_log_probs.push(all_old_log_probs[idx]);
                shuffled_old_values.push(all_old_values[idx]);
                shuffled_returns.push(all_returns[idx]);
                shuffled_advantages.push(all_advantages[idx]);
                if let (Some(sm), Some(am)) = (&mut shuffled_masks, &all_masks) {
                    sm.extend_from_slice(&am[idx * mask_dim..(idx + 1) * mask_dim]);
                }
            }

            let states_tensor =
                Tensor::from_vec(shuffled_states, (total_n, state_dim), &self.device)?;
            let actions_tensor =
                Tensor::from_vec(shuffled_actions, (total_n, enc_dim), &self.device)?;
            let old_log_probs_tensor =
                Tensor::from_vec(shuffled_log_probs, (total_n,), &self.device)?;
            let old_values_tensor =
                Tensor::from_vec(shuffled_old_values, (total_n,), &self.device)?;
            let returns_tensor = Tensor::from_vec(shuffled_returns, (total_n,), &self.device)?;
            let advantages_tensor =
                Tensor::from_vec(shuffled_advantages, (total_n,), &self.device)?;
            let masks_tensor = if let Some(sm) = shuffled_masks {
                Some(Tensor::from_vec(sm, (total_n, mask_dim), &self.device)?)
            } else {
                None
            };

            let mut start_idx = 0;
            while start_idx < total_n {
                let end_idx = (start_idx + mini_batch_size).min(total_n);
                let mb_len = end_idx - start_idx;

                let mb_states = states_tensor.narrow(0, start_idx, mb_len)?;
                let mb_actions = actions_tensor.narrow(0, start_idx, mb_len)?;
                let mb_old_log_probs = old_log_probs_tensor.narrow(0, start_idx, mb_len)?;
                let mb_old_values = old_values_tensor.narrow(0, start_idx, mb_len)?;
                let mb_returns = returns_tensor.narrow(0, start_idx, mb_len)?;
                let mb_advantages = advantages_tensor.narrow(0, start_idx, mb_len)?;
                let mb_masks = if let Some(ref mt) = masks_tensor {
                    Some(mt.narrow(0, start_idx, mb_len)?)
                } else {
                    None
                };

                // Mini-Batch 内部优势重归一化 (CleanRL / PPO2 Detail)
                let mb_advantages_norm = if mb_len > 1 {
                    let mean = mb_advantages.mean_all()?;
                    let diff = mb_advantages.broadcast_sub(&mean)?;
                    let var = (&diff * &diff)?.mean_all()?;
                    let std = (var + 1e-8)?.sqrt()?;
                    diff.broadcast_div(&std)?
                } else {
                    mb_advantages
                };

                let (new_log_probs, new_values, entropy) = self.actor_critic.evaluate_actions(
                    &mb_states,
                    &mb_actions,
                    mb_masks.as_ref(),
                )?;

                let log_ratio = (&new_log_probs - &mb_old_log_probs)?;
                let ratio = log_ratio.exp()?;

                let surr1 = (&ratio * &mb_advantages_norm)?;
                let clamped_ratio =
                    ratio.clamp(1.0 - self.config.clip_eps, 1.0 + self.config.clip_eps)?;
                let surr2 = (&clamped_ratio * &mb_advantages_norm)?;

                let policy_loss = surr1.minimum(&surr2)?.neg()?.mean_all()?;

                // Value Loss: PPO2 Clipped Value Loss
                let value_loss = if self.config.clip_vloss {
                    let v_diff = (&new_values - &mb_old_values)?;
                    let v_clamped_diff =
                        v_diff.clamp(-self.config.clip_eps, self.config.clip_eps)?;
                    let v_clipped = (&mb_old_values + &v_clamped_diff)?;
                    let v_loss_unclipped = (&new_values - &mb_returns)?.powf(2.0)?;
                    let v_loss_clipped = (&v_clipped - &mb_returns)?.powf(2.0)?;
                    v_loss_unclipped
                        .maximum(&v_loss_clipped)?
                        .mean_all()?
                        .affine(0.5, 0.0)?
                } else {
                    let val_diff = (&new_values - &mb_returns)?;
                    (&val_diff * &val_diff)?.mean_all()?.affine(0.5, 0.0)?
                };

                let entropy_loss = entropy.neg()?.mean_all()?;

                let kl = (&ratio - 1.0 - &log_ratio)?.mean_all()?;
                let clip_frac = (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                    + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
                .mean_all()?;

                let p_loss_val: f32 = policy_loss.to_scalar()?;
                let v_loss_val: f32 = value_loss.to_scalar()?;
                let e_loss_val: f32 = entropy_loss.to_scalar()?;
                let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
                let kl_val: f32 = kl.to_scalar()?;
                let clip_frac_val: f32 = clip_frac.to_scalar()?;

                let c1_val = (&policy_loss + (value_loss.affine(self.config.c1 as f64, 0.0)?))?;
                let total_loss = (c1_val + (entropy_loss.affine(self.config.c2 as f64, 0.0)?))?;
                let tot_loss_val: f32 = total_loss.to_scalar()?;

                let mut grads = total_loss.backward()?;
                self.clip_grad_norm(&mut grads)?;
                self.optimizer.step(&grads)?;

                last_stats = PPOStats {
                    policy_loss: p_loss_val,
                    value_loss: v_loss_val,
                    entropy_loss: e_loss_val,
                    entropy: entropy_val,
                    total_loss: tot_loss_val,
                    kl: kl_val,
                    clip_frac: clip_frac_val,
                };

                start_idx += mini_batch_size;
            }
        }

        Ok(last_stats)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn save_load_roundtrip() -> Result<()> {
        let state_dim = 17;
        let hidden_dim = 64;
        let action_dim = 9;
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            ActionSpace::Discrete(action_dim),
            config.clone(),
            device.clone(),
        )?;

        let obs_vec: Vec<f32> = (0..state_dim).map(|i| i as f32 * 0.1).collect();
        let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device)?;
        let (probs_before, _) = agent.actor_critic.forward(&state)?;
        let probs_before_vec: Vec<f32> = probs_before.squeeze(0)?.to_vec1()?;

        let tmp_dir = std::env::temp_dir().join("moon_lol_test");
        std::fs::create_dir_all(&tmp_dir).ok();
        let save_path = tmp_dir.join("test_ckpt.safetensors");
        let _ = std::fs::remove_file(&save_path);
        agent.save(&save_path)?;
        assert!(save_path.exists());
        assert!(save_path.metadata().unwrap().len() > 0);

        let loaded = PPOAgent::load(
            state_dim,
            hidden_dim,
            ActionSpace::Discrete(action_dim),
            config.clone(),
            device.clone(),
            &save_path,
        )?;
        let (probs_after, _) = loaded.actor_critic.forward(&state)?;
        let probs_after_vec: Vec<f32> = probs_after.squeeze(0)?.to_vec1()?;

        for (i, (b, a)) in probs_before_vec
            .iter()
            .zip(probs_after_vec.iter())
            .enumerate()
        {
            assert!(
                (b - a).abs() < 1e-4,
                "action {} prob mismatch: before={}, after={}",
                i,
                b,
                a
            );
        }

        let _ = std::fs::remove_file(&save_path);
        Ok(())
    }

    #[test]
    fn load_empty_file_fails() {
        let tmp_dir = std::env::temp_dir().join("moon_lol_test_empty");
        std::fs::create_dir_all(&tmp_dir).ok();
        let empty_path = tmp_dir.join("empty.safetensors");
        std::fs::write(&empty_path, []).ok();

        let result = PPOAgent::load(
            17,
            64,
            ActionSpace::Discrete(9),
            PPOConfig::default(),
            Device::Cpu,
            &empty_path,
        );
        assert!(result.is_err());

        let _ = std::fs::remove_file(&empty_path);
    }

    #[test]
    fn load_custom_hidden_dim_auto_detect() -> Result<()> {
        let state_dim = 17;
        let hidden_dim = 256;
        let action_dim = 5;
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            ActionSpace::Discrete(action_dim),
            config.clone(),
            device.clone(),
        )?;

        let tmp_dir = std::env::temp_dir().join("moon_lol_test_256");
        std::fs::create_dir_all(&tmp_dir).ok();
        let save_path = tmp_dir.join("test_ckpt_256.safetensors");
        let _ = std::fs::remove_file(&save_path);
        agent.save(&save_path)?;

        // Load with dummy hidden_dim=64, it should auto-detect 256 from safetensors file
        let loaded = PPOAgent::load(
            state_dim,
            64,
            ActionSpace::Discrete(action_dim),
            config,
            device.clone(),
            &save_path,
        )?;
        let state = Tensor::zeros((1, state_dim), DType::F32, &device)?;
        let (probs, val) = loaded.actor_critic.forward(&state)?;
        assert_eq!(probs.dim(1)?, action_dim);
        assert_eq!(val.dim(1)?, 1);

        let _ = std::fs::remove_file(&save_path);
        Ok(())
    }

    #[test]
    fn load_nonexistent_file_fails() {
        let result = PPOAgent::load(
            17,
            64,
            ActionSpace::Discrete(9),
            PPOConfig::default(),
            Device::Cpu,
            &PathBuf::from("/nonexistent/path/checkpoint.safetensors"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn hybrid_ppo_smoke() -> Result<()> {
        let state_dim = 9;
        let hidden_dim = 32;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 2,
        };
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
        )?;

        let mut buffer = RolloutBuffer::new();
        for _ in 0..8 {
            let obs_vec: Vec<f32> = (0..state_dim).map(|i| i as f32 * 0.1).collect();
            let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device)?;
            let (encoded, log_prob, value) = agent.actor_critic.sample_action(&state, None)?;
            assert_eq!(encoded.len(), 3, "hybrid 编码应为 [move_x, move_z, attack]");
            buffer.push_unmasked(obs_vec, encoded, log_prob, 0.1, value, false);
        }

        let stats = agent.update(&buffer, 0.0)?;
        assert!(stats.policy_loss.is_finite(), "policy_loss 应为有限值");
        assert!(stats.value_loss.is_finite(), "value_loss 应为有限值");

        // 保存/加载混合 checkpoint
        let tmp_dir = std::env::temp_dir().join("moon_lol_test_hybrid");
        std::fs::create_dir_all(&tmp_dir).ok();
        let save_path = tmp_dir.join("hybrid_ckpt.safetensors");
        let _ = std::fs::remove_file(&save_path);
        agent.save(&save_path)?;
        let loaded = PPOAgent::load(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            &save_path,
        )?;
        let obs_vec: Vec<f32> = (0..state_dim).map(|i| i as f32 * 0.1).collect();
        let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &loaded.device())?;
        let (encoded_after, _, _) = loaded.actor_critic.sample_action(&state, None)?;
        assert_eq!(encoded_after.len(), 3);

        let _ = std::fs::remove_file(&save_path);
        Ok(())
    }

    #[test]
    fn hybrid_ppo_fiora_v2_smoke() -> Result<()> {
        let state_dim = 58;
        let hidden_dim = 64;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 7,
        };
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
        )?;

        let mut buffer = RolloutBuffer::new();
        for step in 0..16 {
            let mut obs_vec: Vec<f32> = (0..state_dim).map(|i| (i as f32 * 0.05).sin()).collect();
            obs_vec[16] = if step % 2 == 0 { 1.5 } else { 3.0 };
            let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device)?;
            let mask = Some(vec![true, true, step % 2 == 0, true, true, true, true]);
            let (encoded, log_prob, value) =
                agent.actor_critic.sample_action(&state, mask.as_deref())?;
            assert_eq!(
                encoded.len(),
                3,
                "FioraV2 hybrid 动作编码应为 [offset_x, offset_z, discrete_idx]"
            );
            let disc_idx = encoded[2] as usize;
            assert!(disc_idx < 7, "离散动作索引应在 [0, 6] 范围内");
            buffer.push(obs_vec, encoded, log_prob, 0.25, value, false, mask);
        }

        let stats = agent.update(&buffer, 0.0)?;
        assert!(
            stats.policy_loss.is_finite(),
            "V2 policy_loss 应为有效有限值"
        );
        assert!(stats.value_loss.is_finite(), "V2 value_loss 应为有效有限值");
        assert!(
            stats.entropy_loss.is_finite(),
            "V2 entropy_loss 应为有效有限值"
        );

        // 验证批量采样 sample_batch
        let states = Tensor::zeros((4, state_dim), DType::F32, &device)?;
        let batch_samples = agent.actor_critic.sample_batch(&states, None)?;
        assert_eq!(batch_samples.len(), 4);

        // 验证策略可视化显示
        let dummy_state = Tensor::zeros((1, state_dim), DType::F32, &device)?;
        let labels = ["NoOp", "Move", "Attack", "Q", "E", "R", "Flash"];
        let display = agent
            .actor_critic
            .policy_display_real(&dummy_state, None, &labels)?;
        match display {
            lol_rl_protocol::PolicyDisplay::HybridMulti {
                continuous_means,
                discrete_probs,
            } => {
                assert_eq!(continuous_means.len(), 2);
                assert_eq!(discrete_probs.len(), 7);
                let sum_prob: f32 = discrete_probs.iter().map(|p| p.prob).sum();
                assert!((sum_prob - 1.0).abs() < 1e-3, "离散概率之和应为 1.0");
            }
            other => panic!("预期返回 PolicyDisplay::HybridMulti，实际为 {:?}", other),
        }

        // 验证多 Buffer 掩码 Mini-Batch 更新
        let stats_multi = agent.update_multi_buffer(&[buffer], &[0.0], 8)?;
        assert!(
            stats_multi.policy_loss.is_finite(),
            "update_multi_buffer policy_loss 应为有效有限值"
        );
        assert!(
            stats_multi.value_loss.is_finite(),
            "update_multi_buffer value_loss 应为有效有限值"
        );

        Ok(())
    }

    #[test]
    fn selfplay_single_policy_smoke() -> Result<()> {
        let state_dim = 60; // 包含 role_id 与 40 维修饰符槽位
        let hidden_dim = 64;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 8,
        };
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
        )?;

        let mut buffer_f = RolloutBuffer::new();
        let mut buffer_r = RolloutBuffer::new();

        // 模拟自博弈推演：双方 Agent 各自维护独立的轨迹 Buffer
        for step in 0..10 {
            // 1. Fiora 视角
            let mut obs_f = vec![0.0f32; state_dim];
            obs_f[0] = 0.0; // role_id = 0.0 (Fiora)
            obs_f[17] = 1.2; // distance / 100
            let state_f = Tensor::from_vec(obs_f.clone(), (1, state_dim), &device)?;
            let mask_f = Some(vec![true, true, true, true, true, true, true, true]);
            let (act_f, log_prob_f, val_f) = agent
                .actor_critic
                .sample_action(&state_f, mask_f.as_deref())?;
            assert_eq!(act_f.len(), 3);
            let reward_f = if step % 2 == 0 { 0.5 } else { -0.5 };
            buffer_f.push(obs_f, act_f, log_prob_f, reward_f, val_f, false, mask_f);

            // 2. Riven 视角
            let mut obs_r = vec![0.0f32; state_dim];
            obs_r[0] = 1.0; // role_id = 1.0 (Riven)
            obs_r[17] = 1.2; // distance / 100
            let state_r = Tensor::from_vec(obs_r.clone(), (1, state_dim), &device)?;
            let mask_r = Some(vec![true, true, true, true, true, true, true, true]);
            let (act_r, log_prob_r, val_r) = agent
                .actor_critic
                .sample_action(&state_r, mask_r.as_deref())?;
            assert_eq!(act_r.len(), 3);
            let reward_r = -reward_f; // 严格零和
            buffer_r.push(obs_r, act_r, log_prob_r, reward_r, val_r, false, mask_r);
        }

        assert_eq!(buffer_f.len(), 10);
        assert_eq!(buffer_r.len(), 10);

        // 执行单模型多角色样本独立 GAE + 联合 Mini-Batch PPO 更新
        let stats = agent.update_multi_buffer(&[buffer_f, buffer_r], &[0.0, 0.0], 8)?;
        assert!(stats.policy_loss.is_finite());
        assert!(stats.value_loss.is_finite());
        assert!(stats.entropy_loss.is_finite());

        Ok(())
    }

    #[test]
    fn test_orthogonal_weight_properties() {
        use crate::policy::orthogonal_weight;
        let out_dim = 16;
        let in_dim = 32;
        let gain = 1.414f32;
        let w = orthogonal_weight(out_dim, in_dim, gain);
        assert_eq!(w.len(), out_dim * in_dim);

        // 验证行向量正交性：W * W^T ≈ gain^2 * I
        for r1 in 0..out_dim {
            for r2 in 0..out_dim {
                let dot: f32 = (0..in_dim)
                    .map(|c| w[r1 * in_dim + c] * w[r2 * in_dim + c])
                    .sum();
                if r1 == r2 {
                    let expected = gain * gain;
                    assert!(
                        (dot - expected).abs() < 1e-3,
                        "对角元素 dot ({dot}) 应接近 gain^2 ({expected})"
                    );
                } else {
                    assert!(dot.abs() < 1e-3, "非对角元素 dot ({dot}) 应接近 0 (正交)");
                }
            }
        }
    }

    #[test]
    fn test_industrial_ppo_clip_vloss_and_grad_norm() -> Result<()> {
        let state_dim = 8;
        let hidden_dim = 32;
        let action_space = ActionSpace::Discrete(4);
        let mut config = PPOConfig::default();
        config.clip_vloss = true;
        config.max_grad_norm = 0.5;
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(state_dim, hidden_dim, action_space, config, device)?;
        agent.set_lr(1e-4)?;

        let mut buffer = RolloutBuffer::new();
        for _ in 0..10 {
            let obs = vec![0.5f32; state_dim];
            buffer.push_unmasked(obs, vec![0.0], -0.5, 1.0, 0.2, false);
        }

        let stats = agent.update(&buffer, 0.0)?;
        assert!(stats.policy_loss.is_finite());
        assert!(stats.value_loss.is_finite());
        assert!(stats.total_loss.is_finite());
        assert!(stats.kl.is_finite());
        assert!(stats.clip_frac >= 0.0 && stats.clip_frac <= 1.0);

        Ok(())
    }

    #[test]
    fn test_truncation_vs_termination_gae() -> Result<()> {
        let state_dim = 4;
        let hidden_dim = 16;
        let action_space = ActionSpace::Discrete(2);
        let mut config = PPOConfig::default();
        config.gamma = 0.99;
        config.gae_lambda = 0.95;
        let device = Device::Cpu;

        let agent = PPOAgent::new(state_dim, hidden_dim, action_space, config, device)?;

        // 场景 1: 真正终止 (terminated = true, truncated = false)
        let mut buffer_term = RolloutBuffer::new();
        buffer_term.push_full(
            vec![0.0; state_dim],
            vec![0.0],
            -0.1,
            1.0,
            0.5,
            true,  // terminated
            false, // truncated
            None,
            None,
        );
        let (_, adv_term) = agent.compute_gae(&buffer_term, 2.0);
        // delta = reward(1.0) + gamma * next_val(2.0) * 0.0 - val(0.5) = 0.5
        assert!(
            (adv_term[0] - 0.5).abs() < 1e-5,
            "真正终止不应 bootstrap 任何未来价值"
        );

        // 场景 2: 超时截断 (terminated = false, truncated = true, 指定真实残局价值 3.0)
        let mut buffer_trunc = RolloutBuffer::new();
        buffer_trunc.push_full(
            vec![0.0; state_dim],
            vec![0.0],
            -0.1,
            1.0,
            0.5,
            false,     // terminated
            true,      // truncated
            Some(3.0), // 真实残局价值
            None,
        );
        // 传入 last_val = 0.0 (开局重置价值)，但应优先使用 3.0 真实残局价值
        let (_, adv_trunc) = agent.compute_gae(&buffer_trunc, 0.0);
        // delta = reward(1.0) + gamma(0.99) * next_val(3.0) * 1.0 - val(0.5) = 1.0 + 2.97 - 0.5 = 3.47
        assert!(
            (adv_trunc[0] - 3.47).abs() < 1e-4,
            "超时截断必须优先使用真实残局价值进行无偏 bootstrap"
        );

        Ok(())
    }

    #[test]
    fn test_hero_id_embedding_selfplay_and_checkpoint() -> Result<()> {
        let state_dim = 36;
        let hidden_dim = 64;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 8,
        };
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let hero_cfg = HeroEmbedConfig {
            num_heroes: 2,
            embed_dim: 16,
        };

        let mut agent = PPOAgent::with_hero_embed(
            state_dim,
            hidden_dim,
            action_space.clone(),
            config.clone(),
            device.clone(),
            hero_cfg,
        )?;
        assert!(agent.actor_critic.has_hero_embed());

        // 验证两种角色的前向推理与采样
        let mut buffer_f = RolloutBuffer::new();
        let mut buffer_r = RolloutBuffer::new();

        for _ in 0..10 {
            // Fiora 视角 (role_id = 0.0)
            let mut obs_f = vec![0.0f32; state_dim];
            obs_f[0] = 0.0;
            obs_f[17] = 1.0;
            let state_f = Tensor::from_vec(obs_f.clone(), (1, state_dim), &device)?;
            let (act_f, log_prob_f, val_f) = agent.actor_critic.sample_action(&state_f, None)?;
            assert_eq!(act_f.len(), 3);
            buffer_f.push_unmasked(obs_f, act_f, log_prob_f, 1.0, val_f, false);

            // Riven 视角 (role_id = 1.0)
            let mut obs_r = vec![0.0f32; state_dim];
            obs_r[0] = 1.0;
            obs_r[17] = 1.0;
            let state_r = Tensor::from_vec(obs_r.clone(), (1, state_dim), &device)?;
            let (act_r, log_prob_r, val_r) = agent.actor_critic.sample_action(&state_r, None)?;
            assert_eq!(act_r.len(), 3);
            buffer_r.push_unmasked(obs_r, act_r, log_prob_r, -1.0, val_r, false);
        }

        // PPO 联合更新
        let stats = agent.update_multi_buffer(&[buffer_f, buffer_r], &[0.0, 0.0], 8)?;
        assert!(stats.policy_loss.is_finite());
        assert!(stats.value_loss.is_finite());

        // 保存并恢复 checkpoint
        let tmp_dir = std::env::temp_dir();
        let ckpt_path = tmp_dir.join("test_hero_embed_model.safetensors");
        agent.save(&ckpt_path)?;

        let loaded_agent = PPOAgent::load(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device.clone(),
            &ckpt_path,
        )?;
        assert!(loaded_agent.actor_critic.has_hero_embed());

        // 验证加载后的模型与原模型输出一致
        let test_obs = vec![1.0f32; state_dim];
        let test_t = Tensor::from_vec(test_obs, (1, state_dim), &device)?;
        let orig_v = agent.actor_critic.get_values(&test_t)?;
        let loaded_v = loaded_agent.actor_critic.get_values(&test_t)?;
        assert!((orig_v[0] - loaded_v[0]).abs() < 1e-5);

        let _ = std::fs::remove_file(&ckpt_path);
        Ok(())
    }

    #[test]
    fn test_mamba_policy_forward_and_ssm() -> Result<()> {
        let state_dim = 16;
        let hidden_dim = 32;
        let action_space = ActionSpace::Discrete(5);
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let agent = PPOAgent::new(state_dim, hidden_dim, action_space, config, device.clone())?;

        // 1. 2D Tensor (batch, state_dim)
        let obs_2d = Tensor::randn(0.0f32, 1.0f32, (4, state_dim), &device)?;
        let (logits_2d, values_2d) = agent.actor_critic.forward(&obs_2d)?;
        assert_eq!(logits_2d.dims(), &[4, 5]);
        assert_eq!(values_2d.dims(), &[4, 1]);

        // 2. 3D Tensor 序列 (batch, seq_len, state_dim)
        let obs_3d = Tensor::randn(0.0f32, 1.0f32, (2, 8, state_dim), &device)?;
        let (logits_3d, values_3d) = agent.actor_critic.forward(&obs_3d)?;
        assert_eq!(logits_3d.dims(), &[2, 8, 5]);
        assert_eq!(values_3d.dims(), &[2, 8, 1]);

        // 3. 验证 Mamba 参数梯度反向传播
        let (log_probs, values, entropy) = agent.actor_critic.evaluate_actions(
            &obs_2d,
            &Tensor::zeros((4, 1), DType::F32, &device)?,
            None,
        )?;
        let loss = ((&log_probs + &values)? + &entropy)?.sum_all()?;
        let _grads = loss.backward()?;
        assert!(loss.to_scalar::<f32>()?.is_finite());

        Ok(())
    }

    #[test]
    fn test_belief_state_mamba() -> Result<()> {
        let state_dim = 20;
        let hidden_dim = 32;
        let belief_dim = 8;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 4,
        };
        let device = Device::Cpu;

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let mamba_config = crate::policy::MambaConfig::new(hidden_dim);
        let ac = ActorCritic::with_hero_embed_and_mamba(
            state_dim,
            hidden_dim,
            action_space,
            HeroEmbedConfig::default(),
            mamba_config,
            Some(belief_dim),
            vb,
        )?;

        assert!(ac.belief_head().is_some());
        assert_eq!(ac.belief_head().unwrap().belief_dim, belief_dim);

        let dummy_state = Tensor::zeros((2, state_dim), DType::F32, &device)?;
        let belief_res = ac.forward_belief(&dummy_state)?;
        assert!(belief_res.is_some());
        let (mu, std) = belief_res.unwrap();
        assert_eq!(mu.dims(), &[2, belief_dim]);
        assert_eq!(std.dims(), &[2, belief_dim]);

        // 验证 std 全部为正数
        let std_vec = std.flatten_all()?.to_vec1::<f32>()?;
        for s in std_vec {
            assert!(s > 0.0, "Belief std 必须大于 0");
        }

        // 设备迁移验证
        let ac_cpu = ac.to_device(&Device::Cpu)?;
        assert!(ac_cpu.belief_head().is_some());

        Ok(())
    }

    #[test]
    fn test_mamba_stateful_step() -> Result<()> {
        let hidden_dim = 16;
        let device = Device::Cpu;
        let cfg = crate::policy::MambaConfig::new(hidden_dim);

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let mamba = crate::policy::MambaBlock::new(&cfg, vb)?;

        let mut state = crate::policy::MambaState::new(1, &cfg, &device)?;

        // 模拟多步单帧推演
        for step in 0..5 {
            let x = Tensor::new(&[[step as f32 * 0.1; 16]], &device)?;
            let y = mamba.step(&x, &mut state)?;
            assert_eq!(y.dims(), &[1, 16]);
            assert_eq!(state.pos, step + 1);
        }

        // 状态重置验证
        state.reset(1, &cfg, &device)?;
        assert_eq!(state.pos, 0);

        Ok(())
    }

    #[test]
    fn test_mamba_forward_seq_vs_step_equivalence() -> Result<()> {
        let hidden_dim = 16;
        let device = Device::Cpu;
        let cfg = crate::policy::MambaConfig::new(hidden_dim);

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let mamba = crate::policy::MambaBlock::new(&cfg, vb)?;

        let seq_len = 8;
        let mut xs_vec = Vec::with_capacity(seq_len * hidden_dim);
        for t in 0..seq_len {
            for d in 0..hidden_dim {
                xs_vec.push(((t + 1) as f32 * 0.1 + (d as f32) * 0.05).sin());
            }
        }

        // 路径 1: forward_seq (并行因果卷积 + Selective Scan)
        let xs_3d = Tensor::from_vec(xs_vec.clone(), (1, seq_len, hidden_dim), &device)?;
        let ys_seq = mamba.forward_seq(&xs_3d)?;
        let ys_seq_vec: Vec<Vec<f32>> = ys_seq.squeeze(0)?.to_vec2()?;

        // 路径 2: 循环 step (单步递推状态)
        let mut state = crate::policy::MambaState::new(1, &cfg, &device)?;
        let mut ys_step_vec = Vec::with_capacity(seq_len);
        for t in 0..seq_len {
            let x_t = xs_3d.narrow(1, t, 1)?.squeeze(1)?;
            let y_t = mamba.step(&x_t, &mut state)?;
            ys_step_vec.push(y_t.squeeze(0)?.to_vec1::<f32>()?);
        }

        for t in 0..seq_len {
            for d in 0..hidden_dim {
                let seq_val = ys_seq_vec[t][d];
                let step_val = ys_step_vec[t][d];
                let diff = (seq_val - step_val).abs();
                println!("t={t}, d={d}: seq={seq_val:.6}, step={step_val:.6}, diff={diff:.6}");
                assert!(
                    diff < 1e-4,
                    "t={t}, d={d}: seq={seq_val} vs step={step_val}, diff={diff}"
                );
            }
        }

        Ok(())
    }

    #[test]
    fn test_dual_backbone_mlp_and_mamba_roundtrip() -> Result<()> {
        let state_dim = 16;
        let hidden_dim = 64;
        let action_space = ActionSpace::Discrete(4);
        let config = PPOConfig::default();
        let device = Device::Cpu;

        // 1. 测试 MLP 主干
        let mlp_agent = PPOAgent::create_for_env_with_backbone::<lol_env::FioraV2Env>(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
            PolicyBackbone::Mlp,
        )?;
        assert_eq!(
            mlp_agent.actor_critic.backbone().backbone_type(),
            PolicyBackbone::Mlp
        );

        let tmp_dir = std::env::temp_dir();
        let mlp_ckpt = tmp_dir.join("test_mlp_ckpt.safetensors");
        mlp_agent.save(&mlp_ckpt)?;

        let loaded_mlp = PPOAgent::load(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
            &mlp_ckpt,
        )?;
        assert_eq!(
            loaded_mlp.actor_critic.backbone().backbone_type(),
            PolicyBackbone::Mlp
        );
        let _ = std::fs::remove_file(&mlp_ckpt);

        // 2. 测试 Mamba 主干
        let mamba_agent = PPOAgent::create_for_env_with_backbone::<lol_env::FioraV2Env>(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
            PolicyBackbone::Mamba,
        )?;
        assert_eq!(
            mamba_agent.actor_critic.backbone().backbone_type(),
            PolicyBackbone::Mamba
        );

        let mamba_ckpt = tmp_dir.join("test_mamba_ckpt.safetensors");
        mamba_agent.save(&mamba_ckpt)?;

        let loaded_mamba = PPOAgent::load(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            &mamba_ckpt,
        )?;
        assert_eq!(
            loaded_mamba.actor_critic.backbone().backbone_type(),
            PolicyBackbone::Mamba
        );
        let _ = std::fs::remove_file(&mamba_ckpt);

        Ok(())
    }

    #[test]
    fn test_mamba_chunk_sequence_ppo_update() -> Result<()> {
        let state_dim = 16;
        let hidden_dim = 32;
        let action_space = ActionSpace::Discrete(4);
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let mut agent = PPOAgent::create_for_env_with_backbone::<lol_env::FioraV2Env>(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            PolicyBackbone::Mamba,
        )?;

        // 构造两个轨迹 Buffer（分别包含 25 个连续时间步）
        let mut buf1 = RolloutBuffer::new();
        let mut buf2 = RolloutBuffer::new();
        for t in 0..25 {
            let state = vec![t as f32 * 0.1; state_dim];
            let act = vec![(t % 4) as f32];
            buf1.push_unmasked(state.clone(), act.clone(), -1.0, 1.0, 0.5, t == 24);
            buf2.push_unmasked(state, act, -1.0, -1.0, -0.5, t == 24);
        }

        let stats = agent.update_multi_buffer(&[buf1, buf2], &[0.5, -0.5], 16)?;
        assert!(stats.policy_loss.is_finite());
        assert!(stats.value_loss.is_finite());
        assert!(stats.total_loss.is_finite());

        Ok(())
    }
}
