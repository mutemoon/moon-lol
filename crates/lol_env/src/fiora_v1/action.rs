use lol_rl_protocol::ActionSchema;

/// 真实移动缩放：策略网络输出的 `move_x/move_z ∈ [-1, 1]` 映射为相对瑞雯 `±100.0` 单位的目标点
pub const MOVE_SCALE: f32 = 100.0;

pub static FIORA_V1_ACTION_SCHEMA: std::sync::LazyLock<ActionSchema> =
    std::sync::LazyLock::new(|| {
        super::FIORA_V1_SPEC
            .action_schema
            .clone()
            .expect("SPEC_FIORA_V1 缺少 action_schema")
    });

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FioraVsRivenRealAction {
    pub move_x: f32,
    pub move_z: f32,
    pub attack: bool,
}

impl FioraVsRivenRealAction {
    pub const fn new(move_x: f32, move_z: f32, attack: bool) -> Self {
        Self {
            move_x,
            move_z,
            attack,
        }
    }

    pub fn from_encoding(encoded: &[f32]) -> Self {
        let move_x = encoded.first().copied().unwrap_or(0.0);
        let move_z = encoded.get(1).copied().unwrap_or(0.0);
        let attack = encoded.get(2).copied().unwrap_or(0.0) > 0.5;
        Self {
            move_x,
            move_z,
            attack,
        }
    }

    pub fn to_encoding(&self) -> Vec<f32> {
        vec![
            self.move_x,
            self.move_z,
            if self.attack { 1.0 } else { 0.0 },
        ]
    }

    pub fn preset_from_index(index: usize) -> Self {
        match index {
            0 => Self::new(0.5, 0.0, false),
            1 => Self::new(-0.5, 0.0, false),
            2 => Self::new(0.0, 0.5, false),
            3 => Self::new(0.0, -0.5, false),
            4 => Self::new(0.0, 0.0, false),
            5 => Self::new(0.0, 0.0, true),
            _ => Self::new(0.0, 0.0, true),
        }
    }

    pub fn preset_index(&self) -> usize {
        if self.attack {
            5
        } else if self.move_x > 0.25 {
            0
        } else if self.move_x < -0.25 {
            1
        } else if self.move_z > 0.25 {
            2
        } else if self.move_z < -0.25 {
            3
        } else {
            4
        }
    }

    pub fn desc(&self) -> &'static str {
        match self.preset_index() {
            0 => "东移 50u",
            1 => "西移 50u",
            2 => "北移 50u",
            3 => "南移 50u",
            4 => "追击瑞雯",
            5 => "攻击瑞雯",
            _ => "未知",
        }
    }
}
