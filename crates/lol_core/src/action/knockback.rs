use bevy::prelude::*;

use crate::buffs::cc_debuffs::DebuffKnockup;
use crate::movement::{CastBlock, CommandMovement, MovementAction, MovementWay};

#[derive(EntityEvent, Debug, Clone)]
pub struct CommandKnockback {
    pub entity: Entity,        // 被击退的目标
    pub source: Entity,        // 击退来源（用于计算方向）
    pub distance: f32,         // 击退距离
    pub speed: f32,            // 击退速度
    pub duration: Option<f32>, // 击飞持续时间（None 则根据 distance/speed 计算）
}

pub fn on_command_knockback(
    trigger: On<CommandKnockback>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
) {
    let target = trigger.event_target();
    let source = trigger.source;

    let Ok(target_transform) = q_transform.get(target) else {
        return;
    };
    let Ok(source_transform) = q_transform.get(source) else {
        return;
    };

    let diff = target_transform.translation.xz() - source_transform.translation.xz();
    let direction = diff.normalize_or_zero();

    // 如果位置重叠，默认向后方击退（这里简单处理，实际可能需要 source 的 forward）
    let direction = if direction == Vec2::ZERO {
        Vec2::new(0.0, 1.0)
    } else {
        direction
    };

    let dest_xz = target_transform.translation.xz() + direction * trigger.distance;
    let destination = Vec3::new(dest_xz.x, target_transform.translation.y, dest_xz.y);

    let duration = trigger.duration.unwrap_or(trigger.distance / trigger.speed);

    info!(
        "触发击退: {:?} -> {:?}, 距离: {}, 速度: {}, 持续时间: {}",
        source, target, trigger.distance, trigger.speed, duration
    );

    // 触发位移
    commands.entity(target).trigger(|e| CommandMovement {
        entity: e,
        priority: 100, // 高优先级
        action: MovementAction::Start {
            way: MovementWay::Path(vec![destination]),
            speed: Some(trigger.speed),
            source: "Knockback".to_string(),
        },
    });

    // 注入击飞控制（不添加 MovementBlock，否则位移系统会跳过该实体）
    commands
        .entity(target)
        .insert((DebuffKnockup::new(duration), CastBlock));
}
