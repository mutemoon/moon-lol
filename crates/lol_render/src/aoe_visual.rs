//! AoE 视觉渲染：按 [`AoEVisual`] 的形状构建地面 mesh，并将生命周期系统驱动的
//! `alpha` 同步到材质。
//!
//! - `aoe_visual_spawn_system`：在实体首次出现时按 `shape` 构建 XZ 平面 mesh 与
//!   唯一的半透明（AlphaMode::Blend）材质。mesh 尺寸为形状真实大小，故
//!   Transform.scale 只承载生命周期相位因子（生长/爆发/褪去）。
//! - `aoe_visual_sync_system`：`AoEVisual.alpha` 变化时更新材质 base_color 的 alpha。
//!
//! mesh 为双面 winding（每三角形正反各一份），避免单面剔除导致地面贴花从一侧不可见。

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use lol_core::action::damage::DamageShape;
use lol_core::action::delayed_damage::AoEVisual;

/// 扇形/圆盘/环形的分段数（越大越圆滑）
const SEGMENTS: usize = 32;

#[derive(Default)]
pub struct PluginAoEVisual;

impl Plugin for PluginAoEVisual {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (aoe_visual_spawn_system, aoe_visual_sync_system));
    }
}

fn aoe_visual_spawn_system(
    mut commands: Commands,
    query: Query<(Entity, &AoEVisual), Added<AoEVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, visual) in query.iter() {
        let mesh = build_shape_mesh(&visual.shape);
        let material = StandardMaterial {
            base_color: visual.color.with_alpha(visual.alpha),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        };
        commands.entity(entity).insert((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(material)),
        ));
    }
}

fn aoe_visual_sync_system(
    visuals: Query<(&AoEVisual, &MeshMaterial3d<StandardMaterial>), Changed<AoEVisual>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (visual, mat_handle) in visuals.iter() {
        if let Some(mut mat) = materials.get_mut(mat_handle.0.id()) {
            mat.base_color = visual.color.with_alpha(visual.alpha);
        }
    }
}

/// 按 `DamageShape` 构建 XZ 平面（y=0）flat mesh，尺寸为形状真实大小。
fn build_shape_mesh(shape: &DamageShape) -> Mesh {
    let (positions, indices) = shape_geometry(shape);
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    let uvs = vec![[0.0, 0.0]; positions.len()];
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(double_wind(&indices)));
    mesh
}

/// 返回 (positions, indices)，几何在 XZ 平面（y=0），朝向 +X（由 Transform 旋转定向）。
fn shape_geometry(shape: &DamageShape) -> (Vec<[f32; 3]>, Vec<u32>) {
    match shape {
        DamageShape::Circle { radius } => disc_geometry(*radius),
        // Nearest 视觉上以最大距离为半径的圆盘
        DamageShape::Nearest { max_distance } => disc_geometry(*max_distance),
        DamageShape::Sector { radius, angle } => sector_geometry(*radius, *angle),
        DamageShape::Annular {
            inner_radius,
            outer_radius,
        } => annular_geometry(*inner_radius, *outer_radius),
        DamageShape::Rectangle {
            width,
            length,
            start_distance,
        } => rectangle_geometry(*width, *length, *start_distance),
    }
}

/// 全圆盘（中心在原点）
fn disc_geometry(radius: f32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut positions = vec![[0.0, 0.0, 0.0]]; // index 0 = 圆心
    for i in 0..SEGMENTS {
        let t = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        positions.push([radius * t.cos(), 0.0, radius * t.sin()]);
    }
    let n = SEGMENTS as u32;
    let mut indices = Vec::with_capacity(SEGMENTS * 3);
    for i in 0..n {
        let pi = i + 1;
        let pj = (i + 1) % n + 1;
        indices.extend_from_slice(&[0, pi, pj]);
    }
    (positions, indices)
}

/// 扇形（顶点在原点，沿 +X 方向，张角 angle 度）
fn sector_geometry(radius: f32, angle_deg: f32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let half = angle_deg.to_radians() / 2.0;
    let span = angle_deg.to_radians();
    let mut positions = vec![[0.0, 0.0, 0.0]]; // index 0 = 顶点（施法者）
    for i in 0..=SEGMENTS {
        let t = -half + (i as f32 / SEGMENTS as f32) * span;
        positions.push([radius * t.cos(), 0.0, radius * t.sin()]);
    }
    let mut indices = Vec::with_capacity(SEGMENTS * 3);
    for i in 0..SEGMENTS {
        let pi = i as u32 + 1;
        let pj = i as u32 + 2;
        indices.extend_from_slice(&[0, pi, pj]);
    }
    (positions, indices)
}

/// 环形（内半径 inner，外半径 outer）
fn annular_geometry(inner: f32, outer: f32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut positions = Vec::with_capacity(SEGMENTS * 2);
    for i in 0..SEGMENTS {
        let t = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        positions.push([inner * t.cos(), 0.0, inner * t.sin()]); // inner[i] @ i
    }
    for i in 0..SEGMENTS {
        let t = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        positions.push([outer * t.cos(), 0.0, outer * t.sin()]); // outer[i] @ N+i
    }
    let n = SEGMENTS as u32;
    let mut indices = Vec::with_capacity(SEGMENTS * 6);
    for i in 0..n {
        let ii = i; // inner[i]
        let ij = (i + 1) % n; // inner[i+1]
        let oi = n + i; // outer[i]
        let oj = n + (i + 1) % n; // outer[i+1]
        indices.extend_from_slice(&[ii, ij, oj, ii, oj, oi]);
    }
    (positions, indices)
}

/// 矩形（沿 +X 从 start_distance 延伸 length，左右各 width/2）
fn rectangle_geometry(width: f32, length: f32, start_distance: f32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let hw = width / 2.0;
    let s = start_distance;
    let e = start_distance + length;
    let positions = vec![
        [s, 0.0, -hw], // 0 近左
        [e, 0.0, -hw], // 1 远左
        [e, 0.0, hw],  // 2 远右
        [s, 0.0, hw],  // 3 近右
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    (positions, indices)
}

/// 双面 winding：每个三角形追加反向一份，避免背面剔除。
fn double_wind(indices: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(indices.len() * 2);
    for tri in indices.chunks_exact(3) {
        out.extend_from_slice(tri);
        out.extend_from_slice(&[tri[0], tri[2], tri[1]]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disc_geometry_counts() {
        let (positions, indices) = disc_geometry(325.0);
        assert_eq!(positions.len(), SEGMENTS + 1); // 圆心 + N 周界
        assert_eq!(indices.len(), SEGMENTS * 3); // N 三角形
    }

    #[test]
    fn sector_geometry_counts() {
        let (positions, indices) = sector_geometry(650.0, 35.0);
        assert_eq!(positions.len(), SEGMENTS + 2); // 顶点 + (N+1) 周界
        assert_eq!(indices.len(), SEGMENTS * 3);
    }

    #[test]
    fn annular_geometry_counts() {
        let (positions, indices) = annular_geometry(150.0, 350.0);
        assert_eq!(positions.len(), SEGMENTS * 2); // 内外各 N
        assert_eq!(indices.len(), SEGMENTS * 6); // N 四边形 * 2 三角形 * 3
    }

    #[test]
    fn rectangle_geometry_corners() {
        let (positions, indices) = rectangle_geometry(160.0, 625.0, 400.0);
        assert_eq!(positions.len(), 4);
        assert_eq!(indices.len(), 6);
        // 近端 X = start_distance，远端 X = start_distance + length
        assert_eq!(positions[0][0], 400.0);
        assert_eq!(positions[1][0], 1025.0);
        // 左右半宽
        assert_eq!(positions[0][2], -80.0);
        assert_eq!(positions[2][2], 80.0);
    }

    #[test]
    fn build_shape_mesh_smoke_all_shapes() {
        // 构造不 panic 即可（mesh 为纯数据，无需渲染设备）
        let _ = build_shape_mesh(&DamageShape::Circle { radius: 100.0 });
        let _ = build_shape_mesh(&DamageShape::Nearest {
            max_distance: 200.0,
        });
        let _ = build_shape_mesh(&DamageShape::Sector {
            radius: 300.0,
            angle: 75.0,
        });
        let _ = build_shape_mesh(&DamageShape::Annular {
            inner_radius: 150.0,
            outer_radius: 350.0,
        });
        let _ = build_shape_mesh(&DamageShape::Rectangle {
            width: 160.0,
            length: 625.0,
            start_distance: 400.0,
        });
    }

    #[test]
    fn double_wind_doubles_triangles() {
        let out = double_wind(&[0, 1, 2, 3, 4, 5]);
        // 2 个三角形 -> 4 个（正反各一）
        assert_eq!(out.len(), 12);
        // 第二个为第一个的反向
        assert_eq!(out[0..3], [0, 1, 2]);
        assert_eq!(out[3..6], [0, 2, 1]);
    }
}
