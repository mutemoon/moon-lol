use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Neg, Sub};

use serde::{Deserialize, Serialize};

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

/// 结构化特征计算表达式 AST（类似于 RewardExpr，用于声明式特征工程与归一化计算）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ObsExpr {
    Constant(f32),
    Variable(String),
    Add(Box<ObsExpr>, Box<ObsExpr>),
    Sub(Box<ObsExpr>, Box<ObsExpr>),
    Mul(Box<ObsExpr>, Box<ObsExpr>),
    Div(Box<ObsExpr>, Box<ObsExpr>),
    Clamp {
        expr: Box<ObsExpr>,
        min: f32,
        max: f32,
    },
    IfElse {
        cond: Box<ObsExpr>,
        then_branch: Box<ObsExpr>,
        else_branch: Box<ObsExpr>,
    },
    Gt(Box<ObsExpr>, Box<ObsExpr>),
    Lt(Box<ObsExpr>, Box<ObsExpr>),
    Max(Box<ObsExpr>, Box<ObsExpr>),
    Min(Box<ObsExpr>, Box<ObsExpr>),
}

impl ObsExpr {
    pub fn var(name: impl Into<String>) -> Self {
        Self::Variable(name.into())
    }

    pub fn c(val: f32) -> Self {
        Self::Constant(val)
    }

    pub fn clamp(expr: Self, min: f32, max: f32) -> Self {
        Self::Clamp {
            expr: Box::new(expr),
            min,
            max,
        }
    }

    pub fn max(a: Self, b: Self) -> Self {
        Self::Max(Box::new(a), Box::new(b))
    }

    pub fn min(a: Self, b: Self) -> Self {
        Self::Min(Box::new(a), Box::new(b))
    }

    pub fn if_else(cond: Self, then_branch: Self, else_branch: Self) -> Self {
        Self::IfElse {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        }
    }

    pub fn gt(a: Self, b: Self) -> Self {
        Self::Gt(Box::new(a), Box::new(b))
    }

    pub fn lt(a: Self, b: Self) -> Self {
        Self::Lt(Box::new(a), Box::new(b))
    }

    /// 在给定的环境变量上下文中对表达式求值
    pub fn eval(&self, vars: &HashMap<String, f32>) -> f32 {
        match self {
            Self::Constant(c) => *c,
            Self::Variable(name) => vars.get(name).copied().unwrap_or(0.0),
            Self::Add(a, b) => a.eval(vars) + b.eval(vars),
            Self::Sub(a, b) => a.eval(vars) - b.eval(vars),
            Self::Mul(a, b) => a.eval(vars) * b.eval(vars),
            Self::Div(a, b) => {
                let denom = b.eval(vars);
                if denom.abs() < 1e-7 {
                    0.0
                } else {
                    a.eval(vars) / denom
                }
            }
            Self::Clamp { expr, min, max } => expr.eval(vars).clamp(*min, *max),
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
            Self::Lt(a, b) => {
                if a.eval(vars) < b.eval(vars) {
                    1.0
                } else {
                    0.0
                }
            }
            Self::Max(a, b) => a.eval(vars).max(b.eval(vars)),
            Self::Min(a, b) => a.eval(vars).min(b.eval(vars)),
        }
    }
}

// ── 运算符重载 ─────────────────────────────────────────────────────────────

impl Add for ObsExpr {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::Add(Box::new(self), Box::new(rhs))
    }
}

impl Add<f32> for ObsExpr {
    type Output = Self;
    fn add(self, rhs: f32) -> Self {
        Self::Add(Box::new(self), Box::new(Self::Constant(rhs)))
    }
}

impl Sub for ObsExpr {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::Sub(Box::new(self), Box::new(rhs))
    }
}

impl Sub<f32> for ObsExpr {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self {
        Self::Sub(Box::new(self), Box::new(Self::Constant(rhs)))
    }
}

impl Mul for ObsExpr {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::Mul(Box::new(self), Box::new(rhs))
    }
}

impl Mul<f32> for ObsExpr {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::Mul(Box::new(self), Box::new(Self::Constant(rhs)))
    }
}

impl Mul<ObsExpr> for f32 {
    type Output = ObsExpr;
    fn mul(self, rhs: ObsExpr) -> ObsExpr {
        ObsExpr::Mul(Box::new(ObsExpr::Constant(self)), Box::new(rhs))
    }
}

impl Div for ObsExpr {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self::Div(Box::new(self), Box::new(rhs))
    }
}

impl Div<f32> for ObsExpr {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self::Div(Box::new(self), Box::new(Self::Constant(rhs)))
    }
}

impl Neg for ObsExpr {
    type Output = Self;
    fn neg(self) -> Self {
        Self::Mul(Box::new(Self::Constant(-1.0)), Box::new(self))
    }
}

/// 原始观测特征上下文字典（包含未归一化的物理量、基础标量和重复实体列表）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ObsContext {
    #[serde(default)]
    pub vars: HashMap<String, f32>,
    #[serde(default)]
    pub repeated: HashMap<String, Vec<ObsContext>>,
}

impl ObsContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_var(mut self, name: impl Into<String>, value: f32) -> Self {
        self.vars.insert(name.into(), value);
        self
    }

    pub fn set_var(&mut self, name: impl Into<String>, value: f32) {
        self.vars.insert(name.into(), value);
    }

    pub fn with_repeated(mut self, key: impl Into<String>, items: Vec<ObsContext>) -> Self {
        self.repeated.insert(key.into(), items);
        self
    }

    pub fn set_repeated(&mut self, key: impl Into<String>, items: Vec<ObsContext>) {
        self.repeated.insert(key.into(), items);
    }
}

/// 观测空间 AST 节点定义
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ObsNode {
    /// 连续标量特征（如 HP%, 距离, 倒计时等，占 1 个 float 输入）
    Scalar {
        name: String,
        #[serde(default)]
        expr: Option<ObsExpr>,
        #[serde(default)]
        min: f32,
        #[serde(default = "default_max_one")]
        max: f32,
    },
    /// 连续向量特征（如 相对坐标 [x, z], 技能冷却数组等，占 dim 个 float 输入）
    Vector {
        name: String,
        #[serde(default)]
        exprs: Vec<ObsExpr>,
        dim: usize,
    },
    /// 离散/类别特征（映射到 Embedding 表，占 1 个 float 类别索引输入）
    Categorical {
        name: String,
        #[serde(default)]
        expr: Option<ObsExpr>,
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
            expr: None,
            min,
            max,
        }
    }

    pub fn scalar_expr(name: impl Into<String>, expr: ObsExpr) -> Self {
        Self::Scalar {
            name: name.into(),
            expr: Some(expr),
            min: 0.0,
            max: 1.0,
        }
    }

    pub fn vector(name: impl Into<String>, dim: usize) -> Self {
        Self::Vector {
            name: name.into(),
            exprs: Vec::new(),
            dim,
        }
    }

    pub fn vector_exprs(name: impl Into<String>, exprs: Vec<ObsExpr>) -> Self {
        let dim = exprs.len();
        Self::Vector {
            name: name.into(),
            exprs,
            dim,
        }
    }

    pub fn categorical(name: impl Into<String>, num_classes: usize, embed_dim: usize) -> Self {
        Self::Categorical {
            name: name.into(),
            expr: None,
            num_classes,
            embed_dim,
        }
    }

    pub fn categorical_expr(
        name: impl Into<String>,
        expr: ObsExpr,
        num_classes: usize,
        embed_dim: usize,
    ) -> Self {
        Self::Categorical {
            name: name.into(),
            expr: Some(expr),
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
            Self::Vector { name, dim, .. } => {
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
            Self::Vector { name, dim, .. } => ObsValueNode::Vector {
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

    /// 依据当前 AST 表达式对给定的原始观测上下文求值，输出展平的一维浮点向量
    pub fn eval_to_vector(&self, ctx: &ObsContext) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.raw_dim());
        for node in &self.nodes {
            node.eval_to_vector(ctx, &mut out);
        }
        out
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

impl ObsNode {
    /// 依据当前 AST 节点对给定的原始观测上下文求值，并将计算结果追加写入扁平浮点向量
    pub fn eval_to_vector(&self, ctx: &ObsContext, out: &mut Vec<f32>) {
        match self {
            Self::Scalar { name, expr, .. } => {
                let val = if let Some(e) = expr {
                    e.eval(&ctx.vars)
                } else {
                    ctx.vars.get(name).copied().unwrap_or(0.0)
                };
                out.push(val);
            }
            Self::Vector { name, exprs, dim } => {
                if !exprs.is_empty() {
                    for e in exprs {
                        out.push(e.eval(&ctx.vars));
                    }
                } else {
                    for i in 0..*dim {
                        let key = format!("{}[{}]", name, i);
                        let val = ctx
                            .vars
                            .get(&key)
                            .or_else(|| ctx.vars.get(name))
                            .copied()
                            .unwrap_or(0.0);
                        out.push(val);
                    }
                }
            }
            Self::Categorical { name, expr, .. } => {
                let val = if let Some(e) = expr {
                    e.eval(&ctx.vars)
                } else {
                    ctx.vars.get(name).copied().unwrap_or(0.0)
                };
                out.push(val);
            }
            Self::Struct { fields, .. } => {
                for f in fields {
                    f.eval_to_vector(ctx, out);
                }
            }
            Self::Repeated {
                name,
                max_count,
                item,
                ..
            } => {
                let items_ctx = ctx.repeated.get(name);
                let item_raw = item.raw_dim();
                for i in 0..*max_count {
                    if let Some(sub_ctx) = items_ctx.and_then(|list| list.get(i)) {
                        item.eval_to_vector(sub_ctx, out);
                    } else {
                        out.extend(std::iter::repeat(0.0).take(item_raw));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obs_expr_eval_and_operators() {
        let mut vars = HashMap::new();
        vars.insert("self_hp".to_string(), 80.0);
        vars.insert("self_max_hp".to_string(), 100.0);
        vars.insert("pos_x".to_string(), 250.0);
        vars.insert("target_x".to_string(), 100.0);

        let hp_pct_expr = ObsExpr::clamp(
            ObsExpr::var("self_hp") / ObsExpr::max(ObsExpr::var("self_max_hp"), ObsExpr::c(1.0)),
            0.0,
            1.0,
        );
        assert_eq!(hp_pct_expr.eval(&vars), 0.8);

        let rel_x_expr = (ObsExpr::var("pos_x") - ObsExpr::var("target_x")) / 100.0;
        assert_eq!(rel_x_expr.eval(&vars), 1.5);
    }

    #[test]
    fn test_obs_schema_eval_to_vector() {
        let schema = ObsSchema::new(vec![
            ObsNode::categorical_expr("role", ObsExpr::var("role_id"), 4, 12),
            ObsNode::structure(
                "spatial",
                vec![
                    ObsNode::scalar_expr(
                        "rel_x",
                        (ObsExpr::var("x1") - ObsExpr::var("x2")) / 100.0,
                    ),
                    ObsNode::scalar_expr("dist", ObsExpr::var("dist") / 100.0),
                ],
            ),
            ObsNode::repeated(
                "units",
                2,
                ObsNode::structure(
                    "unit",
                    vec![
                        ObsNode::categorical_expr("type", ObsExpr::var("unit_type"), 4, 8),
                        ObsNode::scalar_expr("hp_pct", ObsExpr::var("hp_pct")),
                    ],
                ),
                EntityEncoderSpec::PassThrough,
            ),
        ]);

        assert_eq!(schema.raw_dim(), 1 + 2 + 2 * 2); // 1 + 2 + 4 = 7

        let mut ctx = ObsContext::new()
            .with_var("role_id", 0.0)
            .with_var("x1", 300.0)
            .with_var("x2", 100.0)
            .with_var("dist", 200.0);

        // 仅提供 1 个单位槽位，第 2 个槽位应自动补 0.0
        let u0 = ObsContext::new()
            .with_var("unit_type", 2.0)
            .with_var("hp_pct", 0.5);
        ctx.set_repeated("units", vec![u0]);

        let vec = schema.eval_to_vector(&ctx);
        assert_eq!(vec.len(), 7);
        assert_eq!(vec[0], 0.0); // role
        assert_eq!(vec[1], 2.0); // rel_x: (300 - 100)/100
        assert_eq!(vec[2], 2.0); // dist: 200/100
        assert_eq!(vec[3], 2.0); // units[0].type
        assert_eq!(vec[4], 0.5); // units[0].hp_pct
        assert_eq!(vec[5], 0.0); // units[1].type (padding)
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
    }
}
