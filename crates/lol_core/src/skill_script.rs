use bevy::prelude::*;

use crate::action::{Action, CommandAction};
use crate::game::FixedFrameCount;
use crate::life::Death;
use crate::skill::SkillPoints;

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = SkillScriptTargets)]
pub struct SkillScriptTarget(pub Entity);

#[derive(Component, Debug, Clone, Reflect)]
#[relationship_target(relationship = SkillScriptTarget, linked_spawn)]
#[reflect(Component)]
pub struct SkillScriptTargets(Vec<Entity>);

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = SkillScriptSources)]
pub struct SkillScriptSource(pub Entity);

#[derive(Component, Debug, Clone, Reflect)]
#[relationship_target(relationship = SkillScriptSource, linked_spawn)]
#[reflect(Component)]
pub struct SkillScriptSources(Vec<Entity>);

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct SkillScriptStep {
    pub frame: u32,
    pub command: SkillScriptCommand,
}

impl SkillScriptStep {
    pub fn action(frame: u32, action: Action) -> Self {
        Self {
            frame,
            command: SkillScriptCommand::Action(action),
        }
    }

    pub fn set_skill_points(frame: u32, value: u32) -> Self {
        Self {
            frame,
            command: SkillScriptCommand::SetSkillPoints(value),
        }
    }
}

#[derive(Clone, Reflect, Debug)]
pub enum SkillScriptCommand {
    Action(Action),
    SetSkillPoints(u32),
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct SkillScriptStepExecuted;

#[derive(Default)]
pub struct PluginSkillScript;

impl Plugin for PluginSkillScript {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, run_skill_script_steps);
    }
}

pub fn run_skill_script_steps(
    mut commands: Commands,
    frame: Option<Res<FixedFrameCount>>,
    q_steps: Query<
        (
            Entity,
            &SkillScriptStep,
            &SkillScriptSource,
            Option<&SkillScriptTarget>,
        ),
        Without<SkillScriptStepExecuted>,
    >,
    mut q_actor: Query<Option<&mut SkillPoints>, Without<Death>>,
) {
    let Some(frame) = frame else {
        return;
    };

    for (step_entity, step, source, target) in q_steps.iter() {
        if step.frame > frame.0 {
            continue;
        }
        commands.entity(step_entity).insert(SkillScriptStepExecuted);

        let actor = source.0;
        info!(
            "[SkillScriptStep] frame={} actor={:?} step={:?} target={:?}",
            frame.0, actor, step, target
        );

        let Ok(mut skill_points) = q_actor.get_mut(actor) else {
            continue;
        };
        match &step.command {
            SkillScriptCommand::Action(action) => {
                let mut action = action.clone();
                if let Some(target) = target {
                    if let Action::Attack(ref mut entity) = action {
                        *entity = target.0;
                    }
                }
                commands.trigger(CommandAction {
                    entity: actor,
                    action,
                });
            }
            SkillScriptCommand::SetSkillPoints(value) => {
                if let Some(skill_points) = skill_points.as_deref_mut() {
                    skill_points.0 = *value;
                }
            }
        }
    }
}
