use std::{f32::MAX, sync::Arc};

use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;
use lol_core::Team;

use crate::SkillEffectContext;

type BundleSpawner = Arc<dyn Fn(&mut EntityCommands) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct ActionCommand {
    pub bundle: BundleSpawner,
}

pub fn on_action_command(
    trigger: Trigger<BehaveTrigger<ActionCommand>>,
    mut commands: Commands,
    q_skill_effect_ctx: Query<&SkillEffectContext>,
    q_transform: Query<&Transform>,
    q_target: Query<(Entity, &Team)>,
    q_team: Query<&Team>,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();
    let behave_entity = ctx.behave_entity();

    let skill_effect_ctx = q_skill_effect_ctx.get(behave_entity).ok();
    let translation = skill_effect_ctx.unwrap().point;

    let mut min_distance = MAX;
    let mut target_bundle: Option<Entity> = None;

    let team = q_team.get(entity).unwrap();

    for (target, target_team) in q_target.iter() {
        if target_team == team {
            continue;
        }

        let Ok(target_transform) = q_transform.get(target) else {
            continue;
        };

        let distance = target_transform.translation.xz().distance(translation);
        if distance < min_distance {
            min_distance = distance;
            target_bundle = Some(target);
        }
    }

    let Some(target) = target_bundle else {
        commands.trigger(ctx.failure());
        return;
    };

    (event.bundle)(&mut commands.entity(target));

    commands.trigger(ctx.success());
}
