//! SpawnPreset 子系统的领域层（出生点预设）。

use serde::{Deserialize, Serialize};

/// 阵营。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Team {
    Order,
    Chaos,
}

impl Team {
    pub fn as_str(&self) -> &'static str {
        match self {
            Team::Order => "order",
            Team::Chaos => "chaos",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "order" => Some(Team::Order),
            "chaos" => Some(Team::Chaos),
            _ => None,
        }
    }
}

/// 可见性。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Private,
    Friends,
    Public,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Private => "private",
            Visibility::Friends => "friends",
            Visibility::Public => "public",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "private" => Some(Visibility::Private),
            "friends" => Some(Visibility::Friends),
            "public" => Some(Visibility::Public),
            _ => None,
        }
    }
}

/// 出生点预设：命名坐标 + 阵营 + 可见性。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpawnPreset {
    pub id: uuid::Uuid,
    pub owner_id: i32,
    pub name: String,
    pub x: f32,
    pub z: f32,
    pub team: Team,
    pub visibility: Visibility,
}

/// 创建出生点预设的输入（不含 id 和 owner_id）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPresetInput {
    pub name: String,
    pub x: f32,
    pub z: f32,
    pub team: Team,
    pub visibility: Visibility,
}

/// 坐标范围校验：游戏地图 15000×15000。
pub const MAP_MAX: f32 = 15000.0;
pub const MAP_MIN: f32 = 0.0;

pub fn validate_coord(x: f32, z: f32) -> bool {
    x >= MAP_MIN && x <= MAP_MAX && z >= MAP_MIN && z <= MAP_MAX
}

pub fn validate_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty() && n.len() <= 64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_roundtrip() {
        assert_eq!(Team::from_str("order"), Some(Team::Order));
        assert_eq!(Team::from_str("CHAOS"), Some(Team::Chaos));
        assert_eq!(Team::from_str("invalid"), None);
        assert_eq!(Team::Order.as_str(), "order");
    }

    #[test]
    fn visibility_roundtrip() {
        assert_eq!(Visibility::from_str("private"), Some(Visibility::Private));
        assert_eq!(Visibility::from_str("PUBLIC"), Some(Visibility::Public));
        assert_eq!(Visibility::from_str("x"), None);
    }

    #[test]
    fn coord_in_bounds() {
        assert!(validate_coord(7500.0, 7500.0));
        assert!(validate_coord(0.0, 15000.0));
    }

    #[test]
    fn coord_out_of_bounds() {
        assert!(!validate_coord(-1.0, 7500.0), "x 负数");
        assert!(!validate_coord(7500.0, 15001.0), "z 超界");
        assert!(!validate_coord(f32::NAN, 7500.0), "NaN 不合法");
    }

    #[test]
    fn name_validation() {
        assert!(validate_name("上路一塔"));
        assert!(validate_name("  spaced  ")); // trim 后非空
        assert!(!validate_name(""));
        assert!(!validate_name("   "));
        assert!(!validate_name(&"x".repeat(65)));
    }
}
