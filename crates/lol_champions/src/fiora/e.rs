use bevy::prelude::*;
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::{Buff, Buffs};

#[derive(Component, Clone, Debug)]
#[require(Buff = Buff { name: "FioraE" })]
pub struct BuffFioraE {
    pub left: i32,
}

impl Default for BuffFioraE {
    fn default() -> Self {
        Self { left: 2 }
    }
}

pub fn on_event_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_buffs: Query<&Buffs>,
    mut q_buff_fiora_e: Query<&mut BuffFioraE>,
) {
    let entity = trigger.event_target();
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
