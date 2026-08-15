use std::collections::HashMap;

use lol_rl_protocol::{RewardExpr, RewardFormulaSpec, RewardItem, RewardTermSpec};

/// 统一的环境单步奖励模型 Trait
pub trait RewardModel: Send + Sync {
    type Context;

    /// 获取环境定义的结构化奖励公式（单一事实来源）
    fn formula_spec(&self) -> RewardFormulaSpec;

    /// 从当前环境步骤上下文中提取特征变量
    fn extract_variables(&self, ctx: &Self::Context) -> HashMap<String, f32>;

    /// 严格基于结构化表达式 AST 计算奖励与 Breakdown，返回 (总奖励, 细拆项, 环境变量字典)
    fn evaluate(&self, ctx: &Self::Context) -> (f32, Vec<RewardItem>, HashMap<String, f32>) {
        let vars = self.extract_variables(ctx);
        let (total, items) = self.formula_spec().compute(&vars);
        (total, items, vars)
    }
}

/// Fiora 对战环境单步奖励计算上下文
#[derive(Debug, Clone, Default)]
pub struct FioraRewardContext {
    pub prev_aligned: bool,
    pub curr_aligned: bool,
    pub is_vital_break: bool,
    pub is_attack: bool,
    pub prev_riven_hp: f32,
    pub curr_riven_hp: f32,
    pub elapsed_secs: f32,
}

/// Fiora vs Riven 环境的奖励模型实现
pub struct FioraVsRivenRewardModel;

impl RewardModel for FioraVsRivenRewardModel {
    type Context = FioraRewardContext;

    fn formula_spec(&self) -> RewardFormulaSpec {
        RewardFormulaSpec {
            name: "无双剑姬打破绽标准公式".to_string(),
            terms: vec![
                RewardTermSpec::new(
                    "time_penalty",
                    "时间惩罚 (Time Penalty)",
                    RewardExpr::Constant(-0.002),
                ),
                RewardTermSpec::new(
                    "alignment",
                    "对齐破绽方向 (Alignment Bonus)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(0.02)),
                        Box::new(RewardExpr::Variable("is_newly_aligned".into())),
                    ),
                ),
                RewardTermSpec::new(
                    "misalignment",
                    "错误方向移动 (Misalignment Penalty)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(-0.02)),
                        Box::new(RewardExpr::Variable("is_misaligned_move".into())),
                    ),
                ),
                RewardTermSpec::new(
                    "attack_miss",
                    "空挥攻击 (Attack Miss Penalty)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(-0.1)),
                        Box::new(RewardExpr::Variable("is_attack_missed".into())),
                    ),
                ),
                RewardTermSpec::new(
                    "vital_break",
                    "打破绽成功 (Vital Break)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(0.8)),
                        Box::new(RewardExpr::Variable("is_vital_break".into())),
                    ),
                ),
                RewardTermSpec::new(
                    "kill_reward",
                    "击杀基础奖励 (Kill Reward)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(2.0)),
                        Box::new(RewardExpr::Variable("is_kill".into())),
                    ),
                ),
                RewardTermSpec::new(
                    "quick_kill_bonus",
                    "极速击杀时效奖励 (Quick Kill Time Reward)",
                    RewardExpr::IfElse {
                        cond: Box::new(RewardExpr::Variable("is_kill".into())),
                        then_branch: Box::new(RewardExpr::Variable("quick_kill_reward".into())),
                        else_branch: Box::new(RewardExpr::Constant(0.0)),
                    },
                ),
            ],
        }
    }

    fn extract_variables(&self, ctx: &FioraRewardContext) -> HashMap<String, f32> {
        let mut vars = HashMap::new();
        let is_newly_aligned = if !ctx.prev_aligned && ctx.curr_aligned {
            1.0
        } else {
            0.0
        };
        let is_misaligned_move = if ctx.prev_aligned && !ctx.curr_aligned {
            1.0
        } else {
            0.0
        };
        let is_attack_missed = if ctx.is_attack && !ctx.is_vital_break {
            1.0
        } else {
            0.0
        };
        let is_vital_break = if ctx.is_vital_break { 1.0 } else { 0.0 };
        let is_kill = if ctx.curr_riven_hp <= 0.0 && ctx.prev_riven_hp > 0.0 {
            1.0
        } else {
            0.0
        };

        // 极速击杀时效奖励：越快越高，指数上升；4s 为零界限（>4s 严格为负）；接近 1s 时奖励达到 ~15.15（高于击杀基础分）
        let quick_kill_reward = if is_kill > 0.0 {
            let t = ctx.elapsed_secs.max(0.05);
            let exp_term = 3.0 * ((0.6 * (4.0 - t)).exp() - 1.0);
            let overtime_penalty = (t - 4.0).max(0.0) * 1.0;
            exp_term - overtime_penalty
        } else {
            0.0
        };

        vars.insert("is_vital_break".into(), is_vital_break);
        vars.insert("is_kill".into(), is_kill);
        vars.insert("is_newly_aligned".into(), is_newly_aligned);
        vars.insert("is_misaligned_move".into(), is_misaligned_move);
        vars.insert("is_attack_missed".into(), is_attack_missed);
        vars.insert("quick_kill_reward".into(), quick_kill_reward);
        vars.insert("elapsed_secs".into(), ctx.elapsed_secs);
        vars.insert("step_tick".into(), 1.0);
        vars
    }
}
