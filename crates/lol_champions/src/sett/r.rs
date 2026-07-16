//! Sett R - 消防官 (The Show Stopper)
//!
//! 向指针方向突进（最大 475），落地 AoE 砸地 + 物理伤害 + 40% 减速 1.5s。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::base::buff::BuffOf;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::movement::{EventMovementEnd, MovementSource};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_value};
use lol_core::team::Team;

use crate::sett::Sett;
use crate::sett::buffs::{SettRLandingPending, SETT_R_TAG, SETT_R_SLOW_PERCENT, SETT_R_SLOW_DURATION};

/// R AoE 半径
pub const SETT_R_RADIUS: f32 = 200.0;
/// R 突进最大范围
pub const SETT_R_DASH_MAX_RANGE: f32 = 475.0;
/// R 突进速度
pub const SETT_R_DASH_SPEED: f32 = 1500.0;

pub fn on_sett_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Pointer {
            max: SETT_R_DASH_MAX_RANGE,
        },
        speed: SETT_R_DASH_SPEED,
    });

    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let damage = get_skill_value(spell_obj, "damage_calc", skill.level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    commands.entity(entity).insert(SettRLandingPending {
        damage,
        slow_percent: SETT_R_SLOW_PERCENT,
        slow_duration: SETT_R_SLOW_DURATION,
    });
}

/// R 突进落地：以落点为圆心 AoE 砸地 + 物理伤害（带 SETT_R_TAG）+ 减速。
pub fn on_sett_r_dash_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_pending: Query<&SettRLandingPending>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
) {
    if trigger.event().source != MovementSource::Dash {
        return;
    }

    let entity = trigger.event_target();
    let Ok(pending) = q_pending.get(entity) else {
        return;
    };
    let Ok(sett_tf) = q_transform.get(entity) else {
        return;
    };
    let Ok(sett_team) = q_team.get(entity) else {
        return;
    };
    let land_pos = sett_tf.translation.xz();

    for (enemy, enemy_tf, enemy_team) in q_enemies.iter() {
        if enemy_team == sett_team {
            continue;
        }
        let dist = enemy_tf.translation.xz().distance(land_pos);
        if dist > SETT_R_RADIUS {
            continue;
        }
        if pending.damage > 0.0 {
            commands.entity(enemy).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount: pending.damage,
                tag: Some(SETT_R_TAG),
            });
        }
        commands.entity(enemy).with_related::<BuffOf>(DebuffSlow::new(
            pending.slow_percent,
            pending.slow_duration,
        ));
    }

    commands.entity(entity).remove::<SettRLandingPending>();
}