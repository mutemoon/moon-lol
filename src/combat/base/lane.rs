use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub enum Lane {
    #[default]
    Top,
    Mid,
    Bot,
}
