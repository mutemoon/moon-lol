use bevy::prelude::*;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::missile::CommandAttachedFieldCreate;
use lol_core::movement::EventMovementEnd;
use lol_core::skill::SkillRecastWindow;
use lol_core::team::Team;

use crate::riven::buffs::RivenQ3Pending;
use crate::riven::passive::BuffRivenPassive;

const RIVEN_Q_RECAST_WINDOW: f32 = 4.0;
const RIVEN_Q3_KNOCKBACK_DISTANCE: f32 = 75.0;
const RIVEN_Q3_KNOCKBACK_RADIUS: f32 = 250.0;
const RIVEN_Q_FIELD_DURATION: f32 = 0.5;

/// Q 段对应的半径：Q1 < Q2 < Q3
const RIVEN_Q_RADII: [f32; 3] = [175.0, 215.0, 250.0];

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

    // 纯位移（伤害由 AttachedField 处理）
    commands.trigger(ActionDash {
        entity,
        point,
        skill: Handle::default(),
        move_type: DashMoveType::Fixed(250.0),
        damage: None,
        speed: 1000.0,
    });

    // 生成附着在锐雯身上的通用伤害场，随锐雯移动
    // 半径从小变大，匹配位移过程中的碰撞区增长
    commands.trigger(CommandAttachedFieldCreate {
        entity,
        radius,
        damage: damage_amount,
        duration: RIVEN_Q_FIELD_DURATION,
        grow_from: Some(65.0),
        grow_duration: Some(0.25), // dash 250unit @ 1000speed = 0.25s
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRivenPassive);

    if stage >= 3 {
        // Q3：标记击退，位移结束后触发
        commands.entity(entity).insert(RivenQ3Pending);
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
    } else {
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            stage + 1,
            3,
            RIVEN_Q_RECAST_WINDOW,
        ));
    }

}

/// 锐雯 Q3 位移结束后触发击退
pub fn on_riven_dash_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_pending: Query<&RivenQ3Pending>,
    q_transform: Query<&Transform>,
    q_targets: Query<(Entity, &Team, &Transform)>,
    q_team: Query<&Team>,
) {
    let entity = trigger.event_target();
    let Ok(_) = q_pending.get(entity) else {
        return;
    };
    let Ok(riven_transform) = q_transform.get(entity) else {
        return;
    };
    let Ok(riven_team) = q_team.get(entity) else {
        return;
    };

    let riven_pos = riven_transform.translation;

    for (target, target_team, target_transform) in q_targets.iter() {
        if target_team == riven_team {
            continue;
        }
        let diff = target_transform.translation - riven_pos;
        let distance = diff.length();
        if distance > RIVEN_Q3_KNOCKBACK_RADIUS || distance < 0.001 {
            continue;
        }

        let direction = diff / distance;
        let new_pos = target_transform.translation + direction * RIVEN_Q3_KNOCKBACK_DISTANCE;
        commands.entity(target).insert(Transform {
            translation: new_pos,
            ..*target_transform
        });
    }

    commands.entity(entity).remove::<RivenQ3Pending>();
}
