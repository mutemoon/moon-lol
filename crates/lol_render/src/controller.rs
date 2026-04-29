use std::collections::HashMap;
use std::collections::hash_map::Iter;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::CommandSkinParticleSpawn;
use lol_core::action::{Action, CommandAction};
use lol_core::team::Team;

use crate::camera::CameraState;
use crate::map::Map;

#[derive(Default)]
pub struct PluginController;

impl Plugin for PluginController {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, on_key_pressed);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
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
                (0, KeyCode::KeyQ),
                (1, KeyCode::KeyW),
                (2, KeyCode::KeyE),
                (3, KeyCode::KeyR),
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
    camera: Single<(&Camera, &GlobalTransform), With<CameraState>>,
    mut ray_cast: MeshRayCast,
    _q_children: Query<&ChildOf>,
    q_controller: Query<(Entity, &Team, &Controller)>,
    q_map: Query<Entity, With<Map>>,
    q_target: Query<(Entity, &Transform, &Team)>,
    res_input: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
) {
    // 无按键输入则跳过
    if res_input.get_just_pressed().next().is_none() {
        return;
    }

    debug!("=== on_key_pressed 开始 ===");
    debug!(
        "刚刚按下的键: {:?}",
        res_input.get_just_pressed().collect::<Vec<_>>()
    );

    for (entity, _team, _controller) in q_controller.iter() {
        if res_input.just_pressed(KeyCode::KeyV) {
            debug!("检测到 KeyV 按下，触发 Riven_Q_03_Detonate 粒子特效");
            commands.trigger(CommandSkinParticleSpawn {
                entity,
                hash: hash_bin("Riven_Q_03_Detonate"),
            });
            return;
        };
    }

    let Some(viewport_position) = window.cursor_position() else {
        debug!("无鼠标位置，跳过");
        return;
    };
    debug!("鼠标视口位置: {:?}", viewport_position);

    let (camera, camera_transform) = *camera;

    let Ok(ray) = camera.viewport_to_world(camera_transform, viewport_position) else {
        debug!("从视口位置到世界的射线转换失败");
        return;
    };
    debug!("射线: origin={:?}, dir={:?}", ray.origin, ray.direction);

    let filter = |v| {
        if q_map.contains(v) {
            return true;
        }
        false
    };

    let mesh_ray_cast_settings = MeshRayCastSettings::default().with_filter(&filter);

    let hits = ray_cast.cast_ray(ray, &mesh_ray_cast_settings);
    debug!("射线检测命中数量: {:?}", hits.len());

    let Some(hit) = hits.first() else {
        debug!("无命中点，跳过");
        return;
    };

    let position = hit.1.point;
    debug!("命中点位置: {:?}", position);

    for (entity, team, controller) in q_controller.iter() {
        debug!("检查 Controller Entity: {:?}, Team: {:?}", entity, team);
        let action = if res_input.just_pressed(controller.attack_key()) {
            debug!("检测到攻击键 {:?} 按下", controller.attack_key());
            let mut min_distance = f32::MAX;
            let mut target = None;
            for (entity, transform, target_team) in q_target.iter() {
                let distance = position.distance(transform.translation);
                debug!(
                    "  目标 {:?} 距离: {:.2}, team: {:?}",
                    entity, distance, target_team
                );
                if distance < min_distance && target_team != team {
                    min_distance = distance;
                    target = Some(entity);
                }
            }

            let Some(target) = target else {
                debug!("  未找到有效目标");
                continue;
            };
            debug!("  选择目标: {:?}, 距离: {:.2}", target, min_distance);
            Some(Action::Attack(target))
        } else if res_input.just_pressed(controller.stop_key()) {
            debug!("检测到停止键 {:?} 按下", controller.stop_key());
            Some(Action::Stop)
        } else {
            let mut action = None;
            for (skill_id, key) in controller.iter_skill_keys() {
                if res_input.just_pressed(*key) {
                    debug!("检测到技能键 {:?} 按下, skill_id: {}", key, skill_id);
                    action = Some(Action::Skill {
                        index: *skill_id,
                        point: position.xz(),
                    });
                    break;
                }
            }
            action
        };

        let Some(action) = action else {
            continue;
        };

        debug!("释放 Action {:?}", action);
        commands.trigger(CommandAction { entity, action });
    }
    debug!("=== on_key_pressed 结束 ===");
}
