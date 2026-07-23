use bevy::prelude::*;
use league_utils::{get_shader_handle, get_shader_handle_by_hash};

use crate::loaders::shader::ShaderMapAsset;
use crate::particle::particle::quad::ParticleMaterialQuad;
use crate::particle::utils::MaterialPath;

#[derive(Resource, Default)]
pub struct ResourceShaderMapHandle(pub Handle<ShaderMapAsset>);

/// 记录已插入的 Shader handle，用于 debug 检测
#[derive(Resource, Default)]
pub struct DebugShaderHandles(pub Vec<Handle<Shader>>);

/// 每 5 秒检测一次 Shader 是否能取出
#[derive(Resource)]
pub struct ShaderCheckTimer(pub Timer);

impl Default for ShaderCheckTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}

pub fn startup_load_shaders(
    asset_server: Res<AssetServer>,
    mut res_shader_map_handle: ResMut<ResourceShaderMapHandle>,
) {
    res_shader_map_handle.0 = asset_server.load("shaders/map.ron");
}

pub fn update_shaders(
    mut commands: Commands,
    res_shader_map_handle: Option<Res<ResourceShaderMapHandle>>,
    mut res_assets_shader_map: ResMut<Assets<ShaderMapAsset>>,
    mut res_assets_shader: ResMut<Assets<Shader>>,
    mut debug_handles: ResMut<DebugShaderHandles>,
) {
    let Some(handle) = res_shader_map_handle.as_ref() else {
        return;
    };
    let Some(shader_map) = res_assets_shader_map.remove(handle.0.id()) else {
        return;
    };

    for (shader_type, inner_map) in shader_map.entries {
        for (u64_hash, shader) in inner_map {
            let shader_handle = get_shader_handle_by_hash(shader_type, u64_hash);
            let _ = res_assets_shader.insert(shader_handle.id(), shader);
            debug_handles.0.push(shader_handle);
        }
    }

    info!("update_shaders: 已注入 {} 个 Shader", debug_handles.0.len());
    commands.remove_resource::<ResourceShaderMapHandle>();
}

pub fn debug_check_shaders(
    time: Res<Time>,
    mut timer: ResMut<ShaderCheckTimer>,
    res_assets_shader: Res<Assets<Shader>>,
    debug_handles: Res<DebugShaderHandles>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    let total = debug_handles.0.len();
    let valid = debug_handles
        .0
        .iter()
        .filter(|h| res_assets_shader.get(h.id()).is_some())
        .count();
    let test_handle = get_shader_handle(ParticleMaterialQuad::VERT_SHADER, &vec![]);
    let has_test_shader = res_assets_shader.get(test_handle.id()).is_some();
    info!(
        "debug_check_shaders: ParticleMaterialQuad VERT_SHADER ({:?}) exists: {}",
        test_handle, has_test_shader
    );

    let test_frag_handle = get_shader_handle(ParticleMaterialQuad::FRAG_SHADER, &vec![]);
    let has_test_frag_shader = res_assets_shader.get(test_frag_handle.id()).is_some();
    info!(
        "debug_check_shaders: ParticleMaterialQuad FRAG_SHADER ({:?}) exists: {}",
        test_frag_handle, has_test_frag_shader
    );

    info!(
        "debug_check_shaders: Shader 有效 {}/{} ({:.0}%)",
        valid,
        total,
        (valid as f32 / total as f32 * 100.0)
    );
    // 如果上次有丢失的，列出丢失的 hash
    if valid < total {
        for (i, h) in debug_handles.0.iter().enumerate() {
            if res_assets_shader.get(h.id()).is_none() {
                let id = h.id();
                info!("debug_check_shaders:   丢失[{i}] id={id:?}");
            }
        }
    }
}
