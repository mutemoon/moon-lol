use winnow::Parser;
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated};
use winnow::error::ContextError;

use crate::dsl::common::{PResult, ident, number_f32, number_usize, symbol, ws};
use crate::dsl::expr_parser::parse_obs_expr;
use crate::obs::{EntityEncoderSpec, ObsNode, ObsSchema, PoolType};

/// 解析分类节点: `category role: 4 -> embed(12) = role_id;`
fn parse_category_node<'i>(input: &mut &'i str) -> PResult<ObsNode, ContextError> {
    preceded(
        symbol("category"),
        (
            ident,
            preceded(symbol(":"), number_usize),
            opt(preceded(
                symbol("->"),
                delimited((symbol("embed"), symbol("(")), number_usize, symbol(")")),
            )),
            opt(preceded(symbol("="), parse_obs_expr)),
            symbol(";"),
        ),
    )
    .map(|(name, num_classes, embed_dim, expr, _)| {
        let dim = embed_dim.unwrap_or(num_classes);
        if let Some(e) = expr {
            ObsNode::categorical_expr(name, e, num_classes, dim)
        } else {
            ObsNode::categorical(name, num_classes, dim)
        }
    })
    .parse_next(input)
}

/// 解析标量节点: `scalar distance = distance / 100.0;`
fn parse_scalar_node<'i>(input: &mut &'i str) -> PResult<ObsNode, ContextError> {
    preceded(
        symbol("scalar"),
        (
            ident,
            opt(preceded(
                symbol("in"),
                delimited(
                    symbol("["),
                    (number_f32, symbol(","), number_f32),
                    symbol("]"),
                ),
            )),
            opt(preceded(symbol("="), parse_obs_expr)),
            symbol(";"),
        ),
    )
    .map(|(name, range, expr, _)| {
        let (min, max) = range.map(|(min, _, max)| (min, max)).unwrap_or((0.0, 1.0));
        if let Some(e) = expr {
            ObsNode::scalar_expr(name, e)
        } else {
            ObsNode::scalar(name, min, max)
        }
    })
    .parse_next(input)
}

/// 解析向量节点: `vector target_rel_pos: 2 = [(x - ox) / 100.0, (z - oz) / 100.0];`
fn parse_vector_node<'i>(input: &mut &'i str) -> PResult<ObsNode, ContextError> {
    preceded(
        symbol("vector"),
        (
            ident,
            opt(preceded(symbol(":"), number_usize)),
            opt(preceded(
                symbol("="),
                delimited(
                    symbol("["),
                    separated(1.., parse_obs_expr, symbol(",")),
                    symbol("]"),
                ),
            )),
            symbol(";"),
        ),
    )
    .map(|(name, explicit_dim, exprs, _)| {
        if let Some(expr_list) = exprs {
            ObsNode::vector_exprs(name, expr_list)
        } else {
            let dim = explicit_dim.unwrap_or(1);
            ObsNode::vector(name, dim)
        }
    })
    .parse_next(input)
}

/// 解析结构体节点: `struct spatial { ... }`
fn parse_struct_node<'i>(input: &mut &'i str) -> PResult<ObsNode, ContextError> {
    preceded(
        symbol("struct"),
        (
            ident,
            delimited(symbol("{"), repeat(0.., parse_obs_node), symbol("}")),
        ),
    )
    .map(|(name, fields)| ObsNode::structure(name, fields))
    .parse_next(input)
}

/// 解析隐藏层数组: `[32, 16]`
fn parse_hidden_dims<'i>(input: &mut &'i str) -> PResult<Vec<usize>, ContextError> {
    delimited(
        symbol("["),
        separated(0.., number_usize, symbol(",")),
        symbol("]"),
    )
    .parse_next(input)
}

/// 解析池化方式: `Max` 或 `Mean`
fn parse_pool_type<'i>(input: &mut &'i str) -> PResult<PoolType, ContextError> {
    alt((
        symbol("Max").value(PoolType::Max),
        symbol("Mean").value(PoolType::Mean),
        symbol("max").value(PoolType::Max),
        symbol("mean").value(PoolType::Mean),
    ))
    .parse_next(input)
}

/// 解析实体编码器规范: `-> encoder: SharedMlpPool(hidden=[32, 16], pool=Max)`
fn parse_encoder_spec<'i>(input: &mut &'i str) -> PResult<EntityEncoderSpec, ContextError> {
    preceded(
        (symbol("->"), symbol("encoder"), symbol(":")),
        alt((
            // SharedMlpPool
            delimited(
                (symbol("SharedMlpPool"), symbol("(")),
                (
                    preceded((symbol("hidden"), symbol("=")), parse_hidden_dims),
                    opt(preceded(
                        (symbol(","), symbol("pool"), symbol("=")),
                        parse_pool_type,
                    )),
                ),
                symbol(")"),
            )
            .map(|(hidden, pool)| EntityEncoderSpec::SharedMlpPool {
                hidden_dims: hidden,
                pool_type: pool.unwrap_or(PoolType::Max),
            }),
            // SharedMlpFlatten
            delimited(
                (symbol("SharedMlpFlatten"), symbol("(")),
                preceded((symbol("hidden"), symbol("=")), parse_hidden_dims),
                symbol(")"),
            )
            .map(|hidden| EntityEncoderSpec::SharedMlpFlatten {
                hidden_dims: hidden,
            }),
            // PassThrough
            symbol("PassThrough").value(EntityEncoderSpec::PassThrough),
        )),
    )
    .parse_next(input)
}

/// 解析重复实体列表节点: `repeated visible_units[20] -> encoder: ... { ... }`
fn parse_repeated_node<'i>(input: &mut &'i str) -> PResult<ObsNode, ContextError> {
    preceded(
        symbol("repeated"),
        (
            ident,
            delimited(symbol("["), number_usize, symbol("]")),
            opt(parse_encoder_spec),
            delimited(symbol("{"), repeat(0.., parse_obs_node), symbol("}")),
        ),
    )
    .map(
        |(name, max_count, encoder, fields): (
            String,
            usize,
            Option<EntityEncoderSpec>,
            Vec<ObsNode>,
        )| {
            let item_node = if fields.len() == 1 {
                fields.into_iter().next().unwrap()
            } else {
                ObsNode::structure("item", fields)
            };
            let encoder_spec = encoder.unwrap_or_else(|| EntityEncoderSpec::SharedMlpFlatten {
                hidden_dims: vec![16],
            });
            ObsNode::repeated(name, max_count, item_node, encoder_spec)
        },
    )
    .parse_next(input)
}

/// 解析任意单个 Obs 节点
pub fn parse_obs_node<'i>(input: &mut &'i str) -> PResult<ObsNode, ContextError> {
    alt((
        parse_category_node,
        parse_scalar_node,
        parse_vector_node,
        parse_struct_node,
        parse_repeated_node,
    ))
    .parse_next(input)
}

/// 解析顶级 obs 块: `obs SoloV0Obs { ... }`
pub fn parse_obs_schema<'i>(input: &mut &'i str) -> PResult<(String, ObsSchema), ContextError> {
    ws.parse_next(input)?;
    preceded(
        symbol("obs"),
        (
            ident,
            delimited(symbol("{"), repeat(0.., parse_obs_node), symbol("}")),
        ),
    )
    .map(|(name, nodes)| (name, ObsSchema::new(nodes)))
    .parse_next(input)
}
