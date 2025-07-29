use crate::game::GameState;
use bevy::{
    app::{App, Plugin},
    prelude::*,
};

pub struct PluginClassic;

impl Plugin for PluginClassic {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.add_systems(OnEnter(GameState::Setup), setup);
    }
}

pub fn setup(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}
