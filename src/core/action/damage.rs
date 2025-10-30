use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;
use lol_core::Team;

use crate::core::{rotate_to_direction, CommandDamageCreate, DamageType};

#[derive(Debug, Clone)]
pub struct ActionDamage;

pub fn on_attack_damage(
    trigger: Trigger<BehaveTrigger<ActionDamage>>,
    mut commands: Commands,
    mut q_transform: Query<&mut Transform>,
    q_target: Query<(Entity, &Team)>,
    q_team: Query<&Team>,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let _event = trigger.inner();

    let mut min_distance = 300.;
    let mut target_bundle: Option<(Entity, &Transform)> = None;

    let team = q_team.get(entity).unwrap();
    let transform = q_transform.get(entity).unwrap();

    for (target, target_team) in q_target.iter() {
        if target_team == team {
            continue;
        }

        let Ok(target_transform) = q_transform.get(target) else {
            continue;
        };

        let distance = target_transform.translation.distance(transform.translation);
        if distance < min_distance {
            min_distance = distance;
            target_bundle = Some((target, target_transform));
        }
    }

    let Some((target, target_transform)) = target_bundle else {
        commands.trigger(ctx.failure());
        return;
    };

    let direction = (target_transform.translation - transform.translation).xz();
    let mut transform = q_transform.get_mut(entity).unwrap();
    rotate_to_direction(&mut transform, direction);

    commands.entity(target).trigger(CommandDamageCreate {
        source: entity,
        damage_type: DamageType::Physical,
        amount: 100.0,
    });
    commands.trigger(ctx.success());
}
