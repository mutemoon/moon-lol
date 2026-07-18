//! Volibear R - 风暴之怒 (Stormbringer)
//!
//! 突进 + 落地 AoE 物理伤害 + 减速 + 增加最大生命。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::movement::{EventMovementEnd, MovementSource};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value, get_skill_value};
use lol_core::team::Team;

use crate::volibear::Volibear;
use crate::volibear::buffs::VolibearRLandingPending;

/// R 伤害标签
pub const VOLIBEAR_R_TAG: u32 = 2;
/// R 落地 AoE 半径
pub const VOLIBEAR_R_LANDING_RADIUS: f32 = 300.0;
/// R 最大突进距离
pub const VOLIBEAR_R_MAX_RANGE: f32 = 550.0;
/// R 突进速度
pub const VOLIBEAR_R_DASH_SPEED: f32 = 750.0;

pub fn on_volibear_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
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

    // 突进
    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Pointer {
            max: VOLIBEAR_R_MAX_RANGE,
        },
        speed: VOLIBEAR_R_DASH_SPEED,
    });

    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let damage = get_skill_value(
        spell_obj,
        "sweet_spot_damage_tooltip",
        skill.level,
        |stat| {
            if stat == 2 { ad } else { 0.0 }
        },
    )
    .unwrap_or(0.0);
    let slow_percent = get_skill_data_value(spell_obj, "SlowAmount", skill.level).unwrap_or(0.5);
    let slow_duration = get_skill_data_value(spell_obj, "SlowDuration", skill.level).unwrap_or(1.0);
    let bonus_hp = get_skill_data_value(spell_obj, "HealthAmount", skill.level).unwrap_or(0.0);

    // 落地后 AoE + 减速 + 增加生命由 on_volibear_r_dash_end 结算
    commands.entity(entity).insert(VolibearRLandingPending {
        damage,
        slow_percent,
        slow_duration,
        bonus_hp,
    });

    debug!("Volibear R: 风暴之怒，突进落地");
}

/// R 突进落地：以落点为圆心 AoE 砸地 + 物理伤害 + 减速 + 增加最大生命。
pub fn on_volibear_r_dash_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_pending: Query<&VolibearRLandingPending>,
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
    let Ok(bear_tf) = q_transform.get(entity) else {
        return;
    };
    let Ok(bear_team) = q_team.get(entity) else {
        return;
    };
    let land_pos = bear_tf.translation.xz();

    for (enemy, enemy_tf, enemy_team) in q_enemies.iter() {
        if enemy_team == bear_team {
            continue;
        }
        let dist = enemy_tf.translation.xz().distance(land_pos);
        if dist > VOLIBEAR_R_LANDING_RADIUS {
            continue;
        }
        if pending.damage > 0.0 {
            commands.entity(enemy).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount: pending.damage,
                tag: Some(VOLIBEAR_R_TAG),
            });
        }
        commands
            .entity(enemy)
            .with_related::<BuffOf>(DebuffSlow::new(pending.slow_percent, pending.slow_duration));
    }

    // 增加最大生命（HealthAmount，当前生命同步增加）
    let bonus_hp = pending.bonus_hp;
    if bonus_hp > 0.0 {
        commands.entity(entity).queue(move |mut e: EntityWorldMut| {
            if let Some(mut health) = e.get_mut::<Health>() {
                health.max += bonus_hp;
                health.value = (health.value + bonus_hp).min(health.max);
            }
        });
    }

    commands.entity(entity).remove::<VolibearRLandingPending>();
}
