//! 粒子渲染 server：因为英雄列表与英雄粒子的加载已移交 apps/client，
//! 所以本进程只负责「播放」——不再扫描 assets、不再加载 ConfigVfx。
//! 唯一的播放输入是一段 `ConfigVfxSystemDefinition` 的 RON 字符串。
//!
//! RPC 命令：
//!   - play_particle { def }  → 把 RON 反序列化为 ConfigVfxSystemDefinition，
//!                              解析贴图为线性 handle，注册进 Assets 并在锚点实体上播放（自动停止上一个）
//!   - stop_particle          → 停止当前播放的粒子系统

use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use clap::Parser;
use lol_base::hash_key::{HashKey, LoadHashKeyTrait};
use lol_base_render::camera::{Focus, PluginCamera};
use lol_base_render::particle::{
    CommandParticleDespawn, CommandParticleSpawn, ConfigVfxSystemDefinition, VfxTexture,
};
use lol_core::map::{MAP_HEIGHT, MAP_WIDTH, PluginMap};
use lol_particle::PluginParticle;
use lol_render::map::PluginRenderMap;
use lol_rpc::{CommandWsRequest, RpcAppExt, respond};
use lol_server::PluginWsServer;
use serde::Deserialize;
use serde_json::json;

#[derive(Parser)]
#[command(name = "lol_particle")]
struct Args {
    /// WebSocket 服务端口（与游戏 server 默认的 9001 区分）
    #[arg(long, default_value = "9002")]
    ws_port: u16,
}

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    // 因为更新后的 Bevy 仅在设备启用 PASSTHROUGH_SHADERS 时才将 SpirV 走 wgpu passthrough（原样使用），
                    // 否则回退 naga 解析而无法处理这些 DXC 编译的 League SPIR-V（报 Unable to find entry point 'main'），
                    // 所以显式向设备请求该特性（在保留默认 TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES 的基础上追加）。
                    render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                        features: WgpuFeatures::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                            | WgpuFeatures::PASSTHROUGH_SHADERS,
                        ..default()
                    })),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "粒子渲染 Server".to_string(),
                        resolution: (300, 300).into(),
                        position: WindowPosition::At((0, 1000).into()),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(PluginParticle)
        .add_plugins(PluginMap)
        .add_plugins(PluginRenderMap)
        .add_plugins(PluginCamera)
        .add_plugins(PluginWsServer {
            ws_port: args.ws_port,
        })
        .add_plugins(PluginParticleRpc)
        .run();
}

// ---------------------------------------------------------------------------
// RPC 插件：注册粒子播放相关命令
// ---------------------------------------------------------------------------

struct PluginParticleRpc;

impl Plugin for PluginParticleRpc {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayingVfx>();

        app.register_rpc::<PlayParticleParams>("play_particle");
        app.register_rpc::<StopParticleParams>("stop_particle");

        app.add_observer(on_play_particle);
        app.add_observer(on_stop_particle);

        app.add_systems(Startup, setup_stage);
    }
}

#[derive(Deserialize)]
struct PlayParticleParams {
    /// 一段 ConfigVfxSystemDefinition 的 RON 字符串（由 apps/client 从英雄粒子配置中提取）
    def: String,
}

#[derive(Deserialize)]
struct StopParticleParams {}

/// 当前播放状态：因为播放输入是临时 RON，所以为每次播放分配自增 hash 注册定义，
/// 切换/停止时用该 hash 找回并 despawn。
#[derive(Resource, Default)]
struct PlayingVfx {
    /// 正在播放的粒子 hash（用于停止上一个）
    current: Option<u32>,
    /// 自增计数器，为每次播放分配唯一 hash
    next: u32,
}

/// 粒子播放锚点（所有播放命令的粒子都在此实体上生成）
#[derive(Component)]
struct ParticleAnchor;

fn setup_stage(mut commands: Commands) {
    // 粒子播放锚点，放在地图中心，添加 Focus 聚焦视角
    let center = Vec3::new(MAP_WIDTH / 2.0, 100.0, MAP_HEIGHT / 2.0);
    commands.spawn((ParticleAnchor, Focus, Transform::from_translation(center)));
}

fn on_play_particle(
    event: On<CommandWsRequest<PlayParticleParams>>,
    q_anchor: Query<Entity, With<ParticleAnchor>>,
    asset_server: Res<AssetServer>,
    mut assets_def: ResMut<Assets<ConfigVfxSystemDefinition>>,
    mut playing: ResMut<PlayingVfx>,
    mut commands: Commands,
) {
    let Ok(anchor) = q_anchor.single() else {
        respond(&event, Err("找不到粒子锚点实体".to_string()));
        return;
    };

    // 唯一输入：ConfigVfxSystemDefinition 的 RON 字符串
    let mut def: ConfigVfxSystemDefinition = match ron::from_str(&event.params.def) {
        Ok(def) => def,
        Err(e) => {
            respond(
                &event,
                Err(format!("解析 ConfigVfxSystemDefinition RON 失败: {e}")),
            );
            return;
        }
    };

    // 因为绕过了 ConfigVfxLoader，所以在此复刻其贴图解析：把每个 VfxTexture 的 path
    // 以线性色彩空间（material override 的 base_texture 例外为 sRGB）加载并回填 handle。
    resolve_system_textures(&mut def, &asset_server);

    let particle_name = def.particle_name.clone();

    // 停止上一个粒子
    if let Some(prev) = playing.current.take() {
        commands
            .entity(anchor)
            .trigger(move |entity| CommandParticleDespawn {
                entity,
                vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(prev),
            });
    }

    // 为本次播放分配唯一 hash 并注册定义，供 spawn observer 通过 load_hash 查回
    playing.next = playing.next.wrapping_add(1).max(1);
    let hash = playing.next;
    assets_def.add_hash(hash, def);

    commands
        .entity(anchor)
        .trigger(move |entity| CommandParticleSpawn {
            entity,
            vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(hash),
            rotation: None,
        });
    playing.current = Some(hash);
    respond(
        &event,
        Ok(json!({ "hash": hash, "particle_name": particle_name })),
    );
}

fn on_stop_particle(
    event: On<CommandWsRequest<StopParticleParams>>,
    q_anchor: Query<Entity, With<ParticleAnchor>>,
    mut playing: ResMut<PlayingVfx>,
    mut commands: Commands,
) {
    let Ok(anchor) = q_anchor.single() else {
        respond(&event, Err("找不到粒子锚点实体".to_string()));
        return;
    };

    if let Some(prev) = playing.current.take() {
        commands
            .entity(anchor)
            .trigger(move |entity| CommandParticleDespawn {
                entity,
                vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(prev),
            });
    }
    respond(&event, Ok(json!({})));
}

// ---------------------------------------------------------------------------
// 辅助函数：贴图解析（复刻 ConfigVfxLoader::resolve_texture 的色彩空间规则）
// ---------------------------------------------------------------------------

/// 将单个 VfxTexture 的 path 以指定色彩空间加载为 Handle<Image> 并回填。
fn resolve_texture(texture: &mut Option<VfxTexture>, asset_server: &AssetServer, is_srgb: bool) {
    if let Some(texture) = texture.as_mut() {
        let path = texture.path.clone();
        texture.handle = asset_server
            .load_with_settings(path, move |settings: &mut ImageLoaderSettings| {
                settings.is_srgb = is_srgb
            });
    }
}

/// 遍历系统内 complex/simple 全部发射器，解析其五类 VfxTexture 字段。
fn resolve_system_textures(system: &mut ConfigVfxSystemDefinition, asset_server: &AssetServer) {
    let emitter_lists = [
        system.complex_emitter_definition_data.as_mut(),
        system.simple_emitter_definition_data.as_mut(),
    ];
    for emitters in emitter_lists.into_iter().flatten() {
        for emitter in emitters.iter_mut() {
            resolve_texture(&mut emitter.texture, asset_server, false);
            resolve_texture(&mut emitter.particle_color_texture, asset_server, false);

            if let Some(distortion) = emitter.distortion_definition.as_mut() {
                resolve_texture(&mut distortion.normal_map_texture, asset_server, false);
            }

            if let Some(overrides) = emitter.material_override_definitions.as_mut() {
                for material_override in overrides.iter_mut() {
                    resolve_texture(&mut material_override.base_texture, asset_server, true);
                }
            }

            if let Some(texture_mult) = emitter.texture_mult.as_mut() {
                resolve_texture(&mut texture_mult.texture_mult, asset_server, false);
            }
        }
    }
}
