use std::path::Path;

use candle_core::{DType, Device, Result, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW, VarBuilder, VarMap};
use lol_rl_protocol::{ActionSchema, ActionSpace, ObsSchema, PolicyBackbone};

use crate::algo::grpo::config::GRPOConfig;
use crate::policy::{HeroEmbedConfig, ModelParamSummary, PolicyNetwork};

pub struct GRPOAgent {
    pub policy: PolicyNetwork,
    pub(crate) varmap: VarMap,
    pub(crate) optimizer: AdamW,
    pub(crate) config: GRPOConfig,
    pub(crate) device: Device,
    pub(crate) hero_embed_config: HeroEmbedConfig,
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
}
