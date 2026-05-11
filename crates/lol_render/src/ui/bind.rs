use bevy::prelude::*;
use lol_base::ui_components::{UIBindData, UIBindOf};

use crate::camera::CameraState;

#[derive(Default)]
pub struct PluginUIBind;

impl Plugin for PluginUIBind {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_ui_bind);
    }
}

fn update_ui_bind(
    mut commands: Commands,
    camera_info: Single<(&Camera, &GlobalTransform), With<CameraState>>,
    q_global_transform: Query<&GlobalTransform>,
    mut q_ui_bind: Query<(Entity, &mut Node, &UIBindOf, &UIBindData)>,
) {
    let (camera, camera_global_transform) = camera_info.into_inner();

    for (entity, mut node, ui_bind_of, ui_bind_data) in q_ui_bind.iter_mut() {
        let Ok(bind_target) = q_global_transform.get(ui_bind_of.0) else {
            commands.entity(entity).despawn();
            continue;
        };

        let Ok(viewport_position) = camera.world_to_viewport(
            camera_global_transform,
            bind_target.translation() + ui_bind_data.position,
        ) else {
            continue;
        };

        commands.entity(entity).insert(Visibility::Visible);

        let viewport_position = viewport_position - ui_bind_data.anchor;
        node.left = Val::Px(viewport_position.x);
        node.top = Val::Px(viewport_position.y);
    }
}
