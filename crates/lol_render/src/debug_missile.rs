use bevy::prelude::*;
use lol_base::debug_missile::DebugMissile;

#[derive(Default)]
pub struct PluginDebugMissile;

impl Plugin for PluginDebugMissile {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_missile_system);
    }
}

const MISSILE_HEIGHT: f32 = 10.0;
const MISSILE_VISUAL_LENGTH: f32 = 100.0;

fn debug_missile_system(
    mut commands: Commands,
    query: Query<(Entity, &DebugMissile), Added<DebugMissile>>,
    mut res_materials: ResMut<Assets<StandardMaterial>>,
    mut res_meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, missile) in query.iter() {
        commands.entity(entity).insert((
            Mesh3d(res_meshes.add(Cuboid::new(
                missile.width,
                MISSILE_HEIGHT,
                MISSILE_VISUAL_LENGTH,
            ))),
            MeshMaterial3d(res_materials.add(StandardMaterial {
                base_color: missile.color,
                unlit: true,
                ..Default::default()
            })),
        ));
    }
}
