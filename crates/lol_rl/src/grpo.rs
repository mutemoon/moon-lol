use std::path::Path;

use candle_core::{DType, Device, Result, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW, VarBuilder, VarMap};
use lol_rl_protocol::{ActionSchema, ActionSpace, ObsSchema, PolicyBackbone};

use crate::policy::{HeroEmbedConfig, ModelParamSummary, PolicyNetwork};
use crate::ppo::{PPOStats, RolloutBuffer};

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

/// GRPO 算法 Agent：无 Critic 网络，纯 Actor + 分组相对优势优化
pub struct GRPOAgent {
    pub policy: PolicyNetwork,
    varmap: VarMap,
    optimizer: AdamW,
    config: GRPOConfig,
    device: Device,
    hero_embed_config: HeroEmbedConfig,
}

impl GRPOAgent {
    pub fn new(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: GRPOConfig,
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

    pub fn with_hero_embed(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: GRPOConfig,
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

    /// 创建指定主干架构 (MLP 或 Mamba) 的 GRPOAgent
    pub fn with_hero_embed_and_backbone(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: GRPOConfig,
        device: Device,
        hero_embed_config: HeroEmbedConfig,
        backbone_type: PolicyBackbone,
    ) -> Result<Self> {
        let mut varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let policy = PolicyNetwork::with_hero_embed_and_backbone(
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
                policy.action_space().actor_head_dim(),
                hidden_dim,
                0.01,
            ),
            (policy.action_space().actor_head_dim(), hidden_dim),
            &device,
        )?;
        let _ = varmap.set_one("actor_head.weight", actor_w);

        let params = ParamsAdamW {
            lr: config.lr,
            ..Default::default()
        };
        let optimizer = AdamW::new(varmap.all_vars(), params)?;

        Ok(Self {
            policy,
            varmap,
            optimizer,
            config,
            device,
            hero_embed_config,
        })
    }

    /// 基于 ObsSchema 结构规范自动推导特征提取网络和主干架构的 GRPOAgent
    pub fn from_obs_schema(
        schema: ObsSchema,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: GRPOConfig,
        device: Device,
        backbone_type: PolicyBackbone,
    ) -> Result<Self> {
        let mut varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let policy = PolicyNetwork::from_schema_and_backbone(
            schema.clone(),
            hidden_dim,
            action_space,
            backbone_type,
            None,
            vb,
        )?;

        let in_dim = schema.encoded_dim();
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
                let proj_w = Tensor::from_vec(
                    crate::policy::orthogonal_weight(hidden_dim, in_dim, hidden_gain),
                    (hidden_dim, in_dim),
                    &device,
                )?;
                let _ = varmap.set_one("proj_in.weight", proj_w);
            }
        }

        let params = ParamsAdamW {
            lr: config.lr,
            ..Default::default()
        };
        let optimizer = AdamW::new(varmap.all_vars(), params)?;

        Ok(Self {
            policy,
            varmap,
            optimizer,
            config,
            device,
            hero_embed_config: HeroEmbedConfig::default(),
        })
    }

    /// 基于 ObsSchema + ActionSchema 双 AST 结构规范自动推导特征提取网络和多头 Actor 架构的 GRPOAgent
    pub fn from_schemas(
        obs_schema: ObsSchema,
        action_schema: ActionSchema,
        hidden_dim: usize,
        config: GRPOConfig,
        device: Device,
        backbone_type: PolicyBackbone,
    ) -> Result<Self> {
        let mut varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let policy = PolicyNetwork::from_schemas(
            obs_schema.clone(),
            action_schema,
            hidden_dim,
            backbone_type,
            None,
            vb,
        )?;

        let in_dim = obs_schema.encoded_dim();
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
                let proj_w = Tensor::from_vec(
                    crate::policy::orthogonal_weight(hidden_dim, in_dim, hidden_gain),
                    (hidden_dim, in_dim),
                    &device,
                )?;
                let _ = varmap.set_one("proj_in.weight", proj_w);
            }
        }

        let params = ParamsAdamW {
            lr: config.lr,
            ..Default::default()
        };
        let optimizer = AdamW::new(varmap.all_vars(), params)?;

        Ok(Self {
            policy,
            varmap,
            optimizer,
            config,
            device,
            hero_embed_config: HeroEmbedConfig::default(),
        })
    }

    /// 统一为环境创建 GRPOAgent，支持指定 PolicyBackbone (MLP 或 Mamba)
    pub fn create_for_env_with_backbone<E: lol_env::RlEnvironment>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: GRPOConfig,
        device: Device,
        backbone_type: PolicyBackbone,
    ) -> Result<Self> {
        if let (Some(obs_schema), Some(action_schema)) = (E::obs_schema(), E::action_schema()) {
            Self::from_schemas(
                obs_schema,
                action_schema,
                hidden_dim,
                config,
                device,
                backbone_type,
            )
        } else if let Some(schema) = E::obs_schema() {
            Self::from_obs_schema(
                schema,
                hidden_dim,
                action_space,
                config,
                device,
                backbone_type,
            )
        } else {
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
    }

    pub fn hero_embed_config(&self) -> &HeroEmbedConfig {
        &self.hero_embed_config
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn parameter_summary(&self) -> ModelParamSummary {
        self.policy.parameter_summary()
    }

    pub fn print_parameter_summary(&self) {
        self.policy.parameter_summary().print_summary();
    }

    pub fn lr(&self) -> f64 {
        self.config.lr
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

    pub fn load_for_env<E: lol_env::RlEnvironment>(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: GRPOConfig,
        device: Device,
        path: &Path,
    ) -> Result<Self> {
        if let (Some(obs_schema), Some(action_schema)) = (E::obs_schema(), E::action_schema()) {
            Self::load_from_schemas(obs_schema, action_schema, hidden_dim, config, device, path)
        } else if let Some(schema) = E::obs_schema() {
            Self::load_from_schema(schema, hidden_dim, action_space, config, device, path)
        } else {
            Self::load(state_dim, hidden_dim, action_space, config, device, path)
        }
    }

    pub fn load_from_schemas(
        obs_schema: ObsSchema,
        action_schema: ActionSchema,
        hidden_dim: usize,
        config: GRPOConfig,
        device: Device,
        path: &Path,
    ) -> Result<Self> {
        let meta = std::fs::metadata(path)
            .map_err(|e| candle_core::Error::Msg(format!("checkpoint 文件不存在: {e}")))?;
        if meta.len() == 0 {
            return Err(candle_core::Error::Msg("checkpoint 文件为空".to_string()));
        }
        let tensors = candle_core::safetensors::load(path, &device)?;

        let is_mlp = tensors.contains_key("fc1.weight") || tensors.contains_key("fc1.bias");
        let backbone_type = if is_mlp {
            PolicyBackbone::Mlp
        } else {
            PolicyBackbone::Mamba
        };

        let hidden_dim = tensors
            .get("fc2.bias")
            .or_else(|| tensors.get("fc1.bias"))
            .or_else(|| tensors.get("proj_in.bias"))
            .or_else(|| tensors.get("proj_in.weight"))
            .and_then(|t| t.shape().dims().first().copied())
            .unwrap_or(hidden_dim);

        let mut varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let policy = PolicyNetwork::from_schemas(
            obs_schema,
            action_schema,
            hidden_dim,
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
            policy,
            varmap,
            optimizer,
            config,
            device,
            hero_embed_config: HeroEmbedConfig::default(),
        })
    }

    pub fn load_from_schema(
        schema: ObsSchema,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: GRPOConfig,
        device: Device,
        path: &Path,
    ) -> Result<Self> {
        let meta = std::fs::metadata(path)
            .map_err(|e| candle_core::Error::Msg(format!("checkpoint 文件不存在: {e}")))?;
        if meta.len() == 0 {
            return Err(candle_core::Error::Msg("checkpoint 文件为空".to_string()));
        }
        let tensors = candle_core::safetensors::load(path, &device)?;

        let is_mlp = tensors.contains_key("fc1.weight") || tensors.contains_key("fc1.bias");
        let backbone_type = if is_mlp {
            PolicyBackbone::Mlp
        } else {
            PolicyBackbone::Mamba
        };

        let hidden_dim = tensors
            .get("fc2.bias")
            .or_else(|| tensors.get("fc1.bias"))
            .or_else(|| tensors.get("proj_in.bias"))
            .or_else(|| tensors.get("proj_in.weight"))
            .and_then(|t| t.shape().dims().first().copied())
            .unwrap_or(hidden_dim);

        let mut varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let policy = PolicyNetwork::from_schema_and_backbone(
            schema,
            hidden_dim,
            action_space,
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
            policy,
            varmap,
            optimizer,
            config,
            device,
            hero_embed_config: HeroEmbedConfig::default(),
        })
    }

    pub fn load(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        config: GRPOConfig,
        device: Device,
        path: &Path,
    ) -> Result<Self> {
        let meta = std::fs::metadata(path)
            .map_err(|e| candle_core::Error::Msg(format!("checkpoint 文件不存在: {e}")))?;
        if meta.len() == 0 {
            return Err(candle_core::Error::Msg("checkpoint 文件为空".to_string()));
        }
        let tensors = candle_core::safetensors::load(path, &device)?;

        let is_mlp = tensors.contains_key("fc1.weight") || tensors.contains_key("fc1.bias");
        let backbone_type = if is_mlp {
            PolicyBackbone::Mlp
        } else {
            PolicyBackbone::Mamba
        };

        let hidden_dim = tensors
            .get("fc2.bias")
            .or_else(|| tensors.get("fc1.bias"))
            .or_else(|| tensors.get("proj_in.bias"))
            .or_else(|| tensors.get("proj_in.weight"))
            .and_then(|t| t.shape().dims().first().copied())
            .unwrap_or(hidden_dim);

        let mut varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
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
        let policy = PolicyNetwork::with_hero_embed_and_backbone(
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
            policy,
            varmap,
            optimizer,
            config,
            device,
            hero_embed_config,
        })
    }

    /// 计算 GRPO 分组相对优势 (Group Relative Advantages)
    ///
    /// 对于输入的若干环境轨迹 Buffers，按照 `group_size` 进行分组；
    /// 1. 对每个 buffer 计算其时间步上的未来折扣累计回报 $G_t = \sum_{k=t}^T \gamma^{k-t} r_k$；
    /// 2. 在每个组（Group）内，聚合所有轨迹的各步回报，计算组内均值 $\mu_G$ 与方差 $\sigma_G$；
    /// 3. 计算相对优势 $A_{i,t} = \frac{G_{i,t} - \mu_G}{\sigma_G + \epsilon}$；
    /// 4. 返回所有 buffers 对应的优势列表以及组统计回报（mean, std）。
    pub fn compute_group_advantages(
        &self,
        buffers: &[RolloutBuffer],
        group_size: usize,
    ) -> (Vec<Vec<f32>>, f32, f32) {
        let num_buffers = buffers.len();
        if num_buffers == 0 {
            return (Vec::new(), 0.0, 0.0);
        }

        let g_size = group_size.max(1);
        let mut all_advantages: Vec<Vec<f32>> = Vec::with_capacity(num_buffers);
        for b in buffers {
            all_advantages.push(vec![0.0; b.len()]);
        }

        let mut group_means = Vec::new();
        let mut group_stds = Vec::new();

        // 逐 Group 处理
        let mut start_idx = 0;
        while start_idx < num_buffers {
            let end_idx = (start_idx + g_size).min(num_buffers);
            let cur_group_buffers = &buffers[start_idx..end_idx];

            // 1. 计算当前组内每个 buffer 每步的未来折扣累积回报
            let mut group_discounted_returns: Vec<Vec<f32>> =
                Vec::with_capacity(cur_group_buffers.len());
            let mut all_returns_in_group = Vec::new();

            for buffer in cur_group_buffers {
                let n = buffer.len();
                let mut rets = vec![0.0; n];
                let mut run_ret = 0.0f32;
                for t in (0..n).rev() {
                    let done = buffer.dones.get(t).copied().unwrap_or(false);
                    let reward = buffer.rewards.get(t).copied().unwrap_or(0.0);
                    if done {
                        run_ret = reward;
                    } else {
                        run_ret = reward + self.config.gamma * run_ret;
                    }
                    rets[t] = run_ret;
                    all_returns_in_group.push(run_ret);
                }
                group_discounted_returns.push(rets);
            }

            // 2. 组内均值与标准差标准化
            let total_steps_in_group = all_returns_in_group.len();
            if total_steps_in_group > 0 {
                let mean = all_returns_in_group.iter().sum::<f32>() / (total_steps_in_group as f32);
                let variance = all_returns_in_group
                    .iter()
                    .map(|r| (r - mean).powi(2))
                    .sum::<f32>()
                    / (total_steps_in_group as f32);
                let std = (variance + 1e-8).sqrt();

                group_means.push(mean);
                group_stds.push(std);

                for (local_i, rets) in group_discounted_returns.iter().enumerate() {
                    let buf_idx = start_idx + local_i;
                    for (t, &val) in rets.iter().enumerate() {
                        all_advantages[buf_idx][t] = (val - mean) / std;
                    }
                }
            }

            start_idx += g_size;
        }

        let avg_mean = if group_means.is_empty() {
            0.0
        } else {
            group_means.iter().sum::<f32>() / (group_means.len() as f32)
        };
        let avg_std = if group_stds.is_empty() {
            0.0
        } else {
            group_stds.iter().sum::<f32>() / (group_stds.len() as f32)
        };

        (all_advantages, avg_mean, avg_std)
    }

    /// 多环境 GRPO 相对优势更新（无 Critic 网络，纯 Actor + 策略熵/KL 正则化）
    pub fn update_multi_buffer(
        &mut self,
        buffers: &[RolloutBuffer],
        mini_batch_size: usize,
    ) -> Result<GRPOStats> {
        let total_n: usize = buffers.iter().map(|b| b.len()).sum();
        if total_n == 0 {
            return Ok(GRPOStats {
                policy_loss: 0.0,
                entropy: 0.0,
                total_loss: 0.0,
                kl: 0.0,
                clip_frac: 0.0,
                group_reward_mean: 0.0,
                group_reward_std: 0.0,
            });
        }

        let (group_advantages, grp_mean, grp_std) =
            self.compute_group_advantages(buffers, self.config.group_size);

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

        let is_mamba = self.policy.backbone().backbone_type() == PolicyBackbone::Mamba;

        use rand::seq::SliceRandom;
        let mut rng = rand::rng();

        let mut last_stats = GRPOStats {
            policy_loss: 0.0,
            entropy: 0.0,
            total_loss: 0.0,
            kl: 0.0,
            clip_frac: 0.0,
            group_reward_mean: grp_mean,
            group_reward_std: grp_std,
        };

        // ════════════════════════════════════════════════════════════════
        // 路径 A：Mamba 时序状态空间模型（Chunk-based GRPO 时序切片训练）
        // ════════════════════════════════════════════════════════════════
        if is_mamba {
            let chunk_len = 16.min(total_n).max(1);
            struct TrajChunk {
                states: Vec<f32>,
                actions: Vec<f32>,
                old_log_probs: Vec<f32>,
                advantages: Vec<f32>,
                masks: Option<Vec<f32>>,
            }

            let mut chunks = Vec::new();
            for (i, buffer) in buffers.iter().enumerate() {
                if buffer.is_empty() {
                    continue;
                }
                let advantages = &group_advantages[i];
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

                    // 尾部不足 chunk_len 时做填充
                    if cl < chunk_len {
                        let pad_count = chunk_len - cl;
                        let last_state = &buffer.states[end - 1];
                        for _ in 0..pad_count {
                            c_states.extend_from_slice(last_state);
                            c_actions.extend_from_slice(&buffer.actions[end - 1]);
                            c_log_probs.push(buffer.log_probs[end - 1]);
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

            for _epoch in 0..self.config.grpo_epochs {
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

                    let (new_log_probs, entropy) = self.policy.evaluate_actions(
                        &mb_states_3d,
                        &mb_actions,
                        mb_masks.as_ref(),
                    )?;

                    let log_ratio = (&new_log_probs - &mb_old_log_probs)?;
                    let ratio = log_ratio.exp()?;

                    let surr1 = (&ratio * &mb_advantages)?;
                    let clamped_ratio =
                        ratio.clamp(1.0 - self.config.clip_eps, 1.0 + self.config.clip_eps)?;
                    let surr2 = (&clamped_ratio * &mb_advantages)?;

                    // GRPO 策略损失（Clipped Surrogate）
                    let policy_loss = surr1.minimum(&surr2)?.neg()?.mean_all()?;

                    // KL 散度与指标监控
                    let kl_loss = (&ratio - 1.0 - &log_ratio)?.mean_all()?;
                    let clip_frac =
                        (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                            + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
                        .mean_all()?;

                    let p_loss_val: f32 = policy_loss.to_scalar()?;
                    let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
                    let kl_val: f32 = kl_loss.to_scalar()?;
                    let clip_frac_val: f32 = clip_frac.to_scalar()?;

                    let mut grads = policy_loss.backward()?;
                    self.clip_grad_norm(&mut grads)?;
                    self.optimizer.step(&grads)?;

                    last_stats = GRPOStats {
                        policy_loss: p_loss_val,
                        entropy: entropy_val,
                        total_loss: p_loss_val,
                        kl: kl_val,
                        clip_frac: clip_frac_val,
                        group_reward_mean: grp_mean,
                        group_reward_std: grp_std,
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
        let mut all_advs = Vec::with_capacity(total_n);
        let mut all_masks: Option<Vec<f32>> = if has_masks {
            Some(Vec::with_capacity(total_n * mask_dim))
        } else {
            None
        };

        for (i, buffer) in buffers.iter().enumerate() {
            if buffer.is_empty() {
                continue;
            }
            let advantages = &group_advantages[i];
            for t in 0..buffer.len() {
                all_states.extend_from_slice(&buffer.states[t]);
                all_actions.extend_from_slice(&buffer.actions[t]);
                all_old_log_probs.push(buffer.log_probs[t]);
                all_advs.push(advantages[t]);
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

        let mb_size = mini_batch_size.min(total_n).max(1);

        for _epoch in 0..self.config.grpo_epochs {
            let mut indices: Vec<usize> = (0..total_n).collect();
            indices.shuffle(&mut rng);

            let mut start_idx = 0;
            while start_idx < total_n {
                let end_idx = (start_idx + mb_size).min(total_n);
                let cur_batch_size = end_idx - start_idx;
                let mb_indices = &indices[start_idx..end_idx];

                let mut mb_states_vec = Vec::with_capacity(cur_batch_size * state_dim);
                let mut mb_actions_vec = Vec::with_capacity(cur_batch_size * enc_dim);
                let mut mb_old_log_probs_vec = Vec::with_capacity(cur_batch_size);
                let mut mb_advantages_vec = Vec::with_capacity(cur_batch_size);
                let mut mb_masks_vec = if has_masks {
                    Some(Vec::with_capacity(cur_batch_size * mask_dim))
                } else {
                    None
                };

                for &idx in mb_indices {
                    mb_states_vec
                        .extend_from_slice(&all_states[idx * state_dim..(idx + 1) * state_dim]);
                    mb_actions_vec
                        .extend_from_slice(&all_actions[idx * enc_dim..(idx + 1) * enc_dim]);
                    mb_old_log_probs_vec.push(all_old_log_probs[idx]);
                    mb_advantages_vec.push(all_advs[idx]);
                    if let (Some(mbm), Some(am)) = (&mut mb_masks_vec, &all_masks) {
                        mbm.extend_from_slice(&am[idx * mask_dim..(idx + 1) * mask_dim]);
                    }
                }

                let mb_states =
                    Tensor::from_vec(mb_states_vec, (cur_batch_size, state_dim), &self.device)?;
                let mb_actions =
                    Tensor::from_vec(mb_actions_vec, (cur_batch_size, enc_dim), &self.device)?;
                let mb_old_log_probs =
                    Tensor::from_vec(mb_old_log_probs_vec, (cur_batch_size,), &self.device)?;
                let mb_advantages =
                    Tensor::from_vec(mb_advantages_vec, (cur_batch_size,), &self.device)?;
                let mb_masks = if let Some(mbm) = mb_masks_vec {
                    Some(Tensor::from_vec(
                        mbm,
                        (cur_batch_size, mask_dim),
                        &self.device,
                    )?)
                } else {
                    None
                };

                let (new_log_probs, entropy) =
                    self.policy
                        .evaluate_actions(&mb_states, &mb_actions, mb_masks.as_ref())?;

                let log_ratio = (&new_log_probs - &mb_old_log_probs)?;
                let ratio = log_ratio.exp()?;

                let surr1 = (&ratio * &mb_advantages)?;
                let clamped_ratio =
                    ratio.clamp(1.0 - self.config.clip_eps, 1.0 + self.config.clip_eps)?;
                let surr2 = (&clamped_ratio * &mb_advantages)?;

                let policy_loss = surr1.minimum(&surr2)?.neg()?.mean_all()?;

                let kl_loss = (&ratio - 1.0 - &log_ratio)?.mean_all()?;
                let clip_frac = (ratio.lt(1.0 - self.config.clip_eps)?.to_dtype(DType::F32)?
                    + ratio.gt(1.0 + self.config.clip_eps)?.to_dtype(DType::F32)?)?
                .mean_all()?;

                let p_loss_val: f32 = policy_loss.to_scalar()?;
                let entropy_val: f32 = entropy.mean_all()?.to_scalar()?;
                let kl_val: f32 = kl_loss.to_scalar()?;
                let clip_frac_val: f32 = clip_frac.to_scalar()?;

                let mut grads = policy_loss.backward()?;
                self.clip_grad_norm(&mut grads)?;
                self.optimizer.step(&grads)?;

                last_stats = GRPOStats {
                    policy_loss: p_loss_val,
                    entropy: entropy_val,
                    total_loss: p_loss_val,
                    kl: kl_val,
                    clip_frac: clip_frac_val,
                    group_reward_mean: grp_mean,
                    group_reward_std: grp_std,
                };

                start_idx += mb_size;
            }
        }

        Ok(last_stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpo_advantage_computation() {
        let agent = GRPOAgent::new(
            4,
            16,
            ActionSpace::Discrete(3),
            GRPOConfig {
                group_size: 2,
                gamma: 0.9,
                ..Default::default()
            },
            Device::Cpu,
        )
        .unwrap();

        // 构造 2 个 buffer 作为一组
        let mut b1 = RolloutBuffer::new();
        b1.push_unmasked(vec![0.0; 4], vec![0.0], 0.0, 1.0, 0.0, false);
        b1.push_unmasked(vec![0.0; 4], vec![0.0], 0.0, 2.0, 0.0, true);

        let mut b2 = RolloutBuffer::new();
        b2.push_unmasked(vec![0.0; 4], vec![0.0], 0.0, 0.0, 0.0, false);
        b2.push_unmasked(vec![0.0; 4], vec![0.0], 0.0, 0.0, 0.0, true);

        let (advs, _mean, std) = agent.compute_group_advantages(&[b1, b2], 2);
        assert_eq!(advs.len(), 2);
        assert_eq!(advs[0].len(), 2);
        assert_eq!(advs[1].len(), 2);
        assert!(std > 0.0);
        // 回报高的 b1 其 advantage 应该显著大于回报为 0 的 b2
        assert!(advs[0][0] > advs[1][0]);
    }

    #[test]
    fn test_grpo_update_mlp() {
        let mut agent = GRPOAgent::with_hero_embed_and_backbone(
            4,
            16,
            ActionSpace::Discrete(3),
            GRPOConfig {
                group_size: 2,
                grpo_epochs: 2,
                ..Default::default()
            },
            Device::Cpu,
            HeroEmbedConfig::default(),
            PolicyBackbone::Mlp,
        )
        .unwrap();

        let mut b1 = RolloutBuffer::new();
        b1.push_unmasked(vec![0.0, 1.0, 0.5, 0.2], vec![1.0], -1.0, 1.0, 0.0, false);
        b1.push_unmasked(vec![0.0, 0.8, 0.3, 0.1], vec![2.0], -1.2, 2.0, 0.0, true);

        let mut b2 = RolloutBuffer::new();
        b2.push_unmasked(vec![0.0, 0.5, 0.1, 0.0], vec![0.0], -0.9, -1.0, 0.0, false);
        b2.push_unmasked(vec![0.0, 0.2, 0.0, 0.0], vec![0.0], -0.8, -2.0, 0.0, true);

        let stats = agent.update_multi_buffer(&[b1, b2], 2).unwrap();
        assert!(stats.policy_loss.is_finite());
        assert!(stats.entropy >= 0.0);
    }

    #[test]
    fn test_grpo_update_mamba() {
        let mut agent = GRPOAgent::with_hero_embed_and_backbone(
            4,
            16,
            ActionSpace::Discrete(3),
            GRPOConfig {
                group_size: 2,
                grpo_epochs: 2,
                ..Default::default()
            },
            Device::Cpu,
            HeroEmbedConfig::default(),
            PolicyBackbone::Mamba,
        )
        .unwrap();

        let mut b1 = RolloutBuffer::new();
        for _ in 0..8 {
            b1.push_unmasked(vec![0.0, 1.0, 0.5, 0.2], vec![1.0], -1.0, 1.0, 0.0, false);
        }

        let mut b2 = RolloutBuffer::new();
        for _ in 0..8 {
            b2.push_unmasked(vec![0.0, 0.5, 0.1, 0.0], vec![0.0], -0.9, -1.0, 0.0, false);
        }

        let stats = agent.update_multi_buffer(&[b1, b2], 8).unwrap();
        assert!(stats.policy_loss.is_finite());
    }

    #[test]
    fn test_grpo_has_zero_critic_parameters() {
        use lol_env::{FioraV2Env, RlEnvironment};
        let agent = GRPOAgent::create_for_env_with_backbone::<FioraV2Env>(
            FioraV2Env::state_dim(),
            64,
            FioraV2Env::action_space(),
            GRPOConfig::default(),
            Device::Cpu,
            PolicyBackbone::Mlp,
        )
        .unwrap();

        let summary = agent.parameter_summary();
        agent.print_parameter_summary();

        // 验证 100% 没有任何 Critic 模块或参数
        let categories = summary.category_totals();
        assert!(
            !categories
                .iter()
                .any(|(cat, _)| cat.contains("Critic") || cat.contains("价值"))
        );
        assert!(!summary.layers.iter().any(|l| l.name.contains("critic")));
    }
}
