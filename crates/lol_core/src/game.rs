use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

#[derive(Default)]
pub struct PluginGame;

impl Plugin for PluginGame {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameTime>();
        app.init_resource::<GameScenes>();

        app.configure_sets(FixedUpdate, GameSet.run_if(in_state(GameState::Playing)));

        app.add_systems(Startup, startup_load_game_scenes);
        app.add_systems(FixedFirst, fixed_update_game_time.in_set(GameSet));
    }
}

/// 顶层系统集合
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameSet;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(Resource, Default)]
pub struct GameScenes(pub Vec<String>);

#[derive(Resource, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct GameTime {
    pub frame: u32,
    pub elapsed_secs: f32,
}

impl GameTime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self, delta_secs: f32) {
        self.frame += 1;
        self.elapsed_secs += delta_secs;
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.elapsed_secs = 0.0;
    }

    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed_secs
    }

    pub fn delta_secs(&self, last_time: &GameTime) -> f32 {
        self.elapsed_secs - last_time.elapsed_secs
    }
}

impl GameScenes {
    pub fn new(scenes: Vec<String>) -> Self {
        Self(scenes)
    }
}

fn startup_load_game_scenes(
    mut commands: Commands,
    res_asset_server: Res<AssetServer>,
    scenes: Res<GameScenes>,
) {
    for scene_path in scenes.0.iter() {
        commands
            .spawn(DynamicWorldRoot(res_asset_server.load(scene_path)))
            .observe(|_trigger: On<WorldInstanceReady>| info!("场景加载完成"));
    }
}

fn fixed_update_game_time(mut game_time: ResMut<GameTime>, time: Res<Time>) {
    game_time.tick(time.delta_secs());
}
