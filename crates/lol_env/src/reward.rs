use std::collections::HashMap;

use lol_rl_protocol::{RewardFormulaSpec, RewardItem};

/// 统一的环境单步奖励模型 Trait
pub trait RewardModel: Send + Sync {
    type Context;

    /// 获取环境定义的结构化奖励公式（由 DSL 规范提供，作为单一真实来源）
    fn formula_spec(&self) -> &RewardFormulaSpec;

    /// 从当前环境步骤上下文中提取特征变量
    fn extract_variables(&self, ctx: &Self::Context) -> HashMap<String, f32>;

    /// 严格基于 DSL 结构化表达式 AST 计算奖励与 Breakdown，返回 (总奖励, 细拆项, 环境变量字典)
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

/// Fiora vs Riven 环境的奖励模型实现（纯基于 specs/fiora_v0.rl DSL 规范）
pub struct FioraVsRivenRewardModel;

impl RewardModel for FioraVsRivenRewardModel {
    type Context = FioraRewardContext;

    fn formula_spec(&self) -> &RewardFormulaSpec {
        lol_rl_protocol::SPEC_FIORA_V0
            .reward_formula
            .as_ref()
            .expect("SPEC_FIORA_V0 缺少 reward_formula DSL 规范")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiora_vs_riven_reward_model_dsl_evaluate() {
        let model = FioraVsRivenRewardModel;
        assert_eq!(model.formula_spec().terms.len(), 7);

        // 击破破绽上下文
        let ctx = FioraRewardContext {
            prev_aligned: false,
            curr_aligned: true,
            is_vital_break: true,
            is_attack: true,
            prev_riven_hp: 500.0,
            curr_riven_hp: 400.0,
            elapsed_secs: 1.0,
        };
        let (total, items, vars) = model.evaluate(&ctx);
        assert_eq!(items.len(), 7);
        // time_penalty (-0.002) + alignment (0.02) + vital_break (0.8) = 0.818
        assert!((total - 0.818).abs() < 1e-4);
        assert_eq!(vars.get("is_vital_break").copied(), Some(1.0));
        assert_eq!(vars.get("is_newly_aligned").copied(), Some(1.0));
    }
}
