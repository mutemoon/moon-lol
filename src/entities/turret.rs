use bevy::prelude::*;

use crate::{Aggro, CommandAttackStart, EventAggroTargetFound, HealthBar, HealthBarType};

#[derive(Default)]
pub struct PluginTurret;

impl Plugin for PluginTurret {
    fn build(&self, app: &mut App) {
        app.add_observer(on_event_aggro_target_found);
    }
}

#[derive(Component)]
#[require(Aggro = Aggro { range: 750.0 }, HealthBar = HealthBar { bar_type: HealthBarType::Turret })]
pub struct Turret;

fn on_event_aggro_target_found(
    trigger: On<EventAggroTargetFound>,
    mut commands: Commands,
    q_turret: Query<Entity, With<Turret>>,
) {
    let entity = trigger.event_target();

    if q_turret.get(entity).is_ok() {
        commands.trigger(CommandAttackStart {
            entity,
            target: trigger.target,
        });
    }
}
