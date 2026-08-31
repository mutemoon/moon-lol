use std::collections::HashMap;

use bevy::prelude::*;
use lol_rl_protocol::{ObsFeaturePayload, ObsSchema};

use crate::flash_plugin::extract_flash_obs;
use crate::modifier_obs::{ModifierNameId, ModifierSlotObs, extract_entity_modifiers};
use crate::obs_plugins::{extract_attack_state, extract_champion_base, extract_skill_cds};

/// obs 向量中相对距离归一化列下标
pub const V2_OBS_DISTANCE_IDX: usize = 3;
/// 距离归一化分母
pub const V2_OBS_DISTANCE_SCALE: f32 = 100.0;

pub static FIORA_V2_OBS_SCHEMA: std::sync::LazyLock<ObsSchema> = std::sync::LazyLock::new(|| {
    super::FIORA_V2_SPEC
        .obs_schema
        .clone()
        .expect("FIORA_V2_SPEC 缺少 obs_schema")
});

#[derive(Debug, Clone)]
pub struct FioraV2Obs {
    pub role_id: f32,

    pub fiora_pos: Vec3,
    pub fiora_hp: f32,
    pub fiora_max_hp: f32,
    pub riven_pos: Vec3,
    pub riven_hp: f32,
    pub riven_max_hp: f32,
    pub distance: f32,

    pub attack_state: u8,
    pub attack_is_windup: bool,
    pub attack_is_cooldown: bool,
    pub attack_timer_remaining: f32,

    pub q_ready: bool,
    pub q_cd_remaining: f32,
    pub e_ready: bool,
    pub e_cd_remaining: f32,
    pub r_ready: bool,
    pub r_cd_remaining: f32,

    pub flash_ready: bool,
    pub flash_cd_remaining: f32,

    /// 自身修饰符槽位 (4 槽位 × 5 = 20 维)
    pub self_modifiers: Vec<ModifierSlotObs>,
    /// 目标修饰符槽位 (4 槽位 × 5 = 20 维)
    pub target_modifiers: Vec<ModifierSlotObs>,
}

impl FioraV2Obs {
    pub fn to_context(&self) -> lol_rl_protocol::ObsContext {
        let mut ctx = lol_rl_protocol::ObsContext::new();
        ctx.set_var("role_id", self.role_id);
        ctx.set_var("fiora_x", self.fiora_pos.x);
        ctx.set_var("fiora_z", self.fiora_pos.z);
        ctx.set_var("riven_x", self.riven_pos.x);
        ctx.set_var("riven_z", self.riven_pos.z);
        ctx.set_var("distance", self.distance);

        ctx.set_var(
            "attack_is_ready",
            if self.attack_state == 0 { 1.0 } else { 0.0 },
        );
        ctx.set_var(
            "attack_is_windup",
            if self.attack_is_windup { 1.0 } else { 0.0 },
        );
        ctx.set_var(
            "attack_is_cooldown",
            if self.attack_is_cooldown { 1.0 } else { 0.0 },
        );
        ctx.set_var("attack_timer_remaining", self.attack_timer_remaining);

        ctx.set_var("q_ready", if self.q_ready { 1.0 } else { 0.0 });
        ctx.set_var("q_cd", self.q_cd_remaining);
        ctx.set_var("e_ready", if self.e_ready { 1.0 } else { 0.0 });
        ctx.set_var("e_cd", self.e_cd_remaining);
        ctx.set_var("r_ready", if self.r_ready { 1.0 } else { 0.0 });
        ctx.set_var("r_cd", self.r_cd_remaining);
        ctx.set_var("flash_ready", if self.flash_ready { 1.0 } else { 0.0 });
        ctx.set_var("flash_cd", self.flash_cd_remaining);

        ctx.set_var("fiora_hp", self.fiora_hp);
        ctx.set_var("fiora_max_hp", self.fiora_max_hp);
        ctx.set_var("riven_hp", self.riven_hp);
        ctx.set_var("riven_max_hp", self.riven_max_hp);

        let primary_vital = self
            .target_modifiers
            .iter()
            .find(|m| m.name_id == ModifierNameId::FioraPassiveVital);
        let has_vital = primary_vital.is_some();
        let vital_is_active = primary_vital.map(|v| v.stack_count > 0.5).unwrap_or(false);
        let (vx, vnx, vz, vnz) = match primary_vital {
            Some(v) => (
                if v.param0 > 0.5 { 1.0 } else { 0.0 },
                if v.param0 < -0.5 { 1.0 } else { 0.0 },
                if v.param1 > 0.5 { 1.0 } else { 0.0 },
                if v.param1 < -0.5 { 1.0 } else { 0.0 },
            ),
            None => (0.0, 0.0, 0.0, 0.0),
        };
        ctx.set_var("vital_dir_x", vx);
        ctx.set_var("vital_dir_neg_x", vnx);
        ctx.set_var("vital_dir_z", vz);
        ctx.set_var("vital_dir_neg_z", vnz);
        ctx.set_var("has_vital", if has_vital { 1.0 } else { 0.0 });
        ctx.set_var("vital_is_active", if vital_is_active { 1.0 } else { 0.0 });

        let self_mods: Vec<_> = self.self_modifiers.iter().map(|m| m.to_context()).collect();
        ctx.set_repeated("self_modifiers", self_mods);

        let target_mods: Vec<_> = self
            .target_modifiers
            .iter()
            .map(|m| m.to_context())
            .collect();
        ctx.set_repeated("target_modifiers", target_mods.clone());
        ctx.set_repeated("modifiers", target_mods);

        ctx
    }

    pub fn to_vector(&self) -> Vec<f32> {
        FIORA_V2_OBS_SCHEMA.eval_to_vector(&self.to_context())
    }

    pub fn dim() -> usize {
        FIORA_V2_OBS_SCHEMA.raw_dim()
    }

    pub fn to_payload(&self) -> ObsFeaturePayload {
        let primary_vital = self
            .target_modifiers
            .iter()
            .find(|m| m.name_id == ModifierNameId::FioraPassiveVital);
        let has_vital = primary_vital.is_some();
        let vital_is_active = primary_vital.map(|v| v.stack_count > 0.5).unwrap_or(false);
        let vital_dir = if let Some(v) = primary_vital {
            if v.param0 > 0.5 {
                "+X (东)".to_string()
            } else if v.param0 < -0.5 {
                "-X (西)".to_string()
            } else if v.param1 > 0.5 {
                "+Z (北)".to_string()
            } else if v.param1 < -0.5 {
                "-Z (南)".to_string()
            } else {
                "无".to_string()
            }
        } else {
            "无".to_string()
        };

        ObsFeaturePayload {
            fiora_hp_pct: if self.fiora_max_hp > 0.0 {
                self.fiora_hp / self.fiora_max_hp
            } else {
                1.0
            },
            riven_hp_pct: if self.riven_max_hp > 0.0 {
                self.riven_hp / self.riven_max_hp
            } else {
                1.0
            },
            distance: self.distance,
            q_ready: self.q_ready,
            w_ready: true,
            e_ready: self.e_ready,
            r_ready: self.r_ready,
            has_vital,
            vital_is_active,
            vital_direction: vital_dir,
            tags: HashMap::from([
                ("q_cd".to_string(), format!("{:.1}s", self.q_cd_remaining)),
                ("e_cd".to_string(), format!("{:.1}s", self.e_cd_remaining)),
                ("r_cd".to_string(), format!("{:.1}s", self.r_cd_remaining)),
                (
                    "flash_cd".to_string(),
                    format!("{:.1}s", self.flash_cd_remaining),
                ),
                (
                    "atk_state".to_string(),
                    match self.attack_state {
                        0 => "Ready".to_string(),
                        1 => format!("前摇中({:.2}s)", self.attack_timer_remaining),
                        2 => format!("后摇中({:.2}s)", self.attack_timer_remaining),
                        _ => "未知".to_string(),
                    },
                ),
                (
                    "modifiers_count".to_string(),
                    format!(
                        "Self:{}, Target:{}",
                        self.self_modifiers
                            .iter()
                            .filter(|m| m.name_id != ModifierNameId::None)
                            .count(),
                        self.target_modifiers
                            .iter()
                            .filter(|m| m.name_id != ModifierNameId::None)
                            .count(),
                    ),
                ),
            ]),
            ..Default::default()
        }
    }
}

pub fn get_v2_obs_from_world(world: &World, fiora: Entity, riven: Entity) -> FioraV2Obs {
    let f_base = extract_champion_base(world, fiora);
    let r_base = extract_champion_base(world, riven);
    let dist = f_base.pos.distance(r_base.pos);

    let atk = extract_attack_state(world, fiora);
    let skills = extract_skill_cds(world, fiora);
    let (flash_ready, flash_cd) = extract_flash_obs(world, fiora);

    let self_modifiers = extract_entity_modifiers(world, fiora, 4);
    let target_modifiers = extract_entity_modifiers(world, riven, 4);

    FioraV2Obs {
        role_id: 0.0,
        fiora_pos: f_base.pos,
        fiora_hp: f_base.hp,
        fiora_max_hp: f_base.max_hp,
        riven_pos: r_base.pos,
        riven_hp: r_base.hp,
        riven_max_hp: r_base.max_hp,
        distance: dist,
        attack_state: atk.state_code,
        attack_is_windup: atk.is_windup,
        attack_is_cooldown: atk.is_cooldown,
        attack_timer_remaining: atk.timer_remaining,
        q_ready: skills[0].ready,
        q_cd_remaining: skills[0].cd_remaining,
        e_ready: skills[2].ready,
        e_cd_remaining: skills[2].cd_remaining,
        r_ready: skills[3].ready,
        r_cd_remaining: skills[3].cd_remaining,
        flash_ready,
        flash_cd_remaining: flash_cd,
        self_modifiers,
        target_modifiers,
    }
}
