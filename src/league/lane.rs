use crate::combat::Lane;

pub fn u16_to_lane(value: u16) -> Lane {
    match value {
        0 => Lane::Bot,
        1 => Lane::Mid,
        2 => Lane::Top,
        _ => panic!("Unknown lane value: {}", value),
    }
}
