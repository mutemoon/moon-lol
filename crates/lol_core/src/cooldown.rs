use bevy::prelude::*;

use crate::skill::CoolDownState;

#[derive(Default)]
pub struct PluginCooldown;

impl Plugin for PluginCooldown {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, fixed_update_cooldown);
    }
}

fn fixed_update_cooldown(time: Res<Time>, mut q_cooldown: Query<&mut CoolDownState>) {
    for mut cooldown_state in q_cooldown.iter_mut() {
        cooldown_state.timer.tick(time.delta());
    }
}
