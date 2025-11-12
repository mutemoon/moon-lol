// use bevy::window::CursorGrabMode;
use bevy::math::{Mat4, Vec3A, Vec4};
use bevy::render::camera::{CameraProjection, Projection, SubCameraView};
use bevy::{input::mouse::MouseWheel, prelude::*};

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

pub const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 1911.85, -1289.56);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct CameraInit;

#[derive(Default)]
pub struct PluginCamera;

impl Plugin for PluginCamera {
    fn build(&self, app: &mut App) {
        app.register_type::<Focus>();

        app.add_systems(Startup, setup.in_set(CameraInit));
        app.add_systems(Update, update);
        app.add_systems(FixedUpdate, update_focus);
        app.add_systems(Update, on_wheel);
        app.add_systems(Update, on_mouse_scroll);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
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

fn setup(
    mut commands: Commands,
    // mut window: Query<&mut Window>
) {
    commands.spawn((
        Camera3d::default(),
        CameraState {
            scale: 1.0,
            position: vec3(CAMERA_MIN_X, 0.0, CAMERA_MAX_Y),
        },
        Projection::custom(CustomFlipXProjection::default()),
    ));

    // if let Ok(mut window) = window.single_mut() {
    //     window.cursor_options.grab_mode = CursorGrabMode::Confined;
    // }
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
        movement.z += 1.0;
    } else if cursor_position.y > window_size.y - edge_margin {
        movement.z -= 1.0;
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

#[derive(Clone, Copy, Debug)]
pub struct CustomFlipXProjection {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

impl CameraProjection for CustomFlipXProjection {
    fn get_clip_from_view(&self) -> Mat4 {
        let base = Mat4::perspective_infinite_reverse_rh(self.fov, self.aspect_ratio, self.near);
        let flip = Mat4::from_cols(
            Vec4::new(-1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        flip * base
    }

    fn get_clip_from_view_for_sub(&self, sub_view: &SubCameraView) -> Mat4 {
        let full_width = sub_view.full_size.x as f32;
        let full_height = sub_view.full_size.y as f32;
        let sub_width = sub_view.size.x as f32;
        let sub_height = sub_view.size.y as f32;
        let offset_x = sub_view.offset.x as f32;
        let offset_y = full_height - ((sub_view.offset.y as f32) + sub_height);

        let full_aspect = full_width / full_height;

        let top = self.near * (0.5 * self.fov).tan();
        let bottom = -top;
        let right = top * full_aspect;
        let left = -right;

        let width = right - left;
        let height = top - bottom;

        let left_prime = left + (width * offset_x) / full_width;
        let right_prime = left + (width * (offset_x + sub_width)) / full_width;
        let bottom_prime = bottom + (height * offset_y) / full_height;
        let top_prime = bottom + (height * (offset_y + sub_height)) / full_height;

        let x = (2.0 * self.near) / (right_prime - left_prime);
        let y = (2.0 * self.near) / (top_prime - bottom_prime);
        let a = (right_prime + left_prime) / (right_prime - left_prime);
        let b = (top_prime + bottom_prime) / (top_prime - bottom_prime);

        let proj = Mat4::from_cols(
            Vec4::new(x, 0.0, 0.0, 0.0),
            Vec4::new(0.0, y, 0.0, 0.0),
            Vec4::new(a, b, 0.0, -1.0),
            Vec4::new(0.0, 0.0, self.near, 0.0),
        );

        let flip = Mat4::from_cols(
            Vec4::new(-1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );

        flip * proj
    }

    fn update(&mut self, width: f32, height: f32) {
        if width > 0.0 && height > 0.0 {
            self.aspect_ratio = width / height;
        }
    }

    fn far(&self) -> f32 {
        self.far
    }

    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        let tan_half_fov = (self.fov / 2.0).tan();
        let a = z_near.abs() * tan_half_fov;
        let b = z_far.abs() * tan_half_fov;
        let aspect_ratio = self.aspect_ratio;
        [
            Vec3A::new(a * aspect_ratio, -a, z_near),
            Vec3A::new(a * aspect_ratio, a, z_near),
            Vec3A::new(-a * aspect_ratio, a, z_near),
            Vec3A::new(-a * aspect_ratio, -a, z_near),
            Vec3A::new(b * aspect_ratio, -b, z_far),
            Vec3A::new(b * aspect_ratio, b, z_far),
            Vec3A::new(-b * aspect_ratio, b, z_far),
            Vec3A::new(-b * aspect_ratio, -b, z_far),
        ]
    }
}

impl Default for CustomFlipXProjection {
    fn default() -> Self {
        CustomFlipXProjection {
            fov: core::f32::consts::PI / 4.0,
            near: 0.1,
            far: 1000.0,
            aspect_ratio: 1.0,
        }
    }
}
