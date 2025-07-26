use bevy::{input::mouse::MouseMotion, math::vec3, prelude::*, window::CursorGrabMode};

#[derive(Component)]
struct CameraController {
    move_speed: f32,

    mouse_sensitivity: f32,

    pitch: f32,

    yaw: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_movement, camera_look))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let camera_position = Vec3::new(-1000.0, 1000.0, 1000.0);

    let light_position = Vec3::new(0.0, 1000.0, 0.0);

    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(light_position).looking_at(vec3(0.0, 0.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_position).looking_at(vec3(0.0, 0.0, 0.0), Vec3::Y),
        CameraController {
            move_speed: 500.0,
            mouse_sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
        },
    ));
}

fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &CameraController), With<Camera3d>>,
    time: Res<Time>,
) {
    for (mut transform, controller) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let forward = -*transform.local_z();
        let right = *transform.local_x();
        let up = Vec3::Y;

        if keyboard_input.pressed(KeyCode::KeyW) {
            velocity += forward;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            velocity -= forward;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            velocity -= right;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            velocity += right;
        }

        if keyboard_input.pressed(KeyCode::KeyQ) {
            velocity -= up;
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            velocity += up;
        }

        if velocity.length() > 0.0 {
            velocity = velocity.normalize() * controller.move_speed * time.delta_secs();
            transform.translation += velocity;
        }
    }
}

fn camera_look(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    if window.cursor_options.grab_mode != CursorGrabMode::Locked {
        return;
    }

    for motion in mouse_motion_events.read() {
        for (mut transform, mut controller) in query.iter_mut() {
            controller.yaw -= motion.delta.x * controller.mouse_sensitivity;
            controller.pitch -= motion.delta.y * controller.mouse_sensitivity;

            controller.pitch = controller.pitch.clamp(-1.54, 1.54);

            transform.rotation = Quat::from_axis_angle(Vec3::Y, controller.yaw)
                * Quat::from_axis_angle(Vec3::X, controller.pitch);
        }
    }
}
