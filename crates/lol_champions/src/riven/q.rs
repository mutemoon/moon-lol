use bevy::prelude::*;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn, CommandSkinSoundPlay};
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::action::displace::{
    ActionDisplace, DisplaceCenter, DisplaceEffect, DisplaceMotion, DisplaceTargetSelection,
};
use lol_core::action::knockback::{CommandKnockback, DisplaceDirection, Knockup};
use lol_core::attack::CommandAttackReset;
use lol_core::damage::{Damage, DamageType};
use lol_core::missile::CommandAttachedFieldCreate;
use lol_core::movement::{EventMovementEnd, MovementSource};
use lol_core::skill::{EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_value};
use lol_core::team::Team;

use crate::riven::Riven;
use crate::riven::buffs::RivenQ3Pending;

const RIVEN_Q_RECAST_WINDOW: f32 = 4.0;
const RIVEN_Q3_KNOCKBACK_DISTANCE: f32 = 120.0;
const RIVEN_Q3_KNOCKBACK_SPEED: f32 = 900.0;
const RIVEN_Q3_KNOCKUP_HEIGHT: f32 = 150.0;
const RIVEN_Q3_RADIUS: f32 = 250.0;
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
    let (animation_hash, radius, trail_key, detonate_key) = match stage {
        1 => (
            "Spell1A".to_string(),
            RIVEN_Q_RADII[0],
            "Riven_Q_01_Wpn_Trail",
            "Riven_Q_01_Detonate",
        ),
        2 => (
            "Spell1B".to_string(),
            RIVEN_Q_RADII[1],
            "Riven_Q_02_Wpn_Trail",
            "Riven_Q_02_Detonate",
        ),
        _ => (
            "Spell1C".to_string(),
            RIVEN_Q_RADII[2],
            "Riven_Q_03_Wpn_Trail",
            "Riven_Q_03_Detonate",
        ),
    };

    // 每段各自的武器拖尾粒子
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: trail_key.to_string(),
        rotation: None,
        resolver_entity: None,
    });

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
        commands
            .entity(entity)
            .insert(RivenQ3Pending { damage: q_damage });
        commands
            .entity(trigger.skill_entity)
            .remove::<SkillRecastWindow>();
    } else {
        // 前两段：斩击落地瞬间的起爆粒子
        commands.trigger(CommandSkinParticleSpawn {
            entity,
            hash: detonate_key.to_string(),
            rotation: None,
            resolver_entity: None,
        });
        commands.trigger(CommandSkinSoundPlay {
            entity,
            key: match stage {
                2 => "RivenQ2_OnCast".to_string(),
                _ => "RivenQ_OnCast".to_string(),
            },
        });
        commands.trigger(CommandAttachedFieldCreate {
            entity,
            radius,
            damage: q_damage,
            duration: RIVEN_Q_FIELD_DURATION,
            grow_from: Some(65.0),
            grow_duration: Some(0.25),
        });
        commands
            .entity(trigger.skill_entity)
            .insert(SkillRecastWindow::new(stage + 1, 3, RIVEN_Q_RECAST_WINDOW));
    }
}

/// 锐雯 Q3 位移结束后，以落点为圆心结算：
/// 范围伤害（`ActionDisplace`，无位移）→ 震退 + 击飞（`CommandKnockback`）。
///
/// 震退是平滑的直线强制位移（不进寻路，故不会一帧瞬移），目标沿背离锐雯的方向
/// 滑出 [`RIVEN_Q3_KNOCKBACK_DISTANCE`]；击飞以滞空控制呈现，期间目标无法施法。
pub fn on_riven_dash_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_pending: Query<&RivenQ3Pending>,
    q_team: Query<&Team>,
    q_transform: Query<&Transform>,
    q_targets: Query<(Entity, &Team, &Transform), Without<Riven>>,
) {
    if trigger.event().source != MovementSource::Dash {
        return;
    }

    let entity = trigger.event_target();
    let Ok(pending) = q_pending.get(entity) else {
        return;
    };
    let (Ok(team), Ok(transform)) = (q_team.get(entity), q_transform.get(entity)) else {
        return;
    };

    // Q3 落地起爆粒子
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: "Riven_Q_03_Detonate".to_string(),
        rotation: None,
        resolver_entity: None,
    });
    commands.trigger(CommandSkinSoundPlay {
        entity,
        key: "RivenQ3_OnCast".to_string(),
    });

    commands.trigger(ActionDisplace {
        entity,
        targets: DisplaceTargetSelection::Circle {
            radius: RIVEN_Q3_RADIUS,
            center: DisplaceCenter::Caster,
        },
        motion: DisplaceMotion::None,
        effects: vec![DisplaceEffect::Damage {
            amount: pending.damage,
            damage_type: DamageType::Physical,
            tag: None,
        }],
        cone_hit_policy: None,
    });

    let landed_at = transform.translation;
    for (target, target_team, target_transform) in q_targets.iter() {
        if target_team == team || target_transform.translation.distance(landed_at) > RIVEN_Q3_RADIUS
        {
            continue;
        }
        commands.entity(target).trigger(|e| CommandKnockback {
            entity: e,
            source: entity,
            distance: RIVEN_Q3_KNOCKBACK_DISTANCE,
            speed: RIVEN_Q3_KNOCKBACK_SPEED,
            knockup: Some(Knockup::new(RIVEN_Q3_KNOCKUP_HEIGHT)),
            direction: DisplaceDirection::Away,
        });
    }

    commands.entity(entity).remove::<RivenQ3Pending>();
}
