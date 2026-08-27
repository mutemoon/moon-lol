use crate::action::{ActionMaskRule, ActionNode, ActionSchema};
use crate::dsl::common::{ident, number_usize, string_literal, symbol, ws, PResult};
use crate::dsl::expr_parser::parse_obs_expr;
use crate::obs::ObsExpr;
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated, separated_pair, terminated};
use winnow::error::ContextError;
use winnow::Parser;

/// 解析连续高斯动作: `continuous offset: 2;`
fn parse_continuous_node<'i>(input: &mut &'i str) -> PResult<ActionNode, ContextError> {
    preceded(symbol("continuous"), (
        ident,
        preceded(symbol(":"), number_usize),
        symbol(";"),
    ))
    .map(|(name, dim, _)| ActionNode::continuous(name, dim))
    .parse_next(input)
}

/// 解析单位选择动作: `unit_target target: visible_units[20 -> 16];` 或 `unit_target target;`
fn parse_unit_selection_node<'i>(input: &mut &'i str) -> PResult<ActionNode, ContextError> {
    preceded(symbol("unit_target"), (
        ident,
        opt(preceded(
            symbol(":"),
            (
                ident,
                opt(delimited(
                    symbol("["),
                    (
                        number_usize,
                        opt(preceded(symbol("->"), number_usize)),
                    ),
                    symbol("]"),
                )),
            ),
        )),
        symbol(";"),
    ))
    .map(|(name, details, _)| {
        let (entity_name, max_units, embed_dim) = if let Some((ename, shape)) = details {
            let (max_u, edim) = shape
                .map(|(m, e)| (m, e.unwrap_or(16)))
                .unwrap_or((20, 16));
            (ename, max_u, edim)
        } else {
            ("visible_units".to_string(), 20, 16)
        };
        ActionNode::unit_selection(name, max_units, embed_dim, entity_name)
    })
    .parse_next(input)
}

/// 解析单条离散分支: `0: "NoOp"` 或 `0: NoOp`
fn parse_category_branch<'i>(input: &mut &'i str) -> PResult<(usize, String), ContextError> {
    (
        number_usize,
        symbol(":"),
        alt((string_literal, ident)),
        opt(symbol(",")),
    )
        .map(|(id, _, label, _)| (id, label))
        .parse_next(input)
}

/// 解析分类动作: `category action_type: 8 { 0: "NoOp", 1: "Move", ... }`
fn parse_category_node<'i>(input: &mut &'i str) -> PResult<ActionNode, ContextError> {
    preceded(symbol("category"), (
        ident,
        opt(preceded(symbol(":"), number_usize)),
        alt((
            // 花括号分支表: { 0: "NoOp", 1: "Move", ... }
            delimited(
                symbol("{"),
                repeat(0.., parse_category_branch),
                symbol("}"),
            )
            .map(|branches: Vec<(usize, String)>| {
                if branches.is_empty() {
                    Vec::new()
                } else {
                    let max_idx = branches.iter().map(|(i, _)| *i).max().unwrap_or(0);
                    let mut labels = vec!["Unknown".to_string(); max_idx + 1];
                    for (i, label) in branches {
                        labels[i] = label;
                    }
                    labels
                }
            }),
            // 数组简写: ["NoOp", "Move", ...]
            terminated(
                delimited(
                    symbol("["),
                    separated(0.., alt((string_literal, ident)), symbol(",")),
                    symbol("]"),
                ),
                symbol(";"),
            ),
        )),
    ))
    .map(|(name, num_classes, labels)| {
        let label_count = labels.len();
        let classes = num_classes.unwrap_or(label_count);
        let mut final_labels = labels;
        if final_labels.len() < classes {
            for i in final_labels.len()..classes {
                final_labels.push(format!("Action_{}", i));
            }
        }
        ActionNode::categorical(name, final_labels)
    })
    .parse_next(input)
}

/// 解析复合动作结构体: `struct sub_action { ... }`
fn parse_struct_node<'i>(input: &mut &'i str) -> PResult<ActionNode, ContextError> {
    preceded(symbol("struct"), (
        ident,
        delimited(
            symbol("{"),
            repeat(0.., parse_action_node),
            symbol("}"),
        ),
    ))
    .map(|(name, fields)| ActionNode::structure(name, fields))
    .parse_next(input)
}

/// 解析目标引用: `action_type.Attack` 或 `Attack`
fn parse_target_ref<'i>(input: &mut &'i str) -> PResult<(Option<String>, String), ContextError> {
    alt((
        separated_pair(ident, symbol("."), ident).map(|(h, b)| (Some(h), b)),
        ident.map(|b| (None, b)),
    ))
    .parse_next(input)
}

/// 解析单条掩码规则: `if distance > 22.0 { disable Attack; }`
fn parse_mask_rule_entry<'i>(
    input: &mut &'i str,
) -> PResult<Vec<(ObsExpr, Option<String>, String)>, ContextError> {
    preceded(
        symbol("if"),
        (
            parse_obs_expr,
            delimited(
                symbol("{"),
                preceded(
                    symbol("disable"),
                    alt((
                        delimited(
                            symbol("["),
                            separated(1.., parse_target_ref, symbol(",")),
                            symbol("]"),
                        ),
                        parse_target_ref.map(|r| vec![r]),
                    )),
                ),
                (opt(symbol(";")), symbol("}")),
            ),
        ),
    )
    .map(|(cond, targets)| {
        targets
            .into_iter()
            .map(|(head, branch)| (cond.clone(), head, branch))
            .collect()
    })
    .parse_next(input)
}

/// 解析 `mask { ... }` 块
fn parse_mask_block<'i>(
    input: &mut &'i str,
) -> PResult<Vec<(ObsExpr, Option<String>, String)>, ContextError> {
    preceded(
        symbol("mask"),
        delimited(
            symbol("{"),
            repeat(0.., parse_mask_rule_entry).map(|v: Vec<Vec<_>>| v.into_iter().flatten().collect()),
            symbol("}"),
        ),
    )
    .parse_next(input)
}

/// 解析任意单个 Action 节点
pub fn parse_action_node<'i>(input: &mut &'i str) -> PResult<ActionNode, ContextError> {
    alt((
        parse_continuous_node,
        parse_unit_selection_node,
        parse_category_node,
        parse_struct_node,
    ))
    .parse_next(input)
}

/// 解析顶级 action 块: `action SoloV0Action { ... }`
pub fn parse_action_schema<'i>(input: &mut &'i str) -> PResult<(String, ActionSchema), ContextError> {
    ws.parse_next(input)?;
    preceded(symbol("action"), (
        ident,
        delimited(
            symbol("{"),
            (
                repeat(0.., parse_action_node),
                opt(parse_mask_block),
            ),
            symbol("}"),
        ),
    ))
    .map(|(name, (nodes, raw_masks)): (String, (Vec<ActionNode>, Option<Vec<_>>))| {
        let mut mask_rules = Vec::new();
        if let Some(masks) = raw_masks {
            for (cond, head, branch_name) in masks {
                // 查找对应 Category 节点的分支索引
                let mut resolved_idx = None;
                for node in nodes.iter() {
                    if let ActionNode::Categorical { name: node_name, labels, .. } = node {
                        if head.as_ref().map_or(true, |h| h.as_str() == node_name.as_str()) {
                            if let Some(idx) = labels.iter().position(|l| l == &branch_name) {
                                resolved_idx = Some(idx);
                                break;
                            }
                        }
                    }
                }

                if let Some(idx) = resolved_idx {
                    mask_rules.push(ActionMaskRule::new(cond, head, idx, branch_name));
                }
            }
        }

        (name, ActionSchema::new(nodes).with_mask_rules(mask_rules))
    })
    .parse_next(input)
}
