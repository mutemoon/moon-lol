use bevy::prelude::*;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::action::knockback::{CommandKnockback, DisplaceDirection};
use lol_core::attack::CommandAttackReset;
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::missile::CommandAttachedFieldCreate;
use lol_core::movement::EventMovementEnd;
use lol_core::skill::SkillRecastWindow;
use lol_core::team::Team;

use crate::riven::buffs::RivenQ3Pending;

const RIVEN_Q_RECAST_WINDOW: f32 = 4.0;
const RIVEN_Q3_KNOCKBACK_DISTANCE: f32 = 75.0;
const RIVEN_Q3_KNOCKBACK_RADIUS: f32 = 250.0;
const RIVEN_Q_FIELD_DURATION: f32 = 0.5;
const RIVEN_Q_RADII: [f32; 3] = [100.0, 100.0, 100.0];

pub struct PluginRivenQ;

impl Plugin for PluginRivenQ {
    fn build(&self, app: &mut App) {
        app.add_observer(on_riven_dash_end);
    }
}

pub fn cast_riven_q(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    recast: Option<&SkillRecastWindow>,
    damage_amount: f32,
) {
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

    // 纯位移（Q1/Q2 伤害由 AttachedField 处理，Q3 伤害由落地 observer 结算）
    commands.trigger(ActionDash {
        entity,
        point,
        move_type: DashMoveType::Fixed(250.0),
        speed: 1000.0,
    });

    // 每段 Q 重置普攻计时器（wiki：Q 可重置普攻，Q 后可立即接平A）
    commands.trigger(CommandAttackReset { entity });

    // 被动层数由统一的 on_riven_skill_cast_charge_passive 观察者在 EventSkillCast 上授予，
    // 此处无需再单独授予

    if stage >= 3 {
        // Q3：不在位移路径上造成伤害，标记待落地结算；位移结束后以落点为圆心
        // 造成范围伤害 + 震退（见 on_riven_dash_end）
        commands.entity(entity).insert(RivenQ3Pending {
            damage: damage_amount,
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
    } else {
        // Q1/Q2：生成附着在锐雯身上的通用伤害场，随锐雯移动
        // 半径从小变大，匹配位移过程中的碰撞区增长
        commands.trigger(CommandAttachedFieldCreate {
            entity,
            radius,
            damage: damage_amount,
            duration: RIVEN_Q_FIELD_DURATION,
            grow_from: Some(65.0),
            grow_duration: Some(0.25), // dash 250unit @ 1000speed = 0.25s
        });
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
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
