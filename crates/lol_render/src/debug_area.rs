use bevy::prelude::*;
use lol_base::debug_area::DebugArea;

#[derive(Default)]
pub struct PluginDebugArea;

impl Plugin for PluginDebugArea {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_area_system);
    }
}

fn debug_area_system(
    mut commands: Commands,
    query: Query<(Entity, &DebugArea), Added<DebugArea>>,
    mut res_materials: ResMut<Assets<StandardMaterial>>,
    mut res_meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, area) in query.iter() {
        // 单位半径圆柱体作为地面圆盘指示器，实际大小由 Transform.scale 控制
        // update_attached_fields 每帧根据当前半径更新 scale
        commands.entity(entity).insert((
            Mesh3d(res_meshes.add(Cylinder::new(1.0, 0.2))),
            MeshMaterial3d(res_materials.add(StandardMaterial {
                base_color: area.color,
                unlit: true,
                ..Default::default()
            })),
        ));
    }
}
