use std::f32::consts::PI;

use bevy::prelude::*;
use league_utils::hash_wad;

pub fn rotate_to_direction(transform: &mut Transform, direction: Vec2) {
    transform.rotation = Quat::from_rotation_y(direction_to_angle(direction));
}

/// 计算从Vec2方向到角度的转换
pub fn direction_to_angle(direction: Vec2) -> f32 {
    -(direction.y.atan2(direction.x) - PI / 2.0)
}

/// 计算两个角度之间的最短角度差
pub fn angle_difference(from: f32, to: f32) -> f32 {
    let mut diff = to - from;
    while diff > PI {
        diff -= 2.0 * PI;
    }
    while diff < -PI {
        diff += 2.0 * PI;
    }
    diff
}

/// 使用角速度进行角度插值
pub fn lerp_angle_with_velocity(
    current: f32,
    target: f32,
    angular_velocity: f32,
    delta_time: f32,
) -> f32 {
    let diff = angle_difference(current, target);
    let max_rotation = angular_velocity * delta_time;

    if diff.abs() <= max_rotation {
        target
    } else {
        current + diff.signum() * max_rotation
    }
}

pub struct HashPath {
    pub hash: u64,
    pub ext: String,
}

impl From<&str> for HashPath {
    fn from(value: &str) -> Self {
        let ext = value.split('.').last().unwrap_or("lol");
        let ext = if ext == "tex" || ext == "dds" {
            ext.to_string()
        } else {
            "lol".to_string()
        };
        Self {
            hash: hash_wad(value),
            ext,
        }
    }
}

impl From<&String> for HashPath {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<String> for HashPath {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<u64> for HashPath {
    fn from(hash: u64) -> Self {
        Self {
            hash,
            ext: "lol".to_string(),
        }
    }
}

impl From<&HashPath> for HashPath {
    fn from(value: &HashPath) -> Self {
        value.clone()
    }
}

impl Clone for HashPath {
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            ext: self.ext.clone(),
        }
    }
}

impl PartialEq for HashPath {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.ext == other.ext
    }
}
