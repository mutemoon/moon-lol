use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;

use crate::core::{
    CommandMovement, EventMovementEnd, MovementAction, MovementWay, SkillEffectBehaveCtx,
    SkillEffectContext,
};

#[derive(Debug, Clone)]
pub enum ActionDash {
    Fixed(f32),
    Pointer { speed: f32, max: f32 },
}

pub fn on_action_dash(
    trigger: Trigger<BehaveTrigger<ActionDash>>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
    q_skill_effect_ctx: Query<&SkillEffectContext>,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();
    let behave_entity = ctx.behave_entity();
    match event {
        ActionDash::Fixed(_) => todo!(),
        ActionDash::Pointer { max, speed } => {
            let skill_effect_ctx = q_skill_effect_ctx.get(behave_entity).ok();
            let skill_effect_ctx = skill_effect_ctx.unwrap();
            let transform = q_transform.get(entity).unwrap();
            let vector = skill_effect_ctx.point - transform.translation.xz();
            let distance = vector.length();

            let destination = if distance < *max {
                skill_effect_ctx.point
            } else {
                let direction = vector.normalize();
                let dash_point = transform.translation.xz() + direction * *max;
                dash_point
            };

            commands
                .entity(entity)
                .insert(SkillEffectBehaveCtx(ctx.clone()))
                .trigger(CommandMovement {
                    priority: 100,
                    action: MovementAction::Start {
                        way: MovementWay::Path(vec![destination]),
                        speed: Some(*speed),
                    },
                });
        }
    }
}

pub fn on_action_dash_end(
    trigger: Trigger<EventMovementEnd>,
    mut commands: Commands,
    q: Query<&SkillEffectBehaveCtx>,
) {
    let entity = trigger.target();
    let Ok(SkillEffectBehaveCtx(ctx)) = q.get(entity) else {
        return;
    };

    commands.trigger(ctx.success());
}
