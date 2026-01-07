use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Default)]
pub struct PluginUIBind;

impl Plugin for PluginUIBind {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_ui_bind);
    }
}

#[derive(Component)]
pub struct UIBind {
    pub entity: Entity,
    pub position: Vec3,
    pub offset: Vec2,
    pub anchor: Vec2,
}

fn update_ui_bind(
    mut commands: Commands,
    camera_info: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    q_global_transform: Query<&GlobalTransform>,
    mut q_ui_bind: Query<(Entity, &mut Node, &UIBind, &Children)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let (camera, camera_global_transform) = camera_info.into_inner();

    let Ok(window) = q_window.single() else {
        return;
    };

    for (entity, mut node, ui_bind, children) in q_ui_bind.iter_mut() {
        let Ok(bind_target) = q_global_transform.get(ui_bind.entity) else {
            commands.entity(entity).despawn();
            continue;
        };

        let Ok(viewport_position) = camera.world_to_viewport(
            camera_global_transform,
            bind_target.translation() + ui_bind.position,
        ) else {
            continue;
        };

        let viewport_position = viewport_position + ui_bind.offset;

        if viewport_position.x < 0.0
            || viewport_position.y < 0.0
            || viewport_position.x > window.width()
            || viewport_position.y > window.height()
        {
            commands.entity(entity).insert(Visibility::Hidden);
            for child in children {
                commands.entity(*child).insert(Visibility::Hidden);
            }
            continue;
        }

        for child in children {
            commands.entity(*child).insert(Visibility::Visible);
        }
        commands.entity(entity).insert(Visibility::Visible);

        let viewport_position = viewport_position - ui_bind.anchor;
        node.left = Val::Px(viewport_position.x);
        node.top = Val::Px(viewport_position.y);
    }
}
