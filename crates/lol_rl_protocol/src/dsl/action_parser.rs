use winnow::Parser;
use winnow::combinator::{
    alt, delimited, opt, preceded, repeat, separated, separated_pair, terminated,
};
use winnow::error::ContextError;

use crate::action::{ActionMaskRule, ActionNode, ActionSchema};
use crate::dsl::common::{PResult, ident, number_usize, string_literal, symbol, ws};
use crate::dsl::expr_parser::parse_obs_expr;
use crate::obs::ObsExpr;

/// 解析连续高斯动作: `continuous offset: 2;`
fn parse_continuous_node<'i>(input: &mut &'i str) -> PResult<ActionNode, ContextError> {
    preceded(
        symbol("continuous"),
        (ident, preceded(symbol(":"), number_usize), symbol(";")),
    )
    .map(|(name, dim, _)| ActionNode::continuous(name, dim))
    .parse_next(input)
}

/// 解析单位选择动作: `unit_target target: visible_units[20 -> 16];` 或 `unit_target target;`
fn parse_unit_selection_node<'i>(input: &mut &'i str) -> PResult<ActionNode, ContextError> {
    preceded(
        symbol("unit_target"),
        (
            ident,
            opt(preceded(
                symbol(":"),
                (
                    ident,
                    opt(delimited(
                        symbol("["),
                        (number_usize, opt(preceded(symbol("->"), number_usize))),
                        symbol("]"),
                    )),
                ),
            )),
            symbol(";"),
        ),
    )
    .map(|(name, details, _)| {
        let (entity_name, max_units, embed_dim) = if let Some((ename, shape)) = details {
            let (max_u, edim) = shape.map(|(m, e)| (m, e.unwrap_or(16))).unwrap_or((20, 16));
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
    preceded(
        symbol("category"),
        (
            ident,
            opt(preceded(symbol(":"), number_usize)),
            alt((
                // 花括号分支表: { 0: "NoOp", 1: "Move", ... }
                delimited(symbol("{"), repeat(0.., parse_category_branch), symbol("}")).map(
                    |branches: Vec<(usize, String)>| {
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
                    },
                ),
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
        ),
    )
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
    preceded(
        symbol("struct"),
        (
            ident,
            delimited(symbol("{"), repeat(0.., parse_action_node), symbol("}")),
        ),
    )
    .map(|(name, fields)| ActionNode::structure(name, fields))
    .parse_next(input)
}

/// 解析目标引用: `action_type.Attack` 或 `Attack` 或 `"攻击瑞雯"`
fn parse_target_ref<'i>(input: &mut &'i str) -> PResult<(Option<String>, String), ContextError> {
    alt((
        separated_pair(
            ident,
            symbol("."),
            alt((string_literal, ident, number_usize.map(|n| n.to_string()))),
        )
        .map(|(h, b)| (Some(h), b)),
        string_literal.map(|b| (None, b)),
        ident.map(|b| (None, b)),
        number_usize.map(|n| (None, n.to_string())),
    ))
    .parse_next(input)
}

/// 解析单条掩码规则: `if distance > 22.0 { disable Attack; }` 或 `if distance > 22.0 { disable 2; }`
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

enum RawMaskEntry {
    Global(Vec<(ObsExpr, Option<String>, String)>),
    EntityLoop {
        entity_name: String,
        rules: Vec<(ObsExpr, Option<String>, String)>,
    },
}

fn parse_entity_loop<'i>(input: &mut &'i str) -> PResult<RawMaskEntry, ContextError> {
    preceded(
        symbol("for"),
        (
            ident,
            preceded(symbol("in"), ident),
            delimited(
                symbol("{"),
                repeat(0.., parse_mask_rule_entry)
                    .map(|v: Vec<Vec<_>>| v.into_iter().flatten().collect()),
                symbol("}"),
            ),
        ),
    )
    .map(|(_loop_var, entity_name, rules)| RawMaskEntry::EntityLoop {
        entity_name,
        rules,
    })
    .parse_next(input)
}

fn parse_raw_mask_entry<'i>(input: &mut &'i str) -> PResult<RawMaskEntry, ContextError> {
    alt((parse_entity_loop, parse_mask_rule_entry.map(RawMaskEntry::Global))).parse_next(input)
}

/// 解析 `mask { ... }` 块
fn parse_mask_block<'i>(input: &mut &'i str) -> PResult<Vec<RawMaskEntry>, ContextError> {
    preceded(
        symbol("mask"),
        delimited(symbol("{"), repeat(0.., parse_raw_mask_entry), symbol("}")),
    )
    .parse_next(input)
}

fn resolve_branch_index(
    nodes: &[ActionNode],
    head: &Option<String>,
    branch_name: &str,
) -> Option<usize> {
    if let Ok(idx) = branch_name.parse::<usize>() {
        return Some(idx);
    }
    for node in nodes {
        if let ActionNode::Categorical {
            name: node_name,
            labels,
            ..
        } = node
        {
            if head.as_ref().map_or(true, |h| h == node_name) {
                if let Some(idx) = labels.iter().position(|l| l == branch_name) {
                    return Some(idx);
                }
                if let Some(idx) = labels.iter().position(|l| {
                    l.contains(&format!("({})", branch_name))
                        || l.contains(&format!(" {} ", branch_name))
                        || l.ends_with(&format!(" {}", branch_name))
                        || l.starts_with(&format!("{} ", branch_name))
                }) {
                    return Some(idx);
                }
                if let Some(idx) = labels.iter().position(|l| l.contains(branch_name)) {
                    return Some(idx);
                }
            }
        }
    }
    None
}

fn is_unit_selection_head(nodes: &[ActionNode], name: &str) -> bool {
    nodes.iter().any(|node| match node {
        ActionNode::UnitSelection { name: n, .. } => n == name,
        _ => false,
    })
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
pub fn parse_action_schema<'i>(
    input: &mut &'i str,
) -> PResult<(String, ActionSchema), ContextError> {
    ws.parse_next(input)?;
    preceded(
        symbol("action"),
        (
            ident,
            delimited(
                symbol("{"),
                (repeat(0.., parse_action_node), opt(parse_mask_block)),
                symbol("}"),
            ),
        ),
    )
    .map(
        |(name, (nodes, raw_masks)): (String, (Vec<ActionNode>, Option<Vec<RawMaskEntry>>))| {
            let mut mask_rules = Vec::new();
            if let Some(masks) = raw_masks {
                for raw_entry in masks {
                    match raw_entry {
                        RawMaskEntry::Global(entries) => {
                            for (cond, head, branch_name) in entries {
                                if let Some(idx) =
                                    resolve_branch_index(&nodes, &head, &branch_name)
                                {
                                    mask_rules.push(ActionMaskRule::Global {
                                        condition: cond,
                                        target_head: head,
                                        disabled_branch: idx,
                                        branch_label: branch_name,
                                    });
                                }
                            }
                        }
                        RawMaskEntry::EntityLoop {
                            entity_name, rules, ..
                        } => {
                            for (cond, head, target_or_branch) in rules {
                                if is_unit_selection_head(&nodes, &target_or_branch) {
                                    mask_rules.push(ActionMaskRule::EntitySlot {
                                        entity_name: entity_name.clone(),
                                        condition: cond,
                                        target_head: Some(target_or_branch),
                                    });
                                } else if let Some(idx) =
                                    resolve_branch_index(&nodes, &head, &target_or_branch)
                                {
                                    mask_rules.push(ActionMaskRule::ConditionalTarget {
                                        entity_name: entity_name.clone(),
                                        condition: cond,
                                        target_head: head,
                                        disabled_branch: idx,
                                        branch_label: target_or_branch,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            (name, ActionSchema::new(nodes).with_mask_rules(mask_rules))
        },
    )
    .parse_next(input)
}
