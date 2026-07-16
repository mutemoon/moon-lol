use bevy::prelude::*;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::action::knockback::{CommandKnockback, DisplaceDirection};
use lol_core::attack::CommandAttackReset;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::missile::CommandAttachedFieldCreate;
use lol_core::movement::{EventMovementEnd, MovementSource};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, SkillRecastWindow, get_skill_value};
use lol_core::team::Team;

use crate::riven::Riven;
use crate::riven::buffs::RivenQ3Pending;

const RIVEN_Q_RECAST_WINDOW: f32 = 4.0;
const RIVEN_Q3_KNOCKBACK_DISTANCE: f32 = 75.0;
const RIVEN_Q3_KNOCKBACK_RADIUS: f32 = 250.0;
const RIVEN_Q_FIELD_DURATION: f32 = 0.5;
const RIVEN_Q_RADII: [f32; 3] = [100.0, 100.0, 100.0];

pub fn on_riven_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    q_skill: Query<(&Skill, Option<&SkillRecastWindow>)>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    let Ok((skill, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let damage_value = q_damage.get(entity).map(|d| d.0).unwrap_or(64.0);
    let q_damage = get_skill_value(spell_obj, "first_slash_damage", skill.level, |stat| {
        if stat == 2 { damage_value } else { 0.0 }
    })
    .unwrap_or(0.0);

    let stage = recast.map(|window| window.stage).unwrap_or(1);
    let (animation_hash, radius) = match stage {
        1 => ("Spell1A".to_string(), RIVEN_Q_RADII[0]),
        2 => ("Spell1B".to_string(), RIVEN_Q_RADII[1]),
        _ => ("Spell1C".to_string(), RIVEN_Q_RADII[2]),
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: animation_hash,
        repeat: false,
        duration: None,
    });

    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Fixed(250.0),
        speed: 1000.0,
    });

    commands.trigger(CommandAttackReset { entity });

    if stage >= 3 {
        commands.entity(entity).insert(RivenQ3Pending {
            damage: q_damage,
        });
        commands.entity(trigger.skill_entity).remove::<SkillRecastWindow>();
    } else {
        commands.trigger(CommandAttachedFieldCreate {
            entity,
            radius,
            damage: q_damage,
            duration: RIVEN_Q_FIELD_DURATION,
            grow_from: Some(65.0),
            grow_duration: Some(0.25),
        });
        commands.entity(trigger.skill_entity).insert(SkillRecastWindow::new(
            stage + 1,
            3,
            RIVEN_Q_RECAST_WINDOW,
        ));
    }
}

/// 锐雯 Q3 位移结束后，以落点为圆心造成范围伤害 + 震退
pub fn on_riven_dash_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_pending: Query<&RivenQ3Pending>,
    q_transform: Query<&Transform>,
    q_targets: Query<(Entity, &Team, &Transform)>,
    q_team: Query<&Team>,
) {
    if trigger.event().source != MovementSource::Dash {
        return;
    }

    let entity = trigger.event_target();
    let Ok(pending) = q_pending.get(entity) else {
        return;
    };
    let Ok(riven_transform) = q_transform.get(entity) else {
        return;
    };
    let Ok(riven_team) = q_team.get(entity) else {
        return;
    };

    let riven_pos = riven_transform.translation;
    let damage = pending.damage;

    for (target, target_team, target_transform) in q_targets.iter() {
        if target_team == riven_team {
            continue;
        }
        let distance = (target_transform.translation - riven_pos).length();
        if distance > RIVEN_Q3_KNOCKBACK_RADIUS {
            continue;
        }

        // 落点圆形范围伤害
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Physical,
            amount: damage,
            tag: None,
        });

        // 震退（方向由 CommandKnockback 按 source->target 计算，重叠时退回默认方向）
        commands.entity(target).trigger(|e| CommandKnockback {
            entity: e,
            source: entity,
            distance: RIVEN_Q3_KNOCKBACK_DISTANCE,
            speed: 1200.0,
            duration: Some(0.75),
            direction: DisplaceDirection::Away,
        });
    }

    commands.entity(entity).remove::<RivenQ3Pending>();
}
