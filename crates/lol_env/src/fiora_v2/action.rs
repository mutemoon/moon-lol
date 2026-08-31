use lol_rl_protocol::ActionSchema;

/// 连续偏移缩放系数：[-1, 1] 映射到相对瑞雯 ±100 单位
pub const OFFSET_SCALE: f32 = 100.0;

pub static FIORA_V2_ACTION_SCHEMA: std::sync::LazyLock<ActionSchema> =
    std::sync::LazyLock::new(|| {
        super::FIORA_V2_SPEC
            .action_schema
            .clone()
            .expect("FIORA_V2_SPEC 缺少 action_schema")
    });

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FioraV2DiscreteAction {
    NoOp = 0,
    Move = 1,
    Attack = 2,
    CastQ = 3,
    CastE = 4,
    CastR = 5,
    CastFlash = 6,
}

impl FioraV2DiscreteAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::NoOp,
            1 => Self::Move,
            2 => Self::Attack,
            3 => Self::CastQ,
            4 => Self::CastE,
            5 => Self::CastR,
            6 => Self::CastFlash,
            _ => Self::NoOp,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FioraV2Action {
    pub offset_x: f32,
    pub offset_z: f32,
    pub discrete: FioraV2DiscreteAction,
}

impl FioraV2Action {
    pub const fn new(offset_x: f32, offset_z: f32, discrete: FioraV2DiscreteAction) -> Self {
        Self {
            offset_x,
            offset_z,
            discrete,
        }
    }

    pub fn from_encoding(encoded: &[f32]) -> Self {
        let offset_x = encoded.first().copied().unwrap_or(0.0);
        let offset_z = encoded.get(1).copied().unwrap_or(0.0);
        let discrete_idx = encoded.get(2).copied().unwrap_or(0.0) as u8;
        Self {
            offset_x,
            offset_z,
            discrete: FioraV2DiscreteAction::from_u8(discrete_idx),
        }
    }

    pub fn to_encoding(&self) -> Vec<f32> {
        vec![self.offset_x, self.offset_z, self.discrete.to_u8() as f32]
    }

    pub fn preset_from_index(index: usize) -> Self {
        match index {
            0 => Self::new(0.0, 0.0, FioraV2DiscreteAction::NoOp),
            1 => Self::new(0.5, 0.0, FioraV2DiscreteAction::Move),
            2 => Self::new(-0.5, 0.0, FioraV2DiscreteAction::Move),
            3 => Self::new(0.0, 0.5, FioraV2DiscreteAction::Move),
            4 => Self::new(0.0, -0.5, FioraV2DiscreteAction::Move),
            5 => Self::new(0.0, 0.0, FioraV2DiscreteAction::Move),
            6 => Self::new(0.0, 0.0, FioraV2DiscreteAction::Attack),
            7 => Self::new(0.5, 0.0, FioraV2DiscreteAction::CastQ),
            8 => Self::new(0.0, 0.0, FioraV2DiscreteAction::CastE),
            9 => Self::new(0.0, 0.0, FioraV2DiscreteAction::CastR),
            10 => Self::new(1.0, 0.0, FioraV2DiscreteAction::CastFlash),
            _ => Self::new(0.0, 0.0, FioraV2DiscreteAction::NoOp),
        }
    }

    pub fn preset_index(&self) -> usize {
        match self.discrete {
            FioraV2DiscreteAction::NoOp => 0,
            FioraV2DiscreteAction::Move => {
                if self.offset_x > 0.25 {
                    1
                } else if self.offset_x < -0.25 {
                    2
                } else if self.offset_z > 0.25 {
                    3
                } else if self.offset_z < -0.25 {
                    4
                } else {
                    5
                }
            }
            FioraV2DiscreteAction::Attack => 6,
            FioraV2DiscreteAction::CastQ => 7,
            FioraV2DiscreteAction::CastE => 8,
            FioraV2DiscreteAction::CastR => 9,
            FioraV2DiscreteAction::CastFlash => 10,
        }
    }

    pub fn desc(&self) -> &'static str {
        match self.preset_index() {
            0 => "保持当前 (NoOp)",
            1 => "东移 50u",
            2 => "西移 50u",
            3 => "北移 50u",
            4 => "南移 50u",
            5 => "追击瑞雯",
            6 => "普通攻击",
            7 => "Q-破空斩(东)",
            8 => "E-夺命连刺",
            9 => "R-无双挑战",
            10 => "闪现(东300u)",
            _ => "未知",
        }
    }
}
