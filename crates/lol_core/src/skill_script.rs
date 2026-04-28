use bevy::prelude::*;

use crate::action::{Action, CommandAction};
use crate::game::FixedFrameCount;
use crate::skill::SkillPoints;

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct SkillScript {
    pub steps: Vec<SkillScriptStep>,
}

impl SkillScript {
    pub fn new(steps: Vec<SkillScriptStep>) -> Self {
        Self { steps }
    }
}

#[derive(Clone, Debug, Reflect)]
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
pub struct SkillScriptCursor(pub usize);

#[derive(Default)]
pub struct PluginSkillScript;

impl Plugin for PluginSkillScript {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, run_skill_script);
    }
}

pub fn run_skill_script(
    mut commands: Commands,
    frame: Option<Res<FixedFrameCount>>,
    mut q_actor: Query<(
        Entity,
        &SkillScript,
        &mut SkillScriptCursor,
        Option<&mut SkillPoints>,
    )>,
) {
    let Some(frame) = frame else {
        return;
    };

    let Ok((actor, script, mut cursor, mut skill_points)) = q_actor.single_mut() else {
        return;
    };

    while let Some(step) = script.steps.get(cursor.0) {
        if step.frame > frame.0 {
            break;
        }

        info!(
            "[SkillScript] frame={} actor={:?} step={:?}",
            frame.0, actor, step
        );

        match &step.command {
            SkillScriptCommand::Action(action) => {
                commands.trigger(CommandAction {
                    entity: actor,
                    action: action.clone(),
                });
            }
            SkillScriptCommand::SetSkillPoints(value) => {
                if let Some(skill_points) = skill_points.as_deref_mut() {
                    skill_points.0 = *value;
                }
            }
        }

        cursor.0 += 1;
    }
}
