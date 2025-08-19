use bevy::prelude::*;

#[derive(Component, Default)]
pub enum State {
    #[default]
    Idle,
    Moving,
    Attacking,
}
