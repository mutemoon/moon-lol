pub mod animation;
pub mod mesh;
pub mod mesh_shadow;
pub mod particle;
pub mod skeleton;
pub mod skin;
use bevy::prelude::*;

#[derive(Default)]
pub struct PluginSkin;

impl Plugin for PluginSkin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_character_particle_despawn);
        app.add_observer(on_command_character_particle_spawn);
        app.add_observer(on_command_skin_animation_spawn);
        app.add_observer(on_command_skin_mesh_spawn);
        app.add_observer(on_command_skin_skeleton_spawn);
        app.add_observer(on_command_skin_spawn);

        app.add_systems(Update, update_skin_scale);
        app.add_systems(Update, update_skin_spawn);
        app.add_systems(Update, update_skin_skeleton_spawn);
        app.add_systems(Update, update_skin_animation_spawn);
        app.add_systems(Update, update_skin_mesh_spawn);
    }
}
