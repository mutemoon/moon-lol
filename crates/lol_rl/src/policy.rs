use candle_core::{D, DType, IndexOp, Result, Tensor};
use candle_nn::{Conv1d, Conv1dConfig, Embedding, Linear, Module, VarBuilder};
use lol_rl_protocol::{ActionSpace, PolicyDisplay, PolicyItem};
use rand::Rng;

/// 0.5·ln(2π)，用于高斯策略的 log_prob / 熵。
const HALF_LN_2PI: f32 = 0.9189385;

/// 生成标准正交权重矩阵（Modified Gram-Schmidt），用于工业级深度网络初始化。
pub fn orthogonal_weight(out_dim: usize, in_dim: usize, gain: f32) -> Vec<f32> {
    let rows = out_dim.max(in_dim);
    let cols = out_dim.min(in_dim);
    let mut rng = rand::rng();

    // 生成 rows x cols 的标准正态分布随机矩阵（Box-Muller 变换）
    let mut mat = Vec::with_capacity(rows * cols);
    while mat.len() < rows * cols {
        let u1: f32 = rng.random_range(1e-7..1.0);
        let u2: f32 = rng.random_range(0.0..std::f32::consts::TAU);
        let r = (-2.0 * u1.ln()).sqrt();
        let z0 = r * u2.cos();
        let z1 = r * u2.sin();
        mat.push(z0);
        if mat.len() < rows * cols {
            mat.push(z1);
        }
    }

    // Modified Gram-Schmidt QR 分解正交化每一列
    for j in 0..cols {
        for k in 0..j {
            let mut dot = 0.0f32;
            for r in 0..rows {
                dot += mat[r * cols + j] * mat[r * cols + k];
            }
            for r in 0..rows {
                mat[r * cols + j] -= dot * mat[r * cols + k];
            }
        }
        let mut norm_sq = 0.0f32;
        for r in 0..rows {
            norm_sq += mat[r * cols + j] * mat[r * cols + j];
        }
        let inv_norm = if norm_sq > 1e-12 {
            1.0 / norm_sq.sqrt()
        } else {
            0.0
        };
        for r in 0..rows {
            mat[r * cols + j] *= inv_norm;
        }
    }

    let mut result = vec![0.0f32; out_dim * in_dim];
    if out_dim >= in_dim {
        for r in 0..out_dim {
            for c in 0..in_dim {
                result[r * in_dim + c] = mat[r * cols + c] * gain;
            }
        }
    } else {
        for r in 0..out_dim {
            for c in 0..in_dim {
                result[r * in_dim + c] = mat[c * cols + r] * gain;
            }
        }
    }
    result
}

/// Configuration for hero-id embedding (OpenAI Five style conditional input).
/// The first element of the state vector (obs[0]) is treated as an integer hero index,
/// looked up in an embedding table, and concatenated with the remaining state features.
#[derive(Clone, Debug, PartialEq)]
pub struct HeroEmbedConfig {
    pub num_heroes: usize,
    pub embed_dim: usize,
}

impl Default for HeroEmbedConfig {
    fn default() -> Self {
        Self {
            num_heroes: 4, // 默认支持最多 4 个英雄 (0: Fiora, 1: Riven, 2..3 扩展)
            embed_dim: 16,
        }
    }
}

/// Mamba 结构超参数配置
#[derive(Clone, Debug, PartialEq)]
pub struct MambaConfig {
    pub d_model: usize,
    pub d_state: usize,
    pub d_conv: usize,
    pub expand: usize,
}

impl MambaConfig {
    pub fn new(d_model: usize) -> Self {
        Self {
            d_model,
            d_state: 16,
            d_conv: 4,
            expand: 2,
        }
    }

    pub fn with_state(d_model: usize, d_state: usize, expand: usize) -> Self {
        Self {
            d_model,
            d_state,
            d_conv: 4,
            expand,
        }
    }

    pub fn d_inner(&self) -> usize {
        self.d_model * self.expand
    }

    pub fn dt_rank(&self) -> usize {
        self.d_model.div_ceil(16)
    }
}

impl Default for MambaConfig {
    fn default() -> Self {
        Self::new(64)
    }
}

/// Mamba 单步推演隐状态（用于环境 Rollout 与实时推理）
#[derive(Clone, Debug)]
pub struct MambaState {
    pub h: Tensor,           // (batch, d_inner, d_state)
    pub prev_x: Vec<Tensor>, // d_conv 个 (batch, d_inner)
    pub pos: usize,
}

impl MambaState {
    pub fn new(batch_size: usize, cfg: &MambaConfig, device: &candle_core::Device) -> Result<Self> {
        let d_inner = cfg.d_inner();
        let h = Tensor::zeros((batch_size, d_inner, cfg.d_state), DType::F32, device)?;
        let mut prev_x = Vec::with_capacity(cfg.d_conv);
        for _ in 0..cfg.d_conv {
            prev_x.push(Tensor::zeros((batch_size, d_inner), DType::F32, device)?);
        }
        Ok(Self { h, prev_x, pos: 0 })
    }

    pub fn reset(
        &mut self,
        batch_size: usize,
        cfg: &MambaConfig,
        device: &candle_core::Device,
    ) -> Result<()> {
        let d_inner = cfg.d_inner();
        self.h = Tensor::zeros((batch_size, d_inner, cfg.d_state), DType::F32, device)?;
        self.prev_x.clear();
        for _ in 0..cfg.d_conv {
            self.prev_x
                .push(Tensor::zeros((batch_size, d_inner), DType::F32, device)?);
        }
        self.pos = 0;
        Ok(())
    }
}

/// 基于 RMSNorm 的均方根层归一化（为 Mamba 提供零均值与单位方差数值稳定性）
#[derive(Clone)]
pub struct RMSNorm {
    pub scale: Tensor,
    pub eps: f64,
}

impl RMSNorm {
    pub fn new(dim: usize, eps: f64, vb: VarBuilder) -> Result<Self> {
        let scale = vb.get_with_hints(dim, "scale", candle_nn::Init::Const(1.0))?;
        Ok(Self { scale, eps })
    }

    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x_cont = if x.is_contiguous() {
            x.clone()
        } else {
            x.contiguous()?
        };
        let x_dtype = x_cont.dtype();
        let x_f32 = x_cont.to_dtype(DType::F32)?;
        let variance = x_f32.powf(2.0)?.mean_keepdim(D::Minus1)?;
        let rsqrt = (variance + self.eps)?.sqrt()?.recip()?;
        let normed = x_f32.broadcast_mul(&rsqrt)?;
        let out = normed.broadcast_mul(&self.scale.to_dtype(DType::F32)?)?;
        out.to_dtype(x_dtype)
    }

    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        Ok(Self {
            scale: self.scale.to_device(device)?,
            eps: self.eps,
        })
    }
}

/// 基于 Candle 的 Mamba Selective SSM 核心模块（带残差连接与 RMSNorm）
#[derive(Clone)]
pub struct MambaBlock {
    pub in_proj: Linear,
    pub conv1d: Conv1d,
    pub x_proj: Linear,
    pub dt_proj: Linear,
    pub a_log: Tensor,
    pub d: Tensor,
    pub out_proj: Linear,
    pub norm: RMSNorm,
    pub config: MambaConfig,
}

impl MambaBlock {
    pub fn new(cfg: &MambaConfig, vb: VarBuilder) -> Result<Self> {
        let d_inner = cfg.d_inner();
        let dt_rank = cfg.dt_rank();
        let d_conv = cfg.d_conv;
        let d_state = cfg.d_state;

        let in_proj = candle_nn::linear_no_bias(cfg.d_model, d_inner * 2, vb.pp("in_proj"))?;
        let conv_cfg = Conv1dConfig {
            groups: d_inner,
            padding: 0,
            ..Default::default()
        };
        let conv1d = candle_nn::conv1d(d_inner, d_inner, d_conv, conv_cfg, vb.pp("conv1d"))?;
        let x_proj = candle_nn::linear_no_bias(d_inner, dt_rank + d_state * 2, vb.pp("x_proj"))?;
        let dt_proj = candle_nn::linear(dt_rank, d_inner, vb.pp("dt_proj"))?;
        let a_log = vb.get_with_hints((d_inner, d_state), "A_log", candle_nn::Init::Const(0.0))?;
        let d = vb.get_with_hints(d_inner, "D", candle_nn::Init::Const(1.0))?;
        let out_proj = candle_nn::linear_no_bias(d_inner, cfg.d_model, vb.pp("out_proj"))?;
        let norm = RMSNorm::new(cfg.d_model, 1e-5, vb.pp("norm"))?;

        Ok(Self {
            in_proj,
            conv1d,
            x_proj,
            dt_proj,
            a_log,
            d,
            out_proj,
            norm,
            config: cfg.clone(),
        })
    }

    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        let in_proj_w = self.in_proj.weight().to_device(device)?;
        let in_proj = Linear::new(in_proj_w, None);

        let conv_w = self.conv1d.weight().to_device(device)?;
        let conv_b = self
            .conv1d
            .bias()
            .map(|b| b.to_device(device))
            .transpose()?;
        let conv1d = Conv1d::new(conv_w, conv_b, *self.conv1d.config());

        let x_proj_w = self.x_proj.weight().to_device(device)?;
        let x_proj = Linear::new(x_proj_w, None);

        let dt_proj_w = self.dt_proj.weight().to_device(device)?;
        let dt_proj_b = self
            .dt_proj
            .bias()
            .map(|b| b.to_device(device))
            .transpose()?;
        let dt_proj = Linear::new(dt_proj_w, dt_proj_b);

        let a_log = self.a_log.to_device(device)?;
        let d = self.d.to_device(device)?;

        let out_proj_w = self.out_proj.weight().to_device(device)?;
        let out_proj = Linear::new(out_proj_w, None);
        let norm = self.norm.to_device(device)?;

        Ok(Self {
            in_proj,
            conv1d,
            x_proj,
            dt_proj,
            a_log,
            d,
            out_proj,
            norm,
            config: self.config.clone(),
        })
    }

    /// 前向计算：支持 2D (batch, d_model) 或 3D (batch, seq_len, d_model)
    pub fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        if xs.rank() == 2 {
            let (b_sz, dim) = xs.dims2()?;
            let xs_3d = xs.reshape((b_sz, 1, dim))?;
            let out_3d = self.forward_seq(&xs_3d)?;
            out_3d.reshape((b_sz, dim))
        } else {
            self.forward_seq(xs)
        }
    }

    /// 序列前向计算 (batch, seq_len, d_model) -> (batch, seq_len, d_model)
    pub fn forward_seq(&self, xs: &Tensor) -> Result<Tensor> {
        let xs = if xs.is_contiguous() {
            xs.clone()
        } else {
            xs.contiguous()?
        };
        let (_b_sz, _seq_len, _dim) = xs.dims3()?;
        let xs_proj = self.in_proj.forward(&xs)?;
        let chunks = xs_proj.chunk(2, D::Minus1)?;
        let (u, res) = (&chunks[0], &chunks[1]);

        // 因果 1D 深度卷积：左侧 pad d_conv - 1 个零，进行 padding=0 的因果卷积
        let u_t = u.transpose(1, 2)?.contiguous()?;
        let (b_sz, d_in, _l) = u_t.dims3()?;
        let pad_len = self.config.d_conv.saturating_sub(1);
        let u_conv = if pad_len > 0 {
            let pad = Tensor::zeros((b_sz, d_in, pad_len), u_t.dtype(), u_t.device())?;
            let u_padded = Tensor::cat(&[&pad, &u_t], 2)?.contiguous()?;
            u_padded.apply(&self.conv1d)?
        } else {
            u_t.apply(&self.conv1d)?
        };
        let u_conv = u_conv.transpose(1, 2)?.contiguous()?;
        let u_conv = candle_nn::ops::silu(&u_conv)?;

        let ssm_out = self.ssm(&u_conv)?;
        let ys = (&ssm_out * candle_nn::ops::silu(res))?;
        let ys_cont = if ys.is_contiguous() {
            ys
        } else {
            ys.contiguous()?
        };
        let out = self.out_proj.forward(&ys_cont)?;
        let res = (&xs + &out)?;
        self.norm.forward(&res)
    }

    /// Selective SSM 扫描计算
    pub fn ssm(&self, xs: &Tensor) -> Result<Tensor> {
        let xs_cont = if xs.is_contiguous() {
            xs.clone()
        } else {
            xs.contiguous()?
        };
        let (_d_in, n) = self.a_log.dims2()?;
        let a = self.a_log.to_dtype(DType::F32)?.exp()?.neg()?;
        let d = self.d.to_dtype(DType::F32)?;
        let x_dbl = self.x_proj.forward(&xs_cont)?;
        let dt_rank = self.config.dt_rank();
        let delta = x_dbl.narrow(D::Minus1, 0, dt_rank)?;
        let b = x_dbl.narrow(D::Minus1, dt_rank, n)?;
        let c = x_dbl.narrow(D::Minus1, dt_rank + n, n)?;
        let delta = delta.contiguous()?;
        let delta = self.dt_proj.forward(&delta)?;
        // Softplus 激活步长: ln(1 + exp(delta))
        let delta = (delta.exp()? + 1.0)?.log()?;
        selective_scan(&xs_cont, &delta, &a, &b, &c, &d)
    }

    /// 单步状态化前向推理（用于环境采样循环）
    pub fn step(&self, x: &Tensor, state: &mut MambaState) -> Result<Tensor> {
        let (b_sz, _) = x.dims2()?;
        let x_proj = self.in_proj.forward(x)?;
        let mut chunks = x_proj.chunk(2, D::Minus1)?;
        let proj_silu = chunks.remove(1);
        let proj_conv = chunks.remove(0);

        let d_conv = self.config.d_conv;
        let d_inner = self.config.d_inner();
        let d_state = self.config.d_state;
        let dt_rank = self.config.dt_rank();

        state.prev_x[state.pos % d_conv] = proj_conv.clone();

        // 卷积权重累加
        let conv_w = self.conv1d.weight();
        let mut conv_out = match self.conv1d.bias() {
            Some(bias) => bias.broadcast_as((b_sz, d_inner))?,
            None => Tensor::zeros((b_sz, d_inner), DType::F32, x.device())?,
        };
        for c_idx in 0..d_conv {
            let w_c = conv_w.i((.., 0, c_idx))?;
            let prev = &state.prev_x[(c_idx + 1 + state.pos) % d_conv];
            conv_out = (conv_out + prev.broadcast_mul(&w_c)?)?;
        }
        let conv_out = candle_nn::ops::silu(&conv_out)?;

        let x_dbl = self.x_proj.forward(&conv_out)?;
        let delta = x_dbl.narrow(D::Minus1, 0, dt_rank)?;
        let b = x_dbl.narrow(D::Minus1, dt_rank, d_state)?;
        let c = x_dbl.narrow(D::Minus1, dt_rank + d_state, d_state)?;
        let delta = delta.contiguous()?;
        let delta = self.dt_proj.forward(&delta)?;
        let delta = (delta.exp()? + 1.0)?.log()?;

        let a = self.a_log.to_dtype(DType::F32)?.exp()?.neg()?;
        let d = self.d.to_dtype(DType::F32)?;

        let delta_3d = delta
            .unsqueeze(D::Minus1)?
            .broadcast_as((b_sz, d_inner, d_state))?;
        let a_3d = a.unsqueeze(0)?.broadcast_as((b_sz, d_inner, d_state))?;
        let b_3d = b.unsqueeze(1)?.broadcast_as((b_sz, d_inner, d_state))?;
        let conv_out_3d = conv_out
            .unsqueeze(D::Minus1)?
            .broadcast_as((b_sz, d_inner, d_state))?;

        let da = (&delta_3d * &a_3d)?.exp()?;
        let dbu = (&delta_3d * &b_3d * &conv_out_3d)?;
        state.h = ((&state.h * &da)? + &dbu)?;

        let c_3d = c.unsqueeze(2)?.contiguous()?;
        let ss =
            (state.h.contiguous()?.matmul(&c_3d)?.squeeze(2)? + conv_out.broadcast_mul(&d)?)?;
        let y = (ss * candle_nn::ops::silu(&proj_silu))?;
        state.pos += 1;
        let out = self.out_proj.forward(&y)?;
        let res = (x + &out)?;
        self.norm.forward(&res)
    }
}

/// 可微分 Selective Scan 算子（时序离散扫描）
fn selective_scan(
    u: &Tensor,
    delta: &Tensor,
    a: &Tensor,
    b: &Tensor,
    c: &Tensor,
    d: &Tensor,
) -> Result<Tensor> {
    let (b_sz, l, d_in) = u.dims3()?;
    let n = a.dim(1)?;

    // L = 1 极速向量化路径（单帧批处理时无需循环、三维广播和张量堆叠）
    if l == 1 {
        let u_0 = u.squeeze(1)?;
        let delta_0 = delta.squeeze(1)?;
        let b_0 = b.squeeze(1)?;
        let c_0 = c.squeeze(1)?;

        let bc = (&b_0 * &c_0)?.sum_keepdim(D::Minus1)?;
        let delta_bc = delta_0.broadcast_mul(&bc)?;
        let scale = delta_bc.broadcast_add(d)?;
        let y_0 = (&u_0 * &scale)?;
        return y_0.unsqueeze(1);
    }

    let mut xs = Tensor::zeros((b_sz, d_in, n), DType::F32, u.device())?;
    let mut ys = Vec::with_capacity(l);

    let a_3d = a.unsqueeze(0)?;

    for i in 0..l {
        let u_i = u.narrow(1, i, 1)?.squeeze(1)?;
        let delta_i = delta.narrow(1, i, 1)?.squeeze(1)?;
        let b_i = b.narrow(1, i, 1)?.squeeze(1)?;
        let c_i = c.narrow(1, i, 1)?.squeeze(1)?;

        let delta_3d = delta_i.unsqueeze(2)?.broadcast_as((b_sz, d_in, n))?;
        let a_bcast = a_3d.broadcast_as((b_sz, d_in, n))?;
        let da = (&delta_3d * &a_bcast)?.exp()?;

        let b_3d = b_i.unsqueeze(1)?.broadcast_as((b_sz, d_in, n))?;
        let u_3d = u_i.unsqueeze(2)?.broadcast_as((b_sz, d_in, n))?;
        let dbu = (&delta_3d * &b_3d * &u_3d)?;

        xs = ((&xs * &da)? + &dbu)?;

        let c_3d = c_i.unsqueeze(2)?.contiguous()?;
        let xs_cont = if xs.is_contiguous() {
            xs.clone()
        } else {
            xs.contiguous()?
        };
        let y = (xs_cont.matmul(&c_3d)?.squeeze(2)? + u_i.broadcast_mul(d)?)?;
        ys.push(y);
    }
    Tensor::stack(&ys, 1)
}

/// Belief-State 信念状态估计头（输出真实环境状态估计的高斯分布参数）
#[derive(Clone)]
pub struct BeliefHead {
    pub mu: Linear,
    pub logvar: Linear,
    pub belief_dim: usize,
}

impl BeliefHead {
    pub fn new(d_model: usize, belief_dim: usize, vb: VarBuilder) -> Result<Self> {
        let mu = candle_nn::linear(d_model, belief_dim, vb.pp("belief_mu"))?;
        let logvar = candle_nn::linear(d_model, belief_dim, vb.pp("belief_logvar"))?;
        Ok(Self {
            mu,
            logvar,
            belief_dim,
        })
    }

    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        let mu_w = self.mu.weight().to_device(device)?;
        let mu_b = self.mu.bias().map(|b| b.to_device(device)).transpose()?;
        let mu = Linear::new(mu_w, mu_b);

        let logvar_w = self.logvar.weight().to_device(device)?;
        let logvar_b = self
            .logvar
            .bias()
            .map(|b| b.to_device(device))
            .transpose()?;
        let logvar = Linear::new(logvar_w, logvar_b);

        Ok(Self {
            mu,
            logvar,
            belief_dim: self.belief_dim,
        })
    }

    /// 前向返回 (均值 mu, 标准差 std)
    pub fn forward(&self, features: &Tensor) -> Result<(Tensor, Tensor)> {
        let mu = self.mu.forward(features)?;
        let logvar = self.logvar.forward(features)?;
        let std = logvar.affine(0.5, 0.0)?.exp()?;
        Ok((mu, std))
    }
}

use lol_rl_protocol::PolicyBackbone;

/// 策略网络的主干特征提取层（支持无状态 MLP 或 状态化 Mamba）
#[derive(Clone)]
pub enum Backbone {
    Mlp {
        fc1: Linear,
        fc2: Linear,
        hidden_dim: usize,
    },
    Mamba {
        proj_in: Linear,
        mamba: MambaBlock,
        config: MambaConfig,
    },
}

impl Backbone {
    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x_cont = if x.is_contiguous() {
            x.clone()
        } else {
            x.contiguous()?
        };
        match self {
            Self::Mlp { fc1, fc2, .. } => {
                let h = fc1.forward(&x_cont)?.tanh()?;
                fc2.forward(&h)?.tanh()
            }
            Self::Mamba { proj_in, mamba, .. } => {
                let x_proj = proj_in.forward(&x_cont)?;
                mamba.forward(&x_proj)
            }
        }
    }

    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        match self {
            Self::Mlp {
                fc1,
                fc2,
                hidden_dim,
            } => {
                let fc1_w = fc1.weight().to_device(device)?;
                let fc1_b = fc1.bias().map(|b| b.to_device(device)).transpose()?;
                let fc2_w = fc2.weight().to_device(device)?;
                let fc2_b = fc2.bias().map(|b| b.to_device(device)).transpose()?;
                Ok(Self::Mlp {
                    fc1: Linear::new(fc1_w, fc1_b),
                    fc2: Linear::new(fc2_w, fc2_b),
                    hidden_dim: *hidden_dim,
                })
            }
            Self::Mamba {
                proj_in,
                mamba,
                config,
            } => {
                let proj_in_w = proj_in.weight().to_device(device)?;
                let proj_in_b = proj_in.bias().map(|b| b.to_device(device)).transpose()?;
                let mamba = mamba.to_device(device)?;
                Ok(Self::Mamba {
                    proj_in: Linear::new(proj_in_w, proj_in_b),
                    mamba,
                    config: config.clone(),
                })
            }
        }
    }

    pub fn output_dim(&self) -> usize {
        match self {
            Self::Mlp { hidden_dim, .. } => *hidden_dim,
            Self::Mamba { config, .. } => config.d_model,
        }
    }

    pub fn backbone_type(&self) -> PolicyBackbone {
        match self {
            Self::Mlp { .. } => PolicyBackbone::Mlp,
            Self::Mamba { .. } => PolicyBackbone::Mamba,
        }
    }
}

/// ActorCritic 策略与价值网络（支持 MLP 与 Mamba 双主干）
#[derive(Clone)]
pub struct ActorCritic {
    /// Hero-id embedding (OpenAI Five 风格英雄条件输入).
    hero_embed: Embedding,
    hero_embed_config: HeroEmbedConfig,
    /// 核心特征提取主干（MLP 或 Mamba）
    backbone: Backbone,
    /// 动作输出头：离散分类 logits 或连续动作均值
    actor_head: Linear,
    /// 连续/混合动作：可训练 log_std
    log_std: Option<Tensor>,
    /// 混合动作：离散分类头
    attack_head: Option<Linear>,
    /// 状态价值 Critic 估值头
    critic_head: Linear,
    /// 可选：Belief-State 信念解码头
    belief_head: Option<BeliefHead>,
    action_space: ActionSpace,
}

impl ActorCritic {
    pub fn new(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        vb: VarBuilder,
    ) -> Result<Self> {
        Self::with_hero_embed_and_backbone(
            state_dim,
            hidden_dim,
            action_space,
            HeroEmbedConfig::default(),
            PolicyBackbone::Mamba,
            None,
            vb,
        )
    }

    pub fn with_hero_embed(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        hero_embed_config: HeroEmbedConfig,
        vb: VarBuilder,
    ) -> Result<Self> {
        Self::with_hero_embed_and_backbone(
            state_dim,
            hidden_dim,
            action_space,
            hero_embed_config,
            PolicyBackbone::Mamba,
            None,
            vb,
        )
    }

    pub fn with_hero_embed_and_backbone(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        hero_embed_config: HeroEmbedConfig,
        backbone_type: PolicyBackbone,
        belief_dim: Option<usize>,
        vb: VarBuilder,
    ) -> Result<Self> {
        let emb = candle_nn::embedding(
            hero_embed_config.num_heroes,
            hero_embed_config.embed_dim,
            vb.pp("hero_embed"),
        )?;
        let in_dim = hero_embed_config.embed_dim + state_dim - 1;

        let (backbone, feat_dim) = match backbone_type {
            PolicyBackbone::Mlp => {
                let fc1 = candle_nn::linear(in_dim, hidden_dim, vb.pp("fc1"))?;
                let fc2 = candle_nn::linear(hidden_dim, hidden_dim, vb.pp("fc2"))?;
                (
                    Backbone::Mlp {
                        fc1,
                        fc2,
                        hidden_dim,
                    },
                    hidden_dim,
                )
            }
            PolicyBackbone::Mamba => {
                let mamba_config = MambaConfig::new(hidden_dim);
                let proj_in = candle_nn::linear(in_dim, mamba_config.d_model, vb.pp("proj_in"))?;
                let mamba = MambaBlock::new(&mamba_config, vb.pp("mamba"))?;
                let feat_dim = mamba_config.d_model;
                (
                    Backbone::Mamba {
                        proj_in,
                        mamba,
                        config: mamba_config,
                    },
                    feat_dim,
                )
            }
        };

        let critic_head = candle_nn::linear(feat_dim, 1, vb.pp("critic_head"))?;

        let (actor_out_dim, log_std, attack_head) = match action_space {
            ActionSpace::Discrete(n) => (n, None, None),
            ActionSpace::Continuous(d) => (
                d,
                Some(vb.get_with_hints((d,), "log_std", candle_nn::Init::Const(0.0))?),
                None,
            ),
            ActionSpace::Hybrid {
                continuous_dims,
                discrete_classes,
            } => (
                continuous_dims,
                Some(vb.get_with_hints(
                    (continuous_dims,),
                    "log_std",
                    candle_nn::Init::Const(0.0),
                )?),
                Some(candle_nn::linear(
                    feat_dim,
                    discrete_classes,
                    vb.pp("attack_head"),
                )?),
            ),
        };
        let actor_head = candle_nn::linear(feat_dim, actor_out_dim, vb.pp("actor_head"))?;

        let belief_head = match belief_dim {
            Some(b_dim) => Some(BeliefHead::new(feat_dim, b_dim, vb.pp("belief_head"))?),
            None => None,
        };

        Ok(Self {
            hero_embed: emb,
            hero_embed_config,
            backbone,
            actor_head,
            log_std,
            attack_head,
            critic_head,
            belief_head,
            action_space,
        })
    }

    pub fn with_hero_embed_and_mamba(
        state_dim: usize,
        _hidden_dim: usize,
        action_space: ActionSpace,
        hero_embed_config: HeroEmbedConfig,
        mamba_config: MambaConfig,
        belief_dim: Option<usize>,
        vb: VarBuilder,
    ) -> Result<Self> {
        let emb = candle_nn::embedding(
            hero_embed_config.num_heroes,
            hero_embed_config.embed_dim,
            vb.pp("hero_embed"),
        )?;
        let in_dim = hero_embed_config.embed_dim + state_dim - 1;
        let proj_in = candle_nn::linear(in_dim, mamba_config.d_model, vb.pp("proj_in"))?;
        let mamba = MambaBlock::new(&mamba_config, vb.pp("mamba"))?;
        let feat_dim = mamba_config.d_model;
        let backbone = Backbone::Mamba {
            proj_in,
            mamba,
            config: mamba_config,
        };

        let critic_head = candle_nn::linear(feat_dim, 1, vb.pp("critic_head"))?;

        let (actor_out_dim, log_std, attack_head) = match action_space {
            ActionSpace::Discrete(n) => (n, None, None),
            ActionSpace::Continuous(d) => (
                d,
                Some(vb.get_with_hints((d,), "log_std", candle_nn::Init::Const(0.0))?),
                None,
            ),
            ActionSpace::Hybrid {
                continuous_dims,
                discrete_classes,
            } => (
                continuous_dims,
                Some(vb.get_with_hints(
                    (continuous_dims,),
                    "log_std",
                    candle_nn::Init::Const(0.0),
                )?),
                Some(candle_nn::linear(
                    feat_dim,
                    discrete_classes,
                    vb.pp("attack_head"),
                )?),
            ),
        };
        let actor_head = candle_nn::linear(feat_dim, actor_out_dim, vb.pp("actor_head"))?;

        let belief_head = match belief_dim {
            Some(b_dim) => Some(BeliefHead::new(feat_dim, b_dim, vb.pp("belief_head"))?),
            None => None,
        };

        Ok(Self {
            hero_embed: emb,
            hero_embed_config,
            backbone,
            actor_head,
            log_std,
            attack_head,
            critic_head,
            belief_head,
            action_space,
        })
    }

    /// 将策略网络权重复制并迁移到指定计算设备（例如将 GPU 权重克隆至 CPU）
    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        let hero_w = self.hero_embed.embeddings().to_device(device)?;
        let hero_embed = Embedding::new(hero_w, self.hero_embed_config.embed_dim);

        let backbone = self.backbone.to_device(device)?;

        let actor_w = self.actor_head.weight().to_device(device)?;
        let actor_b = self
            .actor_head
            .bias()
            .map(|b| b.to_device(device))
            .transpose()?;
        let actor_head = Linear::new(actor_w, actor_b);

        let critic_w = self.critic_head.weight().to_device(device)?;
        let critic_b = self
            .critic_head
            .bias()
            .map(|b| b.to_device(device))
            .transpose()?;
        let critic_head = Linear::new(critic_w, critic_b);

        let log_std = self
            .log_std
            .as_ref()
            .map(|s| s.to_device(device))
            .transpose()?;

        let attack_head = self
            .attack_head
            .as_ref()
            .map(|a| -> Result<Linear> {
                let w = a.weight().to_device(device)?;
                let b = a.bias().map(|b| b.to_device(device)).transpose()?;
                Ok(Linear::new(w, b))
            })
            .transpose()?;

        let belief_head = self
            .belief_head
            .as_ref()
            .map(|b| b.to_device(device))
            .transpose()?;

        Ok(Self {
            hero_embed,
            hero_embed_config: self.hero_embed_config.clone(),
            backbone,
            actor_head,
            log_std,
            attack_head,
            critic_head,
            belief_head,
            action_space: self.action_space.clone(),
        })
    }

    pub fn action_space(&self) -> &ActionSpace {
        &self.action_space
    }

    pub fn hero_embed_config(&self) -> &HeroEmbedConfig {
        &self.hero_embed_config
    }

    pub fn backbone(&self) -> &Backbone {
        &self.backbone
    }

    pub fn mamba_config(&self) -> Option<&MambaConfig> {
        match &self.backbone {
            Backbone::Mamba { config, .. } => Some(config),
            Backbone::Mlp { .. } => None,
        }
    }

    pub fn belief_head(&self) -> Option<&BeliefHead> {
        self.belief_head.as_ref()
    }

    pub fn has_hero_embed(&self) -> bool {
        true
    }

    /// Prepare the input by replacing hero_id float with embedding.
    fn prepare_input(&self, state: &Tensor) -> Result<Tensor> {
        if state.rank() == 3 {
            let (_b, _l, state_dim) = state.dims3()?;
            let hero_ids = state.narrow(2, 0, 1)?.squeeze(2)?.to_dtype(DType::U32)?;
            let hero_vecs = self.hero_embed.forward(&hero_ids)?;
            let rest = state.narrow(2, 1, state_dim - 1)?;
            Tensor::cat(&[&hero_vecs, &rest], 2)?.contiguous()
        } else {
            let hero_ids = state.narrow(1, 0, 1)?.squeeze(1)?.to_dtype(DType::U32)?;
            let hero_vecs = self.hero_embed.forward(&hero_ids)?;
            let rest = state.narrow(1, 1, state.dim(1)? - 1)?;
            Tensor::cat(&[&hero_vecs, &rest], 1)?.contiguous()
        }
    }

    /// 特征提取输出（支持 MLP 与 Mamba）。
    fn hidden(&self, state: &Tensor) -> Result<Tensor> {
        let input = self.prepare_input(state)?;
        self.backbone.forward(&input)
    }

    /// Forward pass 返回 (actor_head 原始输出, values)。
    pub fn forward(&self, state: &Tensor) -> Result<(Tensor, Tensor)> {
        let feat = self.hidden(state)?;
        let out = self.actor_head.forward(&feat)?;
        let values = self.critic_head.forward(&feat)?;
        Ok((out, values))
    }

    /// 信念估计前向返回 Option<(mu, std)>
    pub fn forward_belief(&self, state: &Tensor) -> Result<Option<(Tensor, Tensor)>> {
        if let Some(ref bh) = self.belief_head {
            let feat = self.hidden(state)?;
            let (mu, std) = bh.forward(&feat)?;
            Ok(Some((mu, std)))
        } else {
            Ok(None)
        }
    }

    /// 批量获取 Critic 状态价值估值
    pub fn get_values(&self, state: &Tensor) -> Result<Vec<f32>> {
        let feat = self.hidden(state)?;
        let values = self.critic_head.forward(&feat)?;
        values.squeeze(1)?.to_vec1()
    }

    /// 从策略采样一个动作。返回 (编码动作向量, log_prob, value)。
    pub fn sample_action(
        &self,
        state: &Tensor,
        mask: Option<&[bool]>,
    ) -> Result<(Vec<f32>, f32, f32)> {
        let feat = self.hidden(state)?;
        let values = self.critic_head.forward(&feat)?;
        let val_scalar: f32 = values.squeeze(0)?.squeeze(0)?.to_scalar()?;

        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits: Vec<f32> = self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                let masked = mask_logits_slice(&logits, mask);
                let (idx, log_prob) = sample_categorical(&masked);
                Ok((vec![idx as f32], log_prob, val_scalar))
            }
            ActionSpace::Continuous(d) => {
                let means: Vec<f32> = self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                let log_std: Vec<f32> = self.log_std.as_ref().unwrap().to_vec1()?;
                let mut rng = rand::rng();
                let mut encoded = Vec::with_capacity(d);
                let mut log_prob = 0.0;
                for i in 0..d {
                    let std = log_std[i].exp();
                    let a = means[i] + std * sample_gaussian(&mut rng);
                    encoded.push(a);
                    log_prob += gaussian_log_prob(means[i], std, a);
                }
                Ok((encoded, log_prob, val_scalar))
            }
            ActionSpace::Hybrid {
                continuous_dims, ..
            } => {
                let means: Vec<f32> = self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                let log_std: Vec<f32> = self.log_std.as_ref().unwrap().to_vec1()?;
                let mut rng = rand::rng();
                let mut encoded = Vec::with_capacity(continuous_dims + 1);
                let mut log_prob = 0.0;
                for i in 0..continuous_dims {
                    let std = log_std[i].exp();
                    let a = means[i] + std * sample_gaussian(&mut rng);
                    encoded.push(a);
                    log_prob += gaussian_log_prob(means[i], std, a);
                }
                let attack_logits: Vec<f32> = self
                    .attack_head
                    .as_ref()
                    .unwrap()
                    .forward(&feat)?
                    .squeeze(0)?
                    .to_vec1()?;
                let masked = mask_logits_slice(&attack_logits, mask);
                let (idx, cat_log_prob) = sample_categorical(&masked);
                encoded.push(idx as f32);
                log_prob += cat_log_prob;
                Ok((encoded, log_prob, val_scalar))
            }
        }
    }

    /// 批量从策略采样动作（一次 GPU/CPU 前向计算），返回每个样本的 (encoded_action, log_prob, value)。
    pub fn sample_batch(
        &self,
        states: &Tensor,
        masks: Option<&[Option<Vec<bool>>]>,
    ) -> Result<Vec<(Vec<f32>, f32, f32)>> {
        let b = states.dim(0)?;
        if b == 0 {
            return Ok(Vec::new());
        }
        let feat = self.hidden(states)?;
        let values = self.critic_head.forward(&feat)?;
        let val_vec: Vec<f32> = values.squeeze(1)?.to_vec1()?;

        let mut results = Vec::with_capacity(b);

        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits = self.actor_head.forward(&feat)?;
                let logits_mat: Vec<Vec<f32>> = logits.to_vec2()?;
                for i in 0..b {
                    let mask_i = masks.and_then(|ms| ms.get(i)).and_then(|m| m.as_deref());
                    let masked = mask_logits_slice(&logits_mat[i], mask_i);
                    let (idx, log_prob) = sample_categorical(&masked);
                    results.push((vec![idx as f32], log_prob, val_vec[i]));
                }
            }
            ActionSpace::Continuous(d) => {
                let means_mat: Vec<Vec<f32>> = self.actor_head.forward(&feat)?.to_vec2()?;
                let log_std: Vec<f32> = self.log_std.as_ref().unwrap().to_vec1()?;
                let mut rng = rand::rng();
                for i in 0..b {
                    let means = &means_mat[i];
                    let mut encoded = Vec::with_capacity(d);
                    let mut log_prob = 0.0;
                    for j in 0..d {
                        let std = log_std[j].exp();
                        let a = means[j] + std * sample_gaussian(&mut rng);
                        encoded.push(a);
                        log_prob += gaussian_log_prob(means[j], std, a);
                    }
                    results.push((encoded, log_prob, val_vec[i]));
                }
            }
            ActionSpace::Hybrid {
                continuous_dims, ..
            } => {
                let means_mat: Vec<Vec<f32>> = self.actor_head.forward(&feat)?.to_vec2()?;
                let log_std: Vec<f32> = self.log_std.as_ref().unwrap().to_vec1()?;
                let attack_logits_mat: Vec<Vec<f32>> = self
                    .attack_head
                    .as_ref()
                    .unwrap()
                    .forward(&feat)?
                    .to_vec2()?;
                let mut rng = rand::rng();

                for i in 0..b {
                    let means = &means_mat[i];
                    let mut encoded = Vec::with_capacity(continuous_dims + 1);
                    let mut log_prob = 0.0;
                    for j in 0..continuous_dims {
                        let std = log_std[j].exp();
                        let a = means[j] + std * sample_gaussian(&mut rng);
                        encoded.push(a);
                        log_prob += gaussian_log_prob(means[j], std, a);
                    }

                    let mask_i = masks.and_then(|ms| ms.get(i)).and_then(|m| m.as_deref());
                    let masked = mask_logits_slice(&attack_logits_mat[i], mask_i);
                    let (idx, cat_log_prob) = sample_categorical(&masked);
                    encoded.push(idx as f32);
                    log_prob += cat_log_prob;
                    results.push((encoded, log_prob, val_vec[i]));
                }
            }
        }

        Ok(results)
    }

    /// 确定性贪心动作（连续取均值、离散取 argmax），用于可视化与评估。
    pub fn select_greedy_action(&self, state: &Tensor, mask: Option<&[bool]>) -> Result<Vec<f32>> {
        let feat = self.hidden(state)?;
        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits: Vec<f32> = self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                let masked = mask_logits_slice(&logits, mask);
                Ok(vec![argmax(&masked) as f32])
            }
            ActionSpace::Continuous(d) => {
                let means: Vec<f32> = self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                Ok(means[..d].to_vec())
            }
            ActionSpace::Hybrid {
                continuous_dims, ..
            } => {
                let means: Vec<f32> = self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                let attack_logits: Vec<f32> = self
                    .attack_head
                    .as_ref()
                    .unwrap()
                    .forward(&feat)?
                    .squeeze(0)?
                    .to_vec1()?;
                let masked = mask_logits_slice(&attack_logits, mask);
                let mut encoded = means[..continuous_dims].to_vec();
                encoded.push(argmax(&masked) as f32);
                Ok(encoded)
            }
        }
    }

    /// 真实动作空间的策略展示（可视化用）：离散返回逐类概率，混合返回连续均值 + 离散各动作概率。
    pub fn policy_display_real(
        &self,
        state: &Tensor,
        mask: Option<&[bool]>,
        labels: &[&str],
    ) -> Result<PolicyDisplay> {
        let feat = self.hidden(state)?;
        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits: Vec<f32> = self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                let raw_probs_vec = softmax_slice(&logits);
                let masked = mask_logits_slice(&logits, mask);
                let probs_vec = softmax_slice(&masked);
                Ok(PolicyDisplay::Discrete(
                    probs_vec
                        .into_iter()
                        .enumerate()
                        .map(|(i, p)| {
                            let is_masked = mask
                                .map(|m| !m.get(i).copied().unwrap_or(true))
                                .unwrap_or(false);
                            PolicyItem {
                                action_id: i,
                                action: labels
                                    .get(i)
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| format!("Action {}", i)),
                                prob: p,
                                raw_prob: raw_probs_vec.get(i).copied().unwrap_or(0.0),
                                is_masked,
                            }
                        })
                        .collect(),
                ))
            }
            ActionSpace::Continuous(_) => Ok(PolicyDisplay::Discrete(Vec::new())),
            ActionSpace::Hybrid {
                continuous_dims,
                discrete_classes,
            } => {
                let means: Vec<f32> = self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                let attack_logits: Vec<f32> = self
                    .attack_head
                    .as_ref()
                    .unwrap()
                    .forward(&feat)?
                    .squeeze(0)?
                    .to_vec1()?;
                let raw_discrete_probs_vec = softmax_slice(&attack_logits);
                let masked = mask_logits_slice(&attack_logits, mask);
                let discrete_probs_vec = softmax_slice(&masked);

                if discrete_classes == 2 {
                    let p_attack = discrete_probs_vec.last().copied().unwrap_or(0.0);
                    let raw_p_attack = raw_discrete_probs_vec.last().copied().unwrap_or(0.0);
                    let is_attack_masked = mask
                        .map(|m| !m.get(1).copied().unwrap_or(true))
                        .unwrap_or(false);
                    let move_x = means.first().copied().unwrap_or(0.0).clamp(-1.0, 1.0);
                    let move_z = means.get(1).copied().unwrap_or(0.0).clamp(-1.0, 1.0);
                    Ok(PolicyDisplay::Hybrid {
                        move_x,
                        move_z,
                        attack_prob: p_attack,
                        raw_attack_prob: raw_p_attack,
                        is_attack_masked,
                    })
                } else {
                    let discrete_probs = discrete_probs_vec
                        .iter()
                        .enumerate()
                        .map(|(i, &prob)| {
                            let action_label = labels
                                .get(i)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("Act_{}", i));
                            let is_masked = mask
                                .map(|m| !m.get(i).copied().unwrap_or(true))
                                .unwrap_or(false);
                            PolicyItem {
                                action_id: i,
                                action: action_label,
                                prob,
                                raw_prob: raw_discrete_probs_vec.get(i).copied().unwrap_or(0.0),
                                is_masked,
                            }
                        })
                        .collect();
                    let continuous_means = means[..continuous_dims.min(means.len())].to_vec();
                    Ok(PolicyDisplay::HybridMulti {
                        continuous_means,
                        discrete_probs,
                    })
                }
            }
        }
    }

    /// 单步状态化采样（支持环境采样循环中的循环状态递推）
    pub fn step(
        &self,
        state: &Tensor,
        state_obj: &mut Option<MambaState>,
        mask: Option<&[bool]>,
    ) -> Result<(Vec<f32>, f32, f32)> {
        match &self.backbone {
            Backbone::Mlp { .. } => self.sample_action(state, mask),
            Backbone::Mamba {
                proj_in,
                mamba,
                config,
            } => {
                if state_obj.is_none() {
                    *state_obj = Some(MambaState::new(1, config, state.device())?);
                }
                let s = state_obj.as_mut().unwrap();
                let input = self.prepare_input(state)?;
                let x_proj = proj_in.forward(&input)?;
                let feat = mamba.step(&x_proj, s)?;
                let values = self.critic_head.forward(&feat)?;
                let val_scalar: f32 = values.squeeze(0)?.squeeze(0)?.to_scalar()?;

                match self.action_space {
                    ActionSpace::Discrete(_) => {
                        let logits: Vec<f32> =
                            self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                        let masked = mask_logits_slice(&logits, mask);
                        let (idx, log_prob) = sample_categorical(&masked);
                        Ok((vec![idx as f32], log_prob, val_scalar))
                    }
                    ActionSpace::Continuous(d) => {
                        let means: Vec<f32> =
                            self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                        let log_std: Vec<f32> = self.log_std.as_ref().unwrap().to_vec1()?;
                        let mut rng = rand::rng();
                        let mut encoded = Vec::with_capacity(d);
                        let mut log_prob = 0.0;
                        for i in 0..d {
                            let std = log_std[i].exp();
                            let a = means[i] + std * sample_gaussian(&mut rng);
                            encoded.push(a);
                            log_prob += gaussian_log_prob(means[i], std, a);
                        }
                        Ok((encoded, log_prob, val_scalar))
                    }
                    ActionSpace::Hybrid {
                        continuous_dims, ..
                    } => {
                        let means: Vec<f32> =
                            self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                        let log_std: Vec<f32> = self.log_std.as_ref().unwrap().to_vec1()?;
                        let attack_logits: Vec<f32> = self
                            .attack_head
                            .as_ref()
                            .unwrap()
                            .forward(&feat)?
                            .squeeze(0)?
                            .to_vec1()?;
                        let mut rng = rand::rng();
                        let mut encoded = Vec::with_capacity(continuous_dims + 1);
                        let mut log_prob = 0.0;
                        for i in 0..continuous_dims {
                            let std = log_std[i].exp();
                            let a = means[i] + std * sample_gaussian(&mut rng);
                            encoded.push(a);
                            log_prob += gaussian_log_prob(means[i], std, a);
                        }
                        let masked = mask_logits_slice(&attack_logits, mask);
                        let (idx, cat_log_prob) = sample_categorical(&masked);
                        encoded.push(idx as f32);
                        log_prob += cat_log_prob;
                        Ok((encoded, log_prob, val_scalar))
                    }
                }
            }
        }
    }

    /// PPO update：给定 (state, actions, masks) 计算 (log_probs, values, entropy)。
    /// state 支持 2D (n, state_dim) 或 3D (batch_chunks, seq_len, state_dim)。
    /// actions 形状 (n, encoding_dim)，Discrete=1 / Continuous=d / Hybrid=d+1。
    /// masks 形状 (n, num_classes)，用于屏蔽非法离散动作（1.0 = 有效，0.0 = 屏蔽）。
    pub fn evaluate_actions(
        &self,
        state: &Tensor,
        actions: &Tensor,
        masks: Option<&Tensor>,
    ) -> Result<(Tensor, Tensor, Tensor)> {
        let feat = if state.rank() == 3 {
            let (b, t, _) = state.dims3()?;
            let feat_3d = self.hidden(state)?;
            feat_3d.reshape((b * t, self.backbone.output_dim()))?
        } else {
            self.hidden(state)?
        };
        let n = feat.dim(0)?;
        let values = self.critic_head.forward(&feat)?;

        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits = self.actor_head.forward(&feat)?;
                let masked_logits = mask_logits_tensor(&logits, masks)?;
                let log_probs_all = candle_nn::ops::log_softmax(&masked_logits, D::Minus1)?;
                let probs_all = candle_nn::ops::softmax(&masked_logits, D::Minus1)?;
                let act = actions.squeeze(1)?.to_dtype(DType::U32)?;
                let selected_log_probs = log_probs_all.gather(&act.unsqueeze(1)?, 1)?.squeeze(1)?;
                let entropy = (probs_all * log_probs_all)?
                    .neg()?
                    .sum_keepdim(D::Minus1)?
                    .squeeze(1)?;
                Ok((selected_log_probs, values.squeeze(1)?, entropy))
            }
            ActionSpace::Continuous(d) => {
                let means = self.actor_head.forward(&feat)?;
                let log_std = self.log_std.as_ref().unwrap();
                let log_std_b = log_std.broadcast_as((n, d))?;
                let std_b = log_std_b.exp()?;
                let cont = actions.narrow(1, 0, d)?;
                let z = {
                    let diff = (&cont - &means)?;
                    (&diff / &std_b)?
                };
                let log_prob = z
                    .powf(2.0)?
                    .neg()?
                    .affine(0.5, 0.0)?
                    .sub(&log_std_b)?
                    .affine(1.0, -HALF_LN_2PI as f64)?
                    .sum(D::Minus1)?;
                let entropy = log_std_b
                    .affine(1.0, (0.5 + HALF_LN_2PI) as f64)?
                    .sum(D::Minus1)?;
                Ok((log_prob, values.squeeze(1)?, entropy))
            }
            ActionSpace::Hybrid {
                continuous_dims, ..
            } => {
                let means = self.actor_head.forward(&feat)?;
                let log_std = self.log_std.as_ref().unwrap();
                let log_std_b = log_std.broadcast_as((n, continuous_dims))?;
                let std_b = log_std_b.exp()?;
                let cont = actions.narrow(1, 0, continuous_dims)?;
                let z = {
                    let diff = (&cont - &means)?;
                    (&diff / &std_b)?
                };
                let gauss_log_prob = z
                    .powf(2.0)?
                    .neg()?
                    .affine(0.5, 0.0)?
                    .sub(&log_std_b)?
                    .affine(1.0, -HALF_LN_2PI as f64)?
                    .sum(D::Minus1)?;
                let gauss_entropy = log_std_b
                    .affine(1.0, (0.5 + HALF_LN_2PI) as f64)?
                    .sum(D::Minus1)?;

                let attack_logits = self.attack_head.as_ref().unwrap().forward(&feat)?;
                let masked = mask_logits_tensor(&attack_logits, masks)?;
                let log_probs_all = candle_nn::ops::log_softmax(&masked, D::Minus1)?;
                let probs_all = candle_nn::ops::softmax(&masked, D::Minus1)?;
                let act = actions
                    .narrow(1, continuous_dims, 1)?
                    .squeeze(1)?
                    .to_dtype(DType::U32)?;
                let cat_log_prob = log_probs_all.gather(&act.unsqueeze(1)?, 1)?.squeeze(1)?;
                let cat_entropy = (probs_all * log_probs_all)?
                    .neg()?
                    .sum_keepdim(D::Minus1)?
                    .squeeze(1)?;

                let log_prob = (&gauss_log_prob + &cat_log_prob)?;
                let entropy = (&gauss_entropy + &cat_entropy)?;
                Ok((log_prob, values.squeeze(1)?, entropy))
            }
        }
    }
}

/// 对一维切片应用布尔掩码（valid=true 保留，invalid=false 置为 -1e9）
fn mask_logits_slice(logits: &[f32], mask: Option<&[bool]>) -> Vec<f32> {
    match mask {
        Some(m) => logits
            .iter()
            .zip(m.iter())
            .map(|(&l, &valid)| if valid { l } else { -1e9 })
            .collect(),
        None => logits.to_vec(),
    }
}

/// 对 Tensor 应用掩码（mask 形状 (batch, classes)，1.0=有效，0.0=无效置 -1e9）
fn mask_logits_tensor(logits: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
    match mask {
        Some(m) => {
            let m_cast = if m.dtype() != logits.dtype() {
                m.to_dtype(logits.dtype())?
            } else {
                m.clone()
            };
            let penalty = m_cast.affine(1e9, -1e9)?; // valid(1.0)->0.0, invalid(0.0)->-1e9
            logits.broadcast_add(&penalty)
        }
        None => Ok(logits.clone()),
    }
}

fn softmax_slice(logits: &[f32]) -> Vec<f32> {
    let max_l = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits.iter().map(|&x| (x - max_l).exp()).collect();
    let sum_exp: f32 = exps.iter().sum();
    if sum_exp > 0.0 {
        exps.iter().map(|&e| e / sum_exp).collect()
    } else {
        vec![1.0 / logits.len() as f32; logits.len()]
    }
}

fn sample_categorical(logits: &[f32]) -> (usize, f32) {
    let probs = softmax_slice(logits);
    let idx = sample_from_probs(&probs);
    let log_prob = if probs[idx] > 1e-12 {
        probs[idx].ln()
    } else {
        -20.0
    };
    (idx, log_prob)
}

fn sample_from_probs(probs: &[f32]) -> usize {
    let mut rng = rand::rng();
    let r: f32 = rng.random();
    let mut cum_prob = 0.0;
    for (idx, &prob) in probs.iter().enumerate() {
        cum_prob += prob;
        if r <= cum_prob {
            return idx;
        }
    }
    probs.len() - 1
}

fn argmax(values: &[f32]) -> usize {
    let mut max_idx = 0;
    let mut max_val = f32::NEG_INFINITY;
    for (idx, &val) in values.iter().enumerate() {
        if val > max_val {
            max_val = val;
            max_idx = idx;
        }
    }
    max_idx
}

/// Box-Muller 采样标准正态 N(0,1)。
fn sample_gaussian(rng: &mut impl rand::Rng) -> f32 {
    let u1: f64 = rng.random::<f64>().max(f64::EPSILON);
    let u2: f64 = rng.random::<f64>();
    let r = (-2.0 * u1.ln()).sqrt();
    let theta = 2.0 * std::f64::consts::PI * u2;
    (r * theta.cos()) as f32
}

fn gaussian_log_prob(mean: f32, std: f32, action: f32) -> f32 {
    let z = (action - mean) / std;
    -0.5 * z * z - std.ln() - HALF_LN_2PI
}
