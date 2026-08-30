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

/// 声明式动作掩码规则
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ActionMaskRule {
    /// 全局标量规则：当 condition 在当前 ObsContext 下成立 (> 0.0) 时，在 target_head 中禁用指定分支
    Global {
        condition: ObsExpr,
        #[serde(default)]
        target_head: Option<String>,
        disabled_branch: usize,
        branch_label: String,
    },
    /// 实体槽位过滤规则：遍历 entity_name（如 "visible_units"），当 condition (> 0.0) 成立时禁用 target_head 对应槽位
    EntitySlot {
        entity_name: String,
        condition: ObsExpr,
        #[serde(default)]
        target_head: Option<String>,
    },
    /// 针对所选目标的条件动作规则：遍历 entity_name，为每个目标评估，若 condition (> 0.0) 成立则在 target_head 中禁用分支
    ConditionalTarget {
        entity_name: String,
        condition: ObsExpr,
        #[serde(default)]
        target_head: Option<String>,
        disabled_branch: usize,
        branch_label: String,
    },
}

impl ActionMaskRule {
    pub fn global(
        condition: ObsExpr,
        target_head: Option<String>,
        disabled_branch: usize,
        branch_label: impl Into<String>,
    ) -> Self {
        Self::Global {
            condition,
            target_head,
            disabled_branch,
            branch_label: branch_label.into(),
        }
    }

    pub fn entity_slot(
        entity_name: impl Into<String>,
        condition: ObsExpr,
        target_head: Option<String>,
    ) -> Self {
        Self::EntitySlot {
            entity_name: entity_name.into(),
            condition,
            target_head,
        }
    }

    pub fn conditional_target(
        entity_name: impl Into<String>,
        condition: ObsExpr,
        target_head: Option<String>,
        disabled_branch: usize,
        branch_label: impl Into<String>,
    ) -> Self {
        Self::ConditionalTarget {
            entity_name: entity_name.into(),
            condition,
            target_head,
            disabled_branch,
            branch_label: branch_label.into(),
        }
    }

    pub fn new(
        condition: ObsExpr,
        target_head: Option<String>,
        disabled_branch: usize,
        branch_label: impl Into<String>,
    ) -> Self {
        Self::global(condition, target_head, disabled_branch, branch_label)
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
            if let ActionMaskRule::Global {
                disabled_branch,
                condition,
                ..
            } = rule
            {
                if *disabled_branch < mask.len() && condition.eval(&ctx.vars) > 0.0 {
                    mask[*disabled_branch] = false;
                }
            }
        }

        mask
    }

    /// 统一求值：根据当前观测上下文，计算完整的 ActionMasks（包含各分支 branch_masks 与 conditional_target_masks）
    pub fn eval_action_masks(&self, ctx: &ObsContext) -> ActionMasks {
        let flat = self.flat_branches();
        let mut branch_masks = Vec::with_capacity(flat.len());
        let mut unit_selection_info: Option<(usize, String)> = None;
        let mut cat_baseline_mask: Option<Vec<bool>> = None;

        for node in &flat {
            match node {
                ActionNode::Continuous { .. } => {
                    branch_masks.push(None);
                }
                ActionNode::UnitSelection {
                    name,
                    max_units,
                    obs_entity_name,
                    ..
                } => {
                    unit_selection_info = Some((*max_units, obs_entity_name.clone()));
                    let mut slot_mask = vec![true; *max_units];
                    let repeated_units = ctx.repeated.get(obs_entity_name);

                    for i in 0..*max_units {
                        if i == 0 {
                            if let Some(unit) = repeated_units.and_then(|v| v.get(0)) {
                                let mut entity_vars = ctx.vars.clone();
                                for (k, v) in &unit.vars {
                                    entity_vars.insert(k.clone(), *v);
                                    entity_vars.insert(format!("u.{}", k), *v);
                                }
                                for rule in &self.mask_rules {
                                    if let ActionMaskRule::EntitySlot {
                                        entity_name,
                                        condition,
                                        target_head,
                                    } = rule
                                    {
                                        if entity_name == obs_entity_name
                                            && target_head
                                                .as_ref()
                                                .map_or(true, |h| h == name)
                                            && condition.eval(&entity_vars) > 0.0
                                        {
                                            slot_mask[i] = false;
                                            break;
                                        }
                                    }
                                }
                            }
                        } else if let Some(unit) = repeated_units.and_then(|v| v.get(i)) {
                            let mut entity_vars = ctx.vars.clone();
                            for (k, v) in &unit.vars {
                                entity_vars.insert(k.clone(), *v);
                                entity_vars.insert(format!("u.{}", k), *v);
                            }
                            let mut disabled = false;
                            for rule in &self.mask_rules {
                                if let ActionMaskRule::EntitySlot {
                                    entity_name,
                                    condition,
                                    target_head,
                                } = rule
                                {
                                    if entity_name == obs_entity_name
                                        && target_head
                                            .as_ref()
                                            .map_or(true, |h| h == name)
                                        && condition.eval(&entity_vars) > 0.0
                                    {
                                        disabled = true;
                                        break;
                                    }
                                }
                            }
                            slot_mask[i] = !disabled;
                        } else {
                            slot_mask[i] = false;
                        }
                    }
                    branch_masks.push(Some(slot_mask));
                }
                ActionNode::Categorical {
                    name,
                    num_classes,
                    ..
                } => {
                    let mut mask = vec![true; *num_classes];
                    for rule in &self.mask_rules {
                        if let ActionMaskRule::Global {
                            condition,
                            target_head,
                            disabled_branch,
                            ..
                        } = rule
                        {
                            if target_head.as_ref().map_or(true, |h| h == name)
                                && *disabled_branch < mask.len()
                                && condition.eval(&ctx.vars) > 0.0
                            {
                                mask[*disabled_branch] = false;
                            }
                        }
                    }
                    cat_baseline_mask = Some(mask.clone());
                    branch_masks.push(Some(mask));
                }
                ActionNode::Struct { .. } => unreachable!(),
            }
        }

        let conditional_target_masks = if let (Some((max_units, obs_entity_name)), Some(base_mask)) =
            (unit_selection_info, cat_baseline_mask)
        {
            let repeated_units = ctx.repeated.get(&obs_entity_name);
            let mut cond_masks = Vec::with_capacity(max_units);

            for i in 0..max_units {
                if i == 0 {
                    let mut t_mask = base_mask.clone();
                    if let Some(unit) = repeated_units.and_then(|v| v.get(0)) {
                        let mut entity_vars = ctx.vars.clone();
                        for (k, v) in &unit.vars {
                            entity_vars.insert(k.clone(), *v);
                            entity_vars.insert(format!("u.{}", k), *v);
                        }
                        for rule in &self.mask_rules {
                            if let ActionMaskRule::ConditionalTarget {
                                entity_name,
                                condition,
                                disabled_branch,
                                ..
                            } = rule
                            {
                                if entity_name == &obs_entity_name
                                    && *disabled_branch < t_mask.len()
                                    && condition.eval(&entity_vars) > 0.0
                                {
                                    t_mask[*disabled_branch] = false;
                                }
                            }
                        }
                    }
                    cond_masks.push(t_mask);
                } else if let Some(unit) = repeated_units.and_then(|v| v.get(i)) {
                    let mut t_mask = base_mask.clone();
                    let mut entity_vars = ctx.vars.clone();
                    for (k, v) in &unit.vars {
                        entity_vars.insert(k.clone(), *v);
                        entity_vars.insert(format!("u.{}", k), *v);
                    }
                    for rule in &self.mask_rules {
                        if let ActionMaskRule::ConditionalTarget {
                            entity_name,
                            condition,
                            disabled_branch,
                            ..
                        } = rule
                        {
                            if entity_name == &obs_entity_name
                                && *disabled_branch < t_mask.len()
                                && condition.eval(&entity_vars) > 0.0
                            {
                                t_mask[*disabled_branch] = false;
                            }
                        }
                    }
                    cond_masks.push(t_mask);
                } else {
                    let mut fallback = vec![false; base_mask.len()];
                    if !fallback.is_empty() {
                        fallback[0] = true;
                    }
                    if fallback.len() > 1 {
                        fallback[1] = true;
                    }
                    cond_masks.push(fallback);
                }
            }
            Some(cond_masks)
        } else {
            None
        };

        ActionMasks {
            branch_masks,
            conditional_target_masks,
        }
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
