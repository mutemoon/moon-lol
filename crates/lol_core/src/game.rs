use bevy::prelude::*;

#[derive(Default)]
pub struct PluginGame {
    pub scenes: Vec<String>,
}

impl Plugin for PluginGame {
    fn build(&self, app: &mut App) {
        app.init_resource::<FixedFrameCount>();
        app.insert_resource(GameScenes(self.scenes.clone()));

        app.add_systems(Startup, startup_load_game_scenes);
        app.add_systems(FixedLast, fixed_update_frame);
    }
}

#[derive(Resource, Default)]
pub struct FixedFrameCount(pub u32);

#[derive(Resource)]
pub struct GameScenes(pub Vec<String>);

fn startup_load_game_scenes(
    mut commands: Commands,
    res_asset_server: Res<AssetServer>,
    scenes: Res<GameScenes>,
) {
    for scene_path in scenes.0.iter() {
        commands.spawn(DynamicWorldRoot(res_asset_server.load(scene_path)));
    }
}

fn fixed_update_frame(mut frame: ResMut<FixedFrameCount>) {
    frame.0 += 1;
}
