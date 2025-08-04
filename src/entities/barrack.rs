use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Barrack {
    pub initial_spawn_time_secs: f32,
    pub wave_spawn_interval_secs: f32,
    pub minion_spawn_interval_secs: f32,
    pub upgrade_interval_secs: f32,
    pub upgrades_before_late_game_scaling: i32,
    pub move_speed_increase_initial_delay_secs: f32,
    pub move_speed_increase_interval_secs: f32,
    pub move_speed_increase_increment: i32,
    pub move_speed_increase_max_times: i32,
    pub exp_radius: f32,
    pub gold_radius: f32,
}
