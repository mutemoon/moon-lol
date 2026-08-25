use bevy::asset::{RecursiveDependencyLoadState, UntypedHandle};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::world_serialization::WorldInstanceReady;

use crate::character::CharacterReady;

#[derive(Default)]
pub struct PluginGame;

impl Plugin for PluginGame {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<StatesPlugin>() {
            app.add_plugins(StatesPlugin);
        }

        app.init_state::<GameState>();
        app.init_resource::<GameTime>();
        app.init_resource::<GameScenes>();
        app.init_resource::<LoadingTracker>();

        app.configure_sets(FixedFirst, GameSet.run_if(in_state(GameState::Playing)));
        app.configure_sets(FixedUpdate, GameSet.run_if(in_state(GameState::Playing)));

        app.add_systems(Startup, startup_load_game_scenes);
        app.add_systems(
            Update,
            update_loading_progress_and_check_ready.run_if(in_state(GameState::Loading)),
        );
        app.add_systems(FixedFirst, fixed_update_game_time.in_set(GameSet));
        app.add_observer(on_reset_game_time);
    }
}

pub fn on_reset_game_time(
    _trigger: On<crate::action::EventReset>,
    mut game_time: ResMut<GameTime>,
) {
    info!("收到 EventReset，重置 GameTime");
    game_time.reset();
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

/// 等待场景反序列化就绪标记（挂载在 DynamicWorldRoot 所在实体上，收到 WorldInstanceReady 后移除）
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct WaitSceneReady;

/// 等待单个或多个 Asset（及其递归依赖）全部加载完成
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct WaitAssets(pub Vec<UntypedHandle>);

pub type WaitAsset = WaitAssets;

impl WaitAssets {
    pub fn new(handles: Vec<UntypedHandle>) -> Self {
        Self(handles)
    }

    pub fn from_handle<T: Asset>(handle: &Handle<T>) -> Self {
        Self(vec![handle.clone().untyped()])
    }

    pub fn from_untyped(handle: UntypedHandle) -> Self {
        Self(vec![handle])
    }

    pub fn from_handles<T: Asset>(handles: &[Handle<T>]) -> Self {
        Self(handles.iter().map(|h| h.clone().untyped()).collect())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: Asset> From<Handle<T>> for WaitAssets {
    fn from(handle: Handle<T>) -> Self {
        Self(vec![handle.untyped()])
    }
}

impl<T: Asset> From<&Handle<T>> for WaitAssets {
    fn from(handle: &Handle<T>) -> Self {
        Self(vec![handle.clone().untyped()])
    }
}

impl From<UntypedHandle> for WaitAssets {
    fn from(handle: UntypedHandle) -> Self {
        Self(vec![handle])
    }
}

impl From<Vec<UntypedHandle>> for WaitAssets {
    fn from(handles: Vec<UntypedHandle>) -> Self {
        Self(handles)
    }
}

/// 等待当前角色实体拥有 CharacterReady 组件
#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct WaitCharacterReady;

/// 通用自定义等待任务标记（只要存在该组件就阻止进入 GameState::Playing，完成后 remove 或 despawn）
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct WaitTask(pub String);

/// 全局加载进度资源
#[derive(Resource, Default, Debug, Clone)]
pub struct LoadingTracker {
    pub has_started_loading: bool,
    pub pending_scenes: usize,
    pub pending_assets: usize,
    pub pending_characters: usize,
    pub pending_tasks: usize,
}

impl LoadingTracker {
    pub fn is_all_ready(&self) -> bool {
        self.pending_scenes == 0
            && self.pending_assets == 0
            && self.pending_characters == 0
            && self.pending_tasks == 0
    }

    pub fn total_pending(&self) -> usize {
        self.pending_scenes + self.pending_assets + self.pending_characters + self.pending_tasks
    }
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
            .spawn((
                WaitSceneReady,
                DynamicWorldRoot(res_asset_server.load(scene_path)),
            ))
            .observe(|trigger: On<WorldInstanceReady>, mut commands: Commands| {
                info!("场景加载完成");
                commands
                    .entity(trigger.event_target())
                    .remove::<WaitSceneReady>();
            });
    }
}

fn update_loading_progress_and_check_ready(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tracker: ResMut<LoadingTracker>,
    q_scenes: Query<Entity, With<WaitSceneReady>>,
    mut q_wait_assets: Query<(Entity, &mut WaitAssets)>,
    q_wait_characters: Query<Entity, With<WaitCharacterReady>>,
    q_character_ready: Query<&CharacterReady>,
    q_tasks: Query<Entity, With<WaitTask>>,
) {
    let pending_scenes = q_scenes.iter().count();

    // 检查 Assets 加载状态
    let mut pending_assets = 0;
    for (entity, mut wait_assets) in q_wait_assets.iter_mut() {
        wait_assets.0.retain(|handle| {
            let state = asset_server.get_recursive_dependency_load_state(handle);
            !matches!(
                state,
                Some(RecursiveDependencyLoadState::Loaded | RecursiveDependencyLoadState::Failed(_))
            )
        });

        if wait_assets.0.is_empty() {
            commands.entity(entity).remove::<WaitAssets>();
        } else {
            pending_assets += wait_assets.0.len();
        }
    }

    // 检查角色实体就绪状态
    let mut pending_characters = 0;
    for entity in q_wait_characters.iter() {
        if q_character_ready.get(entity).is_ok() {
            commands.entity(entity).remove::<WaitCharacterReady>();
        } else {
            pending_characters += 1;
        }
    }

    let pending_tasks = q_tasks.iter().count();
    let total_pending = pending_scenes + pending_assets + pending_characters + pending_tasks;

    if total_pending > 0 {
        tracker.has_started_loading = true;
    }

    tracker.pending_scenes = pending_scenes;
    tracker.pending_assets = pending_assets;
    tracker.pending_characters = pending_characters;
    tracker.pending_tasks = pending_tasks;

    info!(
        "{}加载进度: 场景: {}, 资源: {}, 角色: {}, 任务: {}, 总计: {}",
        if tracker.is_all_ready() { "✅" } else { "⏳" },
        pending_scenes,
        pending_assets,
        pending_characters,
        pending_tasks,
        total_pending
    );
    if tracker.is_all_ready() {
        info!("所有加载项已全部就绪，进入 GameState::Playing");
        commands.set_state(GameState::Playing);
    }
}

fn fixed_update_game_time(mut game_time: ResMut<GameTime>, time: Res<Time>) {
    debug!(
        "游戏时间更新: frame {}, elapsed_secs {}",
        game_time.frame, game_time.elapsed_secs
    );
    game_time.tick(time.delta_secs());
}
