use bevy::prelude::*;

use crate::{EventAttackStart, EventRunEnd, EventRunStart};

#[derive(Default)]
pub struct PluginState;

impl Plugin for PluginState {
    fn build(&self, app: &mut App) {
        app.add_observer(on_run_start);
        app.add_observer(on_run_end);
        app.add_observer(on_command_attack_start);
    }
}

#[derive(Component, Default, PartialEq, Debug)]
pub enum State {
    #[default]
    Idle,
    Running,
    Attacking,
}

fn on_run_start(trigger: On<EventRunStart>, mut query: Query<&mut State>) {
    let entity = trigger.event_target();

    let Ok(mut state) = query.get_mut(entity) else {
        return;
    };

    *state = State::Running;
}

fn on_run_end(trigger: On<EventRunEnd>, mut query: Query<&mut State>) {
    let entity = trigger.event_target();

    let Ok(mut state) = query.get_mut(entity) else {
        return;
    };

    *state = State::Idle;
}

fn on_command_attack_start(trigger: On<EventAttackStart>, mut query: Query<&mut State>) {
    let entity = trigger.event_target();

    let Ok(mut state) = query.get_mut(entity) else {
        return;
    };

    *state = State::Attacking;
}
