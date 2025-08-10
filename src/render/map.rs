use crate::combat::{AttackState, MovementDestination, Target};
use bevy::{color::palettes, prelude::*};

pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup_map);
        // app.add_systems(Startup, setup_map_placeble);
    }
}

pub fn draw_attack(
    mut gizmos: Gizmos,
    q_attack: Query<(&Transform, &AttackState)>,
    q_movement_destination: Query<(&Transform, &MovementDestination)>,
    q_target: Query<(&Transform, &Target)>,
    q_transform: Query<&Transform>,
) {
    for (transform, attack_info) in q_attack.iter() {
        let Some(target) = attack_info.target else {
            continue;
        };
        let Ok(target_transform) = q_transform.get(target) else {
            continue;
        };
        gizmos.line(
            transform.translation,
            target_transform.translation,
            Color::Srgba(palettes::tailwind::RED_500),
        );
    }

    for (transform, movement_destination) in q_movement_destination.iter() {
        let destination = movement_destination.0;

        gizmos.line(
            transform.translation,
            transform
                .translation
                .clone()
                .with_x(destination.x)
                .with_z(destination.y),
            Color::Srgba(palettes::tailwind::GREEN_500),
        );
    }

    for (transform, target) in q_target.iter() {
        let Ok(target_transform) = q_transform.get(target.0) else {
            continue;
        };
        gizmos.line(
            transform.translation,
            target_transform.translation,
            Color::Srgba(palettes::tailwind::YELLOW_500),
        );
    }
}
