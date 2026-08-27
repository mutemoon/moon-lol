use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RewardItem {
    pub name: String,
    pub value: f32,
}

#[cfg(test)]
mod tests {
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
}
