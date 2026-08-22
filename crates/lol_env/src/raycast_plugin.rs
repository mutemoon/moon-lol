use bevy::prelude::*;

/// 在 Bevy World 中根据屏幕像素坐标进行地平面射线求交。
/// `plane_y` 为地平面的高度（通常为目标角色或地面的 Y 坐标）。
pub fn raycast_ground_plane(world: &mut World, screen_pos: Vec2, plane_y: f32) -> Option<Vec3> {
    // 获取主相机
    let mut camera_query = world.query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>();
    let (camera, camera_transform) = camera_query.iter(world).next()?;

    let ray = camera
        .viewport_to_world(camera_transform, screen_pos)
        .ok()?;
    if ray.direction.y.abs() < 1e-5 {
        return None;
    }

    let t = (plane_y - ray.origin.y) / ray.direction.y;
    if t < 0.0 {
        return None;
    }

    Some(ray.origin + ray.direction * t)
}
