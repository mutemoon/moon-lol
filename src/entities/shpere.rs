use bevy::{color::palettes::tailwind, prelude::*};

#[derive(Component)]
pub struct DebugSphere {
    pub radius: f32,
    pub color: Color,
}

pub struct PluginDebugSphere;

impl Plugin for PluginDebugSphere {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_sphere_system);
    }
}

fn debug_sphere_system(
    mut commands: Commands,
    query: Query<Entity, Added<DebugSphere>>,
    mut res_materials: ResMut<Assets<StandardMaterial>>,
    mut res_meshes: ResMut<Assets<Mesh>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert((
            Mesh3d(res_meshes.add(Sphere::new(20.0))),
            MeshMaterial3d(res_materials.add(StandardMaterial {
                base_color: Color::from(tailwind::RED_500),
                ..Default::default()
            })),
        ));
    }
}
