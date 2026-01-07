use std::f32::consts::PI;

use bevy::prelude::*;
use league_utils::hash_wad;
use lol_loader::ShaderTocSettings;

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

pub trait AssetServerLoadLeague {
    fn load_league<A: Asset>(&self, path: impl Into<HashPath>) -> Handle<A>;

    fn load_league_labeled<'a, A: Asset>(
        &self,
        path: impl Into<HashPath>,
        label: &str,
    ) -> Handle<A>;

    fn load_league_with_settings<'a, A: Asset>(&self, path: &str) -> Handle<A>;
}

impl AssetServerLoadLeague for AssetServer {
    fn load_league<A: Asset>(&self, path: impl Into<HashPath>) -> Handle<A> {
        let path = path.into();
        self.load(format!("data/{:x}.{}", path.hash, path.ext))
    }

    fn load_league_labeled<'a, A: Asset>(
        &self,
        path: impl Into<HashPath>,
        label: &str,
    ) -> Handle<A> {
        let path = path.into();
        self.load(format!("data/{:x}.{}#{label}", path.hash, path.ext))
    }

    fn load_league_with_settings<'a, A: Asset>(&self, path: &str) -> Handle<A> {
        let original_path = path.to_string();
        self.load_with_settings(
            format!("data/{:x}.lol", hash_wad(path)),
            move |settings: &mut ShaderTocSettings| settings.0 = original_path.clone(),
        )
    }
}
