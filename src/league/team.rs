use crate::combat::Team;

pub fn u32_to_team(value: u32) -> Team {
    match value {
        100 => Team::Order,
        200 => Team::Chaos,
        300 => Team::Neutral,
        _ => panic!("Unknown team value: {}", value),
    }
}

pub fn u32_option_to_team(value: Option<u32>) -> Team {
    match value {
        Some(value) => u32_to_team(value),
        None => Team::default(),
    }
}
