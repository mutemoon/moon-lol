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
    q: Query<(&Transform, &Bounding, Option<&Attack>)>,
) {
    let entity = trigger.event_target();
    let target = trigger.target;

    let Ok((transform, bounding, Some(attack))) = q.get(entity) else {
        return;
    };

    let Ok((target_transform, target_bounding, _)) = q.get(target) else {
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
    q_attacker: Query<(
        Entity,
        &AttackAuto,
        &Attack,
        Option<&AttackState>,
        &Transform,
        &Bounding,
    )>,
    q_target: Query<(&Transform, &Bounding)>,
) {
    for (entity, attack_auto, attack, attack_state, transform, bounding) in q_attacker.iter() {
        if let Some(AttackState {
            status: AttackStatus::Windup { .. },
            ..
        }) = attack_state
        {
            continue;
        }

        let target = attack_auto.target;

        let Ok((target_transform, target_bounding)) = q_target.get(target) else {
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
