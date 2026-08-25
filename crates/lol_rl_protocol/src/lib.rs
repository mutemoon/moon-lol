use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub const DEFAULT_RL_SERVER_ADDR: &str = "127.0.0.1:8765";

/// 强化学习动作空间描述，供训练/可视化循环区分离散与连续策略。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActionSpace {
    /// 纯离散分类，动作数 n（legacy 环境）。
    Discrete(usize),
    /// 纯连续高斯，维度 d。
    Continuous(usize),
    /// 混合：连续高斯 d 维 + 一个离散分类 k 类。
    Hybrid {
        continuous_dims: usize,
        discrete_classes: usize,
    },
}

impl ActionSpace {
    /// Actor 头输出维度：Discrete(n)=n，Continuous(d)=d，Hybrid=d+k。
    pub fn actor_head_dim(&self) -> usize {
        match self {
            Self::Discrete(n) => *n,
            Self::Continuous(d) => *d,
            Self::Hybrid {
                continuous_dims,
                discrete_classes,
            } => continuous_dims + discrete_classes,
        }
    }

    /// Rollout 缓冲区中单个动作的扁平编码长度：
    /// Discrete=1（分类索引），Continuous=d，Hybrid=d+1（末位为攻击分类索引）。
    pub fn encoding_dim(&self) -> usize {
        match self {
            Self::Discrete(_) => 1,
            Self::Continuous(d) => *d,
            Self::Hybrid {
                continuous_dims, ..
            } => continuous_dims + 1,
        }
    }
}

/// 结构化奖励表达式 AST
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RewardExpr {
    Constant(f32),
    Variable(String),
    Add(Box<RewardExpr>, Box<RewardExpr>),
    Sub(Box<RewardExpr>, Box<RewardExpr>),
    Mul(Box<RewardExpr>, Box<RewardExpr>),
    IfElse {
        cond: Box<RewardExpr>,
        then_branch: Box<RewardExpr>,
        else_branch: Box<RewardExpr>,
    },
    Gt(Box<RewardExpr>, Box<RewardExpr>),
    Max(Box<RewardExpr>, Box<RewardExpr>),
    Min(Box<RewardExpr>, Box<RewardExpr>),
    Exp(Box<RewardExpr>),
}

impl RewardExpr {
    /// 在给定的环境变量上下文中对表达式求值
    pub fn eval(&self, vars: &HashMap<String, f32>) -> f32 {
        match self {
            Self::Constant(c) => *c,
            Self::Variable(name) => vars.get(name).copied().unwrap_or(0.0),
            Self::Add(a, b) => a.eval(vars) + b.eval(vars),
            Self::Sub(a, b) => a.eval(vars) - b.eval(vars),
            Self::Mul(a, b) => a.eval(vars) * b.eval(vars),
            Self::IfElse {
                cond,
                then_branch,
                else_branch,
            } => {
                if cond.eval(vars) > 0.0 {
                    then_branch.eval(vars)
                } else {
                    else_branch.eval(vars)
                }
            }
            Self::Gt(a, b) => {
                if a.eval(vars) > b.eval(vars) {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Max(a, b) => a.eval(vars).max(b.eval(vars)),
            Self::Min(a, b) => a.eval(vars).min(b.eval(vars)),
            Self::Exp(a) => a.eval(vars).exp(),
        }
    }

    /// 转换为数学展示字符串，如 "80.0 × is_vital_break"
    pub fn to_display_string(&self) -> String {
        match self {
            Self::Constant(c) => {
                if c.fract() == 0.0 {
                    format!("{:.0}", c)
                } else {
                    format!("{:.2}", c)
                }
            }
            Self::Variable(v) => v.clone(),
            Self::Add(a, b) => format!("({} + {})", a.to_display_string(), b.to_display_string()),
            Self::Sub(a, b) => format!("({} - {})", a.to_display_string(), b.to_display_string()),
            Self::Mul(a, b) => format!("{} × {}", a.to_display_string(), b.to_display_string()),
            Self::IfElse {
                cond,
                then_branch,
                else_branch,
            } => {
                format!(
                    "if {} then {} else {}",
                    cond.to_display_string(),
                    then_branch.to_display_string(),
                    else_branch.to_display_string()
                )
            }
            Self::Gt(a, b) => format!("({} > {})", a.to_display_string(), b.to_display_string()),
            Self::Max(a, b) => format!("max({}, {})", a.to_display_string(), b.to_display_string()),
            Self::Min(a, b) => format!("min({}, {})", a.to_display_string(), b.to_display_string()),
            Self::Exp(a) => format!("exp({})", a.to_display_string()),
        }
    }

    /// 转换为 LaTeX 符号公式，如 `0.02 \cdot \mathbb{1}_{\text{newly aligned}}`
    pub fn to_latex(&self) -> String {
        self.to_latex_inner(None)
    }

    /// 转换为代入真实变量值后的 LaTeX 公式，变量叶节点替换为其数值
    pub fn to_latex_substituted(&self, vars: &HashMap<String, f32>) -> String {
        self.to_latex_inner(Some(vars))
    }

    fn to_latex_inner(&self, vars: Option<&HashMap<String, f32>>) -> String {
        match self {
            Self::Constant(c) => fmt_math_num(*c),
            Self::Variable(name) => match vars {
                Some(vars) => fmt_math_num(vars.get(name).copied().unwrap_or(0.0)),
                None => format!(r"\mathbb{{1}}_{{\text{{{}}}}}", latex_var_text(name)),
            },
            Self::Add(a, b) => format!("{} + {}", a.to_latex_inner(vars), b.to_latex_inner(vars)),
            Self::Sub(a, b) => format!("{} - {}", a.to_latex_inner(vars), b.to_latex_inner(vars)),
            Self::Mul(a, b) => format!(
                r"{} \cdot {}",
                a.to_latex_inner(vars),
                b.to_latex_inner(vars)
            ),
            Self::IfElse {
                cond,
                then_branch,
                else_branch,
            } => format!(
                r"\begin{{cases}} {} & \text{{if }} {} \\ {} & \text{{otherwise}} \end{{cases}}",
                then_branch.to_latex_inner(vars),
                cond.to_latex_inner(vars),
                else_branch.to_latex_inner(vars)
            ),
            Self::Gt(a, b) => format!("{} > {}", a.to_latex_inner(vars), b.to_latex_inner(vars)),
            Self::Max(a, b) => {
                format!(
                    r"\max({}, {})",
                    a.to_latex_inner(vars),
                    b.to_latex_inner(vars)
                )
            }
            Self::Min(a, b) => {
                format!(
                    r"\min({}, {})",
                    a.to_latex_inner(vars),
                    b.to_latex_inner(vars)
                )
            }
            Self::Exp(a) => {
                format!(r"\exp\left({}\right)", a.to_latex_inner(vars))
            }
        }
    }
}

/// 把数值格式化成干净的 LaTeX 数字（`2.0`→`2`、`-0.002`→`-0.002`）。
fn fmt_math_num(v: f32) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    let s = format!("{v:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

/// 变量名转可读的指示函数下标：`is_newly_aligned` → `newly aligned`。
fn latex_var_text(name: &str) -> String {
    let stripped = name.strip_prefix("is_").unwrap_or(name);
    stripped.replace('_', " ")
}

/// 用 ` + `/` - ` 拼接各项，避免出现 `+ -0.02` 这类连号。
fn join_with_signs(parts: &[String]) -> String {
    let mut out = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            out.push_str(part);
        } else if let Some(rest) = part.strip_prefix('-') {
            out.push_str(" - ");
            out.push_str(rest);
        } else {
            out.push_str(" + ");
            out.push_str(part);
        }
    }
    out
}

/// 单项奖励定义
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RewardTermSpec {
    pub id: String,
    pub label: String,
    pub expr: RewardExpr,
}

impl RewardTermSpec {
    pub fn new(id: impl Into<String>, label: impl Into<String>, expr: RewardExpr) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            expr,
        }
    }

    pub fn eval(&self, vars: &HashMap<String, f32>) -> f32 {
        self.expr.eval(vars)
    }
}

/// 统一的环境奖励公式规范
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct RewardFormulaSpec {
    pub name: String,
    pub terms: Vec<RewardTermSpec>,
}

impl RewardFormulaSpec {
    /// 依据结构化表达式计算总奖励与分解项
    pub fn compute(&self, vars: &HashMap<String, f32>) -> (f32, Vec<RewardItem>) {
        let mut total = 0.0;
        let mut items = Vec::with_capacity(self.terms.len());
        for term in &self.terms {
            let val = term.eval(vars);
            total += val;
            items.push(RewardItem {
                name: term.label.clone(),
                value: val,
            });
        }
        (total, items)
    }

    /// 符号版总公式：`R = t_1 + t_2 + ...`
    pub fn to_latex(&self) -> String {
        let parts: Vec<String> = self.terms.iter().map(|t| t.expr.to_latex()).collect();
        format!("R = {}", join_with_signs(&parts))
    }

    /// 代入真实变量值后的总公式：`R = t_1 + t_2 + ... = total`
    pub fn to_latex_substituted(&self, vars: &HashMap<String, f32>) -> String {
        let parts: Vec<String> = self
            .terms
            .iter()
            .map(|t| t.expr.to_latex_substituted(vars))
            .collect();
        let total = self.compute(vars).0;
        format!("R = {} = {}", join_with_signs(&parts), fmt_math_num(total))
    }
}

// ── 观测空间 AST 声明式结构体系 (Obs AST) ──────────────────────────────────

fn default_max_one() -> f32 {
    1.0
}

/// 实体聚合池化类型
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolType {
    Max,
    Mean,
    Sum,
}

/// 重复实体集合 (M x N) 的网络编码层规范
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EntityEncoderSpec {
    /// 共享权重 MLP + 槽位展平 Flatten（保留固定槽位/顺序关系）
    SharedMlpFlatten { hidden_dims: Vec<usize> },
    /// 共享权重 MLP + 置换不变性池化 (Max / Mean / Sum)（适用于无序或变长集合）
    SharedMlpPool {
        hidden_dims: Vec<usize>,
        pool_type: PoolType,
    },
    /// 直接透传展平，不额外经过 Entity MLP
    PassThrough,
}

/// 观测空间 AST 节点定义
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ObsNode {
    /// 连续标量特征（如 HP%, 距离, 倒计时等，占 1 个 float 输入）
    Scalar {
        name: String,
        #[serde(default)]
        min: f32,
        #[serde(default = "default_max_one")]
        max: f32,
    },
    /// 连续向量特征（如 相对坐标 [x, z], 技能冷却数组等，占 dim 个 float 输入）
    Vector { name: String, dim: usize },
    /// 离散/类别特征（映射到 Embedding 表，占 1 个 float 类别索引输入）
    Categorical {
        name: String,
        num_classes: usize,
        embed_dim: usize,
    },
    /// 命名复合结构体（包含多个子节点）
    Struct { name: String, fields: Vec<ObsNode> },
    /// 矩阵/重复实体列表 (M x N)
    Repeated {
        name: String,
        max_count: usize,
        item: Box<ObsNode>,
        encoder: EntityEncoderSpec,
    },
}

impl ObsNode {
    pub fn scalar(name: impl Into<String>, min: f32, max: f32) -> Self {
        Self::Scalar {
            name: name.into(),
            min,
            max,
        }
    }

    pub fn vector(name: impl Into<String>, dim: usize) -> Self {
        Self::Vector {
            name: name.into(),
            dim,
        }
    }

    pub fn categorical(name: impl Into<String>, num_classes: usize, embed_dim: usize) -> Self {
        Self::Categorical {
            name: name.into(),
            num_classes,
            embed_dim,
        }
    }

    pub fn structure(name: impl Into<String>, fields: Vec<ObsNode>) -> Self {
        Self::Struct {
            name: name.into(),
            fields,
        }
    }

    pub fn repeated(
        name: impl Into<String>,
        max_count: usize,
        item: ObsNode,
        encoder: EntityEncoderSpec,
    ) -> Self {
        Self::Repeated {
            name: name.into(),
            max_count,
            item: Box::new(item),
            encoder,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Scalar { name, .. } => name,
            Self::Vector { name, .. } => name,
            Self::Categorical { name, .. } => name,
            Self::Struct { name, .. } => name,
            Self::Repeated { name, .. } => name,
        }
    }

    /// 计算在原始扁平输入向量（raw float buffer）中占用的维度大小
    pub fn raw_dim(&self) -> usize {
        match self {
            Self::Scalar { .. } => 1,
            Self::Vector { dim, .. } => *dim,
            Self::Categorical { .. } => 1,
            Self::Struct { fields, .. } => fields.iter().map(|f| f.raw_dim()).sum(),
            Self::Repeated {
                max_count, item, ..
            } => *max_count * item.raw_dim(),
        }
    }

    /// 计算单个实体在 Embedding 映射后、进入 Entity MLP 之前的稠密特征维度
    pub fn embedded_item_dim(&self) -> usize {
        match self {
            Self::Scalar { .. } => 1,
            Self::Vector { dim, .. } => *dim,
            Self::Categorical { embed_dim, .. } => *embed_dim,
            Self::Struct { fields, .. } => fields.iter().map(|f| f.embedded_item_dim()).sum(),
            Self::Repeated { .. } => self.encoded_dim(),
        }
    }

    /// 计算经过特征提取（Embedding + Entity MLP + Pooling/Flatten）后输出给主干 Policy 网络的最终特征维度
    pub fn encoded_dim(&self) -> usize {
        match self {
            Self::Scalar { .. } => 1,
            Self::Vector { dim, .. } => *dim,
            Self::Categorical { embed_dim, .. } => *embed_dim,
            Self::Struct { fields, .. } => fields.iter().map(|f| f.encoded_dim()).sum(),
            Self::Repeated {
                max_count,
                item,
                encoder,
                ..
            } => {
                let item_in_dim = item.embedded_item_dim();
                match encoder {
                    EntityEncoderSpec::SharedMlpFlatten { hidden_dims } => {
                        let item_out_dim = hidden_dims.last().copied().unwrap_or(item_in_dim);
                        *max_count * item_out_dim
                    }
                    EntityEncoderSpec::SharedMlpPool { hidden_dims, .. } => {
                        hidden_dims.last().copied().unwrap_or(item_in_dim)
                    }
                    EntityEncoderSpec::PassThrough => *max_count * item_in_dim,
                }
            }
        }
    }

    /// 自动生成每一个原始输入 float 的说明标签路径
    pub fn to_dim_labels(&self) -> Vec<String> {
        match self {
            Self::Scalar { name, .. } => vec![name.clone()],
            Self::Vector { name, dim } => {
                if *dim == 1 {
                    vec![name.clone()]
                } else {
                    (0..*dim).map(|i| format!("{}[{}]", name, i)).collect()
                }
            }
            Self::Categorical { name, .. } => vec![format!("{}_id", name)],
            Self::Struct { name, fields } => fields
                .iter()
                .flat_map(|f| {
                    f.to_dim_labels()
                        .into_iter()
                        .map(|l| format!("{}.{}", name, l))
                })
                .collect(),
            Self::Repeated {
                name,
                max_count,
                item,
                ..
            } => {
                let item_labels = item.to_dim_labels();
                (0..*max_count)
                    .flat_map(|i| {
                        item_labels
                            .iter()
                            .map(move |l| format!("{}[{}].{}", name, i, l))
                    })
                    .collect()
            }
        }
    }

    /// 将扁平的原始浮点切片按照当前节点定义解析为结构化数据节点
    pub fn decode_value(&self, slice: &[f32]) -> ObsValueNode {
        match self {
            Self::Scalar { name, .. } => ObsValueNode::Scalar {
                name: name.clone(),
                value: slice.first().copied().unwrap_or(0.0),
            },
            Self::Vector { name, dim } => ObsValueNode::Vector {
                name: name.clone(),
                values: slice[..(*dim).min(slice.len())].to_vec(),
            },
            Self::Categorical { name, .. } => {
                let raw_value = slice.first().copied().unwrap_or(0.0);
                ObsValueNode::Categorical {
                    name: name.clone(),
                    class_id: raw_value.round() as usize,
                    raw_value,
                }
            }
            Self::Struct { name, fields } => {
                let mut offset = 0;
                let mut field_nodes = Vec::with_capacity(fields.len());
                for field in fields {
                    let f_raw = field.raw_dim();
                    let f_slice = if offset < slice.len() {
                        &slice[offset..(offset + f_raw).min(slice.len())]
                    } else {
                        &[]
                    };
                    field_nodes.push(field.decode_value(f_slice));
                    offset += f_raw;
                }
                ObsValueNode::Struct {
                    name: name.clone(),
                    fields: field_nodes,
                }
            }
            Self::Repeated {
                name,
                max_count,
                item,
                ..
            } => {
                let item_raw = item.raw_dim();
                let mut items = Vec::with_capacity(*max_count);
                let mut offset = 0;
                for i in 0..*max_count {
                    let i_slice = if offset < slice.len() {
                        &slice[offset..(offset + item_raw).min(slice.len())]
                    } else {
                        &[]
                    };
                    let mut node = item.decode_value(i_slice);
                    if let ObsValueNode::Struct { ref mut name, .. } = node {
                        *name = format!("[{i}] {name}");
                    }
                    items.push(node);
                    offset += item_raw;
                }
                ObsValueNode::Repeated {
                    name: name.clone(),
                    items,
                }
            }
        }
    }
}

/// 结构化观测解析树节点（用于前端动态长度、可折叠、可嵌套呈现）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ObsValueNode {
    Scalar {
        name: String,
        value: f32,
    },
    Vector {
        name: String,
        values: Vec<f32>,
    },
    Categorical {
        name: String,
        class_id: usize,
        raw_value: f32,
    },
    Struct {
        name: String,
        fields: Vec<ObsValueNode>,
    },
    Repeated {
        name: String,
        items: Vec<ObsValueNode>,
    },
}

impl ObsValueNode {
    pub fn name(&self) -> &str {
        match self {
            Self::Scalar { name, .. } => name,
            Self::Vector { name, .. } => name,
            Self::Categorical { name, .. } => name,
            Self::Struct { name, .. } => name,
            Self::Repeated { name, .. } => name,
        }
    }
}

/// 完整的观测空间规范 Schema
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ObsSchema {
    pub nodes: Vec<ObsNode>,
}

impl ObsSchema {
    pub fn new(nodes: Vec<ObsNode>) -> Self {
        Self { nodes }
    }

    /// 原始输入向量总长度
    pub fn raw_dim(&self) -> usize {
        self.nodes.iter().map(|n| n.raw_dim()).sum()
    }

    /// 经特征提取器变换后输出给策略主干的总维度
    pub fn encoded_dim(&self) -> usize {
        self.nodes.iter().map(|n| n.encoded_dim()).sum()
    }

    /// 自动推导所有原始观测维度的标签说明
    pub fn to_dim_labels(&self) -> Vec<String> {
        self.nodes.iter().flat_map(|n| n.to_dim_labels()).collect()
    }

    /// 将原始扁平浮点数组解析为结构化观测树
    pub fn decode_tree(&self, raw_obs: &[f32]) -> Vec<ObsValueNode> {
        let mut offset = 0;
        let mut result = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let r_dim = node.raw_dim();
            let slice = if offset < raw_obs.len() {
                &raw_obs[offset..(offset + r_dim).min(raw_obs.len())]
            } else {
                &[]
            };
            result.push(node.decode_value(slice));
            offset += r_dim;
        }
        result
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetricsRow {
    pub step: usize,
    pub ep_return: f32,
    /// 兼容旧数据：policy_loss + value_loss 之和
    pub loss: f32,
    pub policy_loss: f32,
    pub value_loss: f32,
    pub total_loss: f32,
    pub kl: f32,
    pub entropy: f32,
    /// 本迭代被 clip 的比例（相对 clip_eps 界）
    pub clip_frac: f32,
    /// 迭代内各状态 critic 预测值的均值
    pub value: f32,
    pub fps: usize,
    /// 本迭代完成的各局步数最大值 / 最小值 / 平均值
    pub ep_steps_max: usize,
    pub ep_steps_min: usize,
    pub ep_steps_avg: f32,
    /// 本迭代各奖励项的每步平均贡献（时间惩罚/对齐/错位/空挥/破绽/击杀）
    pub reward_breakdown: Vec<RewardItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ObsFeaturePayload {
    // ── 通用战斗与环境遥测指标 ──
    #[serde(default)]
    pub self_hp_pct: f32,
    #[serde(default)]
    pub target_hp_pct: f32,
    #[serde(default)]
    pub distance: f32,
    #[serde(default)]
    pub metrics: HashMap<String, f32>,
    #[serde(default)]
    pub tags: HashMap<String, String>,

    // ── 向下兼容字段（供特定环境或现有 UI 无缝使用） ──
    #[serde(default)]
    pub fiora_hp_pct: f32,
    #[serde(default)]
    pub riven_hp_pct: f32,
    #[serde(default)]
    pub q_ready: bool,
    #[serde(default)]
    pub w_ready: bool,
    #[serde(default)]
    pub e_ready: bool,
    #[serde(default)]
    pub r_ready: bool,
    #[serde(default)]
    pub has_vital: bool,
    #[serde(default)]
    pub vital_is_active: bool,
    #[serde(default)]
    pub vital_direction: String,
    #[serde(default)]
    pub vital_active_time: f32,
    #[serde(default)]
    pub has_r_vital: bool,
    #[serde(default)]
    pub r_is_active: bool,
    #[serde(default)]
    pub attack_state: String,
    #[serde(default)]
    pub attack_timer: f32,
}

pub const AGENT_PPO_MAMBA: &str = "PPO (Mamba)";
pub const AGENT_PPO_MLP: &str = "PPO (MLP)";

/// 策略网络的主干网络架构类型
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyBackbone {
    /// 经典多层感知机（无状态前馈网络，计算速度极快）
    Mlp,
    /// Selective State Space Model（带时序记忆与门控状态空间）
    Mamba,
}

impl PolicyBackbone {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mlp => "MLP",
            Self::Mamba => "Mamba",
        }
    }
}

impl std::fmt::Display for PolicyBackbone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskConfigPayload {
    pub name: String,
    pub agent_type: String,
    pub env_name: String,
    pub lr: f32,
    pub gamma: f32,
    pub gae_lambda: f32,
    pub clip_eps: f32,
    pub ppo_epochs: usize,
    pub hidden_dim: usize,
    pub parallel_envs: usize,
    pub rollout_steps_per_env: usize,
    pub total_iterations: usize,
    #[serde(default)]
    pub backbone: Option<PolicyBackbone>,
}

pub const ENV_SOLO_V0: &str = "SoloV0";
pub const ENV_FIORA_V2: &str = "FioraV2";
pub const ENV_FIORA_V1: &str = "FioraV1";
pub const ENV_FIORA_V0: &str = "FioraV0";

/// 环境自带的训练超参数规范（作为环境默认超参数的唯一真实来源）。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EnvTrainingParams {
    pub lr: f32,
    pub gamma: f32,
    pub gae_lambda: f32,
    pub clip_eps: f32,
    pub ppo_epochs: usize,
    pub hidden_dim: usize,
    pub rollout_steps_per_env: usize,
    pub total_iterations: usize,
}

/// 环境规范与展示元数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub default_params: EnvTrainingParams,
}

pub const ENV_SOLO_V0_SPEC: EnvSpec = EnvSpec {
    name: ENV_SOLO_V0,
    label: "剑姬 vs 瑞雯 (Solo 1v1 自博弈)",
    tag: "SoloV0",
    description: "单神经网络通过 role_id (0:剑姬, 1:瑞雯) 自博弈对抗，对称零和奖励与自我中心化全技能对决",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 8,
        hidden_dim: 64,
        rollout_steps_per_env: 160,
        total_iterations: 500,
    },
};

pub const ENV_FIORA_V2_SPEC: EnvSpec = EnvSpec {
    name: ENV_FIORA_V2,
    label: "全技能实战 (V2)",
    tag: "V2",
    description: "基于 OpenAI Five 统一结构化 Modifier 槽位与通用表征架构的全技能微操环境",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 8,
        hidden_dim: 64,
        rollout_steps_per_env: 160,
        total_iterations: 300,
    },
};

pub const ENV_FIORA_V1_SPEC: EnvSpec = EnvSpec {
    name: ENV_FIORA_V1,
    label: "真实移动 (V1)",
    tag: "V1",
    description: "模拟真实微操移动与普攻破绽打击，连续空间离散化动作",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 4,
        hidden_dim: 64,
        rollout_steps_per_env: 80,
        total_iterations: 80,
    },
};

pub const ENV_FIORA_V0_SPEC: EnvSpec = EnvSpec {
    name: ENV_FIORA_V0,
    label: "瞬移站位 (V0)",
    tag: "V0",
    description: "简化版瞬移站位打弱点机制，快速收敛验证基础 PPO 策略",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 4,
        hidden_dim: 64,
        rollout_steps_per_env: 80,
        total_iterations: 50,
    },
};

pub const AVAILABLE_ENVS: &[EnvSpec] = &[
    ENV_SOLO_V0_SPEC,
    ENV_FIORA_V2_SPEC,
    ENV_FIORA_V1_SPEC,
    ENV_FIORA_V0_SPEC,
];

pub fn get_env_spec(name: &str) -> Option<&'static EnvSpec> {
    AVAILABLE_ENVS.iter().find(|e| e.name == name)
}

pub fn get_env_training_params(name: &str) -> EnvTrainingParams {
    get_env_spec(name)
        .map(|s| s.default_params.clone())
        .unwrap_or(ENV_FIORA_V2_SPEC.default_params)
}

impl TaskConfigPayload {
    pub fn default_for_env(env_name: &str) -> Self {
        let params = get_env_training_params(env_name);
        Self {
            name: "RL 对战训练任务".to_string(),
            agent_type: AGENT_PPO_MAMBA.to_string(),
            env_name: env_name.to_string(),
            lr: params.lr,
            gamma: params.gamma,
            gae_lambda: params.gae_lambda,
            clip_eps: params.clip_eps,
            ppo_epochs: params.ppo_epochs,
            hidden_dim: params.hidden_dim,
            parallel_envs: 0,
            rollout_steps_per_env: params.rollout_steps_per_env,
            total_iterations: params.total_iterations,
            backbone: Some(PolicyBackbone::Mamba),
        }
    }

    /// 解析当前任务的主干网络架构（优先使用 backbone 字段，其次解析 agent_type）
    pub fn backbone(&self) -> PolicyBackbone {
        if let Some(bb) = self.backbone {
            return bb;
        }
        if self.agent_type.to_lowercase().contains("mlp") {
            PolicyBackbone::Mlp
        } else {
            PolicyBackbone::Mamba
        }
    }
}

impl Default for TaskConfigPayload {
    fn default() -> Self {
        Self::default_for_env(ENV_FIORA_V2)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskOverviewItem {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub env_name: String,
    pub status: String,
    pub current_step: usize,
    pub ep_return: f32,
    pub checkpoints_count: usize,
    pub hidden_dim: usize,
    pub parallel_envs: usize,
    pub lr: f32,
    pub total_iterations: usize,
    pub rollout_steps_per_env: usize,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OutFrame {
    TaskList {
        tasks: Vec<TaskOverviewItem>,
    },
    Status {
        task_id: String,
        status: String,
    },
    Metrics {
        task_id: String,
        step: usize,
        ep_return: f32,
        loss: f32,
        policy_loss: f32,
        value_loss: f32,
        total_loss: f32,
        kl: f32,
        entropy: f32,
        clip_frac: f32,
        /// 本任务固定的 PPO clip 界，前端用于 KL 参考线
        clip_eps: f32,
        value: f32,
        fps: usize,
        ep_steps_max: usize,
        ep_steps_min: usize,
        ep_steps_avg: f32,
        reward_breakdown: Vec<RewardItem>,
        obs_feature: Option<ObsFeaturePayload>,
        reward_formula: Option<RewardFormulaSpec>,
        reward_variables: Option<HashMap<String, f32>>,
    },
    Log {
        task_id: String,
        level: String,
        message: String,
    },
    CheckpointMsg {
        task_id: String,
        checkpoint: CheckpointItem,
    },
    CheckpointLoaded {
        task_id: String,
        checkpoint: CheckpointItem,
    },
    TaskDetail {
        task_id: String,
        checkpoints: Vec<CheckpointItem>,
        metrics_history: Vec<MetricsRow>,
        logs: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolicyItem {
    pub action_id: usize,
    pub action: String,
    /// Mask 后的有效采样概率
    pub prob: f32,
    /// 未 Mask 的网络原始 Softmax 概率
    #[serde(default)]
    pub raw_prob: f32,
    /// 当前步是否被 Action Mask 屏蔽
    #[serde(default)]
    pub is_masked: bool,
}

/// 可视化策略展示：按真实动作空间区分（连续/混合环境不再用 preset 伪分布）。
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PolicyDisplay {
    /// 离散动作空间：每个离散动作的概率分布。
    Discrete(Vec<PolicyItem>),
    /// 混合动作空间（连续移动 + 离散攻击）：连续均值 + 攻击概率。
    Hybrid {
        move_x: f32,
        move_z: f32,
        attack_prob: f32,
        #[serde(default)]
        raw_attack_prob: f32,
        #[serde(default)]
        is_attack_masked: bool,
    },
    /// 增强混合动作空间（多维连续偏移均值 + 多离散动作概率分布）
    HybridMulti {
        continuous_means: Vec<f32>,
        discrete_probs: Vec<PolicyItem>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RewardItem {
    pub name: String,
    pub value: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckpointItem {
    pub id: String,
    pub step: usize,
    pub path: String,
    pub ep_return: f32,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InFrame {
    GetTaskList,
    GetTaskDetail {
        task_id: String,
    },
    CreateTask {
        config: TaskConfigPayload,
    },
    Control {
        task_id: String,
        command: String,
        config_json: Option<String>,
    },
    SaveCheckpoint {
        task_id: String,
    },
    ApplyCheckpoint {
        task_id: String,
        id: String,
    },
    DeleteTask {
        task_id: String,
    },
}

// ── Visual subprocess protocol ──

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisualObsFrame {
    pub step: usize,
    pub obs: ObsFeaturePayload,
    pub reward: f32,
    /// 本局从开始累积到当前步的总奖励（每次对局重置后归零）。
    #[serde(default)]
    pub episode_reward: f32,
    pub reward_breakdown: Vec<RewardItem>,
    pub policy: PolicyDisplay,
    pub terminated: bool,
    pub truncated: bool,
    #[serde(default)]
    pub self_alive: bool,
    #[serde(default)]
    pub target_alive: bool,
    #[serde(default)]
    pub fiora_alive: bool,
    #[serde(default)]
    pub riven_alive: bool,
    pub reward_formula: Option<RewardFormulaSpec>,
    pub reward_variables: Option<HashMap<String, f32>>,
    /// 策略真实输入的观测向量。
    #[serde(default)]
    pub obs_vector: Vec<f32>,
    /// 观测向量每一维的简要说明。
    #[serde(default)]
    pub obs_labels: Vec<String>,
    /// 结构化 AST 观测树（供前端动态长度、可折叠、可嵌套展示）。
    #[serde(default)]
    pub obs_tree: Option<Vec<ObsValueNode>>,
    /// 当前是否处于暂停状态。
    #[serde(default)]
    pub is_paused: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VisualOutFrame {
    Ready {
        checkpoint_path: String,
        env_name: String,
        env_max_steps: usize,
        action_labels: Vec<String>,
        #[serde(default)]
        obs_schema: Option<ObsSchema>,
    },
    Frame(VisualObsFrame),
    Paused(bool),
    Log {
        level: String,
        message: String,
    },
    Exited {
        code: Option<i32>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VisualInFrame {
    Reset,
    Pause,
    Resume,
    StepOnce,
    StepWithAction { action_id: usize },
    SetAutoPause(bool),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_reward_expr_eval() {
        let vars = HashMap::from([("a".to_string(), 2.0), ("b".to_string(), 3.0)]);
        let expr = RewardExpr::Add(
            Box::new(RewardExpr::Variable("a".into())),
            Box::new(RewardExpr::Mul(
                Box::new(RewardExpr::Constant(4.0)),
                Box::new(RewardExpr::Variable("b".into())),
            )),
        );
        assert_eq!(expr.eval(&vars), 2.0 + 4.0 * 3.0);
    }

    #[test]
    fn test_reward_expr_gt_and_if_else() {
        let vars = HashMap::from([("x".to_string(), 5.0)]);
        let gt = RewardExpr::Gt(
            Box::new(RewardExpr::Variable("x".into())),
            Box::new(RewardExpr::Constant(3.0)),
        );
        assert_eq!(gt.eval(&vars), 1.0);
        let if_else = RewardExpr::IfElse {
            cond: Box::new(gt),
            then_branch: Box::new(RewardExpr::Constant(10.0)),
            else_branch: Box::new(RewardExpr::Constant(0.0)),
        };
        assert_eq!(if_else.eval(&vars), 10.0);
    }

    #[test]
    fn test_reward_expr_exp() {
        let vars = HashMap::from([("t".to_string(), 4.0)]);
        let expr = RewardExpr::Sub(
            Box::new(RewardExpr::Mul(
                Box::new(RewardExpr::Constant(3.0)),
                Box::new(RewardExpr::Exp(Box::new(RewardExpr::Mul(
                    Box::new(RewardExpr::Constant(0.6)),
                    Box::new(RewardExpr::Sub(
                        Box::new(RewardExpr::Constant(4.0)),
                        Box::new(RewardExpr::Variable("t".into())),
                    )),
                )))),
            )),
            Box::new(RewardExpr::Constant(3.0)),
        );
        // t = 4.0 => 3.0 * (exp(0) - 1) = 0.0
        assert!((expr.eval(&vars) - 0.0).abs() < 1e-5);

        let vars_1s = HashMap::from([("t".to_string(), 1.0)]);
        // t = 1.0 => 3.0 * (exp(1.8) - 1) = 3.0 * (6.0496 - 1) = 15.1489
        let val_1s = expr.eval(&vars_1s);
        assert!((val_1s - 15.1489).abs() < 1e-2);
    }

    #[test]
    fn test_reward_formula_compute() {
        let spec = RewardFormulaSpec {
            name: "test".into(),
            terms: vec![
                RewardTermSpec::new("c", "常数", RewardExpr::Constant(-0.5)),
                RewardTermSpec::new("v", "变量", RewardExpr::Variable("hit".into())),
            ],
        };
        let vars = HashMap::from([("hit".to_string(), 0.8)]);
        let (total, items) = spec.compute(&vars);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].value, -0.5);
        assert_eq!(items[1].value, 0.8);
        assert!((total - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_action_space_dims() {
        assert_eq!(ActionSpace::Discrete(5).actor_head_dim(), 5);
        assert_eq!(ActionSpace::Discrete(5).encoding_dim(), 1);
        assert_eq!(ActionSpace::Continuous(3).actor_head_dim(), 3);
        assert_eq!(ActionSpace::Continuous(3).encoding_dim(), 3);
        let hybrid = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 2,
        };
        assert_eq!(hybrid.actor_head_dim(), 4);
        assert_eq!(hybrid.encoding_dim(), 3);
    }

    #[test]
    fn test_obs_schema_dims_and_labels() {
        let schema = ObsSchema::new(vec![
            ObsNode::structure(
                "hero",
                vec![
                    ObsNode::categorical("role", 4, 12),
                    ObsNode::scalar("hp_pct", 0.0, 1.0),
                    ObsNode::vector("cooldowns", 2),
                ],
            ),
            ObsNode::repeated(
                "buffs",
                2,
                ObsNode::structure(
                    "slot",
                    vec![
                        ObsNode::categorical("buff", 8, 8),
                        ObsNode::scalar("duration", 0.0, 10.0),
                    ],
                ),
                EntityEncoderSpec::SharedMlpFlatten {
                    hidden_dims: vec![16],
                },
            ),
            ObsNode::repeated(
                "minions",
                3,
                ObsNode::structure(
                    "unit",
                    vec![
                        ObsNode::categorical("unit_type", 4, 6),
                        ObsNode::vector("rel_pos", 2),
                    ],
                ),
                EntityEncoderSpec::SharedMlpPool {
                    hidden_dims: vec![16, 8],
                    pool_type: PoolType::Max,
                },
            ),
        ]);

        // Raw dims:
        // hero: 1 (role) + 1 (hp_pct) + 2 (cooldowns) = 4
        // buffs: 2 * (1 + 1) = 4
        // minions: 3 * (1 + 2) = 9
        // total raw = 17
        assert_eq!(schema.raw_dim(), 17);

        // Encoded dims:
        // hero: 12 (role embed) + 1 (hp_pct) + 2 (cooldowns) = 15
        // buffs: 2 * 16 (shared mlp flatten) = 32
        // minions: 8 (shared mlp max pool to 8) = 8
        // total encoded = 55
        assert_eq!(schema.encoded_dim(), 55);

        let labels = schema.to_dim_labels();
        assert_eq!(labels.len(), 17);
        assert_eq!(labels[0], "hero.role_id");
        assert_eq!(labels[1], "hero.hp_pct");
        assert_eq!(labels[2], "hero.cooldowns[0]");
        assert_eq!(labels[3], "hero.cooldowns[1]");
        assert_eq!(labels[4], "buffs[0].slot.buff_id");
        assert_eq!(labels[5], "buffs[0].slot.duration");
        assert_eq!(labels[6], "buffs[1].slot.buff_id");
        assert_eq!(labels[7], "buffs[1].slot.duration");
        assert_eq!(labels[8], "minions[0].unit.unit_type_id");
        assert_eq!(labels[9], "minions[0].unit.rel_pos[0]");
        assert_eq!(labels[10], "minions[0].unit.rel_pos[1]");

        let dummy_raw = vec![1.0; 17];
        let tree = schema.decode_tree(&dummy_raw);
        assert_eq!(tree.len(), 3);

        let frame = VisualOutFrame::Ready {
            checkpoint_path: "test.safetensors".into(),
            env_name: "SoloV0".into(),
            env_max_steps: 100,
            action_labels: vec!["a1".into()],
            obs_schema: Some(schema),
        };
        let bytes = bincode::serialize(&frame).expect("bincode serialize Ready");
        let decoded: VisualOutFrame = bincode::deserialize(&bytes).expect("bincode deserialize Ready");
        match decoded {
            VisualOutFrame::Ready { obs_schema, .. } => {
                assert!(obs_schema.is_some());
            }
            _ => panic!("Expected Ready"),
        }

        let obs_frame = VisualObsFrame {
            step: 1,
            obs: ObsFeaturePayload::default(),
            reward: 1.0,
            episode_reward: 10.0,
            reward_breakdown: vec![],
            policy: PolicyDisplay::Discrete(vec![]),
            terminated: false,
            truncated: false,
            self_alive: true,
            target_alive: true,
            fiora_alive: true,
            riven_alive: true,
            reward_formula: None,
            reward_variables: None,
            obs_vector: dummy_raw,
            obs_labels: labels,
            obs_tree: Some(tree),
            is_paused: false,
        };
        let out_frame = VisualOutFrame::Frame(obs_frame);
        let bytes2 = bincode::serialize(&out_frame).expect("bincode serialize Frame");
        let decoded2: VisualOutFrame = bincode::deserialize(&bytes2).expect("bincode deserialize Frame");
        match decoded2 {
            VisualOutFrame::Frame(f) => {
                assert_eq!(f.obs_tree.as_ref().unwrap().len(), 3);
            }
            _ => panic!("Expected Frame"),
        }
    }
}
