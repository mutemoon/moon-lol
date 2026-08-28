use winnow::Parser;
use winnow::combinator::{alt, delimited, opt, separated_pair};
use winnow::error::ContextError;

use crate::dsl::common::{PResult, ident, number_f32, symbol, var_ident};
use crate::obs::ObsExpr;
use crate::reward::RewardExpr;

// ── ObsExpr 解析 ─────────────────────────────────────────────────────────────

/// 解析初等 Obs 表达式（常数、括号、函数调用、变量）
fn parse_obs_primary<'i>(input: &mut &'i str) -> PResult<ObsExpr, ContextError> {
    alt((
        // 函数调用: clamp(expr, min, max)
        delimited(
            (symbol("clamp"), symbol("(")),
            (
                parse_obs_expr,
                symbol(","),
                number_f32,
                symbol(","),
                number_f32,
            ),
            symbol(")"),
        )
        .map(|(e, _, min, _, max)| ObsExpr::clamp(e, min, max)),
        // 函数调用: max(a, b)
        delimited(
            (symbol("max"), symbol("(")),
            separated_pair(parse_obs_expr, symbol(","), parse_obs_expr),
            symbol(")"),
        )
        .map(|(a, b)| ObsExpr::max(a, b)),
        // 函数调用: min(a, b)
        delimited(
            (symbol("min"), symbol("(")),
            separated_pair(parse_obs_expr, symbol(","), parse_obs_expr),
            symbol(")"),
        )
        .map(|(a, b)| ObsExpr::min(a, b)),
        // 函数调用: if(cond, then, else)
        delimited(
            (symbol("if"), symbol("(")),
            (
                parse_obs_expr,
                symbol(","),
                parse_obs_expr,
                symbol(","),
                parse_obs_expr,
            ),
            symbol(")"),
        )
        .map(|(cond, _, then_e, _, else_e)| ObsExpr::if_else(cond, then_e, else_e)),
        // 括号表达式: (expr)
        delimited(symbol("("), parse_obs_expr, symbol(")")),
        // 浮点数常数
        number_f32.map(ObsExpr::c),
        // 变量名
        var_ident.map(ObsExpr::var),
    ))
    .parse_next(input)
}

/// 解析一元负号: -expr
fn parse_obs_unary<'i>(input: &mut &'i str) -> PResult<ObsExpr, ContextError> {
    let minus = opt(symbol("-")).parse_next(input)?;
    let expr = parse_obs_primary.parse_next(input)?;
    if minus.is_some() { Ok(-expr) } else { Ok(expr) }
}

/// 解析乘除法项: expr * expr, expr / expr
fn parse_obs_term<'i>(input: &mut &'i str) -> PResult<ObsExpr, ContextError> {
    let mut left = parse_obs_unary.parse_next(input)?;

    while let Ok(op) = alt((symbol("*"), symbol("/"))).parse_next(input) {
        let right = parse_obs_unary.parse_next(input)?;
        left = match op {
            "*" => left * right,
            "/" => left / right,
            _ => unreachable!(),
        };
    }

    Ok(left)
}

/// 解析加减法表达式: term + term, term - term
fn parse_obs_arithmetic<'i>(input: &mut &'i str) -> PResult<ObsExpr, ContextError> {
    let mut left = parse_obs_term.parse_next(input)?;

    while let Ok(op) = alt((symbol("+"), symbol("-"))).parse_next(input) {
        let right = parse_obs_term.parse_next(input)?;
        left = match op {
            "+" => left + right,
            "-" => left - right,
            _ => unreachable!(),
        };
    }

    Ok(left)
}

/// 解析比较运算: a > b, a < b
fn parse_obs_comparison<'i>(input: &mut &'i str) -> PResult<ObsExpr, ContextError> {
    let left = parse_obs_arithmetic.parse_next(input)?;

    if let Ok(op) = alt((symbol(">"), symbol("<"))).parse_next(input) {
        let right = parse_obs_arithmetic.parse_next(input)?;
        match op {
            ">" => Ok(ObsExpr::gt(left, right)),
            "<" => Ok(ObsExpr::lt(left, right)),
            _ => unreachable!(),
        }
    } else {
        Ok(left)
    }
}

/// 解析完整 ObsExpr（包含三元表达式与顶级组合）
pub fn parse_obs_expr<'i>(input: &mut &'i str) -> PResult<ObsExpr, ContextError> {
    parse_obs_comparison.parse_next(input)
}

// ── RewardExpr 解析 ──────────────────────────────────────────────────────────

/// 解析初等 Reward 表达式
fn parse_reward_primary<'i>(input: &mut &'i str) -> PResult<RewardExpr, ContextError> {
    alt((
        // 函数调用: clamp(expr, min, max)
        delimited(
            (symbol("clamp"), symbol("(")),
            (
                parse_reward_expr,
                symbol(","),
                number_f32,
                symbol(","),
                number_f32,
            ),
            symbol(")"),
        )
        .map(|(e, _, min, _, max)| RewardExpr::clamp(e, min, max)),
        // 函数调用: exp(expr)
        delimited((symbol("exp"), symbol("(")), parse_reward_expr, symbol(")"))
            .map(RewardExpr::exp),
        // 函数调用: if(cond, then, else)
        delimited(
            (symbol("if"), symbol("(")),
            (
                parse_reward_expr,
                symbol(","),
                parse_reward_expr,
                symbol(","),
                parse_reward_expr,
            ),
            symbol(")"),
        )
        .map(|(cond, _, then_e, _, else_e)| RewardExpr::if_else(cond, then_e, else_e)),
        // 括号表达式
        delimited(symbol("("), parse_reward_expr, symbol(")")),
        // 浮点数常数
        number_f32.map(RewardExpr::Constant),
        // 变量名
        ident.map(RewardExpr::Variable),
    ))
    .parse_next(input)
}

/// 解析一元负号
fn parse_reward_unary<'i>(input: &mut &'i str) -> PResult<RewardExpr, ContextError> {
    let minus = opt(symbol("-")).parse_next(input)?;
    let expr = parse_reward_primary.parse_next(input)?;
    if minus.is_some() {
        Ok(RewardExpr::Mul(
            Box::new(RewardExpr::Constant(-1.0)),
            Box::new(expr),
        ))
    } else {
        Ok(expr)
    }
}

/// 解析乘除法
fn parse_reward_term<'i>(input: &mut &'i str) -> PResult<RewardExpr, ContextError> {
    let mut left = parse_reward_unary.parse_next(input)?;

    while let Ok(op) = alt((symbol("*"), symbol("/"))).parse_next(input) {
        let right = parse_reward_unary.parse_next(input)?;
        left = match op {
            "*" => RewardExpr::Mul(Box::new(left), Box::new(right)),
            "/" => RewardExpr::Div(Box::new(left), Box::new(right)),
            _ => unreachable!(),
        };
    }

    Ok(left)
}

/// 解析加减法
fn parse_reward_arithmetic<'i>(input: &mut &'i str) -> PResult<RewardExpr, ContextError> {
    let mut left = parse_reward_term.parse_next(input)?;

    while let Ok(op) = alt((symbol("+"), symbol("-"))).parse_next(input) {
        let right = parse_reward_term.parse_next(input)?;
        left = match op {
            "+" => RewardExpr::Add(Box::new(left), Box::new(right)),
            "-" => RewardExpr::Sub(Box::new(left), Box::new(right)),
            _ => unreachable!(),
        };
    }

    Ok(left)
}

/// 解析比较运算
fn parse_reward_comparison<'i>(input: &mut &'i str) -> PResult<RewardExpr, ContextError> {
    let left = parse_reward_arithmetic.parse_next(input)?;

    if let Ok(op) = alt((symbol(">"), symbol("<"))).parse_next(input) {
        let right = parse_reward_arithmetic.parse_next(input)?;
        match op {
            ">" => Ok(RewardExpr::Gt(Box::new(left), Box::new(right))),
            "<" => Ok(RewardExpr::Lt(Box::new(left), Box::new(right))),
            _ => unreachable!(),
        }
    } else {
        Ok(left)
    }
}

/// 解析完整 RewardExpr
pub fn parse_reward_expr<'i>(input: &mut &'i str) -> PResult<RewardExpr, ContextError> {
    parse_reward_comparison.parse_next(input)
}
