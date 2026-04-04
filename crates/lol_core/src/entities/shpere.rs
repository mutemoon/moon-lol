use bevy::prelude::*;

#[derive(Component)]
pub struct DebugSphere {
    pub radius: f32,
    pub color: Color,
}

#[derive(Default)]
pub struct PluginDebugSphere;

impl Plugin for PluginDebugSphere {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_sphere_system);
    }
}

fn debug_sphere_system(
    mut commands: Commands,
    query: Query<(Entity, &DebugSphere), Added<DebugSphere>>,
    mut res_materials: ResMut<Assets<StandardMaterial>>,
    mut res_meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, debug_sphere) in query.iter() {
        commands.entity(entity).insert((
            Mesh3d(res_meshes.add(Sphere::new(debug_sphere.radius))),
            MeshMaterial3d(res_materials.add(StandardMaterial {
                base_color: debug_sphere.color,
                unlit: true,
                ..Default::default()
            })),
        ));
    }
}
