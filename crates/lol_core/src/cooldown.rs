use bevy::prelude::*;

use crate::skill::CoolDown;

#[derive(Default)]
pub struct PluginCooldown;

impl Plugin for PluginCooldown {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, fixed_update_cooldown);
    }
}

fn fixed_update_cooldown(time: Res<Time>, mut q_cooldown: Query<&mut CoolDown>) {
    for mut cooldown_state in q_cooldown.iter_mut() {
        if let Some(ref mut timer) = cooldown_state.timer {
            timer.tick(time.delta());
        }
    }
}
