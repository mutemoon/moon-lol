use bevy::prelude::*;

use crate::{
    Attack, AttackState, AttackStatus, Bounding, CommandAttackStart, CommandAttackStop,
    CommandRunStart, CommandRunStop, RunTarget,
};

#[derive(Default)]
pub struct PluginAttackAuto;

impl Plugin for PluginAttackAuto {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_attack_auto_start);
        app.add_observer(on_command_attack_auto_stop);

        app.add_systems(FixedPreUpdate, update_attack_auto);
    }
}

#[derive(Component)]
pub struct AttackAuto {
    pub target: Entity,
}

#[derive(EntityEvent)]
pub struct CommandAttackAutoStart {
    pub entity: Entity,
    pub target: Entity,
}

#[derive(EntityEvent)]
pub struct CommandAttackAutoStop {
    pub entity: Entity,
}

fn on_command_attack_auto_start(
    trigger: On<CommandAttackAutoStart>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
    q_attack: Query<&Attack>,
    q_bounding: Query<&Bounding>,
) {
    let entity = trigger.event_target();
    let target = trigger.target;

    // 获取自身组件
    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let Ok(attack) = q_attack.get(entity) else {
        return;
    };
    let Ok(bounding) = q_bounding.get(entity) else {
        return;
    };

    // 获取目标组件
    let Ok(target_transform) = q_transform.get(target) else {
        return;
    };
    let Ok(target_bounding) = q_bounding.get(target) else {
        return;
    };

    process_attack_logic(
        &mut commands,
        entity,
        target,
        transform.translation.xz(),
        target_transform.translation.xz(),
        bounding.radius,
        target_bounding.radius,
        attack.range,
    );

    commands.entity(entity).insert(AttackAuto { target });
}

fn on_command_attack_auto_stop(trigger: On<CommandAttackAutoStop>, mut commands: Commands) {
    let entity = trigger.event_target();
    commands.entity(entity).remove::<AttackAuto>();
    commands.trigger(CommandAttackStop { entity });
}

fn update_attack_auto(
    mut commands: Commands,
    // 优化：直接在主查询中获取自身的 Transform 和 Bounding，减少 get 调用
    q_attacker: Query<(Entity, &AttackAuto, &Attack, &Transform, &Bounding)>,
    q_attack_state: Query<&AttackState>,
    q_transform: Query<&Transform>,
    q_bounding: Query<&Bounding>,
) {
    for (entity, attack_auto, attack, transform, bounding) in q_attacker.iter() {
        // 检查攻击状态（如果在前摇中则跳过）
        if let Ok(state) = q_attack_state.get(entity) {
            if matches!(state.status, AttackStatus::Windup { .. }) {
                continue;
            }
        }

        let target = attack_auto.target;

        // 获取目标组件
        let Ok(target_transform) = q_transform.get(target) else {
            continue;
        };
        let Ok(target_bounding) = q_bounding.get(target) else {
            continue;
        };

        process_attack_logic(
            &mut commands,
            entity,
            target,
            transform.translation.xz(),
            target_transform.translation.xz(),
            bounding.radius,
            target_bounding.radius,
            attack.range,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn process_attack_logic(
    commands: &mut Commands,
    entity: Entity,
    target: Entity,
    pos: Vec2,
    target_pos: Vec2,
    radius: f32,
    target_radius: f32,
    range: f32,
) {
    // 优化：使用平方距离避免 sqrt 开方运算
    let dist_sq = pos.distance_squared(target_pos);
    let required_range = range + radius + target_radius;
    let required_range_sq = required_range * required_range;

    if dist_sq > required_range_sq {
        commands.trigger(CommandRunStart {
            entity,
            target: RunTarget::Target(target),
        });

        debug!("{} 停止攻击：离开攻击范围", entity);
        commands.trigger(CommandAttackStop { entity });
    } else {
        commands.trigger(CommandRunStop { entity });

        debug!("{} 开始攻击：进入攻击范围", entity);
        commands.trigger(CommandAttackStart { entity, target });
    }
}
