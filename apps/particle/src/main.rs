use std::collections::VecDeque;
use std::sync::Mutex;

use bevy::asset::AssetMetaCheck;
use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use lol_base::hash_key::{HashKey, LoadHashKeyTrait};
use lol_base_render::camera::{Focus, PluginCamera};
use lol_base_render::particle::{ConfigVfxSystemDefinition, VfxTexture};
use lol_particle::{CommandParticleDespawn, CommandParticleSpawn, PluginParticle};
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// 全局指令队列与 Bridge
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WebCommand {
    #[serde(rename = "PLAY_RON")]
    PlayRon { ron: String },
    #[serde(rename = "PLAY_EMITTER")]
    PlayEmitter { ron: String, index: usize },
    #[serde(rename = "STOP")]
    Stop,
    #[serde(rename = "RESET_CAMERA")]
    ResetCamera,
}

static COMMAND_QUEUE: Mutex<Option<VecDeque<WebCommand>>> = Mutex::new(None);

#[allow(dead_code)]
fn push_command(cmd: WebCommand) {
    if let Ok(mut lock) = COMMAND_QUEUE.lock() {
        if let Some(queue) = lock.as_mut() {
            queue.push_back(cmd);
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn post_bevy_command(json_str: &str) {
    if let Ok(cmd) = serde_json::from_str::<WebCommand>(json_str) {
        push_command(cmd);
    } else {
        web_sys::console::warn_1(&format!("无效的 Bevy 命令 JSON: {}", json_str).into());
    }
}

// ---------------------------------------------------------------------------
// 运行时状态资源
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
struct ParticlePlayerState {
    current_hash: Option<u32>,
    next_hash: u32,
    anchor_entity: Option<Entity>,
}

fn main() {
    // 初始化命令队列
    if let Ok(mut lock) = COMMAND_QUEUE.lock() {
        *lock = Some(VecDeque::new());
    }

    App::new()
        .add_plugins(
            DefaultPlugins
                .build()
                .disable::<bevy::audio::AudioPlugin>()
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(RenderPlugin {
                    // 设备启用 PASSTHROUGH_SHADERS 时将 SpirV 走 wgpu passthrough
                    render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                        features: WgpuFeatures::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                            | WgpuFeatures::PASSTHROUGH_SHADERS,
                        ..default()
                    })),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy WASM 粒子播放器".to_string(),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(PluginParticle)
        .add_plugins(PluginCamera)
        .init_resource::<ParticlePlayerState>()
        .add_systems(Startup, setup_stage)
        .add_systems(Update, handle_web_commands)
        .run();
}

fn setup_stage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<ParticlePlayerState>,
) {
    // 1. 在原点放置一个精致暗灰地面网格
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(600.0)))),
        MeshMaterial3d(std_materials.add(Color::srgb(0.08, 0.09, 0.12))),
        Transform::default().with_translation(Vec3::NEG_Y * 2.0),
    ));

    // 2. 放置粒子播放锚点并聚焦
    let anchor = commands
        .spawn((
            Focus,
            Transform::from_xyz(0.0, 50.0, 0.0),
            GlobalTransform::from_xyz(0.0, 50.0, 0.0),
        ))
        .id();

    state.anchor_entity = Some(anchor);

    // 3. 注册浏览器端 window 消息监听并向父级发送 READY
    #[cfg(target_arch = "wasm32")]
    setup_web_event_listener();
}

#[cfg(target_arch = "wasm32")]
fn setup_web_event_listener() {
    use wasm_bindgen::closure::Closure;

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    // 挂载全局调用函数 window.__BEVY_POST_COMMAND__
    let closure = Closure::wrap(Box::new(|json_str: String| {
        post_bevy_command(&json_str);
    }) as Box<dyn FnMut(String)>);

    let _ = js_sys::Reflect::set(
        &window,
        &"__BEVY_POST_COMMAND__".into(),
        closure.as_ref().unchecked_ref(),
    );
    closure.forget();

    // 监听 window 的 message 事件 (便于 iframe 通信)
    let message_closure = Closure::wrap(Box::new(|event: web_sys::MessageEvent| {
        if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
            let s: String = text.into();
            post_bevy_command(&s);
        }
    }) as Box<dyn FnMut(web_sys::MessageEvent)>);

    let _ = window
        .add_event_listener_with_callback("message", message_closure.as_ref().unchecked_ref());
    message_closure.forget();

    // 通知外部界面 Bevy 已就绪
    let ready_msg = serde_json::json!({ "type": "BEVY_READY" }).to_string();
    if let Ok(parent) = window.parent() {
        if let Some(parent) = parent {
            let _ = parent.post_message(&ready_msg.into(), "*");
        }
    }
}

// ---------------------------------------------------------------------------
// 逐帧处理来自外部 Web 的控制指令
// ---------------------------------------------------------------------------

fn handle_web_commands(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets_def: ResMut<Assets<ConfigVfxSystemDefinition>>,
    mut state: ResMut<ParticlePlayerState>,
    mut q_focus: Query<&mut Transform, With<Focus>>,
) {
    let mut pending = Vec::new();
    if let Ok(mut lock) = COMMAND_QUEUE.lock() {
        if let Some(queue) = lock.as_mut() {
            while let Some(cmd) = queue.pop_front() {
                pending.push(cmd);
            }
        }
    }

    let Some(anchor) = state.anchor_entity else {
        return;
    };

    for cmd in pending {
        match cmd {
            WebCommand::PlayRon { ron } => {
                let mut def: ConfigVfxSystemDefinition = match ron::from_str(&ron) {
                    Ok(d) => d,
                    Err(e) => {
                        error!("解析粒子系统 RON 失败: {}", e);
                        continue;
                    }
                };

                resolve_system_textures(&mut def, &asset_server);

                // 停止上一个正在播放的粒子
                if let Some(prev) = state.current_hash.take() {
                    commands
                        .entity(anchor)
                        .trigger(move |entity| CommandParticleDespawn {
                            entity,
                            vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(prev),
                        });
                }

                state.next_hash = state.next_hash.wrapping_add(1).max(1);
                let hash = state.next_hash;
                assets_def.add_hash(hash, def);

                commands
                    .entity(anchor)
                    .trigger(move |entity| CommandParticleSpawn {
                        entity,
                        vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(hash),
                        rotation: None,
                    });
                state.current_hash = Some(hash);
            }
            WebCommand::PlayEmitter { ron, index } => {
                let mut def: ConfigVfxSystemDefinition = match ron::from_str(&ron) {
                    Ok(d) => d,
                    Err(e) => {
                        error!("解析粒子系统 RON 失败: {}", e);
                        continue;
                    }
                };

                // 仅保留指定 index 的发射器
                if let Some(complex) = def.complex_emitter_definition_data.as_mut() {
                    if index < complex.len() {
                        *complex = vec![complex[index].clone()];
                        def.simple_emitter_definition_data = None;
                    }
                }

                resolve_system_textures(&mut def, &asset_server);

                if let Some(prev) = state.current_hash.take() {
                    commands
                        .entity(anchor)
                        .trigger(move |entity| CommandParticleDespawn {
                            entity,
                            vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(prev),
                        });
                }

                state.next_hash = state.next_hash.wrapping_add(1).max(1);
                let hash = state.next_hash;
                assets_def.add_hash(hash, def);

                commands
                    .entity(anchor)
                    .trigger(move |entity| CommandParticleSpawn {
                        entity,
                        vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(hash),
                        rotation: None,
                    });
                state.current_hash = Some(hash);
            }
            WebCommand::Stop => {
                if let Some(prev) = state.current_hash.take() {
                    commands
                        .entity(anchor)
                        .trigger(move |entity| CommandParticleDespawn {
                            entity,
                            vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(prev),
                        });
                }
            }
            WebCommand::ResetCamera => {
                if let Ok(mut transform) = q_focus.get_mut(anchor) {
                    transform.translation = Vec3::new(0.0, 50.0, 0.0);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 贴图资源解析
// ---------------------------------------------------------------------------

fn resolve_texture(texture: &mut Option<VfxTexture>, asset_server: &AssetServer, is_srgb: bool) {
    if let Some(texture) = texture.as_mut() {
        let mut path = texture.path.clone();
        if !path.starts_with("ASSETS/") && !path.starts_with("assets/") {
            path = format!("ASSETS/{}", path.trim_start_matches('/'));
        }
        texture.handle = asset_server
            .load_builder()
            .with_settings(move |settings: &mut ImageLoaderSettings| {
                settings.is_srgb = is_srgb;
            })
            .load(path);
    }
}

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
