use lol_rl_protocol::ActionSchema;

pub static FIORA_V0_ACTION_SCHEMA: std::sync::LazyLock<ActionSchema> =
    std::sync::LazyLock::new(|| {
        super::FIORA_V0_SPEC
            .action_schema
            .clone()
            .expect("SPEC_FIORA_V0 缺少 action_schema")
    });

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FioraVsRivenAction {
    MoveEast50 = 0,
    MoveWest50 = 1,
    MoveNorth50 = 2,
    MoveSouth50 = 3,
    AttackRiven = 4,
}

impl FioraVsRivenAction {
    #[allow(non_upper_case_globals)]
    pub const TeleportEast50: Self = Self::MoveEast50;
    #[allow(non_upper_case_globals)]
    pub const TeleportWest50: Self = Self::MoveWest50;
    #[allow(non_upper_case_globals)]
    pub const TeleportNorth50: Self = Self::MoveNorth50;
    #[allow(non_upper_case_globals)]
    pub const TeleportSouth50: Self = Self::MoveSouth50;

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::MoveEast50,
            1 => Self::MoveWest50,
            2 => Self::MoveNorth50,
            3 => Self::MoveSouth50,
            4 => Self::AttackRiven,
            _ => Self::AttackRiven,
        }
    }

    pub fn to_index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::MoveEast50 => "东移50u",
            Self::MoveWest50 => "西移50u",
            Self::MoveNorth50 => "北移50u",
            Self::MoveSouth50 => "南移50u",
            Self::AttackRiven => "攻击瑞雯",
        }
    }
}
