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

#[cfg(test)]
mod tests {
    use super::*;

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
