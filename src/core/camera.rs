use crate::{core::ConfigMap, system_debug, system_info};
use bevy::{
    input::keyboard::KeyCode, input::mouse::MouseWheel, prelude::*, window::CursorGrabMode,
};

// 相机基础偏移量
pub const CAMERA_OFFSET_X: f32 = 0.0;
pub const CAMERA_OFFSET_Y: f32 = 1911.85;
pub const CAMERA_OFFSET_Z: f32 = 1289.56;

// 相机距离和位置配置
pub const CAMERA_FAR_Z: f32 = 22000.0;
pub const CAMERA_MIN_X: f32 = 300.0;
pub const CAMERA_MAX_X: f32 = 14400.0;
pub const CAMERA_MIN_Y: f32 = -14765.0;
pub const CAMERA_MAX_Y: f32 = -520.0;

// 键盘轨道控制速度
pub const CAMERA_KEYBOARD_ORBIT_SPEED_X: f32 = 100.0;
pub const CAMERA_KEYBOARD_ORBIT_SPEED_Y: f32 = 50.0;

// 相机地图约束偏移
pub const CAMERA_MAP_CONSTRAIN_OFFSET_LEFT: f32 = -10000.0;
pub const CAMERA_MAP_CONSTRAIN_OFFSET_RIGHT: f32 = 10000.0;
pub const CAMERA_MAP_CONSTRAIN_OFFSET_TOP: f32 = 6000.0;
pub const CAMERA_MAP_CONSTRAIN_OFFSET_BOTTOM: f32 = -2000.0;

pub const CAMERA_OFFSET: Vec3 = Vec3::new(CAMERA_OFFSET_X, CAMERA_OFFSET_Y, CAMERA_OFFSET_Z);

#[derive(Component)]
pub struct Focus;

#[derive(Component)]
pub struct CameraState {
    pub scale: f32,
    pub position: Vec3,
}

impl CameraState {
    pub fn set_position(&mut self, position: Vec3) {
        // self.position.x = position.x.clamp(CAMERA_MIN_X, CAMERA_MAX_X);
        // self.position.z = position.z.clamp(CAMERA_MIN_Y, CAMERA_MAX_Y);
        self.position = position;
    }

    pub fn set_scale(&mut self, scale: f32) {
        // self.scale = scale.clamp(0.1, 1.0);
        self.scale = scale;
    }
}

pub struct PluginCamera;

impl Plugin for PluginCamera {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, update);
        app.add_systems(Update, update_focus);
        app.add_systems(Update, on_wheel);
        app.add_systems(Update, on_mouse_scroll);
        app.add_systems(Update, on_key_m);
    }
}

fn setup(mut commands: Commands, mut window: Query<&mut Window>) {
    commands.spawn((
        Camera3d::default(),
        CameraState {
            scale: 1.0,
            position: vec3(CAMERA_MIN_X, 0.0, CAMERA_MAX_Y),
        },
    ));

    if let Ok(mut window) = window.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Confined;
    }
}

fn update(mut q_camera: Query<(&mut Transform, &CameraState), Changed<CameraState>>) {
    let Ok((mut transform, camera_state)) = q_camera.single_mut() else {
        return;
    };

    transform.translation = camera_state.position + (CAMERA_OFFSET * camera_state.scale);
    transform.look_at(camera_state.position, Vec3::Y);
}

fn update_focus(mut q_camera: Query<&mut CameraState>, q_focus: Query<&Transform, With<Focus>>) {
    let Ok(transform) = q_focus.single() else {
        return;
    };

    let Ok(mut camera_state) = q_camera.single_mut() else {
        return;
    };

    camera_state.set_position(transform.translation);
}

fn on_wheel(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut CameraState, With<Camera3d>>,
) {
    let Ok(mut camera_state) = query.single_mut() else {
        return;
    };

    for event in mouse_wheel_events.read() {
        let new_scale = camera_state.scale - event.y * 0.1;
        camera_state.set_scale(new_scale);
    }
}

fn on_mouse_scroll(window: Query<&Window>, mut camera: Query<&mut CameraState, With<Camera3d>>) {
    let Ok(window) = window.single() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let window_size = Vec2::new(window.width(), window.height());

    let edge_margin = 20.0;

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
    if movement == Vec3::ZERO {
        return;
    }

    let Ok(mut camera_state) = camera.single_mut() else {
        return;
    };

    let new_position = camera_state.position
        + movement
            * Vec3::new(
                CAMERA_KEYBOARD_ORBIT_SPEED_X,
                0.0,
                CAMERA_KEYBOARD_ORBIT_SPEED_Y,
            );

    camera_state.set_position(new_position);
}

fn on_key_m(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    configs: Res<ConfigMap>,
    mut camera: Query<&mut CameraState, With<Camera3d>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        let center_pos = configs.navigation_grid.get_center_pos();

        if let Ok(mut camera_state) = camera.single_mut() {
            camera_state.position = center_pos;
            camera_state.scale = 10.0;
        }
    }
}
