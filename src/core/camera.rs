use crate::{
    core::{map::MAP_HEIGHT, Configs, MAP_WIDTH},
    system_debug, system_info,
};
use bevy::{
    input::keyboard::KeyCode, input::mouse::MouseWheel, prelude::*, window::CursorGrabMode,
};

pub const CAMERA_OFFSET_X: f32 = 0.0;
pub const CAMERA_OFFSET_Y: f32 = 1911.85;
pub const CAMERA_OFFSET_Z: f32 = 769.56;

pub const CAMERA_OFFSET: Vec3 = Vec3::new(CAMERA_OFFSET_X, CAMERA_OFFSET_Y, CAMERA_OFFSET_Z);
pub const CAMERA_START_POSITION: Vec3 = Vec3::new(2500.0, 0.0, -2500.0);

pub const CAMERA_MOVE_SPEED: f32 = 125.0;

#[derive(Component)]
pub struct Focus;

pub struct PluginCamera;

impl Plugin for PluginCamera {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Startup, confine_cursor);
        app.add_systems(Update, camera_focus);
        app.add_systems(Update, camera_zoom);
        app.add_systems(Update, edge_scroll_camera);
        app.add_systems(Update, camera_navigation_center);
    }
}

fn setup(mut commands: Commands) {
    let camera_position = CAMERA_START_POSITION + CAMERA_OFFSET;
    system_info!(
        "camera_setup",
        "Setting up camera at position ({:.1}, {:.1}, {:.1}) looking at ({:.1}, {:.1}, {:.1})",
        camera_position.x,
        camera_position.y,
        camera_position.z,
        CAMERA_START_POSITION.x,
        CAMERA_START_POSITION.y,
        CAMERA_START_POSITION.z
    );

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_position).looking_at(CAMERA_START_POSITION, Vec3::Y),
    ));
}

fn camera_focus(
    mut q_camera: Query<&mut Transform, With<Camera3d>>,
    q_focus: Query<&Transform, (With<Focus>, Without<Camera3d>)>,
) {
    if let Ok(transform) = q_focus.single() {
        if let Ok(mut camera_transform) = q_camera.single_mut() {
            let new_position =
                transform.translation + Vec3::new(0.0, CAMERA_OFFSET_Y, CAMERA_OFFSET_Z);
            camera_transform.translation = new_position;
        }
    }
}

fn camera_zoom(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    for event in mouse_wheel_events.read() {
        system_debug!("camera_zoom", "Mouse wheel event: y={:.2}", event.y);

        if let Ok(mut transform) = query.single_mut() {
            camera_zoom_by_mouse_wheel(&mut transform, event.y);
        }
    }
}

fn camera_zoom_by_mouse_wheel(transform: &mut Transform, y: f32) {
    let before_z = transform.translation.z;
    let pred_z = before_z - (y * CAMERA_OFFSET_Z / 100.0);

    transform.translation.z = pred_z;
    // transform.translation.z = pred_z.clamp(CAMERA_OFFSET_Z / 5.0, CAMERA_OFFSET_Z);

    let delta_z = transform.translation.z - before_z;
    let delta_y = delta_z * CAMERA_OFFSET_Y / CAMERA_OFFSET_Z;

    transform.translation.y = transform.translation.y + delta_y;
}

fn edge_scroll_camera(window: Query<&Window>, mut camera: Query<&mut Transform, With<Camera3d>>) {
    let Ok(window) = window.single() else {
        return;
    };
    if let Some(cursor_position) = window.cursor_position() {
        let window_size = Vec2::new(window.width(), window.height());
        let edge_margin = 20.0; // 边缘检测区域大小

        let mut movement = Vec3::ZERO;

        // 检测左右边缘
        if cursor_position.x < edge_margin {
            movement.x -= 1.0;
        } else if cursor_position.x > window_size.x - edge_margin {
            movement.x += 1.0;
        }

        // 检测上下边缘
        if cursor_position.y < edge_margin {
            movement.z -= 1.0;
        } else if cursor_position.y > window_size.y - edge_margin {
            movement.z += 1.0;
        }

        // 如果有移动，应用到相机
        if movement != Vec3::ZERO {
            if let Ok(mut transform) = camera.single_mut() {
                transform.translation += movement * CAMERA_MOVE_SPEED;

                transform.translation.x = transform.translation.x.clamp(0.0, MAP_WIDTH);

                let max_z = transform.translation.y * CAMERA_OFFSET_Z / CAMERA_OFFSET_Y;
                let min_z = max_z - MAP_HEIGHT;

                transform.translation.z = transform.translation.z.clamp(min_z, max_z);
            }
        }
    }
}

fn camera_navigation_center(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    configs: Res<Configs>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        let center_pos = configs.navigation_grid.get_center_pos();

        if let Ok(mut transform) = camera.single_mut() {
            // 将相机对准导航网格中心
            let new_position = center_pos + Vec3::new(0.0, CAMERA_OFFSET_Y, CAMERA_OFFSET_Z);
            transform.translation = new_position;

            // 调整相机朝向，看向中心点
            transform.look_at(center_pos, Vec3::Y);

            camera_zoom_by_mouse_wheel(&mut transform, -700.0);
        }
    }
}

fn confine_cursor(mut window: Query<&mut Window>) {
    if let Ok(mut window) = window.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Confined;
    }
}
