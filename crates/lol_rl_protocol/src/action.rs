use serde::{Deserialize, Serialize};

use crate::ActionSpace;

/// 动作空间 AST 节点定义（与 ObsNode 对称的声明式动作空间描述）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ActionNode {
    /// 离散分类动作（如主动作类型：NoOp/Move/Attack/CastQ/…）
    Categorical {
        name: String,
        num_classes: usize,
        labels: Vec<String>,
    },
    /// 连续高斯动作（如方向偏移 [x, z]）
    Continuous { name: String, dim: usize },
    /// 单位选择头：从观测中 max_units 个单位嵌入中选一个目标
    /// 网络输出 = dot(query_proj(feat), unit_embeds) → max_units 维 logits
    UnitSelection {
        name: String,
        max_units: usize,
        /// 单位嵌入特征维度（应与 obs schema 中对应 Repeated 的 encoder 输出维一致）
        unit_embed_dim: usize,
        /// 该头引用的观测 Repeated 节点名（如 "visible_units"）
        obs_entity_name: String,
    },
    /// 命名复合元组（将多个子动作组合为一个动作步）
    Struct {
        name: String,
        fields: Vec<ActionNode>,
    },
}

impl ActionNode {
    pub fn name(&self) -> &str {
        match self {
            Self::Categorical { name, .. } => name,
            Self::Continuous { name, .. } => name,
            Self::UnitSelection { name, .. } => name,
            Self::Struct { name, .. } => name,
        }
    }

    pub fn encoding_dim(&self) -> usize {
        match self {
            Self::Categorical { .. } => 1,
            Self::Continuous { dim, .. } => *dim,
            Self::UnitSelection { .. } => 1,
            Self::Struct { fields, .. } => fields.iter().map(|f| f.encoding_dim()).sum(),
        }
    }

    pub fn to_encoding_labels(&self) -> Vec<String> {
        match self {
            Self::Categorical { name, .. } => vec![format!("{}_id", name)],
            Self::Continuous { name, dim } => {
                if *dim == 1 {
                    vec![name.clone()]
                } else {
                    (0..*dim).map(|i| format!("{}[{}]", name, i)).collect()
                }
            }
            Self::UnitSelection { name, .. } => vec![format!("{}_target_id", name)],
            Self::Struct { name, fields } => fields
                .iter()
                .flat_map(|f| {
                    f.to_encoding_labels()
                        .into_iter()
                        .map(|l| format!("{}.{}", name, l))
                })
                .collect(),
        }
    }

    pub fn categorical(name: impl Into<String>, labels: Vec<String>) -> Self {
        Self::Categorical {
            name: name.into(),
            num_classes: labels.len(),
            labels,
        }
    }

    pub fn continuous(name: impl Into<String>, dim: usize) -> Self {
        Self::Continuous {
            name: name.into(),
            dim,
        }
    }

    pub fn unit_selection(
        name: impl Into<String>,
        max_units: usize,
        unit_embed_dim: usize,
        obs_entity_name: impl Into<String>,
    ) -> Self {
        Self::UnitSelection {
            name: name.into(),
            max_units,
            unit_embed_dim,
            obs_entity_name: obs_entity_name.into(),
        }
    }

    pub fn structure(name: impl Into<String>, fields: Vec<ActionNode>) -> Self {
        Self::Struct {
            name: name.into(),
            fields,
        }
    }

    /// 将扁平动作编码切片解析为结构化动作值节点
    pub fn decode_value(&self, slice: &[f32]) -> ActionValueNode {
        match self {
            Self::Categorical {
                name,
                labels,
                num_classes,
            } => {
                let class_id = slice.first().copied().unwrap_or(0.0).round().max(0.0) as usize;
                let label = labels
                    .get(class_id)
                    .cloned()
                    .unwrap_or_else(|| format!("类别 {class_id}"));
                ActionValueNode::Categorical {
                    name: name.clone(),
                    class_id,
                    num_classes: *num_classes,
                    label,
                }
            }
            Self::Continuous { name, dim } => {
                let values = slice[..(*dim).min(slice.len())].to_vec();
                let labels = if *dim == 1 {
                    vec![name.clone()]
                } else {
                    (0..*dim).map(|i| format!("{}[{}]", name, i)).collect()
                };
                ActionValueNode::Continuous {
                    name: name.clone(),
                    values,
                    labels,
                }
            }
            Self::UnitSelection {
                name,
                max_units,
                obs_entity_name,
                ..
            } => {
                let target_idx = slice.first().copied().unwrap_or(0.0).round().max(0.0) as usize;
                ActionValueNode::UnitSelection {
                    name: name.clone(),
                    target_idx,
                    max_units: *max_units,
                    obs_entity_name: obs_entity_name.clone(),
                }
            }
            Self::Struct { name, fields } => {
                let mut offset = 0;
                let mut field_nodes = Vec::with_capacity(fields.len());
                for field in fields {
                    let f_raw = field.encoding_dim();
                    let f_slice = if offset < slice.len() {
                        &slice[offset..(offset + f_raw).min(slice.len())]
                    } else {
                        &[]
                    };
                    field_nodes.push(field.decode_value(f_slice));
                    offset += f_raw;
                }
                ActionValueNode::Struct {
                    name: name.clone(),
                    fields: field_nodes,
                }
            }
        }
    }
}

/// 结构化动作解析树节点（用于前端动态展现执行动作与 AST 结构）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ActionValueNode {
    Categorical {
        name: String,
        class_id: usize,
        num_classes: usize,
        label: String,
    },
    Continuous {
        name: String,
        values: Vec<f32>,
        labels: Vec<String>,
    },
    UnitSelection {
        name: String,
        target_idx: usize,
        max_units: usize,
        obs_entity_name: String,
    },
    Struct {
        name: String,
        fields: Vec<ActionValueNode>,
    },
}

impl ActionValueNode {
    pub fn name(&self) -> &str {
        match self {
            Self::Categorical { name, .. } => name,
            Self::Continuous { name, .. } => name,
            Self::UnitSelection { name, .. } => name,
            Self::Struct { name, .. } => name,
        }
    }
}

use crate::obs::{ObsContext, ObsExpr};

/// 声明式动作掩码规则：当 condition 在当前 ObsContext 下成立 (> 0.0) 时，禁用对应动作分支
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ActionMaskRule {
    /// 禁用触发条件（ObsExpr 求值 > 0.0 时触发禁用）
    pub condition: ObsExpr,
    /// 目标动作头名称（可选，如 "action_type"）
    #[serde(default)]
    pub target_head: Option<String>,
    /// 被禁用的动作分支索引
    pub disabled_branch: usize,
    /// 被禁用的动作标签（如 "Attack"）
    pub branch_label: String,
}

impl ActionMaskRule {
    pub fn new(
        condition: ObsExpr,
        target_head: Option<String>,
        disabled_branch: usize,
        branch_label: impl Into<String>,
    ) -> Self {
        Self {
            condition,
            target_head,
            disabled_branch,
            branch_label: branch_label.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ActionSchema {
    pub nodes: Vec<ActionNode>,
    #[serde(default)]
    pub mask_rules: Vec<ActionMaskRule>,
}

impl ActionSchema {
    pub fn new(nodes: Vec<ActionNode>) -> Self {
        Self {
            nodes,
            mask_rules: Vec::new(),
        }
    }

    pub fn with_mask_rules(mut self, mask_rules: Vec<ActionMaskRule>) -> Self {
        self.mask_rules = mask_rules;
        self
    }

    pub fn encoding_dim(&self) -> usize {
        self.nodes.iter().map(|n| n.encoding_dim()).sum()
    }

    pub fn to_encoding_labels(&self) -> Vec<String> {
        self.nodes
            .iter()
            .flat_map(|n| n.to_encoding_labels())
            .collect()
    }

    /// 根据当前观测上下文，计算扁平离散动作掩码向量（适用于单离散动作头，如 FioraV2）
    pub fn eval_flat_mask(&self, ctx: &ObsContext) -> Vec<bool> {
        let cat_node = self.nodes.iter().find_map(|n| match n {
            ActionNode::Categorical { num_classes, .. } => Some(*num_classes),
            _ => None,
        });

        let num_classes = cat_node.unwrap_or(0);
        let mut mask = vec![true; num_classes];

        for rule in &self.mask_rules {
            if rule.disabled_branch < mask.len() && rule.condition.eval(&ctx.vars) > 0.0 {
                mask[rule.disabled_branch] = false;
            }
        }

        mask
    }

    /// 将扁平的动作编码数组解析为结构化动作树
    pub fn decode_tree(&self, encoded_action: &[f32]) -> Vec<ActionValueNode> {
        let mut offset = 0;
        let mut result = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let dim = node.encoding_dim();
            let slice = if offset < encoded_action.len() {
                &encoded_action[offset..(offset + dim).min(encoded_action.len())]
            } else {
                &[]
            };
            result.push(node.decode_value(slice));
            offset += dim;
        }
        result
    }

    pub fn from_legacy(action_space: &ActionSpace, labels: &[&str]) -> Self {
        let str_labels: Vec<String> = labels.iter().map(|&s| s.to_string()).collect();
        match action_space {
            ActionSpace::Discrete(_) => {
                Self::new(vec![ActionNode::categorical("action", str_labels)])
            }
            ActionSpace::Continuous(dim) => Self::new(vec![ActionNode::continuous("action", *dim)]),
            ActionSpace::Hybrid {
                continuous_dims, ..
            } => Self::new(vec![
                ActionNode::continuous("offset", *continuous_dims),
                ActionNode::categorical("action_type", str_labels),
            ]),
        }
    }

    pub fn num_branches(&self) -> usize {
        self.flat_branches().len()
    }

    pub fn flat_branches(&self) -> Vec<&ActionNode> {
        let mut branches = Vec::new();
        for node in &self.nodes {
            Self::collect_branches(node, &mut branches);
        }
        branches
    }

    fn collect_branches<'a>(node: &'a ActionNode, branches: &mut Vec<&'a ActionNode>) {
        match node {
            ActionNode::Struct { fields, .. } => {
                for field in fields {
                    Self::collect_branches(field, branches);
                }
            }
            _ => branches.push(node),
        }
    }
}

/// 因式分解的动作掩码（每个叶子分支独立的有效性过滤）
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ActionMasks {
    /// 每个叶子分支的掩码。
    /// Categorical → Some(Vec<bool>), UnitSelection → Some(Vec<bool>), Continuous → None
    pub branch_masks: Vec<Option<Vec<bool>>>,
    /// 自回归条件目标动作掩码矩阵（目标维度 -> 动作类别维度有效性布尔切片）
    /// conditional_target_masks.as_ref()[target_idx] 对应选中目标 target_idx 时 action_type 的合法动作掩码
    pub conditional_target_masks: Option<Vec<Vec<bool>>>,
}

impl ActionMasks {
    pub fn new(branch_masks: Vec<Option<Vec<bool>>>) -> Self {
        Self {
            branch_masks,
            conditional_target_masks: None,
        }
    }

    pub fn with_conditional_target_masks(
        branch_masks: Vec<Option<Vec<bool>>>,
        conditional_target_masks: Vec<Vec<bool>>,
    ) -> Self {
        Self {
            branch_masks,
            conditional_target_masks: Some(conditional_target_masks),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_schema_encoding() {
        // 模拟 SoloV0 的动作空间用新 AST 表达
        let schema = ActionSchema::new(vec![
            ActionNode::continuous("offset", 2),
            ActionNode::categorical(
                "action_type",
                vec![
                    "NoOp", "Move", "Attack", "CastQ", "CastW", "CastE", "CastR", "Flash",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            ),
        ]);
        assert_eq!(schema.encoding_dim(), 3); // 2 continuous + 1 categorical index
        assert_eq!(schema.to_encoding_labels().len(), 3);
    }

    #[test]
    fn test_action_schema_with_unit_selection() {
        let schema = ActionSchema::new(vec![
            ActionNode::categorical(
                "action_type",
                vec!["NoOp", "Move", "Attack", "CastQ"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            ),
            ActionNode::unit_selection("target", 16, 32, "visible_units"),
            ActionNode::continuous("offset", 2),
        ]);
        assert_eq!(schema.encoding_dim(), 4); // 1 + 1 + 2
        assert_eq!(schema.num_branches(), 3);
    }

    #[test]
    fn test_action_schema_from_legacy() {
        let hybrid = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 8,
        };
        let labels: Vec<&str> = vec![
            "NoOp", "Move", "Attack", "CastQ", "CastW", "CastE", "CastR", "Flash",
        ];
        let schema = ActionSchema::from_legacy(&hybrid, &labels);
        assert_eq!(schema.encoding_dim(), 3); // 2 + 1
        assert_eq!(schema.num_branches(), 2);
    }

    #[test]
    fn test_action_schema_struct() {
        let schema = ActionSchema::new(vec![
            ActionNode::structure(
                "combat",
                vec![
                    ActionNode::categorical("action_type", vec!["Attack".into(), "Spell".into()]),
                    ActionNode::unit_selection("target", 8, 16, "enemies"),
                ],
            ),
            ActionNode::continuous("move_dir", 2),
        ]);
        assert_eq!(schema.encoding_dim(), 4); // 1+1+2
        let branches = schema.flat_branches();
        assert_eq!(branches.len(), 3); // action_type, target, move_dir

        let encoded = vec![1.0, 3.0, 0.5, -0.5];
        let tree = schema.decode_tree(&encoded);
        assert_eq!(tree.len(), 2);
    }
}
