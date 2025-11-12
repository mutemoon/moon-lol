use bevy::prelude::*;

use crate::{Buff, Buffs, EventAttackEnd};

#[derive(Default)]
pub struct PluginFioraE;

impl Plugin for PluginFioraE {
    fn build(&self, app: &mut App) {
        app.add_observer(on_event_attack_end);
    }
}

#[derive(Component, Debug, Default)]
#[require(Buff = Buff { name: "FioraE" })]
pub struct BuffFioraE {
    pub left: i32,
}

fn on_event_attack_end(
    trigger: Trigger<EventAttackEnd>,
    mut commands: Commands,
    q_buffs: Query<&Buffs>,
    mut q_buff_fiora_e: Query<&mut BuffFioraE>,
) {
    let entity = trigger.target();
    let Ok(buffs) = q_buffs.get(entity) else {
        return;
    };

    for buff in buffs.iter() {
        let Ok(mut buff_fiora_e) = q_buff_fiora_e.get_mut(buff) else {
            continue;
        };

        buff_fiora_e.left -= 1;

        if buff_fiora_e.left <= 0 {
            commands.entity(buff).despawn();
        }
    }
}
