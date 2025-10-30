use std::collections::{hash_map::Iter, HashMap};

use bevy::prelude::*;
use lol_core::Team;

use crate::core::{Action, CommandAction, Map};

#[derive(Default)]
pub struct PluginController;

impl Plugin for PluginController {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, on_key_pressed);
    }
}

#[derive(Component)]
pub struct Controller {
    attack_key: KeyCode,
    stop_key: KeyCode,
    skill_key_map: HashMap<usize, KeyCode>,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            attack_key: KeyCode::KeyA,
            stop_key: KeyCode::KeyS,
            skill_key_map: HashMap::from([
                (1, KeyCode::KeyQ),
                (2, KeyCode::KeyW),
                (3, KeyCode::KeyE),
                (4, KeyCode::KeyR),
            ]),
        }
    }
}

impl Controller {
    pub fn attack_key(&self) -> KeyCode {
        self.attack_key
    }
    pub fn stop_key(&self) -> KeyCode {
        self.stop_key
    }
    pub fn iter_skill_keys(&self) -> Iter<'_, usize, KeyCode> {
        self.skill_key_map.iter()
    }
}

pub fn on_key_pressed(
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut ray_cast: MeshRayCast,
    q_children: Query<&ChildOf>,
    q_controller: Query<(Entity, &Team, &Controller)>,
    q_map: Query<Entity, With<Map>>,
    q_target: Query<(Entity, &Transform, &Team)>,
    res_input: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
) {
    let Some(viewport_position) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = *camera;

    let Ok(ray) = camera.viewport_to_world(camera_transform, viewport_position) else {
        return;
    };

    let filter = |v| {
        for ancestor in q_children.iter_ancestors(v) {
            if q_map.contains(ancestor) {
                return true;
            }
        }
        false
    };

    let mesh_ray_cast_settings = MeshRayCastSettings::default().with_filter(&filter);

    let hits = ray_cast.cast_ray(ray, &mesh_ray_cast_settings);

    let Some(hit) = hits.first() else {
        return;
    };

    let position = hit.1.point;

    for (entity, team, controller) in q_controller.iter() {
        let action = if res_input.just_pressed(controller.attack_key()) {
            let mut min_distance = f32::MAX;
            let mut target = None;
            for (entity, transform, target_team) in q_target.iter() {
                let distance = position.distance(transform.translation);
                if distance < min_distance && target_team != team {
                    min_distance = distance;
                    target = Some(entity);
                }
            }

            let Some(target) = target else {
                continue;
            };
            Some(Action::Attack(target))
        } else if res_input.just_pressed(controller.stop_key()) {
            Some(Action::Stop)
        } else {
            let mut action = None;
            for (skill_id, key) in controller.iter_skill_keys() {
                if res_input.just_pressed(*key) {
                    action = Some(Action::Skill {
                        index: *skill_id,
                        point: position,
                    });
                    break;
                }
            }
            action
        };

        let Some(action) = action else {
            continue;
        };

        commands.entity(entity).trigger(CommandAction { action });
    }
}
