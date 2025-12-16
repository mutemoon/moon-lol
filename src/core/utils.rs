use std::f32::consts::PI;

use bevy::asset::meta::Settings;
use bevy::prelude::*;
use league_utils::hash_wad;
use lol_loader::ImageSettings;

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

pub struct HashPath(pub u64);

impl From<&str> for HashPath {
    fn from(value: &str) -> Self {
        Self(hash_wad(value))
    }
}

impl From<&String> for HashPath {
    fn from(value: &String) -> Self {
        Self(hash_wad(value))
    }
}

impl From<String> for HashPath {
    fn from(value: String) -> Self {
        Self(hash_wad(&value))
    }
}

impl From<&HashPath> for HashPath {
    fn from(value: &HashPath) -> Self {
        Self(value.0)
    }
}

impl Clone for HashPath {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for HashPath {}

impl PartialEq for HashPath {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub trait AssetServerLoadLeague {
    fn load_league<A: Asset>(&self, path: &str) -> Handle<A>;

    fn load_league_labeled<'a, A: Asset>(&self, path: &str, label: &str) -> Handle<A>;
}

impl AssetServerLoadLeague for AssetServer {
    fn load_league<A: Asset>(&self, path: &str) -> Handle<A> {
        self.load(format!("data/{:x}.lol", HashPath::from(path).0))
    }

    fn load_league_labeled<'a, A: Asset>(&self, path: &str, label: &str) -> Handle<A> {
        self.load(format!("data/{:x}.lol#{label}", HashPath::from(path).0))
    }
}
