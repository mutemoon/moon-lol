use bevy::gltf::GltfLoaderSettings;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::light::light_consts::lux;
use bevy::pbr::DefaultOpaqueRendererMethod;
use bevy::prelude::*;
use bevy_gltf_draco::GltfDracoDecoderPlugin;
use lol_render::camera::PluginCamera;

fn main() {
    App::new()
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#lol".to_string()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PluginCamera)
        .add_plugins(GltfDracoDecoderPlugin)
        .add_systems(Startup, setup)
        .init_resource::<LoadingData>()
        .add_systems(Update, update_loading_data)
        .add_systems(Startup, spawn_progress_bar)
        .add_systems(Update, update_progress_bar_ui)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut loading_data: ResMut<LoadingData>,
) {
    let handle = asset_server.load_with_settings(
        GltfAssetLabel::Scene(0).from_asset("maps/output.glb"),
        |s: &mut GltfLoaderSettings| {
            s.validate = false;
        },
    );
    loading_data.loading_assets.push(handle.clone().untyped());
    loading_data.total_assets = 1;
    commands.spawn(WorldAssetRoot(handle));

    ambient_light.brightness = 1500.0;

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 5_000.,
            shadow_maps_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 10000.0,
            ..default()
        }
        .build(),
        Transform::from_xyz(2000.0, 2000.0, 2000.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
//   1. 修改数据结构以记录总量
//   示例中的 LoadingData 只是简单地移除已加载的资产。为了计算进度，你需要保存初始加载的资产总数。

#[derive(Resource, Debug, Default)]
struct LoadingData {
    loading_assets: Vec<UntypedHandle>,
    total_assets: usize, // 新增：记录总数
}

//   2. 计算进度百分比
//   在 update_loading_data 系统中，通过 total_assets 和当前剩余的 loading_assets.len() 计算进度：

fn update_loading_data(
    mut loading_data: ResMut<LoadingData>,
    asset_server: Res<AssetServer>,
    mut mesh_asset_events: MessageReader<AssetEvent<Mesh>>,
) {
    // 移除已加载的资产
    loading_data.loading_assets.retain(|handle| {
        // get_recursive_dependency_load_state 会检查 GLTF 及其引用的所有贴图/缓存
        asset_server
            .get_recursive_dependency_load_state(handle)
            .is_none_or(|state| !state.is_loaded())
    });
    //    12
    // 计算进度 (0.0 到 1.0)
    let progress = if loading_data.total_assets == 0 {
        1.0
    } else {
        (loading_data.total_assets - loading_data.loading_assets.len()) as f32
            / loading_data.total_assets as f32
    };
    //    20
    // 你可以将这个 progress 写入一个 Resource 或直接更新 UI
}

//   3. 创建 UI 进度条
//   在 Bevy UI 中，进度条通常由两个 Node 组成：一个背景容器（固定宽度）和一个前景填充块（宽度百分比随进度变化）。

//     1 // 标记进度条填充部分的组件
#[derive(Component)]
struct ProgressBarFill;
//     4
fn spawn_progress_bar(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::BLACK.into()),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(0.0), // 初始为 0
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE.into()),
                ProgressBarFill,
            ));
        });
}
//   4. 更新 UI 宽度
//   创建一个系统，根据计算出的进度更新进度条的 width：

fn update_progress_bar_ui(
    loading_data: Res<LoadingData>,
    mut query: Query<&mut Node, With<ProgressBarFill>>,
) {
    let progress = (loading_data.total_assets - loading_data.loading_assets.len()) as f32
        / loading_data.total_assets as f32;
    //     7
    for mut node in &mut query {
        node.width = Val::Percent(progress * 100.0);
    }
}
