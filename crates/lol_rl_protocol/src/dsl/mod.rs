pub mod action_parser;
pub mod common;
pub mod env_parser;
pub mod expr_parser;
pub mod obs_parser;
pub mod reward_parser;

use serde::{Deserialize, Serialize};
use winnow::Parser;
use winnow::combinator::repeat;

use crate::action::ActionSchema;
use crate::dsl::action_parser::parse_action_schema;
use crate::dsl::common::ws;
use crate::dsl::env_parser::{EnvMetaBlock, parse_env_meta_block};
use crate::dsl::obs_parser::parse_obs_schema;
use crate::dsl::reward_parser::parse_reward_formula;
use crate::env_spec::{EnvSpec, EnvTrainingParams};
use crate::obs::ObsSchema;
use crate::reward::RewardFormulaSpec;

/// 完整的环境 DSL 规范定义
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct EnvDslSpec {
    pub name: String,
    pub label: Option<String>,
    pub tag: Option<String>,
    pub description: Option<String>,
    pub num_agents: Option<usize>,
    pub default_params: Option<EnvTrainingParams>,
    pub obs_schema: Option<ObsSchema>,
    pub action_schema: Option<ActionSchema>,
    pub reward_formula: Option<RewardFormulaSpec>,
}

impl EnvDslSpec {
    /// 转换为环境展示与默认超参数规范
    pub fn to_env_spec(&self) -> EnvSpec {
        EnvSpec {
            name: Box::leak(self.name.clone().into_boxed_str()),
            label: Box::leak(
                self.label
                    .clone()
                    .unwrap_or_else(|| self.name.clone())
                    .into_boxed_str(),
            ),
            tag: Box::leak(
                self.tag
                    .clone()
                    .unwrap_or_else(|| self.name.clone())
                    .into_boxed_str(),
            ),
            description: Box::leak(
                self.description
                    .clone()
                    .unwrap_or_default()
                    .into_boxed_str(),
            ),
            num_agents: self.num_agents.unwrap_or(1),
            default_params: self.default_params.clone().unwrap_or_default(),
        }
    }
}

enum DslBlock {
    Env(EnvMetaBlock),
    Obs(String, ObsSchema),
    Action(String, ActionSchema),
    Reward(RewardFormulaSpec),
}

/// 解析包含多个块（env, obs, action, reward）的完整 `.rl` 脚本
pub fn parse_env_dsl(mut input: &str) -> Result<EnvDslSpec, String> {
    let mut spec = EnvDslSpec::default();

    let blocks: Vec<DslBlock> = repeat(
        0..,
        winnow::combinator::alt((
            parse_env_meta_block.map(DslBlock::Env),
            parse_obs_schema.map(|(n, s)| DslBlock::Obs(n, s)),
            parse_action_schema.map(|(n, a)| DslBlock::Action(n, a)),
            parse_reward_formula.map(DslBlock::Reward),
        )),
    )
    .parse_next(&mut input)
    .map_err(|e| format!("DSL 解析失败: {:?}", e))?;

    ws.parse_next(&mut input)
        .map_err(|e| format!("空白符处理失败: {:?}", e))?;

    if !input.trim().is_empty() {
        return Err(format!("无法识别的 DSL 尾随内容: {:?}", input));
    }

    for block in blocks {
        match block {
            DslBlock::Env(meta) => {
                spec.name = meta.name;
                spec.label = meta.label;
                spec.tag = meta.tag;
                spec.description = meta.description;
                spec.num_agents = meta.num_agents;
                spec.default_params = meta.params;
            }
            DslBlock::Obs(name, obs) => {
                if spec.name.is_empty() {
                    spec.name = name;
                }
                spec.obs_schema = Some(obs);
            }
            DslBlock::Action(name, action) => {
                if spec.name.is_empty() {
                    spec.name = name;
                }
                spec.action_schema = Some(action);
            }
            DslBlock::Reward(reward) => {
                if spec.name.is_empty() {
                    spec.name = reward.name.clone();
                }
                spec.reward_formula = Some(reward);
            }
        }
    }

    Ok(spec)
}

impl ObsSchema {
    /// 从 DSL 脚本解析 ObsSchema
    pub fn from_dsl(mut input: &str) -> Result<Self, String> {
        let (_, schema) = parse_obs_schema
            .parse_next(&mut input)
            .map_err(|e| format!("Obs DSL 解析错误: {:?}", e))?;
        Ok(schema)
    }
}

impl ActionSchema {
    /// 从 DSL 脚本解析 ActionSchema
    pub fn from_dsl(mut input: &str) -> Result<Self, String> {
        let (_, schema) = parse_action_schema
            .parse_next(&mut input)
            .map_err(|e| format!("Action DSL 解析错误: {:?}", e))?;
        Ok(schema)
    }
}

impl RewardFormulaSpec {
    /// 从 DSL 脚本解析 RewardFormulaSpec
    pub fn from_dsl(mut input: &str) -> Result<Self, String> {
        let schema = parse_reward_formula
            .parse_next(&mut input)
            .map_err(|e| format!("Reward DSL 解析错误: {:?}", e))?;
        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionMaskRule;
    use crate::obs::ObsContext;

    #[test]
    fn test_dsl_solo_v0_roundtrip() {
        let dsl_src = r#"
        // ── 观测空间 ──────────────────────────────────────────
        obs SoloV0Obs {
            category role: 4 -> embed(12) = role_id;

            struct spatial {
                vector target_rel_pos: 2 = [
                    (self_x - target_x) / 100.0,
                    (self_z - target_z) / 100.0
                ];
                scalar distance = distance / 100.0;
            }

            struct cooldowns {
                scalar q_ready = q_ready;
                scalar q_cd = q_cd / 10.0;
                scalar flash_ready = flash_ready;
                scalar flash_cd = flash_cd / 300.0;
            }

            repeated visible_units[20] -> encoder: SharedMlpPool(hidden=[32, 16], pool=Max) {
                category unit_type: 6 -> embed(8) = unit_type;
                vector rel_pos: 2 = [rel_pos[0] / 100.0, rel_pos[1] / 100.0];
                scalar hp_pct = hp_pct;
                scalar is_enemy = is_enemy;
            }
        }

        // ── 动作空间 ──────────────────────────────────────────
        action SoloV0Action {
            continuous offset: 2;
            unit_target target: visible_units[20 -> 16];
            category action_type: 8 {
                0: "NoOp",
                1: "Move",
                2: "Attack",
                3: "CastQ",
                4: "CastW",
                5: "CastE",
                6: "CastR",
                7: "CastFlash",
            }
        }

        // ── 奖励公式 ──────────────────────────────────────────
        reward SoloV0Reward {
            term vital_hit   : "击破破绽" = 80.0 * is_vital_break;
            term damage_deal : "造成伤害" = 100.0 * (enemy_damage / enemy_max_hp);
            term step_cost   : "时间惩罚" = -0.005;
        }
        "#;

        let spec = parse_env_dsl(dsl_src).expect("DSL 脚本应成功解析");
        assert_eq!(spec.name, "SoloV0Obs");

        // 验证 ObsSchema
        let obs = spec.obs_schema.expect("应包含 obs_schema");
        // role(1) + spatial(3) + cooldowns(4) + visible_units(20 * 5 = 100) = 108 维 raw_dim
        assert_eq!(obs.raw_dim(), 1 + 3 + 4 + 20 * 5);

        // 验证求值
        let mut ctx = ObsContext::new();
        ctx.set_var("role_id", 0.0);
        ctx.set_var("self_x", 150.0);
        ctx.set_var("target_x", 50.0);
        ctx.set_var("self_z", 0.0);
        ctx.set_var("target_z", 0.0);
        ctx.set_var("distance", 100.0);
        ctx.set_var("q_ready", 1.0);
        ctx.set_var("q_cd", 5.0);
        ctx.set_var("flash_ready", 0.0);
        ctx.set_var("flash_cd", 150.0);

        let vec = obs.eval_to_vector(&ctx);
        assert_eq!(vec.len(), obs.raw_dim());
        assert_eq!(vec[0], 0.0); // role
        assert_eq!(vec[1], 1.0); // target_rel_pos[0] = (150 - 50) / 100 = 1.0
        assert_eq!(vec[2], 0.0); // target_rel_pos[1]
        assert_eq!(vec[3], 1.0); // distance = 100 / 100 = 1.0
        assert_eq!(vec[4], 1.0); // q_ready
        assert_eq!(vec[5], 0.5); // q_cd = 5 / 10 = 0.5

        // 验证 ActionSchema
        let act = spec.action_schema.expect("应包含 action_schema");
        assert_eq!(act.encoding_dim(), 2 + 1 + 1); // continuous(2) + target(1) + action_type(1) = 4

        // 验证 RewardFormulaSpec
        let rew = spec.reward_formula.expect("应包含 reward_formula");
        assert_eq!(rew.terms.len(), 3);
        assert_eq!(rew.terms[0].id, "vital_hit");
        assert_eq!(rew.terms[0].label, "击破破绽");
    }

    #[test]
    fn test_dsl_math_and_functions() {
        let obs_src = r#"
        obs MathTest {
            scalar hp_pct = clamp(hp / max(max_hp, 1.0), 0.0, 1.0);
            scalar cond_val = if(hp > 50.0, 1.0, 0.0);
            scalar complex = (a + b * c) / 2.0;
        }
        "#;
        let obs = ObsSchema::from_dsl(obs_src).expect("应成功解析");
        assert_eq!(obs.raw_dim(), 3);

        let mut ctx = ObsContext::new();
        ctx.set_var("hp", 80.0);
        ctx.set_var("max_hp", 100.0);
        ctx.set_var("a", 10.0);
        ctx.set_var("b", 4.0);
        ctx.set_var("c", 5.0);

        let vec = obs.eval_to_vector(&ctx);
        assert_eq!(vec[0], 0.8); // 80 / 100
        assert_eq!(vec[1], 1.0); // 80 > 50 -> 1.0
        assert_eq!(vec[2], 15.0); // (10 + 4 * 5) / 2 = 30 / 2 = 15.0
    }

    #[test]
    fn test_dsl_action_and_reward_standalone() {
        let act_src = r#"
        action SimpleAction {
            continuous move: 2;
            category skill [NoOp, Q, W, E, R];
        }
        "#;
        let action = ActionSchema::from_dsl(act_src).expect("Action 应解析成功");
        assert_eq!(action.encoding_dim(), 3);
        assert_eq!(action.nodes.len(), 2);

        let rew_src = r#"
        reward SimpleReward {
            term kill: "击杀奖励" = 100.0 * is_kill;
            term step = -0.01;
        }
        "#;
        let reward = RewardFormulaSpec::from_dsl(rew_src).expect("Reward 应解析成功");
        assert_eq!(reward.terms.len(), 2);
        assert_eq!(reward.terms[1].label, "step");
    }

    #[test]
    fn test_dsl_action_mask_rules() {
        let act_src = r#"
        action FioraV2Action {
            continuous offset: 2;
            category action_type: 7 {
                0: "NoOp",
                1: "Move",
                2: "Attack",
                3: "CastQ",
                4: "CastE",
                5: "CastR",
                6: "CastFlash",
            }

            mask {
                if distance > 22.0 { disable Attack; }
                if q_ready < 0.5   { disable CastQ; }
                if e_ready < 0.5   { disable CastE; }
                if r_ready < 0.5   { disable CastR; }
                if flash_ready < 0.5 { disable CastFlash; }
            }
        }
        "#;
        let action = ActionSchema::from_dsl(act_src).expect("Action 应成功解析掩码规则");
        assert_eq!(action.mask_rules.len(), 5);
        match &action.mask_rules[0] {
            ActionMaskRule::Global {
                branch_label,
                disabled_branch,
                ..
            } => {
                assert_eq!(branch_label, "Attack");
                assert_eq!(*disabled_branch, 2);
            }
            _ => panic!("Expected Global mask rule"),
        }

        let mut ctx = ObsContext::new();
        ctx.set_var("distance", 30.0); // > 22.0 -> Attack disabled
        ctx.set_var("q_ready", 1.0); // CastQ enabled
        ctx.set_var("e_ready", 0.0); // < 0.5 -> CastE disabled
        ctx.set_var("r_ready", 1.0); // CastR enabled
        ctx.set_var("flash_ready", 0.0); // CastFlash disabled

        let mask = action.eval_flat_mask(&ctx);
        assert_eq!(mask, vec![true, true, false, true, false, true, false]);
    }

    #[test]
    fn test_dsl_conditional_target_masks() {
        let act_src = r#"
        action CombatAction {
            continuous offset: 2;
            unit_target target: visible_units[3 -> 16];
            category action_type: 4 {
                0: "NoOp",
                1: "Move",
                2: "Attack",
                3: "CastQ",
            }

            mask {
                for u in visible_units {
                    if u.unit_type <= 0.0 { disable target; }
                }

                if distance > 200.0 { disable Attack; }
                if q_ready < 0.5    { disable CastQ; }

                for u in visible_units {
                    if u.is_enemy <= 0.5 {
                        disable [Attack, CastQ];
                    }
                }
            }
        }
        "#;
        let action = ActionSchema::from_dsl(act_src).expect("CombatAction 应成功解析掩码规则");
        assert_eq!(action.mask_rules.len(), 5); // 1 EntitySlot + 2 Global + 2 ConditionalTarget (Attack, CastQ)

        let mut ctx = ObsContext::new();
        ctx.set_var("distance", 150.0);
        ctx.set_var("q_ready", 1.0);
        ctx.set_repeated(
            "visible_units",
            vec![
                ObsContext::new()
                    .with_var("unit_type", 1.0)
                    .with_var("is_enemy", 1.0),
                ObsContext::new()
                    .with_var("unit_type", 2.0)
                    .with_var("is_enemy", 0.0),
            ],
        );

        let action_masks = action.eval_action_masks(&ctx);
        assert_eq!(action_masks.branch_masks.len(), 3);
        assert!(action_masks.branch_masks[0].is_none()); // continuous offset

        let target_mask = action_masks.branch_masks[1].as_ref().unwrap();
        assert_eq!(target_mask, &vec![true, true, false]); // slot 0 (enemy), slot 1 (ally), slot 2 (empty)

        let base_mask = action_masks.branch_masks[2].as_ref().unwrap();
        assert_eq!(base_mask, &vec![true, true, true, true]); // NoOp, Move, Attack, CastQ all enabled

        let cond_masks = action_masks.conditional_target_masks.as_ref().unwrap();
        assert_eq!(cond_masks.len(), 4); // 4 action classes
        // 0: NoOp -> slot 0 (enemy), slot 1 (ally) both valid, slot 2 (empty) invalid
        assert_eq!(cond_masks[0], vec![true, true, false]);
        // 1: Move -> slot 0, slot 1 valid, slot 2 invalid
        assert_eq!(cond_masks[1], vec![true, true, false]);
        // 2: Attack -> only slot 0 (enemy) valid; slot 1 (ally) & slot 2 (empty) invalid
        assert_eq!(cond_masks[2], vec![true, false, false]);
        // 3: CastQ -> only slot 0 (enemy) valid; slot 1 (ally) & slot 2 (empty) invalid
        assert_eq!(cond_masks[3], vec![true, false, false]);
    }

    #[test]
    fn test_dsl_when_hierarchical_conditional_branch_masks() {
        let act_src = r#"
        action HierarchicalAction {
            continuous offset: 2;
            category action_type: 5 {
                0: "NoOp",
                1: "Move",
                2: "Attack",
                3: "CastSkill",
                4: "LevelUpSkill",
            }
            category skill_slot: 4 {
                0: "Q",
                1: "W",
                2: "E",
                3: "R",
            }
            unit_target target: visible_units[12 -> 32];

            mask {
                for u in visible_units {
                    if u.unit_type <= 0.0 { disable target; }
                }

                if attack_is_cooldown > 0.5 { disable Attack; }
                if can_cast_any < 0.5       { disable CastSkill; }
                if can_level_up_any < 0.5   { disable LevelUpSkill; }

                when CastSkill {
                    if q_ready < 0.5 { disable skill_slot.Q; }
                    if w_ready < 0.5 { disable skill_slot.W; }
                    if e_ready < 0.5 { disable skill_slot.E; }
                    if r_ready < 0.5 { disable skill_slot.R; }
                }

                when LevelUpSkill {
                    if can_level_up_q < 0.5 { disable skill_slot.Q; }
                    if can_level_up_w < 0.5 { disable skill_slot.W; }
                    if can_level_up_e < 0.5 { disable skill_slot.E; }
                    if can_level_up_r < 0.5 { disable skill_slot.R; }
                }

                when Attack {
                    for u in visible_units {
                        if u.is_enemy <= 0.5 {
                            disable target;
                        }
                    }
                }
            }
        }
        "#;

        let action = ActionSchema::from_dsl(act_src).expect("HierarchicalAction 应成功解析");
        assert_eq!(action.nodes.len(), 4);
        assert_eq!(action.encoding_dim(), 5); // 2 continuous + 1 action_type + 1 skill_slot + 1 target

        let mut ctx = ObsContext::new();
        ctx.set_var("attack_is_cooldown", 0.0);
        ctx.set_var("can_cast_any", 1.0);
        ctx.set_var("can_level_up_any", 1.0);

        // 技能施放就绪状态：Q 就绪，W/E/R 冷却或未学
        ctx.set_var("q_ready", 1.0);
        ctx.set_var("w_ready", 0.0);
        ctx.set_var("e_ready", 0.0);
        ctx.set_var("r_ready", 0.0);

        // 技能升级状态：Q/W 可升级，E/R 不可升级
        ctx.set_var("can_level_up_q", 1.0);
        ctx.set_var("can_level_up_w", 1.0);
        ctx.set_var("can_level_up_e", 0.0);
        ctx.set_var("can_level_up_r", 0.0);

        ctx.set_repeated(
            "visible_units",
            vec![
                ObsContext::new()
                    .with_var("unit_type", 2.0)
                    .with_var("is_enemy", 1.0),
                ObsContext::new()
                    .with_var("unit_type", 2.0)
                    .with_var("is_enemy", 0.0),
            ],
        );

        let action_masks = action.eval_action_masks(&ctx);

        // 1. 验证主动作掩码
        let act_type_mask = action_masks.branch_masks[1].as_ref().unwrap();
        assert_eq!(act_type_mask, &vec![true, true, true, true, true]);

        // 2. 验证自回归条件分支掩码 (conditional_branch_masks)
        let branch_map = action_masks
            .conditional_branch_masks
            .as_ref()
            .expect("应包含条件分支掩码");
        let skill_slot_matrix = branch_map
            .get("skill_slot")
            .expect("应包含 skill_slot 条件掩码矩阵");
        assert_eq!(skill_slot_matrix.len(), 5); // 对应 5 种 action_type

        // 当 action_type = 3 (CastSkill) 时：Q 开放，W/E/R 禁用
        assert_eq!(skill_slot_matrix[3], vec![true, false, false, false]);

        // 当 action_type = 4 (LevelUpSkill) 时：Q/W 开放，E/R 禁用
        assert_eq!(skill_slot_matrix[4], vec![true, true, false, false]);

        // 当 action_type = 0 (NoOp) / 1 (Move) / 2 (Attack) 时：默认开放
        assert_eq!(skill_slot_matrix[0], vec![true, true, true, true]);
        assert_eq!(skill_slot_matrix[1], vec![true, true, true, true]);
        assert_eq!(skill_slot_matrix[2], vec![true, true, true, true]);
    }
}
