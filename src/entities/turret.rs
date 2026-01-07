use bevy::prelude::*;

use crate::{Aggro, CommandAttackAutoStart, EventAggroTargetFound};

#[derive(Default)]
pub struct PluginTurret;

impl Plugin for PluginTurret {
    fn build(&self, app: &mut App) {
        app.add_observer(on_event_aggro_target_found);
    }
}

#[derive(Component)]
#[require(Aggro = Aggro { range: 1000.0 })]
pub struct Turret;

fn on_event_aggro_target_found(
    trigger: On<EventAggroTargetFound>,
    mut commands: Commands,
    q_turret: Query<Entity, With<Turret>>,
) {
    let entity = trigger.event_target();

    if q_turret.get(entity).is_ok() {
        debug!("{} 对仇恨目标 {} 发起攻击", entity, trigger.target);

        commands.trigger(CommandAttackAutoStart {
            entity,
            target: trigger.target,
        });
    }
}
