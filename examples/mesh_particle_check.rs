//! Mesh 粒子排查专用：仿 particle_quad_spirv_dynamic 手动构造
//! ConfigVfxSystemDefinition（对齐 Fiora_Base_BA 的 Mesh 发射器），
//! 寿命拉到常驻，spawn 后定点截图，便于离线核对渲染输出。
//! 绕过 ConfigVfxLoader 后所有 uniform/贴图输入均可控，方便做对照实验。
use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use bevy::render::view::window::screenshot::{Screenshot, save_to_disk};
use lol_base::hash_key::{HashKey, LoadHashKeyTrait};
use lol_base_render::particle::{
    ConfigVfxEmitterDefinition, ConfigVfxPrimitive, ConfigVfxShape, ConfigVfxSystemDefinition,
    Sampler, StochasticSampler, VfxTexture,
};
use lol_base_render::camera::PluginCamera;
use lol_particle::{CommandParticleSpawn, PluginParticle};

fn const_sampler<T: Clone>(v: T) -> StochasticSampler<T> {
    StochasticSampler {
        base_sampler: Sampler::Constant(v),
        prob_curves: vec![],
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            // 因为 League SPIR-V 由 DXC 编译、naga 无法解析（报 Unable to find
            // entry point 'main'），所以必须启用 PASSTHROUGH_SHADERS 原样提交
            render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                features: WgpuFeatures::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                    | WgpuFeatures::PASSTHROUGH_SHADERS,
                ..default()
            })),
            ..default()
        }))
        .add_plugins(PluginParticle)
        .add_plugins(PluginCamera)
        .add_systems(Startup, setup)
        .add_systems(Update, capture_screenshots)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut vfx_system_assets: ResMut<Assets<ConfigVfxSystemDefinition>>,
) {
    // 地面参照物，便于确认相机方位
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)))),
        MeshMaterial3d(std_materials.add(Color::srgb(0.2, 0.2, 0.2))),
        Transform::default().with_translation(Vec3::NEG_Y * 10.0),
    ));

    // 手动构造与 Fiora_Base_BA 的 "Mesh" 发射器对齐的定义，仅调长寿命：
    // 因为 COLOR_LOOKUP_UV=(life,life) 而 common_color-rampdown.png 在 u=0
    // 处是纯黑纹素，所以粒子寿命不能取 99999（life≈0 会采中黑纹素导致
    // RGB 全黑），取 20s 让 2~4s 截图时 life≈0.1~0.2 落在 ramp 最亮区
    let emitter_def = ConfigVfxEmitterDefinition {
        emitter_name: Some("Mesh".into()),
        lifetime: Some(99999.0),
        birth_acceleration: const_sampler(Vec3::ZERO),
        birth_color: const_sampler(Vec4::ONE),
        birth_rotation0: const_sampler(Vec3::ZERO),
        birth_scale0: const_sampler(Vec3::ONE),
        birth_uv_offset: const_sampler(Vec2::ZERO),
        birth_uv_scroll_rate: const_sampler(Vec2::ZERO),
        birth_velocity: const_sampler(Vec3::ZERO),
        bind_weight: const_sampler(1.0),
        color: const_sampler(Vec4::ONE),
        scale0: const_sampler(Vec3::ONE),
        particle_lifetime: const_sampler(20.0),
        rate: const_sampler(1.0),
        emitter_position: const_sampler(Vec3::ZERO),
        distortion_definition: None,
        num_frames: None,
        blend_mode: Some(4),
        material_override_definitions: None,
        primitive: Some(ConfigVfxPrimitive::VfxPrimitiveMesh {
            align_pitch_to_camera: None,
            align_yaw_to_camera: None,
            simple_mesh_name: Some(
                "ASSETS/Characters/Fiora/Skins/Base/Particles/Fiora_WeaponTrail.scb".into(),
            ),
        }),
        is_single_particle: Some(true),
        is_uniform_scale: None,
        is_random_start_frame: None,
        is_local_orientation: None,
        // 因为绕过 ConfigVfxLoader 直接注入 Assets，所以需手动以线性方式加载贴图
        texture: Some(VfxTexture {
            path: "ASSETS/Characters/Fiora/Skins/Base/Particles/Fiora_mesh_Weapontrail.png".into(),
            handle: asset_server.load_with_settings(
                "ASSETS/Characters/Fiora/Skins/Base/Particles/Fiora_mesh_Weapontrail.png",
                |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
            ),
        }),
        particle_color_texture: Some(VfxTexture {
            path: "ASSETS/Characters/Fiora/Skins/Base/Particles/common_color-rampdown.png".into(),
            handle: asset_server.load_with_settings(
                "ASSETS/Characters/Fiora/Skins/Base/Particles/common_color-rampdown.png",
                |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
            ),
        }),
        tex_div: None,
        slice_technique_range: None,
        texture_mult: None,
        alpha_ref: None,
        spawn_shape: Some(ConfigVfxShape::Unk0xee39916f {
            emit_offset: Some(Vec3::new(50.0, 100.0, -50.0)),
        }),
    };

    let test_vfx_hash = league_utils::hash_bin("mesh_check_manual");
    let system_def = ConfigVfxSystemDefinition {
        particle_name: "mesh_check_manual".into(),
        particle_path: "".into(),
        complex_emitter_definition_data: Some(vec![emitter_def]),
        simple_emitter_definition_data: None,
    };
    vfx_system_assets.add_hash(test_vfx_hash, system_def);

    let anchor = commands
        .spawn((Transform::default(), GlobalTransform::default()))
        .id();
    commands
        .entity(anchor)
        .trigger(move |entity| CommandParticleSpawn {
            entity,
            vfx_handle: HashKey::<ConfigVfxSystemDefinition>::from(test_vfx_hash),
            rotation: None,
        });
    info!("[mesh-check] 已手动 spawn 常驻 Mesh 粒子");
}

/// 启动后 2/3/4 秒各截一张（粒子常驻，必然在画面里）
fn capture_screenshots(time: Res<Time>, mut taken: Local<u32>, mut commands: Commands) {
    if *taken >= 3 || time.elapsed_secs() < (*taken + 2) as f32 {
        return;
    }
    *taken += 1;
    let path = format!("shader_debug/mesh_check_{}.png", *taken);
    info!("[mesh-check] 截屏 -> {path}");
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}
