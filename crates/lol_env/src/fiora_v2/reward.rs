use std::collections::HashMap;

use bevy::prelude::*;
use lol_rl_protocol::RewardFormulaSpec;

use super::obs::FioraV2Obs;
use crate::modifier_obs::ModifierNameId;
use crate::reward::RewardModel;

#[derive(Debug, Clone, Default)]
pub struct FioraV2RewardContext {
    pub prev_aligned: bool,
    pub curr_aligned: bool,
    pub is_vital_break: bool,
    pub prev_riven_hp: f32,
    pub curr_riven_hp: f32,
    pub riven_max_hp: f32,
    pub elapsed_secs: f32,
}

pub struct FioraV2RewardModel;

impl RewardModel for FioraV2RewardModel {
    type Context = FioraV2RewardContext;

    fn formula_spec(&self) -> &RewardFormulaSpec {
        super::FIORA_V2_SPEC
            .reward_formula
            .as_ref()
            .expect("FIORA_V2_SPEC 缺少 reward_formula DSL 规范")
    }

    fn extract_variables(&self, ctx: &FioraV2RewardContext) -> HashMap<String, f32> {
        let hp_diff = (ctx.prev_riven_hp - ctx.curr_riven_hp).max(0.0);
        let max_hp = if ctx.riven_max_hp > 0.0 {
            ctx.riven_max_hp
        } else {
            10000.0
        };
        let damage_ratio = hp_diff / max_hp;
        let is_kill = if ctx.curr_riven_hp <= 0.0 && ctx.prev_riven_hp > 0.0 {
            1.0
        } else {
            0.0
        };

        HashMap::from([
            ("damage_ratio".to_string(), damage_ratio),
            ("hp_diff".to_string(), hp_diff),
            ("is_kill".to_string(), is_kill),
            ("elapsed_secs".to_string(), ctx.elapsed_secs),
            ("step_tick".to_string(), 1.0),
        ])
    }
}

pub fn is_v2_aligned_with_vital(fpos: Vec3, rpos: Vec3, obs: &FioraV2Obs) -> bool {
    let delta_x = fpos.x - rpos.x;
    let delta_z = fpos.z - rpos.z;
    let abs_delta_x = delta_x.abs();
    let abs_delta_z = delta_z.abs();

    let primary_vital = obs
        .target_modifiers
        .iter()
        .find(|m| m.name_id == ModifierNameId::FioraPassiveVital);

    if let Some(v) = primary_vital {
        if v.param0 > 0.5 {
            delta_x > 0.0 && abs_delta_x > abs_delta_z
        } else if v.param0 < -0.5 {
            delta_x < 0.0 && abs_delta_x > abs_delta_z
        } else if v.param1 > 0.5 {
            delta_z > 0.0 && abs_delta_z > abs_delta_x
        } else if v.param1 < -0.5 {
            delta_z < 0.0 && abs_delta_z > abs_delta_x
        } else {
            false
        }
    } else {
        false
    }
}
