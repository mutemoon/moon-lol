use lol_rl_protocol::ActionSchema;

pub const FIORA_V3_OFFSET_SCALE: f32 = 100.0;

pub static FIORA_V3_ACTION_SCHEMA: std::sync::LazyLock<ActionSchema> =
    std::sync::LazyLock::new(|| {
        super::FIORA_V3_SPEC
            .action_schema
            .clone()
            .expect("FIORA_V3_SPEC 缺少 action_schema")
    });

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FioraV3DiscreteAction {
    NoOp = 0,
    Move = 1,
    Attack = 2,
}

impl FioraV3DiscreteAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::NoOp,
            1 => Self::Move,
            2 => Self::Attack,
            _ => Self::NoOp,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FioraV3Action {
    pub offset_x: f32,
    pub offset_z: f32,
    pub target_idx: u8,
    pub discrete: FioraV3DiscreteAction,
}

impl FioraV3Action {
    pub const fn new(offset_x: f32, offset_z: f32, discrete: FioraV3DiscreteAction) -> Self {
        Self {
            offset_x,
            offset_z,
            target_idx: 0,
            discrete,
        }
    }

    pub const fn with_target(
        offset_x: f32,
        offset_z: f32,
        target_idx: u8,
        discrete: FioraV3DiscreteAction,
    ) -> Self {
        Self {
            offset_x,
            offset_z,
            target_idx,
            discrete,
        }
    }

    pub fn from_encoding(encoded: &[f32]) -> Self {
        let offset_x = encoded.first().copied().unwrap_or(0.0);
        let offset_z = encoded.get(1).copied().unwrap_or(0.0);
        if encoded.len() >= 4 {
            let discrete_idx = encoded.get(2).copied().unwrap_or(0.0) as u8;
            let target_idx = encoded.get(3).copied().unwrap_or(0.0) as u8;
            Self {
                offset_x,
                offset_z,
                target_idx,
                discrete: FioraV3DiscreteAction::from_u8(discrete_idx),
            }
        } else {
            let discrete_idx = encoded.get(2).copied().unwrap_or(0.0) as u8;
            Self {
                offset_x,
                offset_z,
                target_idx: 0,
                discrete: FioraV3DiscreteAction::from_u8(discrete_idx),
            }
        }
    }

    pub fn to_encoding(&self) -> Vec<f32> {
        vec![
            self.offset_x,
            self.offset_z,
            self.discrete.to_u8() as f32,
            self.target_idx as f32,
        ]
    }

    pub fn preset_from_index(index: usize) -> Self {
        match index {
            0 => Self::new(0.0, 0.0, FioraV3DiscreteAction::NoOp),
            1 => Self::new(0.5, 0.0, FioraV3DiscreteAction::Move),
            2 => Self::new(0.0, 0.0, FioraV3DiscreteAction::Attack),
            _ => Self::new(0.0, 0.0, FioraV3DiscreteAction::NoOp),
        }
    }

    pub fn preset_index(&self) -> usize {
        match self.discrete {
            FioraV3DiscreteAction::NoOp => 0,
            FioraV3DiscreteAction::Move => 1,
            FioraV3DiscreteAction::Attack => 2,
        }
    }

    pub fn desc(&self) -> &'static str {
        match self.discrete {
            FioraV3DiscreteAction::NoOp => "保持当前 (NoOp)",
            FioraV3DiscreteAction::Move => "移动",
            FioraV3DiscreteAction::Attack => "普通攻击",
        }
    }
}
