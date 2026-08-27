use crate::dsl::common::{ident, string_literal, symbol, ws, PResult};
use crate::dsl::expr_parser::parse_reward_expr;
use crate::reward::{RewardFormulaSpec, RewardTermSpec};
use winnow::combinator::{delimited, opt, preceded, repeat};
use winnow::error::ContextError;
use winnow::Parser;

/// 解析单条奖励分项: `term vital_hit: "击破破绽" = 80.0 * is_vital_break;`
fn parse_reward_term_node<'i>(input: &mut &'i str) -> PResult<RewardTermSpec, ContextError> {
    preceded(symbol("term"), (
        ident,
        opt(preceded(symbol(":"), string_literal)),
        preceded(symbol("="), parse_reward_expr),
        symbol(";"),
    ))
    .map(|(id, label, expr, _)| {
        let final_label = label.unwrap_or_else(|| id.clone());
        RewardTermSpec::new(id, final_label, expr)
    })
    .parse_next(input)
}

/// 解析顶级 reward 块: `reward SoloV0Reward { ... }`
pub fn parse_reward_formula<'i>(input: &mut &'i str) -> PResult<RewardFormulaSpec, ContextError> {
    ws.parse_next(input)?;
    preceded(symbol("reward"), (
        ident,
        delimited(
            symbol("{"),
            repeat(0.., parse_reward_term_node),
            symbol("}"),
        ),
    ))
    .map(|(name, terms)| RewardFormulaSpec { name, terms })
    .parse_next(input)
}
