use bevy::prelude::*;
use lol_base::debug_sphere::DebugSphere;
use lol_core::team::Team;

#[derive(Default)]
pub struct PluginDebugSphere;

impl Plugin for PluginDebugSphere {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_sphere_system);
    }
}

fn debug_sphere_system(
    mut commands: Commands,
    query: Query<(Entity, &DebugSphere, Option<&Team>), Added<DebugSphere>>,
    mut res_materials: ResMut<Assets<StandardMaterial>>,
    mut res_meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, debug_sphere, team) in query.iter() {
        let mut final_color = debug_sphere.color;
        if let Some(t) = team {
            final_color = match t {
                Team::Order => Color::srgba(0.0, 0.5, 1.0, 1.0), // 友方蓝色
                Team::Chaos => Color::srgba(1.0, 0.0, 0.0, 1.0), // 敌方红色
                Team::Neutral => Color::srgba(1.0, 1.0, 0.0, 1.0), // 中立黄色
            };
        }

        commands.entity(entity).insert((
            Mesh3d(res_meshes.add(Sphere::new(debug_sphere.radius))),
            MeshMaterial3d(res_materials.add(StandardMaterial {
                base_color: final_color,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            })),
        ));
    }
}
