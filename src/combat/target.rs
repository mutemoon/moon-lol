use bevy::prelude::*;

#[derive(Component)]
pub struct Target(pub Entity);

#[derive(Event, Debug)]
pub struct ActionSetTarget {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct ActionRemoveTarget;

pub struct PluginTarget;

impl Plugin for PluginTarget {
    fn build(&self, app: &mut App) {
        app.add_observer(action_set_target);
        app.add_observer(action_remove_target);
    }
}

fn action_set_target(trigger: Trigger<ActionSetTarget>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .insert(Target(trigger.target));
}

fn action_remove_target(trigger: Trigger<ActionRemoveTarget>, mut commands: Commands) {
    commands.entity(trigger.target()).remove::<Target>();
}
