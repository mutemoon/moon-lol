use std::time::Duration;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, update_fixed)
        .run();
}

fn setup(mut virtual_time: ResMut<Time<Virtual>>) {
    println!("加速");
    // 只是尽可能加速
    virtual_time.set_relative_speed(10000.0);
}

fn update_fixed(mut times: Local<u64>, fixed: Res<Time<Fixed>>, real: Res<Time<Real>>) {
    *times += 1;
    if *times % 10000 == 0 {
        println!(
            "fps: {}, fixed delta: {}, real delta: {}, fixed elapsed: {}, real elapsed: {}",
            *times as f32 / real.elapsed_secs(),
            fixed.delta_secs(),
            real.delta_secs(),
            fixed.elapsed_secs(),
            real.elapsed_secs(),
        );
    }
}
