use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub fn is_in_direction(source: Vec2, target: Vec2, direction: &Direction) -> bool {
    let delta_x = source.x - target.x;
    let delta_y = source.y - target.y;

    let abs_delta_x = delta_x.abs();
    let abs_delta_y = delta_y.abs();

    match direction {
        Direction::Up => delta_y < 0.0 && abs_delta_y > abs_delta_x,

        Direction::Down => delta_y > 0.0 && abs_delta_y > abs_delta_x,

        Direction::Right => delta_x > 0.0 && abs_delta_x > abs_delta_y,

        Direction::Left => delta_x < 0.0 && abs_delta_x > abs_delta_y,
    }
}
