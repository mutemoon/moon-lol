use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, States, Default)]
pub enum GameState {
    #[default]
    Setup,
    Playing,
}
