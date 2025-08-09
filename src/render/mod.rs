mod animation;
mod camera;
mod character_cache;
mod map;
mod resource;
mod ui;

pub use animation::*;
pub use camera::*;
pub use character_cache::*;
pub use map::*;
pub use resource::*;
pub use ui::*;

use bevy::prelude::*;

pub struct PluginRender;

impl Plugin for PluginRender {
    fn build(&self, app: &mut App) {
        // Picking plugin will be added when needed
        // app.add_plugins(MeshPickingPlugin);

        // app.insert_resource(ClearColor(palettes::css::BLACK.into()));
        // app.init_resource::<GltfAssets>();

        // app.add_systems(Startup, setup);
        // app.add_systems(Update, load_scene);

        app.add_plugins((PluginCamera, PluginResource, PluginMap));
        app.add_plugins(PluginAnimation);
        app.init_resource::<CharacterResourceCache>();
    }
}

// pub fn setup(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut gltf_assets: ResMut<GltfAssets>,
// ) {
//     system_info!(
//         "render_setup",
//         "Setting up render system with directional light"
//     );

//     system_debug!("render_setup", "Spawned directional light");

//     gltf_assets.load_all(&asset_server);
//     system_info!("render_setup", "Loaded all GLTF assets");
// }

// fn load_scene(
//     mut commands: Commands,
//     q_scene_handle: Query<(Entity, &GltfSceneHandle)>,
//     mut gltf_assets: ResMut<GltfAssets>,
//     gltfs: Res<Assets<Gltf>>,
// ) {
//     let scene_count = q_scene_handle.iter().count();
//     if scene_count > 0 {
//         system_debug!("load_scene", "Loading {} GLTF scenes", scene_count);
//     }

//     q_scene_handle
//         .iter()
//         .for_each(|(entity, gltf_scene_handle)| {
//             system_debug!(
//                 "load_scene",
//                 "Loading scene for entity {:?} with asset {:?}",
//                 entity,
//                 gltf_scene_handle.0
//             );
//             gltf_assets.insert_scene_root(entity, gltf_scene_handle, &gltfs, &mut commands);
//         });
// }
