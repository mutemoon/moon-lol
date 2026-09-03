use candle_core::{D, DType, IndexOp, Result, Tensor};
use candle_nn::{Conv1d, Conv1dConfig, Embedding, Linear, Module, VarBuilder};
use lol_rl_protocol::{
    ActionBranchDisplay, ActionSpace, EntityEncoderSpec, ObsNode, ObsSchema, PolicyBackbone,
    PolicyDisplay, PolicyItem, PoolType,
};
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

    pub fn collect_params(&self, out: &mut Vec<LayerParamInfo>) {
        match self {
            Self::Mlp { fc1, fc2, .. } => {
                out.push(LayerParamInfo::new(
                    "Backbone (MLP)",
                    "fc1.weight",
                    fc1.weight().dims(),
                ));
                if let Some(b) = fc1.bias() {
                    out.push(LayerParamInfo::new("Backbone (MLP)", "fc1.bias", b.dims()));
                }
                out.push(LayerParamInfo::new(
                    "Backbone (MLP)",
                    "fc2.weight",
                    fc2.weight().dims(),
                ));
                if let Some(b) = fc2.bias() {
                    out.push(LayerParamInfo::new("Backbone (MLP)", "fc2.bias", b.dims()));
                }
            }
            Self::Mamba { proj_in, mamba, .. } => {
                out.push(LayerParamInfo::new(
                    "Backbone (Mamba)",
                    "proj_in.weight",
                    proj_in.weight().dims(),
                ));
                if let Some(b) = proj_in.bias() {
                    out.push(LayerParamInfo::new(
                        "Backbone (Mamba)",
                        "proj_in.bias",
                        b.dims(),
                    ));
                }
                out.push(LayerParamInfo::new(
                    "Backbone (Mamba)",
                    "mamba.in_proj.weight",
                    mamba.in_proj.weight().dims(),
                ));
                out.push(LayerParamInfo::new(
                    "Backbone (Mamba)",
                    "mamba.conv1d.weight",
                    mamba.conv1d.weight().dims(),
                ));
                if let Some(b) = mamba.conv1d.bias() {
                    out.push(LayerParamInfo::new(
                        "Backbone (Mamba)",
                        "mamba.conv1d.bias",
                        b.dims(),
                    ));
                }
                out.push(LayerParamInfo::new(
                    "Backbone (Mamba)",
                    "mamba.x_proj.weight",
                    mamba.x_proj.weight().dims(),
                ));
                out.push(LayerParamInfo::new(
                    "Backbone (Mamba)",
                    "mamba.dt_proj.weight",
                    mamba.dt_proj.weight().dims(),
                ));
                if let Some(b) = mamba.dt_proj.bias() {
                    out.push(LayerParamInfo::new(
                        "Backbone (Mamba)",
                        "mamba.dt_proj.bias",
                        b.dims(),
                    ));
                }
                out.push(LayerParamInfo::new(
                    "Backbone (Mamba)",
                    "mamba.A_log",
                    mamba.a_log.dims(),
                ));
                out.push(LayerParamInfo::new(
                    "Backbone (Mamba)",
                    "mamba.D",
                    mamba.d.dims(),
                ));
                out.push(LayerParamInfo::new(
                    "Backbone (Mamba)",
                    "mamba.out_proj.weight",
                    mamba.out_proj.weight().dims(),
                ));
                out.push(LayerParamInfo::new(
                    "Backbone (Mamba)",
                    "mamba.norm.scale",
                    mamba.norm.scale.dims(),
                ));
            }
        }
    }
}

// ── AST 声明式观测特征提取模块 (Obs Feature Extractor) ──────────────────────

/// 单个叶子特征或实体特征的提取器
#[derive(Clone)]
pub enum NodeExtractor {
    PassThrough {
        dim: usize,
    },
    Categorical {
        embed: Embedding,
        embed_dim: usize,
    },
    Struct {
        field_extractors: Vec<NodeExtractor>,
        field_raw_dims: Vec<usize>,
    },
    Repeated {
        name: String,
        max_count: usize,
        item_raw_dim: usize,
        item_extractor: Box<NodeExtractor>,
        mlp_layers: Vec<Linear>,
        skip_proj: Option<Linear>,
        encoder_spec: EntityEncoderSpec,
    },
}

/// 观测特征提取结果（主干聚合特征 + 可选的命名实体嵌入）
pub struct ExtractedFeatures {
    /// 聚合后的主干输入特征 (batch, encoded_dim) 或 (batch, seq, encoded_dim)
    pub aggregated: Tensor,
    /// 池化前的命名实体嵌入，key 为 Repeated 节点名
    /// 值 shape: (batch, max_count, item_embed_dim)
    pub entity_embeds: std::collections::HashMap<String, Tensor>,
}

impl NodeExtractor {
    pub fn from_node(node: &ObsNode, vb: VarBuilder) -> Result<Self> {
        match node {
            ObsNode::Scalar { .. } => Ok(Self::PassThrough { dim: 1 }),
            ObsNode::Vector { dim, .. } => Ok(Self::PassThrough { dim: *dim }),
            ObsNode::Categorical {
                name,
                num_classes,
                embed_dim,
                ..
            } => {
                let embed = candle_nn::embedding(
                    *num_classes,
                    *embed_dim,
                    vb.pp(format!("{}_embed", name)),
                )?;
                Ok(Self::Categorical {
                    embed,
                    embed_dim: *embed_dim,
                })
            }
            ObsNode::Struct { name, fields } => {
                let mut field_extractors = Vec::with_capacity(fields.len());
                let mut field_raw_dims = Vec::with_capacity(fields.len());
                let struct_vb = vb.pp(name);
                for field in fields {
                    field_raw_dims.push(field.raw_dim());
                    field_extractors.push(NodeExtractor::from_node(field, struct_vb.clone())?);
                }
                Ok(Self::Struct {
                    field_extractors,
                    field_raw_dims,
                })
            }
            ObsNode::Repeated {
                name,
                max_count,
                item,
                encoder,
            } => {
                let rep_vb = vb.pp(name);
                let item_raw_dim = item.raw_dim();
                let item_extractor = Box::new(NodeExtractor::from_node(item, rep_vb.pp("item"))?);

                let item_in_dim = item.embedded_item_dim();
                let mut mlp_layers = Vec::new();
                let hidden_dims = match encoder {
                    EntityEncoderSpec::SharedMlpFlatten { hidden_dims } => hidden_dims.as_slice(),
                    EntityEncoderSpec::SharedMlpPool { hidden_dims, .. } => hidden_dims.as_slice(),
                    EntityEncoderSpec::PassThrough => &[],
                };

                let mut cur_dim = item_in_dim;
                for (idx, &h_dim) in hidden_dims.iter().enumerate() {
                    let linear =
                        candle_nn::linear(cur_dim, h_dim, rep_vb.pp(format!("mlp_{}", idx)))?;
                    mlp_layers.push(linear);
                    cur_dim = h_dim;
                }

                // 实体线性残差直通 (Linear Residual Skip-Connection)
                let skip_proj = if !mlp_layers.is_empty() {
                    let out_dim = *hidden_dims.last().unwrap();
                    if item_in_dim != out_dim {
                        Some(candle_nn::linear(
                            item_in_dim,
                            out_dim,
                            rep_vb.pp("skip_proj"),
                        )?)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(Self::Repeated {
                    name: name.clone(),
                    max_count: *max_count,
                    item_raw_dim,
                    item_extractor,
                    mlp_layers,
                    skip_proj,
                    encoder_spec: encoder.clone(),
                })
            }
        }
    }

    pub fn forward_2d_with_entities(
        &self,
        x: &Tensor,
    ) -> Result<(Tensor, std::collections::HashMap<String, Tensor>)> {
        match self {
            Self::PassThrough { .. } => Ok((x.clone(), std::collections::HashMap::new())),
            Self::Categorical { embed, .. } => {
                let ids = x.squeeze(1)?.to_dtype(DType::U32)?;
                Ok((embed.forward(&ids)?, std::collections::HashMap::new()))
            }
            Self::Struct {
                field_extractors,
                field_raw_dims,
            } => {
                let mut offset = 0;
                let mut parts = Vec::with_capacity(field_extractors.len());
                let mut all_embeds = std::collections::HashMap::new();
                for (ext, &r_dim) in field_extractors.iter().zip(field_raw_dims.iter()) {
                    let slice = x.narrow(1, offset, r_dim)?;
                    let (part_feat, part_embeds) = ext.forward_2d_with_entities(&slice)?;
                    parts.push(part_feat);
                    all_embeds.extend(part_embeds);
                    offset += r_dim;
                }
                let agg = if parts.len() == 1 {
                    parts.remove(0)
                } else {
                    Tensor::cat(&parts, 1)?
                };
                Ok((agg, all_embeds))
            }
            Self::Repeated {
                name,
                max_count,
                item_raw_dim,
                item_extractor,
                mlp_layers,
                skip_proj,
                encoder_spec,
            } => {
                let (b, _total_raw) = x.dims2()?;
                let item_input = x.reshape((b * max_count, *item_raw_dim))?;
                let (in_feat, mut embeds) = item_extractor.forward_2d_with_entities(&item_input)?;

                let mut mlp_feat = in_feat.clone();
                for layer in mlp_layers {
                    mlp_feat = layer.forward(&mlp_feat)?.tanh()?;
                }

                // 线性残差融合：非线性特征 + 线性直通特征
                let feat = if !mlp_layers.is_empty() {
                    let skip_feat = match skip_proj {
                        Some(proj) => proj.forward(&in_feat)?,
                        None => in_feat,
                    };
                    (mlp_feat + skip_feat)?
                } else {
                    in_feat
                };

                let item_feat_dim = feat.dim(1)?;
                let feat_3d = feat.reshape((b, *max_count, item_feat_dim))?;

                embeds.insert(name.clone(), feat_3d.clone());

                let agg = match encoder_spec {
                    EntityEncoderSpec::SharedMlpFlatten { .. } | EntityEncoderSpec::PassThrough => {
                        feat_3d.reshape((b, max_count * item_feat_dim))?
                    }
                    EntityEncoderSpec::SharedMlpPool { pool_type, .. } => match pool_type {
                        PoolType::Max => feat_3d.max(1)?,
                        PoolType::Mean => feat_3d.mean(1)?,
                        PoolType::Sum => feat_3d.sum(1)?,
                    },
                };
                Ok((agg, embeds))
            }
        }
    }

    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        match self {
            Self::PassThrough { dim } => Ok(Self::PassThrough { dim: *dim }),
            Self::Categorical { embed, embed_dim } => {
                let w = embed.embeddings().to_device(device)?;
                Ok(Self::Categorical {
                    embed: Embedding::new(w, *embed_dim),
                    embed_dim: *embed_dim,
                })
            }
            Self::Struct {
                field_extractors,
                field_raw_dims,
            } => {
                let mut moved_fields = Vec::with_capacity(field_extractors.len());
                for ext in field_extractors {
                    moved_fields.push(ext.to_device(device)?);
                }
                Ok(Self::Struct {
                    field_extractors: moved_fields,
                    field_raw_dims: field_raw_dims.clone(),
                })
            }
            Self::Repeated {
                name,
                max_count,
                item_raw_dim,
                item_extractor,
                mlp_layers,
                skip_proj,
                encoder_spec,
            } => {
                let moved_item = Box::new(item_extractor.to_device(device)?);
                let mut moved_layers = Vec::with_capacity(mlp_layers.len());
                for layer in mlp_layers {
                    let w = layer.weight().to_device(device)?;
                    let b = layer.bias().map(|b| b.to_device(device)).transpose()?;
                    moved_layers.push(Linear::new(w, b));
                }
                let moved_skip = match skip_proj {
                    Some(proj) => {
                        let w = proj.weight().to_device(device)?;
                        let b = proj.bias().map(|b| b.to_device(device)).transpose()?;
                        Some(Linear::new(w, b))
                    }
                    None => None,
                };
                Ok(Self::Repeated {
                    name: name.clone(),
                    max_count: *max_count,
                    item_raw_dim: *item_raw_dim,
                    item_extractor: moved_item,
                    mlp_layers: moved_layers,
                    skip_proj: moved_skip,
                    encoder_spec: encoder_spec.clone(),
                })
            }
        }
    }

    pub fn collect_params(&self, prefix: &str, out: &mut Vec<LayerParamInfo>) {
        match self {
            Self::PassThrough { .. } => {}
            Self::Categorical { embed, .. } => {
                out.push(LayerParamInfo::new(
                    "Obs 特征提取",
                    format!("{prefix}.embed"),
                    embed.embeddings().dims(),
                ));
            }
            Self::Struct {
                field_extractors, ..
            } => {
                for (idx, ext) in field_extractors.iter().enumerate() {
                    ext.collect_params(&format!("{prefix}.field_{idx}"), out);
                }
            }
            Self::Repeated {
                name,
                item_extractor,
                mlp_layers,
                skip_proj,
                ..
            } => {
                let pfx = if prefix.is_empty() || prefix == name {
                    name.as_str()
                } else {
                    prefix
                };
                item_extractor.collect_params(&format!("{pfx}.item"), out);
                for (idx, layer) in mlp_layers.iter().enumerate() {
                    out.push(LayerParamInfo::new(
                        "Obs 特征提取",
                        format!("{pfx}.mlp_{idx}.weight"),
                        layer.weight().dims(),
                    ));
                    if let Some(b) = layer.bias() {
                        out.push(LayerParamInfo::new(
                            "Obs 特征提取",
                            format!("{pfx}.mlp_{idx}.bias"),
                            b.dims(),
                        ));
                    }
                }
                if let Some(proj) = skip_proj {
                    out.push(LayerParamInfo::new(
                        "Obs 特征提取 (Residual Skip)",
                        format!("{pfx}.skip_proj.weight"),
                        proj.weight().dims(),
                    ));
                    if let Some(b) = proj.bias() {
                        out.push(LayerParamInfo::new(
                            "Obs 特征提取 (Residual Skip)",
                            format!("{pfx}.skip_proj.bias"),
                            b.dims(),
                        ));
                    }
                }
            }
        }
    }
}

/// AST 声明式观测特征提取器
#[derive(Clone)]
pub struct ObsFeatureExtractor {
    schema: ObsSchema,
    node_extractors: Vec<NodeExtractor>,
    node_raw_dims: Vec<usize>,
}

impl ObsFeatureExtractor {
    pub fn new(schema: ObsSchema, vb: VarBuilder) -> Result<Self> {
        let mut node_extractors = Vec::with_capacity(schema.nodes.len());
        let mut node_raw_dims = Vec::with_capacity(schema.nodes.len());
        for node in &schema.nodes {
            node_raw_dims.push(node.raw_dim());
            node_extractors.push(NodeExtractor::from_node(node, vb.clone())?);
        }
        Ok(Self {
            schema,
            node_extractors,
            node_raw_dims,
        })
    }

    pub fn schema(&self) -> &ObsSchema {
        &self.schema
    }

    pub fn forward_with_entities(&self, state: &Tensor) -> Result<ExtractedFeatures> {
        let is_3d = state.rank() == 3;
        let (state_2d, b, l) = if is_3d {
            let (b, l, raw_dim) = state.dims3()?;
            (state.reshape((b * l, raw_dim))?, b, l)
        } else {
            (state.clone(), 0, 0)
        };

        let mut offset = 0;
        let mut parts = Vec::with_capacity(self.node_extractors.len());
        let mut all_embeds = std::collections::HashMap::new();
        for (ext, &r_dim) in self.node_extractors.iter().zip(self.node_raw_dims.iter()) {
            let slice = state_2d.narrow(1, offset, r_dim)?;
            let (part_feat, part_embeds) = ext.forward_2d_with_entities(&slice)?;
            parts.push(part_feat);
            all_embeds.extend(part_embeds);
            offset += r_dim;
        }

        let mut aggregated = if parts.len() == 1 {
            parts.remove(0)
        } else {
            Tensor::cat(&parts, 1)?.contiguous()?
        };

        if is_3d {
            let feat_dim = aggregated.dim(1)?;
            aggregated = aggregated.reshape((b, l, feat_dim))?.contiguous()?;
        }

        Ok(ExtractedFeatures {
            aggregated,
            entity_embeds: all_embeds,
        })
    }

    pub fn forward(&self, state: &Tensor) -> Result<Tensor> {
        self.forward_with_entities(state).map(|f| f.aggregated)
    }

    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        let mut moved_nodes = Vec::with_capacity(self.node_extractors.len());
        for ext in &self.node_extractors {
            moved_nodes.push(ext.to_device(device)?);
        }
        Ok(Self {
            schema: self.schema.clone(),
            node_extractors: moved_nodes,
            node_raw_dims: self.node_raw_dims.clone(),
        })
    }

    pub fn collect_params(&self, out: &mut Vec<LayerParamInfo>) {
        for (node, ext) in self.schema.nodes.iter().zip(self.node_extractors.iter()) {
            ext.collect_params(node.name(), out);
        }
    }
}

/// PolicyNetwork 纯策略网络（专注特征提取、主干网络与动作采样/分布推演，100% 纯 Actor，无 Critic）
#[derive(Clone)]
pub struct PolicyNetwork {
    /// 可选的结构化多头 Actor（当环境提供 ActionSchema 时使用）
    pub structured_action_head: Option<StructuredActionHead>,
    /// AST 声明式特征提取器
    feature_extractor: ObsFeatureExtractor,
    /// 核心特征提取主干（MLP 或 Mamba）
    backbone: Backbone,
    /// 动作输出头：离散分类 logits 或连续动作均值
    actor_head: Linear,
    /// 连续/混合动作：可训练 log_std
    log_std: Option<Tensor>,
    /// 混合动作：离散分类头
    attack_head: Option<Linear>,
    /// 可选：Belief-State 信念解码头
    belief_head: Option<BeliefHead>,
    action_space: ActionSpace,
}

impl PolicyNetwork {
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

    /// 基于 ObsSchema 结构规范自动推导特征提取与主干架构
    pub fn from_schema_and_backbone(
        schema: ObsSchema,
        hidden_dim: usize,
        action_space: ActionSpace,
        backbone_type: PolicyBackbone,
        belief_dim: Option<usize>,
        vb: VarBuilder,
    ) -> Result<Self> {
        let extractor = ObsFeatureExtractor::new(schema, vb.clone())?;
        let in_dim = extractor.schema().encoded_dim();

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
            structured_action_head: None,
            feature_extractor: extractor,
            backbone,
            actor_head,
            log_std,
            attack_head,
            belief_head,
            action_space,
        })
    }

    pub fn from_schema(
        schema: ObsSchema,
        hidden_dim: usize,
        action_space: ActionSpace,
        vb: VarBuilder,
    ) -> Result<Self> {
        Self::from_schema_and_backbone(
            schema,
            hidden_dim,
            action_space,
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
        let schema = ObsSchema::new(vec![
            ObsNode::categorical(
                "hero",
                hero_embed_config.num_heroes,
                hero_embed_config.embed_dim,
            ),
            ObsNode::vector("rest", state_dim.saturating_sub(1)),
        ]);
        Self::from_schema_and_backbone(
            schema,
            hidden_dim,
            action_space,
            backbone_type,
            belief_dim,
            vb,
        )
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
        Self::with_hero_embed_and_backbone(
            state_dim,
            mamba_config.d_model,
            action_space,
            hero_embed_config,
            PolicyBackbone::Mamba,
            belief_dim,
            vb,
        )
    }

    /// 基于 ObsSchema + ActionSchema 自动推导完整的 Policy 网络
    pub fn from_schemas(
        obs_schema: ObsSchema,
        action_schema: ActionSchema,
        hidden_dim: usize,
        backbone_type: PolicyBackbone,
        belief_dim: Option<usize>,
        vb: VarBuilder,
    ) -> Result<Self> {
        let extractor = ObsFeatureExtractor::new(obs_schema, vb.clone())?;
        let in_dim = extractor.schema().encoded_dim();

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

        let belief_head = match belief_dim {
            Some(b_dim) => Some(BeliefHead::new(feat_dim, b_dim, vb.pp("belief_head"))?),
            None => None,
        };

        let structured_action_head =
            StructuredActionHead::new(action_schema, feat_dim, vb.pp("structured_actor"))?;

        let action_space = ActionSpace::Discrete(1);
        let actor_head = candle_nn::linear(feat_dim, 1, vb.pp("dummy_actor_head"))?;

        Ok(Self {
            structured_action_head: Some(structured_action_head),
            feature_extractor: extractor,
            backbone,
            actor_head,
            log_std: None,
            attack_head: None,
            belief_head,
            action_space,
        })
    }

    pub fn action_space(&self) -> &ActionSpace {
        &self.action_space
    }

    pub fn feature_extractor(&self) -> &ObsFeatureExtractor {
        &self.feature_extractor
    }

    pub fn schema(&self) -> &ObsSchema {
        self.feature_extractor.schema()
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

    pub fn prepare_input(&self, state: &Tensor) -> Result<Tensor> {
        self.feature_extractor.forward(state)
    }

    pub fn hidden(&self, state: &Tensor) -> Result<Tensor> {
        let input = self.prepare_input(state)?;
        self.backbone.forward(&input)
    }

    pub fn forward_actor(&self, state: &Tensor) -> Result<Tensor> {
        let feat = self.hidden(state)?;
        self.actor_head.forward(&feat)
    }

    pub fn forward_belief(&self, state: &Tensor) -> Result<Option<(Tensor, Tensor)>> {
        if let Some(ref bh) = self.belief_head {
            let feat = self.hidden(state)?;
            let (mu, std) = bh.forward(&feat)?;
            Ok(Some((mu, std)))
        } else {
            Ok(None)
        }
    }

    /// 从策略采样一个动作。返回 (编码动作向量, log_prob)。
    pub fn sample_action(&self, state: &Tensor, mask: Option<&[bool]>) -> Result<(Vec<f32>, f32)> {
        self.sample_action_with_structured_masks(state, None, mask)
    }

    /// 从策略采样一个动作（支持结构化掩码 ActionMasks）。
    pub fn sample_action_with_structured_masks(
        &self,
        state: &Tensor,
        structured_mask: Option<&ActionMasks>,
        mask: Option<&[bool]>,
    ) -> Result<(Vec<f32>, f32)> {
        if let Some(ref structured_head) = self.structured_action_head {
            let ext = self.feature_extractor.forward_with_entities(state)?;
            let feat = self.backbone.forward(&ext.aggregated)?;
            let (encoded, log_prob) =
                structured_head.sample(&feat, structured_mask, &ext.entity_embeds)?;
            return Ok((encoded, log_prob));
        }

        let feat = self.hidden(state)?;
        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits: Vec<f32> = self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                let masked = mask_logits_slice(&logits, mask);
                let (idx, log_prob) = sample_categorical(&masked);
                Ok((vec![idx as f32], log_prob))
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
                Ok((encoded, log_prob))
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
                Ok((encoded, log_prob))
            }
        }
    }

    /// 批量从策略采样动作（一次 GPU/CPU 前向计算），返回每个样本的 (encoded_action, log_prob)。
    pub fn sample_batch(
        &self,
        states: &Tensor,
        masks: Option<&[Option<Vec<bool>>]>,
    ) -> Result<Vec<(Vec<f32>, f32)>> {
        self.sample_batch_with_structured_masks(states, None, masks)
    }

    /// 批量从策略采样动作（支持结构化掩码 ActionMasks 列表）。
    pub fn sample_batch_with_structured_masks(
        &self,
        states: &Tensor,
        structured_masks: Option<&[Option<ActionMasks>]>,
        masks: Option<&[Option<Vec<bool>>]>,
    ) -> Result<Vec<(Vec<f32>, f32)>> {
        let b = states.dim(0)?;
        if b == 0 {
            return Ok(Vec::new());
        }

        if let Some(ref structured_head) = self.structured_action_head {
            let ext = self.feature_extractor.forward_with_entities(states)?;
            let feat = self.backbone.forward(&ext.aggregated)?;
            let mut results = Vec::with_capacity(b);
            for i in 0..b {
                let feat_i = feat.narrow(0, i, 1)?;
                let mut embeds_i = HashMap::new();
                for (k, v) in &ext.entity_embeds {
                    embeds_i.insert(k.clone(), v.narrow(0, i, 1)?);
                }
                let mask_i = structured_masks
                    .and_then(|ms| ms.get(i))
                    .and_then(|m| m.as_ref());
                let (encoded, log_prob) = structured_head.sample(&feat_i, mask_i, &embeds_i)?;
                results.push((encoded, log_prob));
            }
            return Ok(results);
        }

        let feat = self.hidden(states)?;
        let mut results = Vec::with_capacity(b);

        match self.action_space {
            ActionSpace::Discrete(_) => {
                let logits = self.actor_head.forward(&feat)?;
                let logits_mat: Vec<Vec<f32>> = logits.to_vec2()?;
                for i in 0..b {
                    let mask_i = masks.and_then(|ms| ms.get(i)).and_then(|m| m.as_deref());
                    let masked = mask_logits_slice(&logits_mat[i], mask_i);
                    let (idx, log_prob) = sample_categorical(&masked);
                    results.push((vec![idx as f32], log_prob));
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
                    results.push((encoded, log_prob));
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
                    results.push((encoded, log_prob));
                }
            }
        }

        Ok(results)
    }

    /// 确定性贪心动作（连续取均值、离散取 argmax），用于可视化与评估。
    pub fn select_greedy_action(&self, state: &Tensor, mask: Option<&[bool]>) -> Result<Vec<f32>> {
        self.select_greedy_action_with_structured_masks(state, None, mask)
    }

    /// 确定性贪心动作（支持结构化掩码 ActionMasks）。
    pub fn select_greedy_action_with_structured_masks(
        &self,
        state: &Tensor,
        structured_mask: Option<&ActionMasks>,
        mask: Option<&[bool]>,
    ) -> Result<Vec<f32>> {
        if let Some(ref structured_head) = self.structured_action_head {
            let ext = self.feature_extractor.forward_with_entities(state)?;
            let feat = self.backbone.forward(&ext.aggregated)?;
            let (encoded, _lp) =
                structured_head.sample(&feat, structured_mask, &ext.entity_embeds)?;
            return Ok(encoded);
        }

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
        if let Some(ref structured_head) = self.structured_action_head {
            let ext = self.feature_extractor.forward_with_entities(state)?;
            let feat = self.backbone.forward(&ext.aggregated)?;
            let displays = structured_head.policy_display(&feat, &ext.entity_embeds, None)?;
            return Ok(PolicyDisplay::Structured(displays));
        }

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
    ) -> Result<(Vec<f32>, f32)> {
        self.step_with_structured_masks(state, state_obj, None, mask)
    }

    /// 单步状态化采样（支持结构化掩码 ActionMasks）
    pub fn step_with_structured_masks(
        &self,
        state: &Tensor,
        state_obj: &mut Option<MambaState>,
        structured_mask: Option<&ActionMasks>,
        mask: Option<&[bool]>,
    ) -> Result<(Vec<f32>, f32)> {
        if let Some(ref structured_head) = self.structured_action_head {
            match &self.backbone {
                Backbone::Mlp { .. } => {
                    return self.sample_action_with_structured_masks(state, structured_mask, mask);
                }
                Backbone::Mamba {
                    proj_in,
                    mamba,
                    config,
                } => {
                    if state_obj.is_none() {
                        *state_obj = Some(MambaState::new(1, config, state.device())?);
                    }
                    let s = state_obj.as_mut().unwrap();
                    let ext = self.feature_extractor.forward_with_entities(state)?;
                    let x_proj = proj_in.forward(&ext.aggregated)?;
                    let feat = mamba.step(&x_proj, s)?;
                    let (encoded, log_prob) =
                        structured_head.sample(&feat, structured_mask, &ext.entity_embeds)?;
                    return Ok((encoded, log_prob));
                }
            }
        }

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

                match self.action_space {
                    ActionSpace::Discrete(_) => {
                        let logits: Vec<f32> =
                            self.actor_head.forward(&feat)?.squeeze(0)?.to_vec1()?;
                        let masked = mask_logits_slice(&logits, mask);
                        let (idx, log_prob) = sample_categorical(&masked);
                        Ok((vec![idx as f32], log_prob))
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
                        Ok((encoded, log_prob))
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
                        Ok((encoded, log_prob))
                    }
                }
            }
        }
    }

    /// 给定 (state, actions, masks) 计算 (log_probs, entropy)
    pub fn evaluate_actions(
        &self,
        state: &Tensor,
        actions: &Tensor,
        masks: Option<&Tensor>,
    ) -> Result<(Tensor, Tensor)> {
        if let Some(ref structured_head) = self.structured_action_head {
            let (feat, entity_embeds) = if state.rank() == 3 {
                let (b, t, _) = state.dims3()?;
                let ext = self.feature_extractor.forward_with_entities(state)?;
                let feat_3d = self.backbone.forward(&ext.aggregated)?;
                let feat_2d = feat_3d.reshape((b * t, self.backbone.output_dim()))?;
                let mut flat_embeds = HashMap::new();
                for (k, v) in ext.entity_embeds {
                    let (_, max_u, d) = v.dims3()?;
                    flat_embeds.insert(k, v.reshape((b * t, max_u, d))?);
                }
                (feat_2d, flat_embeds)
            } else {
                let ext = self.feature_extractor.forward_with_entities(state)?;
                let feat = self.backbone.forward(&ext.aggregated)?;
                (feat, ext.entity_embeds)
            };
            let (log_prob, entropy) =
                structured_head.evaluate(&feat, actions, None, &entity_embeds, masks)?;
            return Ok((log_prob, entropy));
        }

        let feat = if state.rank() == 3 {
            let (b, t, _) = state.dims3()?;
            let feat_3d = self.hidden(state)?;
            feat_3d.reshape((b * t, self.backbone.output_dim()))?
        } else {
            self.hidden(state)?
        };
        let n = feat.dim(0)?;

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
                Ok((selected_log_probs, entropy))
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
                Ok((log_prob, entropy))
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
                Ok((log_prob, entropy))
            }
        }
    }

    /// 将策略网络权重复制并迁移到指定计算设备
    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        let feature_extractor = self.feature_extractor.to_device(device)?;
        let backbone = self.backbone.to_device(device)?;

        let actor_w = self.actor_head.weight().to_device(device)?;
        let actor_b = self
            .actor_head
            .bias()
            .map(|b| b.to_device(device))
            .transpose()?;
        let actor_head = Linear::new(actor_w, actor_b);

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

        let structured_action_head = self
            .structured_action_head
            .as_ref()
            .map(|h| h.to_device(device))
            .transpose()?;

        Ok(Self {
            structured_action_head,
            feature_extractor,
            backbone,
            actor_head,
            log_std,
            attack_head,
            belief_head,
            action_space: self.action_space.clone(),
        })
    }

    pub fn collect_params(&self, out: &mut Vec<LayerParamInfo>) {
        self.feature_extractor.collect_params(out);
        self.backbone.collect_params(out);
        if let Some(ref structured_head) = self.structured_action_head {
            structured_head.collect_params(out);
        } else {
            out.push(LayerParamInfo::new(
                "Actor 策略头",
                "actor_head.weight",
                self.actor_head.weight().dims(),
            ));
            if let Some(b) = self.actor_head.bias() {
                out.push(LayerParamInfo::new(
                    "Actor 策略头",
                    "actor_head.bias",
                    b.dims(),
                ));
            }
            if let Some(ref log_std) = self.log_std {
                out.push(LayerParamInfo::new(
                    "Actor 策略头",
                    "log_std",
                    log_std.dims(),
                ));
            }
            if let Some(ref attack_head) = self.attack_head {
                out.push(LayerParamInfo::new(
                    "Actor 策略头",
                    "attack_head.weight",
                    attack_head.weight().dims(),
                ));
                if let Some(b) = attack_head.bias() {
                    out.push(LayerParamInfo::new(
                        "Actor 策略头",
                        "attack_head.bias",
                        b.dims(),
                    ));
                }
            }
        }
        if let Some(ref belief_head) = self.belief_head {
            out.push(LayerParamInfo::new(
                "Belief 信念头",
                "belief_mu.weight",
                belief_head.mu.weight().dims(),
            ));
            if let Some(b) = belief_head.mu.bias() {
                out.push(LayerParamInfo::new(
                    "Belief 信念头",
                    "belief_mu.bias",
                    b.dims(),
                ));
            }
            out.push(LayerParamInfo::new(
                "Belief 信念头",
                "belief_logvar.weight",
                belief_head.logvar.weight().dims(),
            ));
            if let Some(b) = belief_head.logvar.bias() {
                out.push(LayerParamInfo::new(
                    "Belief 信念头",
                    "belief_logvar.bias",
                    b.dims(),
                ));
            }
        }
    }

    /// 提取模型所有可训练参数的层级明细与参数量统计（单位 K / M）
    pub fn parameter_summary(&self) -> ModelParamSummary {
        let mut out = Vec::new();
        self.collect_params(&mut out);
        ModelParamSummary::new(out)
    }

    pub fn print_parameter_summary(&self) {
        self.parameter_summary().print_summary();
    }
}

/// ValueHead 状态价值估计头（Critic，专供需要 Value 学习的算法如 PPO）
#[derive(Clone)]
pub struct ValueHead {
    pub head: Linear,
}

impl ValueHead {
    pub fn new(feat_dim: usize, vb: VarBuilder) -> Result<Self> {
        let head = candle_nn::linear(feat_dim, 1, vb.pp("critic_head"))?;
        Ok(Self { head })
    }

    pub fn forward(&self, feat: &Tensor) -> Result<Tensor> {
        self.head.forward(feat)
    }

    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        let w = self.head.weight().to_device(device)?;
        let b = self.head.bias().map(|b| b.to_device(device)).transpose()?;
        Ok(Self {
            head: Linear::new(w, b),
        })
    }

    pub fn collect_params(&self, out: &mut Vec<LayerParamInfo>) {
        out.push(LayerParamInfo::new(
            "Critic 价值头",
            "critic_head.weight",
            self.head.weight().dims(),
        ));
        if let Some(b) = self.head.bias() {
            out.push(LayerParamInfo::new(
                "Critic 价值头",
                "critic_head.bias",
                b.dims(),
            ));
        }
    }
}

/// ActorCritic 策略与价值网络组合（由 PolicyNetwork + ValueHead 组合而成，专供 PPO 算法）
#[derive(Clone)]
pub struct ActorCritic {
    pub policy: PolicyNetwork,
    pub critic: ValueHead,
}

impl ActorCritic {
    pub fn new(
        state_dim: usize,
        hidden_dim: usize,
        action_space: ActionSpace,
        vb: VarBuilder,
    ) -> Result<Self> {
        let policy = PolicyNetwork::new(state_dim, hidden_dim, action_space, vb.clone())?;
        let feat_dim = policy.backbone().output_dim();
        let critic = ValueHead::new(feat_dim, vb)?;
        Ok(Self { policy, critic })
    }

    pub fn from_schema_and_backbone(
        schema: ObsSchema,
        hidden_dim: usize,
        action_space: ActionSpace,
        backbone_type: PolicyBackbone,
        belief_dim: Option<usize>,
        vb: VarBuilder,
    ) -> Result<Self> {
        let policy = PolicyNetwork::from_schema_and_backbone(
            schema,
            hidden_dim,
            action_space,
            backbone_type,
            belief_dim,
            vb.clone(),
        )?;
        let feat_dim = policy.backbone().output_dim();
        let critic = ValueHead::new(feat_dim, vb)?;
        Ok(Self { policy, critic })
    }

    pub fn from_schema(
        schema: ObsSchema,
        hidden_dim: usize,
        action_space: ActionSpace,
        vb: VarBuilder,
    ) -> Result<Self> {
        Self::from_schema_and_backbone(
            schema,
            hidden_dim,
            action_space,
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
        let policy = PolicyNetwork::with_hero_embed_and_backbone(
            state_dim,
            hidden_dim,
            action_space,
            hero_embed_config,
            backbone_type,
            belief_dim,
            vb.clone(),
        )?;
        let feat_dim = policy.backbone().output_dim();
        let critic = ValueHead::new(feat_dim, vb)?;
        Ok(Self { policy, critic })
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
        Self::with_hero_embed_and_backbone(
            state_dim,
            mamba_config.d_model,
            action_space,
            hero_embed_config,
            PolicyBackbone::Mamba,
            belief_dim,
            vb,
        )
    }

    pub fn from_schemas(
        obs_schema: ObsSchema,
        action_schema: ActionSchema,
        hidden_dim: usize,
        backbone_type: PolicyBackbone,
        belief_dim: Option<usize>,
        vb: VarBuilder,
    ) -> Result<Self> {
        let policy = PolicyNetwork::from_schemas(
            obs_schema,
            action_schema,
            hidden_dim,
            backbone_type,
            belief_dim,
            vb.clone(),
        )?;
        let feat_dim = policy.backbone().output_dim();
        let critic = ValueHead::new(feat_dim, vb)?;
        Ok(Self { policy, critic })
    }

    pub fn policy(&self) -> &PolicyNetwork {
        &self.policy
    }

    pub fn policy_mut(&mut self) -> &mut PolicyNetwork {
        &mut self.policy
    }

    pub fn critic(&self) -> &ValueHead {
        &self.critic
    }

    pub fn action_space(&self) -> &ActionSpace {
        self.policy.action_space()
    }

    pub fn feature_extractor(&self) -> &ObsFeatureExtractor {
        self.policy.feature_extractor()
    }

    pub fn schema(&self) -> &ObsSchema {
        self.policy.schema()
    }

    pub fn backbone(&self) -> &Backbone {
        self.policy.backbone()
    }

    pub fn mamba_config(&self) -> Option<&MambaConfig> {
        self.policy.mamba_config()
    }

    pub fn belief_head(&self) -> Option<&BeliefHead> {
        self.policy.belief_head()
    }

    pub fn has_hero_embed(&self) -> bool {
        self.policy.has_hero_embed()
    }

    pub fn forward(&self, state: &Tensor) -> Result<(Tensor, Tensor)> {
        let feat = self.policy.hidden(state)?;
        let actor_out = self.policy.actor_head.forward(&feat)?;
        let values = self.critic.forward(&feat)?;
        Ok((actor_out, values))
    }

    pub fn forward_belief(&self, state: &Tensor) -> Result<Option<(Tensor, Tensor)>> {
        self.policy.forward_belief(state)
    }

    pub fn policy_display_real(
        &self,
        state: &Tensor,
        mask: Option<&[bool]>,
        labels: &[&str],
    ) -> Result<PolicyDisplay> {
        self.policy.policy_display_real(state, mask, labels)
    }

    pub fn select_greedy_action(&self, state: &Tensor, mask: Option<&[bool]>) -> Result<Vec<f32>> {
        self.policy.select_greedy_action(state, mask)
    }

    pub fn select_greedy_action_with_structured_masks(
        &self,
        state: &Tensor,
        structured_mask: Option<&ActionMasks>,
        mask: Option<&[bool]>,
    ) -> Result<Vec<f32>> {
        self.policy
            .select_greedy_action_with_structured_masks(state, structured_mask, mask)
    }

    pub fn get_values(&self, state: &Tensor) -> Result<Vec<f32>> {
        let feat = self.policy.hidden(state)?;
        let values = self.critic.forward(&feat)?;
        values.squeeze(1)?.to_vec1()
    }

    pub fn sample_action(
        &self,
        state: &Tensor,
        mask: Option<&[bool]>,
    ) -> Result<(Vec<f32>, f32, f32)> {
        let feat = self.policy.hidden(state)?;
        let values = self.critic.forward(&feat)?;
        let val_scalar: f32 = values.squeeze(0)?.squeeze(0)?.to_scalar()?;
        let (action, log_prob) = self.policy.sample_action(state, mask)?;
        Ok((action, log_prob, val_scalar))
    }

    pub fn sample_action_with_structured_masks(
        &self,
        state: &Tensor,
        structured_mask: Option<&ActionMasks>,
        mask: Option<&[bool]>,
    ) -> Result<(Vec<f32>, f32, f32)> {
        let feat = self.policy.hidden(state)?;
        let values = self.critic.forward(&feat)?;
        let val_scalar: f32 = values.squeeze(0)?.squeeze(0)?.to_scalar()?;
        let (action, log_prob) =
            self.policy
                .sample_action_with_structured_masks(state, structured_mask, mask)?;
        Ok((action, log_prob, val_scalar))
    }

    pub fn sample_batch(
        &self,
        states: &Tensor,
        masks: Option<&[Option<Vec<bool>>]>,
    ) -> Result<Vec<(Vec<f32>, f32, f32)>> {
        let feat = self.policy.hidden(states)?;
        let values = self.critic.forward(&feat)?;
        let val_vec: Vec<f32> = values.squeeze(1)?.to_vec1()?;
        let act_lps = self.policy.sample_batch(states, masks)?;
        let mut res = Vec::with_capacity(act_lps.len());
        for (i, (act, lp)) in act_lps.into_iter().enumerate() {
            res.push((act, lp, val_vec[i]));
        }
        Ok(res)
    }

    pub fn sample_batch_with_structured_masks(
        &self,
        states: &Tensor,
        structured_masks: Option<&[Option<ActionMasks>]>,
        masks: Option<&[Option<Vec<bool>>]>,
    ) -> Result<Vec<(Vec<f32>, f32, f32)>> {
        let feat = self.policy.hidden(states)?;
        let values = self.critic.forward(&feat)?;
        let val_vec: Vec<f32> = values.squeeze(1)?.to_vec1()?;
        let act_lps =
            self.policy
                .sample_batch_with_structured_masks(states, structured_masks, masks)?;
        let mut res = Vec::with_capacity(act_lps.len());
        for (i, (act, lp)) in act_lps.into_iter().enumerate() {
            res.push((act, lp, val_vec[i]));
        }
        Ok(res)
    }

    pub fn step(
        &self,
        state: &Tensor,
        state_obj: &mut Option<MambaState>,
        mask: Option<&[bool]>,
    ) -> Result<(Vec<f32>, f32, f32)> {
        self.step_with_structured_masks(state, state_obj, None, mask)
    }

    pub fn step_with_structured_masks(
        &self,
        state: &Tensor,
        state_obj: &mut Option<MambaState>,
        structured_mask: Option<&ActionMasks>,
        mask: Option<&[bool]>,
    ) -> Result<(Vec<f32>, f32, f32)> {
        let feat = self.policy.hidden(state)?;
        let values = self.critic.forward(&feat)?;
        let val_scalar: f32 = values.squeeze(0)?.squeeze(0)?.to_scalar()?;
        let (action, log_prob) =
            self.policy
                .step_with_structured_masks(state, state_obj, structured_mask, mask)?;
        Ok((action, log_prob, val_scalar))
    }

    pub fn evaluate_actions(
        &self,
        state: &Tensor,
        actions: &Tensor,
        masks: Option<&Tensor>,
    ) -> Result<(Tensor, Tensor, Tensor)> {
        let feat = if state.rank() == 3 {
            let (b, t, _) = state.dims3()?;
            let feat_3d = self.policy.hidden(state)?;
            feat_3d.reshape((b * t, self.policy.backbone().output_dim()))?
        } else {
            self.policy.hidden(state)?
        };
        let values = self.critic.forward(&feat)?;
        let (log_prob, entropy) = self.policy.evaluate_actions(state, actions, masks)?;
        Ok((log_prob, values.squeeze(1)?, entropy))
    }

    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        Ok(Self {
            policy: self.policy.to_device(device)?,
            critic: self.critic.to_device(device)?,
        })
    }

    pub fn parameter_summary(&self) -> ModelParamSummary {
        let mut out = Vec::new();
        self.policy.collect_params(&mut out);
        self.critic.collect_params(&mut out);
        ModelParamSummary::new(out)
    }

    pub fn print_parameter_summary(&self) {
        self.parameter_summary().print_summary();
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

use std::collections::HashMap;

use lol_rl_protocol::{ActionMasks, ActionNode, ActionSchema};

/// 单个动作分支的输出头
#[derive(Clone)]
pub enum ActionBranchHead {
    Categorical {
        head: Linear,
        num_classes: usize,
        name: String,
    },
    Continuous {
        head: Linear,
        log_std: Tensor,
        dim: usize,
        name: String,
    },
    UnitSelection {
        w_h: Linear,
        w_a: candle_nn::Embedding,
        w_e: Linear,
        v_a: Linear,
        proj_dim: usize,
        num_classes: usize,
        max_units: usize,
        unit_embed_dim: usize,
        name: String,
        obs_entity_name: String,
    },
}

impl ActionBranchHead {
    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        match self {
            Self::Categorical {
                head,
                num_classes,
                name,
            } => {
                let w = head.weight().to_device(device)?;
                let b = head.bias().map(|b| b.to_device(device)).transpose()?;
                Ok(Self::Categorical {
                    head: Linear::new(w, b),
                    num_classes: *num_classes,
                    name: name.clone(),
                })
            }
            Self::Continuous {
                head,
                log_std,
                dim,
                name,
            } => {
                let w = head.weight().to_device(device)?;
                let b = head.bias().map(|b| b.to_device(device)).transpose()?;
                let l_std = log_std.to_device(device)?;
                Ok(Self::Continuous {
                    head: Linear::new(w, b),
                    log_std: l_std,
                    dim: *dim,
                    name: name.clone(),
                })
            }
            Self::UnitSelection {
                w_h,
                w_a,
                w_e,
                v_a,
                proj_dim,
                num_classes,
                max_units,
                unit_embed_dim,
                name,
                obs_entity_name,
            } => {
                let w_h_w = w_h.weight().to_device(device)?;
                let w_h_b = w_h.bias().map(|b| b.to_device(device)).transpose()?;
                let w_a_embeddings = w_a.embeddings().to_device(device)?;
                let w_e_w = w_e.weight().to_device(device)?;
                let w_e_b = w_e.bias().map(|b| b.to_device(device)).transpose()?;
                let v_a_w = v_a.weight().to_device(device)?;
                let v_a_b = v_a.bias().map(|b| b.to_device(device)).transpose()?;
                Ok(Self::UnitSelection {
                    w_h: Linear::new(w_h_w, w_h_b),
                    w_a: candle_nn::Embedding::new(w_a_embeddings, *proj_dim),
                    w_e: Linear::new(w_e_w, w_e_b),
                    v_a: Linear::new(v_a_w, v_a_b),
                    proj_dim: *proj_dim,
                    num_classes: *num_classes,
                    max_units: *max_units,
                    unit_embed_dim: *unit_embed_dim,
                    name: name.clone(),
                    obs_entity_name: obs_entity_name.clone(),
                })
            }
        }
    }
}

/// 基于 ActionSchema 的因式分解多头 Actor
#[derive(Clone)]
pub struct StructuredActionHead {
    branches: Vec<ActionBranchHead>,
    schema: ActionSchema,
}

impl StructuredActionHead {
    pub fn new(schema: ActionSchema, feat_dim: usize, vb: VarBuilder) -> Result<Self> {
        let flat = schema.flat_branches();
        let cat_classes = flat
            .iter()
            .find_map(|node| match node {
                ActionNode::Categorical { num_classes, .. } => Some(*num_classes),
                _ => None,
            })
            .unwrap_or(1);

        let mut branches = Vec::new();
        for node in flat.iter() {
            let branch_vb = vb.pp(format!("action_{}", node.name()));
            match node {
                ActionNode::Categorical {
                    num_classes, name, ..
                } => {
                    let head = candle_nn::linear(feat_dim, *num_classes, branch_vb)?;
                    branches.push(ActionBranchHead::Categorical {
                        head,
                        num_classes: *num_classes,
                        name: name.clone(),
                    });
                }
                ActionNode::Continuous { dim, name, .. } => {
                    let head = candle_nn::linear(feat_dim, *dim, branch_vb.pp("mean"))?;
                    let log_std = branch_vb.get_with_hints(
                        (*dim,),
                        "log_std",
                        candle_nn::Init::Const(0.0),
                    )?;
                    branches.push(ActionBranchHead::Continuous {
                        head,
                        log_std,
                        dim: *dim,
                        name: name.clone(),
                    });
                }
                ActionNode::UnitSelection {
                    max_units,
                    unit_embed_dim,
                    name,
                    obs_entity_name,
                    ..
                } => {
                    let proj_dim = *unit_embed_dim;
                    let w_h = candle_nn::linear(feat_dim, proj_dim, branch_vb.pp("w_h"))?;
                    let w_a = candle_nn::embedding(cat_classes, proj_dim, branch_vb.pp("w_a"))?;
                    let w_e = candle_nn::linear(*unit_embed_dim, proj_dim, branch_vb.pp("w_e"))?;
                    let v_a = candle_nn::linear(proj_dim, 1, branch_vb.pp("v_a"))?;
                    branches.push(ActionBranchHead::UnitSelection {
                        w_h,
                        w_a,
                        w_e,
                        v_a,
                        proj_dim,
                        num_classes: cat_classes,
                        max_units: *max_units,
                        unit_embed_dim: *unit_embed_dim,
                        name: name.clone(),
                        obs_entity_name: obs_entity_name.clone(),
                    });
                }
                ActionNode::Struct { .. } => {
                    unreachable!("Struct should be flattened")
                }
            }
        }
        Ok(Self { branches, schema })
    }

    pub fn schema(&self) -> &ActionSchema {
        &self.schema
    }

    pub fn sample(
        &self,
        feat: &Tensor,
        masks: Option<&ActionMasks>,
        entity_embeds: &HashMap<String, Tensor>,
    ) -> Result<(Vec<f32>, f32)> {
        let mut encoded = Vec::new();
        let mut total_log_prob = 0.0f32;
        let mut rng = rand::rng();
        let mut chosen_action_idx = 0usize;

        for (i, branch) in self.branches.iter().enumerate() {
            let mask_i = masks
                .and_then(|m| m.branch_masks.get(i))
                .and_then(|m| m.as_deref());
            match branch {
                ActionBranchHead::Categorical { head, .. } => {
                    let logits: Vec<f32> = head.forward(feat)?.squeeze(0)?.to_vec1()?;
                    let masked = mask_logits_slice(&logits, mask_i);
                    let (idx, lp) = sample_categorical(&masked);
                    chosen_action_idx = idx;
                    encoded.push(idx as f32);
                    total_log_prob += lp;
                }
                ActionBranchHead::Continuous {
                    head, log_std, dim, ..
                } => {
                    let means: Vec<f32> = head.forward(feat)?.squeeze(0)?.to_vec1()?;
                    let lstd: Vec<f32> = log_std.to_vec1()?;
                    for j in 0..*dim {
                        let std = lstd[j].exp();
                        let a = means[j] + std * sample_gaussian(&mut rng);
                        encoded.push(a);
                        total_log_prob += gaussian_log_prob(means[j], std, a);
                    }
                }
                ActionBranchHead::UnitSelection {
                    w_h,
                    w_a,
                    w_e,
                    v_a,
                    obs_entity_name,
                    ..
                } => {
                    let target_mask: Option<&[bool]> = if let Some(ctm) =
                        masks.and_then(|m| m.conditional_target_masks.as_ref())
                    {
                        ctm.get(chosen_action_idx).map(|v| v.as_slice())
                    } else {
                        mask_i
                    };

                    let embeds = entity_embeds.get(obs_entity_name).ok_or_else(|| {
                        candle_core::Error::Msg(format!(
                            "Missing entity embeds for '{}'",
                            obs_entity_name
                        ))
                    })?;

                    let act_t = Tensor::new(&[chosen_action_idx as u32], feat.device())?;
                    let h_proj = w_h.forward(feat)?.unsqueeze(1)?;
                    let a_proj = w_a.forward(&act_t)?.unsqueeze(1)?;
                    let e_proj = w_e.forward(embeds)?;
                    let ha = (&h_proj + &a_proj)?;
                    let sum = ha.broadcast_add(&e_proj)?;
                    let act = sum.tanh()?;
                    let logits_t = v_a.forward(&act)?.squeeze(2)?;
                    let logits: Vec<f32> = logits_t.squeeze(0)?.to_vec1()?;
                    let masked = mask_logits_slice(&logits, target_mask);
                    let (idx, lp) = sample_categorical(&masked);
                    encoded.push(idx as f32);
                    total_log_prob += lp;
                }
            }
        }
        Ok((encoded, total_log_prob))
    }

    pub fn evaluate(
        &self,
        feat: &Tensor,
        actions: &Tensor,
        masks: Option<&ActionMasks>,
        entity_embeds: &HashMap<String, Tensor>,
        batch_masks: Option<&Tensor>,
    ) -> Result<(Tensor, Tensor)> {
        let n = feat.dim(0)?;
        let mut all_log_probs: Vec<Tensor> = Vec::new();
        let mut all_entropies: Vec<Tensor> = Vec::new();

        let mut cat_action_offset: Option<usize> = None;
        let mut current_off = 0usize;
        for branch in &self.branches {
            match branch {
                ActionBranchHead::Categorical { .. } => {
                    if cat_action_offset.is_none() {
                        cat_action_offset = Some(current_off);
                    }
                    current_off += 1;
                }
                ActionBranchHead::Continuous { dim, .. } => {
                    current_off += *dim;
                }
                ActionBranchHead::UnitSelection { .. } => {
                    current_off += 1;
                }
            }
        }

        let cat_actions = if let Some(cat_off) = cat_action_offset {
            Some(
                actions
                    .narrow(1, cat_off, 1)?
                    .squeeze(1)?
                    .to_dtype(DType::U32)?,
            )
        } else {
            None
        };

        let mut action_offset = 0usize;

        for (_i, branch) in self.branches.iter().enumerate() {
            match branch {
                ActionBranchHead::Categorical {
                    head, num_classes, ..
                } => {
                    let logits = head.forward(feat)?;
                    let masked_logits = if let Some(bm) = batch_masks {
                        if bm.dim(1)? == *num_classes {
                            mask_logits_tensor(&logits, Some(bm))?
                        } else {
                            logits
                        }
                    } else {
                        logits
                    };

                    let log_probs_all = candle_nn::ops::log_softmax(&masked_logits, D::Minus1)?;
                    let probs_all = candle_nn::ops::softmax(&masked_logits, D::Minus1)?;
                    let act = actions
                        .narrow(1, action_offset, 1)?
                        .squeeze(1)?
                        .to_dtype(DType::U32)?;
                    let sel_lp = log_probs_all.gather(&act.unsqueeze(1)?, 1)?.squeeze(1)?;
                    let ent = (probs_all * log_probs_all)?
                        .neg()?
                        .sum_keepdim(D::Minus1)?
                        .squeeze(1)?;
                    all_log_probs.push(sel_lp);
                    all_entropies.push(ent);
                    action_offset += 1;
                }
                ActionBranchHead::Continuous {
                    head, log_std, dim, ..
                } => {
                    let means = head.forward(feat)?;
                    let log_std_b = log_std.broadcast_as((n, *dim))?;
                    let std_b = log_std_b.exp()?;
                    let cont = actions.narrow(1, action_offset, *dim)?;
                    let z = ((&cont - &means)? / &std_b)?;
                    let lp = z
                        .powf(2.0)?
                        .neg()?
                        .affine(0.5, 0.0)?
                        .sub(&log_std_b)?
                        .affine(1.0, -HALF_LN_2PI as f64)?
                        .sum(D::Minus1)?;
                    let ent = log_std_b
                        .affine(1.0, (0.5 + HALF_LN_2PI) as f64)?
                        .sum(D::Minus1)?;
                    all_log_probs.push(lp);
                    all_entropies.push(ent);
                    action_offset += *dim;
                }
                ActionBranchHead::UnitSelection {
                    w_h,
                    w_a,
                    w_e,
                    v_a,
                    max_units,
                    obs_entity_name,
                    ..
                } => {
                    let target_actions = actions
                        .narrow(1, action_offset, 1)?
                        .squeeze(1)?
                        .to_dtype(DType::U32)?;

                    let chosen_acts = if let Some(ref cat_act) = cat_actions {
                        cat_act.clone()
                    } else {
                        Tensor::zeros((n,), DType::U32, feat.device())?
                    };

                    let embeds = entity_embeds.get(obs_entity_name).ok_or_else(|| {
                        candle_core::Error::Msg(format!(
                            "Missing entity embeds for '{}'",
                            obs_entity_name
                        ))
                    })?;

                    let h_proj = w_h.forward(feat)?.unsqueeze(1)?;
                    let a_proj = w_a.forward(&chosen_acts)?.unsqueeze(1)?;
                    let e_proj = w_e.forward(embeds)?;
                    let ha = (&h_proj + &a_proj)?;
                    let sum = ha.broadcast_add(&e_proj)?;
                    let act = sum.tanh()?;
                    let raw_logits = v_a.forward(&act)?.squeeze(2)?;

                    let masked_logits = if let Some(ref ctm) =
                        masks.and_then(|m| m.conditional_target_masks.as_ref())
                    {
                        let chosen_act_vec: Vec<u32> = chosen_acts.to_vec1()?;
                        let mut flat_mask = Vec::with_capacity(n * *max_units);
                        let default_row = vec![true; *max_units];
                        for &a_idx in &chosen_act_vec {
                            let row = ctm.get(a_idx as usize).unwrap_or(&default_row);
                            for &valid in row {
                                flat_mask.push(if valid { 1.0f32 } else { 0.0f32 });
                            }
                        }
                        let mask_tensor =
                            Tensor::from_vec(flat_mask, (n, *max_units), feat.device())?;
                        mask_logits_tensor(&raw_logits, Some(&mask_tensor))?
                    } else if let Some(bm) = batch_masks {
                        if bm.dim(1)? == *max_units {
                            mask_logits_tensor(&raw_logits, Some(bm))?
                        } else {
                            raw_logits
                        }
                    } else {
                        raw_logits
                    };

                    let log_probs_all = candle_nn::ops::log_softmax(&masked_logits, D::Minus1)?;
                    let probs_all = candle_nn::ops::softmax(&masked_logits, D::Minus1)?;
                    let sel_lp = log_probs_all
                        .gather(&target_actions.unsqueeze(1)?, 1)?
                        .squeeze(1)?;
                    let ent = (probs_all * log_probs_all)?
                        .neg()?
                        .sum_keepdim(D::Minus1)?
                        .squeeze(1)?;
                    all_log_probs.push(sel_lp);
                    all_entropies.push(ent);
                    action_offset += 1;
                }
            }
        }

        let total_lp = sum_tensors(&all_log_probs)?;
        let total_ent = sum_tensors(&all_entropies)?;
        Ok((total_lp, total_ent))
    }

    /// 提取多头因式分解动作分布的展示结构（用于可视化调试）
    pub fn policy_display(
        &self,
        feat: &Tensor,
        entity_embeds: &HashMap<String, Tensor>,
        masks: Option<&ActionMasks>,
    ) -> Result<Vec<ActionBranchDisplay>> {
        let flat = self.schema.flat_branches();
        let mut displays = Vec::with_capacity(self.branches.len());

        for (i, branch) in self.branches.iter().enumerate() {
            let mask_i = masks
                .and_then(|m| m.branch_masks.get(i))
                .and_then(|m| m.as_deref());
            let node = flat.get(i);
            match branch {
                ActionBranchHead::Categorical {
                    head,
                    num_classes,
                    name,
                } => {
                    let logits: Vec<f32> = head.forward(feat)?.squeeze(0)?.to_vec1()?;
                    let raw_probs = softmax_slice(&logits);
                    let masked = mask_logits_slice(&logits, mask_i);
                    let probs = softmax_slice(&masked);
                    let labels = if let Some(ActionNode::Categorical { labels, .. }) = node {
                        labels.clone()
                    } else {
                        (0..*num_classes).map(|c| format!("Class {c}")).collect()
                    };
                    let items = (0..*num_classes)
                        .map(|c| {
                            let label = labels
                                .get(c)
                                .cloned()
                                .unwrap_or_else(|| format!("Class {c}"));
                            let is_masked = mask_i
                                .map(|m| !m.get(c).copied().unwrap_or(true))
                                .unwrap_or(false);
                            PolicyItem {
                                action_id: c,
                                action: label,
                                prob: probs.get(c).copied().unwrap_or(0.0),
                                raw_prob: raw_probs.get(c).copied().unwrap_or(0.0),
                                is_masked,
                            }
                        })
                        .collect();
                    displays.push(ActionBranchDisplay::Categorical {
                        name: name.clone(),
                        items,
                    });
                }
                ActionBranchHead::Continuous {
                    head, dim, name, ..
                } => {
                    let means: Vec<f32> = head.forward(feat)?.squeeze(0)?.to_vec1()?;
                    let labels = (0..*dim).map(|j| format!("{}[{}]", name, j)).collect();
                    displays.push(ActionBranchDisplay::Continuous {
                        name: name.clone(),
                        means,
                        labels,
                    });
                }
                ActionBranchHead::UnitSelection {
                    w_h,
                    w_a,
                    w_e,
                    v_a,
                    max_units,
                    name,
                    obs_entity_name,
                    ..
                } => {
                    let target_mask: Option<&[bool]> = if let Some(ctm) =
                        masks.and_then(|m| m.conditional_target_masks.as_ref())
                    {
                        ctm.first().map(|v| v.as_slice())
                    } else {
                        mask_i
                    };

                    let items = if let Some(embeds) = entity_embeds.get(obs_entity_name) {
                        let act_0 = Tensor::new(&[0u32], feat.device())?;
                        let h_proj = w_h.forward(feat)?.unsqueeze(1)?;
                        let a_proj = w_a.forward(&act_0)?.unsqueeze(1)?;
                        let e_proj = w_e.forward(embeds)?;
                        let ha = (&h_proj + &a_proj)?;
                        let sum = ha.broadcast_add(&e_proj)?;
                        let act = sum.tanh()?;
                        let logits_t = v_a.forward(&act)?.squeeze(2)?;
                        let logits: Vec<f32> = logits_t.squeeze(0)?.to_vec1()?;
                        let raw_probs = softmax_slice(&logits);
                        let masked = mask_logits_slice(&logits, target_mask);
                        let probs = softmax_slice(&masked);
                        (0..*max_units)
                            .map(|j| {
                                let label = if j == 0 {
                                    format!("Slot {j} (目标英雄)")
                                } else {
                                    format!("Slot {j} (小兵/单位)")
                                };
                                let is_masked = target_mask
                                    .map(|m| !m.get(j).copied().unwrap_or(true))
                                    .unwrap_or(false);
                                PolicyItem {
                                    action_id: j,
                                    action: label,
                                    prob: probs.get(j).copied().unwrap_or(0.0),
                                    raw_prob: raw_probs.get(j).copied().unwrap_or(0.0),
                                    is_masked,
                                }
                            })
                            .collect()
                    } else {
                        Vec::new()
                    };
                    displays.push(ActionBranchDisplay::UnitSelection {
                        name: name.clone(),
                        obs_entity_name: obs_entity_name.clone(),
                        items,
                    });
                }
            }
        }

        Ok(displays)
    }

    pub fn to_device(&self, device: &candle_core::Device) -> Result<Self> {
        let mut moved_branches = Vec::with_capacity(self.branches.len());
        for branch in &self.branches {
            moved_branches.push(branch.to_device(device)?);
        }
        Ok(Self {
            branches: moved_branches,
            schema: self.schema.clone(),
        })
    }

    /// 收集多头输出层的所有参数张量信息
    pub fn collect_params(&self, out: &mut Vec<LayerParamInfo>) {
        for branch in &self.branches {
            match branch {
                ActionBranchHead::Categorical { head, name, .. } => {
                    out.push(LayerParamInfo::new(
                        "Actor 策略头",
                        format!("action_{}.weight", name),
                        head.weight().dims(),
                    ));
                    if let Some(b) = head.bias() {
                        out.push(LayerParamInfo::new(
                            "Actor 策略头",
                            format!("action_{}.bias", name),
                            b.dims(),
                        ));
                    }
                }
                ActionBranchHead::Continuous {
                    head,
                    log_std,
                    name,
                    ..
                } => {
                    out.push(LayerParamInfo::new(
                        "Actor 策略头",
                        format!("action_{}.mean.weight", name),
                        head.weight().dims(),
                    ));
                    if let Some(b) = head.bias() {
                        out.push(LayerParamInfo::new(
                            "Actor 策略头",
                            format!("action_{}.mean.bias", name),
                            b.dims(),
                        ));
                    }
                    out.push(LayerParamInfo::new(
                        "Actor 策略头",
                        format!("action_{}.log_std", name),
                        log_std.dims(),
                    ));
                }
                ActionBranchHead::UnitSelection {
                    w_h,
                    w_a,
                    w_e,
                    v_a,
                    name,
                    ..
                } => {
                    out.push(LayerParamInfo::new(
                        "Actor 策略头",
                        format!("action_{}.w_h.weight", name),
                        w_h.weight().dims(),
                    ));
                    if let Some(b) = w_h.bias() {
                        out.push(LayerParamInfo::new(
                            "Actor 策略头",
                            format!("action_{}.w_h.bias", name),
                            b.dims(),
                        ));
                    }
                    out.push(LayerParamInfo::new(
                        "Actor 策略头",
                        format!("action_{}.w_a.embeddings", name),
                        w_a.embeddings().dims(),
                    ));
                    out.push(LayerParamInfo::new(
                        "Actor 策略头",
                        format!("action_{}.w_e.weight", name),
                        w_e.weight().dims(),
                    ));
                    if let Some(b) = w_e.bias() {
                        out.push(LayerParamInfo::new(
                            "Actor 策略头",
                            format!("action_{}.w_e.bias", name),
                            b.dims(),
                        ));
                    }
                    out.push(LayerParamInfo::new(
                        "Actor 策略头",
                        format!("action_{}.v_a.weight", name),
                        v_a.weight().dims(),
                    ));
                    if let Some(b) = v_a.bias() {
                        out.push(LayerParamInfo::new(
                            "Actor 策略头",
                            format!("action_{}.v_a.bias", name),
                            b.dims(),
                        ));
                    }
                }
            }
        }
    }
}

// ── 参数量明细统计与格式化 (K / M 单位) ──────────────────────────────────────

/// 单个参数张量的元信息
#[derive(Debug, Clone)]
pub struct LayerParamInfo {
    /// 模块类别：如 "Obs 特征提取", "Backbone (Mamba)", "Actor 策略头", "Critic 价值头", "Belief 信念头"
    pub category: &'static str,
    /// 参数/层名称：如 "proj_in.weight", "fc1.weight", "critic_head.bias"
    pub name: String,
    /// 张量维度：如 [64, 115]
    pub shape: Vec<usize>,
    /// 参数量数值
    pub count: usize,
}

impl LayerParamInfo {
    pub fn new(category: &'static str, name: impl Into<String>, shape: &[usize]) -> Self {
        let shape_vec = shape.to_vec();
        let count = shape_vec.iter().product();
        Self {
            category,
            name: name.into(),
            shape: shape_vec,
            count,
        }
    }
}

/// 格式化参数量，以 K 或 M 为单位
pub fn format_param_k_m(count: usize) -> String {
    if count >= 1_000_000 {
        format!("{:.3} M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.2} K", count as f64 / 1_000.0)
    } else {
        format!("{:.2} K", count as f64 / 1_000.0)
    }
}

/// 网络模型参数量统计汇总
#[derive(Debug, Clone)]
pub struct ModelParamSummary {
    pub layers: Vec<LayerParamInfo>,
    pub total_params: usize,
}

impl ModelParamSummary {
    pub fn new(layers: Vec<LayerParamInfo>) -> Self {
        let total_params = layers.iter().map(|l| l.count).sum();
        Self {
            layers,
            total_params,
        }
    }

    /// 按分类汇总
    pub fn category_totals(&self) -> Vec<(&'static str, usize)> {
        let mut map: std::collections::BTreeMap<&'static str, usize> =
            std::collections::BTreeMap::new();
        let mut ordered_cats = Vec::new();
        for layer in &self.layers {
            if !map.contains_key(layer.category) {
                ordered_cats.push(layer.category);
            }
            *map.entry(layer.category).or_insert(0) += layer.count;
        }
        ordered_cats
            .into_iter()
            .map(|cat| (cat, map[cat]))
            .collect()
    }

    /// 格式化为美观的表格字符串
    pub fn format_table(&self) -> String {
        let mut s = String::new();
        s.push_str("\n========================================================================================\n");
        s.push_str("🧠 [RL 策略与价值网络参数量明细 (Model Parameter Breakdown)]\n");
        s.push_str("----------------------------------------------------------------------------------------\n");
        s.push_str(&format!(
            "{:<18} {:<34} {:<18} {:>10} {:>10}\n",
            "模块分类", "参数层名称", "形状 (Shape)", "参数量", "规格 (K/M)"
        ));
        s.push_str("----------------------------------------------------------------------------------------\n");

        for layer in &self.layers {
            let shape_str = format!("{:?}", layer.shape);
            s.push_str(&format!(
                "{:<18} {:<34} {:<18} {:>10} {:>10}\n",
                layer.category,
                layer.name,
                shape_str,
                layer.count,
                format_param_k_m(layer.count)
            ));
        }

        s.push_str("----------------------------------------------------------------------------------------\n");
        s.push_str("📊 [模块分类汇总]:\n");
        for (cat, count) in self.category_totals() {
            s.push_str(&format!(
                "  ├─ {:<18}: {:>8} 参数 ({:>8})\n",
                cat,
                count,
                format_param_k_m(count)
            ));
        }
        s.push_str(&format!(
            "  └─ 👉 总可训练参数量 : {:>8} 参数 ({:>8} / {:.4} M)\n",
            self.total_params,
            format_param_k_m(self.total_params),
            self.total_params as f64 / 1_000_000.0
        ));
        s.push_str("========================================================================================\n");
        s
    }

    pub fn print_summary(&self) {
        println!("{}", self.format_table());
        tracing::info!("{}", self.format_table());
    }
}

fn sum_tensors(tensors: &[Tensor]) -> Result<Tensor> {
    let mut result = tensors[0].clone();
    for t in &tensors[1..] {
        result = (&result + t)?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use candle_nn::VarMap;

    use super::*;

    #[test]
    fn test_structured_action_head_sample() -> Result<()> {
        let device = candle_core::Device::Cpu;
        let schema = ActionSchema::new(vec![
            ActionNode::continuous("offset", 2),
            ActionNode::categorical(
                "action_type",
                vec!["NoOp", "Move", "Attack", "CastQ"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            ),
        ]);
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let head = StructuredActionHead::new(schema, 64, vb)?;
        let feat = Tensor::zeros((1, 64), DType::F32, &device)?;
        let (encoded, _lp) = head.sample(&feat, None, &HashMap::new())?;
        assert_eq!(encoded.len(), 3);
        Ok(())
    }

    #[test]
    fn test_structured_action_head_with_unit_selection() -> Result<()> {
        let device = candle_core::Device::Cpu;
        let schema = ActionSchema::new(vec![
            ActionNode::categorical(
                "action_type",
                vec!["Attack", "Move"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            ),
            ActionNode::unit_selection("target", 8, 16, "visible_units"),
        ]);
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let head = StructuredActionHead::new(schema, 32, vb)?;
        let feat = Tensor::zeros((1, 32), DType::F32, &device)?;
        let mut entity_embeds = HashMap::new();
        entity_embeds.insert(
            "visible_units".to_string(),
            Tensor::randn(0.0f32, 1.0, (1, 8, 16), &device)?,
        );
        let (encoded, _lp) = head.sample(&feat, None, &entity_embeds)?;
        assert_eq!(encoded.len(), 2);
        assert!(encoded[1] >= 0.0 && encoded[1] < 8.0);
        Ok(())
    }

    #[test]
    fn test_obs_schema_feature_extractor_2d_and_3d() -> Result<()> {
        let device = candle_core::Device::Cpu;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let schema = ObsSchema::new(vec![
            ObsNode::categorical("hero", 4, 8),
            ObsNode::scalar("hp_pct", 0.0, 1.0),
            ObsNode::vector("spatial", 2),
            ObsNode::repeated(
                "modifiers",
                2,
                ObsNode::structure(
                    "slot",
                    vec![
                        ObsNode::categorical("name", 8, 4),
                        ObsNode::scalar("dur", 0.0, 1.0),
                    ],
                ),
                EntityEncoderSpec::SharedMlpFlatten {
                    hidden_dims: vec![16],
                },
            ),
        ]);

        // Raw dim: 1 + 1 + 2 + 2 * (1 + 1) = 8
        // Encoded dim: 8 + 1 + 2 + 2 * 16 = 43
        assert_eq!(schema.raw_dim(), 8);
        assert_eq!(schema.encoded_dim(), 43);

        let extractor = ObsFeatureExtractor::new(schema.clone(), vb.pp("test_ext"))?;

        // 2D forward: (batch=3, raw_dim=8)
        let raw_2d = Tensor::zeros((3, 8), DType::F32, &device)?;
        let out_2d = extractor.forward(&raw_2d)?;
        assert_eq!(out_2d.dims(), &[3, 43]);

        // 3D forward: (batch=2, seq_len=5, raw_dim=8)
        let raw_3d = Tensor::zeros((2, 5, 8), DType::F32, &device)?;
        let out_3d = extractor.forward(&raw_3d)?;
        assert_eq!(out_3d.dims(), &[2, 5, 43]);

        // ActorCritic integration
        let ac = ActorCritic::from_schema(schema, 64, ActionSpace::Discrete(8), vb.pp("test_ac"))?;
        let (logits_2d, vals_2d) = ac.forward(&raw_2d)?;
        assert_eq!(logits_2d.dims(), &[3, 8]);
        assert_eq!(vals_2d.dims(), &[3, 1]);

        let (logits_3d, vals_3d) = ac.forward(&raw_3d)?;
        assert_eq!(logits_3d.dims(), &[2, 5, 8]);
        assert_eq!(vals_3d.dims(), &[2, 5, 1]);

        Ok(())
    }

    #[test]
    fn test_parameter_summary_output() -> Result<()> {
        let device = candle_core::Device::Cpu;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let obs_schema = ObsSchema::new(vec![
            ObsNode::categorical("hero", 4, 12),
            ObsNode::vector("spatial", 3),
            ObsNode::repeated(
                "visible_units",
                7,
                ObsNode::structure(
                    "unit",
                    vec![
                        ObsNode::categorical("unit_type", 6, 8),
                        ObsNode::vector("rel_pos", 2),
                        ObsNode::scalar("hp_pct", 0.0, 1.0),
                    ],
                ),
                EntityEncoderSpec::SharedMlpPool {
                    hidden_dims: vec![32, 16],
                    pool_type: PoolType::Max,
                },
            ),
        ]);

        let action_schema = ActionSchema::new(vec![
            ActionNode::continuous("offset", 2),
            ActionNode::categorical(
                "action_type",
                vec!["NoOp", "Move", "Attack", "CastQ"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            ),
            ActionNode::unit_selection("target", 7, 16, "visible_units"),
        ]);

        let ac = ActorCritic::from_schemas(
            obs_schema,
            action_schema,
            64,
            PolicyBackbone::Mamba,
            None,
            vb,
        )?;

        let summary = ac.parameter_summary();
        assert!(summary.total_params > 0);
        summary.print_summary();

        Ok(())
    }

    #[test]
    fn test_structured_action_head_conditional_target_masking() -> Result<()> {
        let device = candle_core::Device::Cpu;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let schema = ActionSchema::new(vec![
            ActionNode::categorical(
                "action_type",
                vec!["NoOp", "Move", "Attack", "CastQ"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            ),
            ActionNode::unit_selection("target", 2, 16, "visible_units"),
        ]);

        let head = StructuredActionHead::new(schema, 32, vb)?;
        let feat = Tensor::zeros((1, 32), DType::F32, &device)?;

        let mut entity_embeds = HashMap::new();
        entity_embeds.insert(
            "visible_units".to_string(),
            Tensor::randn(0.0f32, 1.0, (1, 2, 16), &device)?,
        );

        // 动作 0 (NoOp) / 1 (Move): 两个槽位均允许
        // 动作 2 (Attack) / 3 (CastQ): 仅允许槽位 0 (敌方)，槽位 1 (友方) 禁用
        let cond_masks = vec![
            vec![true, true],  // 0: NoOp
            vec![true, true],  // 1: Move
            vec![true, false], // 2: Attack (友方 Slot 1 禁用)
            vec![true, false], // 3: CastQ (友方 Slot 1 禁用)
        ];

        // 1. 强制选择 action_type = 2 (Attack) 时测试目标采样
        let masks_attack = ActionMasks::with_conditional_target_masks(
            vec![
                Some(vec![false, false, true, false]), // 强制选 Attack
                Some(vec![true, true]),
            ],
            cond_masks.clone(),
        );

        for _ in 0..20 {
            let (encoded, _lp) = head.sample(&feat, Some(&masks_attack), &entity_embeds)?;
            assert_eq!(encoded.len(), 2);
            assert_eq!(encoded[0], 2.0, "动作类型应强制为 2 (Attack)");
            assert_eq!(
                encoded[1], 0.0,
                "在 Attack 动作下，目标必须选择合法敌军 Slot 0，决不能选中友军 Slot 1"
            );
        }

        // 2. 测试 evaluate 条件评估对齐
        // Sample A: action_type=2 (Attack), target=0 (敌军) -> 合法
        // Sample B: action_type=1 (Move), target=1 (友军) -> 合法
        let actions = Tensor::from_vec(vec![2.0f32, 0.0, 1.0, 1.0], (2, 2), &device)?;
        let feat_2 = Tensor::zeros((2, 32), DType::F32, &device)?;
        let mut entity_embeds_2 = HashMap::new();
        entity_embeds_2.insert(
            "visible_units".to_string(),
            Tensor::randn(0.0f32, 1.0, (2, 2, 16), &device)?,
        );

        let masks_all = ActionMasks::with_conditional_target_masks(
            vec![Some(vec![true, true, true, true]), Some(vec![true, true])],
            cond_masks,
        );

        let (total_lp, total_ent) =
            head.evaluate(&feat_2, &actions, Some(&masks_all), &entity_embeds_2, None)?;

        assert_eq!(total_lp.dims(), &[2]);
        assert_eq!(total_ent.dims(), &[2]);

        Ok(())
    }
}
