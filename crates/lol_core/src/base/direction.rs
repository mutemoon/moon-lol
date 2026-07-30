use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 世界 XZ 平面上的轴向方位，用世界轴（X / Z 正负轴）而非屏幕上下左右描述，
/// 因为屏幕方位依赖相机朝向，所以轴向语义更通用。
///
/// 二维参数约定：`Vec2.x` 对应世界 X 轴，`Vec2.y` 对应世界 Z 轴。
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Direction {
    /// X 正轴
    X,
    /// X 负轴
    NegX,
    /// Z 正轴
    Z,
    /// Z 负轴
    NegZ,
}

pub fn is_in_direction(source: Vec2, target: Vec2, direction: &Direction) -> bool {
    let delta_x = source.x - target.x;
    let delta_z = source.y - target.y;

    let abs_delta_x = delta_x.abs();
    let abs_delta_z = delta_z.abs();

    match direction {
        Direction::Z => delta_z > 0.0 && abs_delta_z > abs_delta_x,

        Direction::NegZ => delta_z < 0.0 && abs_delta_z > abs_delta_x,

        Direction::X => delta_x > 0.0 && abs_delta_x > abs_delta_z,

        Direction::NegX => delta_x < 0.0 && abs_delta_x > abs_delta_z,
    }
}
