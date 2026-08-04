use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use lol_base::hash_key::{HashKey, LoadHashKeyTrait};
use lol_base_render::camera::PluginCamera;
use lol_base_render::particle::{
    ConfigVfxAlphaErosionDefinition, ConfigVfxEmitterDefinition, ConfigVfxFlexShapeDefinition,
    ConfigVfxPrimitive, ConfigVfxShape, ConfigVfxSystemDefinition, ProbabilityCurve, Sampler,
    StochasticSampler, VfxTexture,
};
use lol_particle::{CommandParticleSpawn, PluginParticle};

fn const_sampler<T: Clone>(v: T) -> StochasticSampler<T> {
    StochasticSampler {
        base_sampler: Sampler::Constant(v),
        prob_curves: vec![],
    }
}

fn curve_sampler<T: Clone + bevy::math::StableInterpolate>(
    samples: Vec<(f32, T)>,
) -> StochasticSampler<T> {
    StochasticSampler {
        base_sampler: Sampler::new_curve(samples).unwrap(),
        prob_curves: vec![],
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            // 因为更新后的 Bevy 仅在设备启用 PASSTHROUGH_SHADERS 时才将 SpirV 走 wgpu passthrough（原样使用），
            // 否则回退 naga 解析而无法处理这些 DXC 编译的 League SPIR-V（报 Unable to find entry point 'main'），
            // 所以显式向设备请求该特性（在保留默认 TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES 的基础上追加）。
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
        // .add_systems(Update, auto_exit)
        .run();
}

fn auto_exit(mut exit_writer: MessageWriter<AppExit>, time: Res<Time>) {
    if time.elapsed_secs() > 1.0 {
        info!("已拥有足够调试信息，自动退出");
        exit_writer.write(AppExit::Success);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut vfx_system_assets: ResMut<Assets<ConfigVfxSystemDefinition>>,
) {
    // 8. 在原点放置一个全部默认的红色 Plane3d 物体
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)))),
        MeshMaterial3d(std_materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::default().with_translation(Vec3::NEG_Y * 10.),
    ));

    // 9. 手动构造 ConfigVfxSystemDefinition，并在 x: 100 生成对比发射器
    // 9. 构造与 fiora 皮肤 ConfigVfx 中的 ground_stack_01 真实粒子对齐的数据 (生命周期维持 99999.0 供常驻对比)
    let emitter_def = ConfigVfxEmitterDefinition {
        emitter_name: Some("End".into()),
        lifetime: Some(100.2),
        birth_acceleration: const_sampler(Vec3::ZERO),
        birth_color: const_sampler(Vec4::ONE),
        birth_rotation0: const_sampler(Vec3::new(90.0, -90.0, 0.0)),
        birth_scale0: StochasticSampler {
            base_sampler: Sampler::Constant(Vec3::new(150.0, 70.0, 1.0)),
            prob_curves: vec![
                Some(ProbabilityCurve::new_curve(vec![(0.0, 0.3), (1.0, 1.0)]).unwrap()),
                Some(ProbabilityCurve::new_curve(vec![(0.0, 0.5), (1.0, 1.0)]).unwrap()),
                None,
            ],
        },
        birth_uv_offset: const_sampler(Vec2::ZERO),
        birth_uv_scroll_rate: const_sampler(Vec2::ZERO),
        birth_velocity: StochasticSampler {
            base_sampler: Sampler::Constant(Vec3::new(0.0, 200.0, -500.0)),
            prob_curves: vec![
                Some(ProbabilityCurve::new_curve(vec![(0.0, 0.7), (1.0, 1.0)]).unwrap()),
                None,
                None,
            ],
        },
        bind_weight: const_sampler(0.0),
        color: curve_sampler(vec![
            (0.0, Vec4::new(0.49019608, 0.45882353, 0.40392157, 1.0)),
            (1.0, Vec4::new(0.49019608, 0.45882353, 0.40392157, 1.0)),
        ]),
        scale0: curve_sampler(vec![
            (0.0, Vec3::new(1.0, 1.0, 1.0)),
            (1.0, Vec3::new(0.8, 1.0, 0.0)),
        ]),
        particle_lifetime: StochasticSampler {
            base_sampler: Sampler::Constant(0.2),
            prob_curves: vec![Some(
                ProbabilityCurve::new_curve(vec![(0.0, 0.5), (1.0, 2.0)]).unwrap(),
            )],
        },
        rate: const_sampler(3.0),
        emitter_position: const_sampler(Vec3::new(0.0, 50.0, 0.0)),
        distortion_definition: None,
        num_frames: None,
        blend_mode: Some(1),
        material_override_definitions: None,
        sound_on_create: None,
        sound_persistent: None,
        primitive: Some(ConfigVfxPrimitive::VfxPrimitiveArbitraryQuad),
        is_single_particle: Some(false),
        is_uniform_scale: None,
        is_random_start_frame: None,
        is_local_orientation: None,
        is_direction_oriented: Some(true),
        texture: Some(VfxTexture {
            // path: "ASSETS/Characters/Riven/Skins/Base/Particles/sword_profile_glow_02.png".into(),
            path: "ASSETS/Characters/Riven/Skins/Base/Particles/Riven_Base_Q_SmokeShapes.png"
                .into(),
            handle: asset_server.load_with_settings(
                // "ASSETS/Characters/Riven/Skins/Base/Particles/sword_profile_glow_02.png",
                "ASSETS/Characters/Riven/Skins/Base/Particles/Riven_Base_Q_SmokeShapes.png",
                |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
            ),
        }),
        particle_color_texture: None,
        tex_div: None,
        slice_technique_range: None,
        texture_mult: None,
        alpha_ref: Some(15),
        spawn_shape: Some(ConfigVfxShape::Legacy {
            emit_offset: const_sampler(Vec3::new(10.0, 0.0, -30.0)),
            emit_rotation_angles: vec![
                StochasticSampler {
                    base_sampler: Sampler::Constant(0.0),
                    prob_curves: vec![Some(
                        ProbabilityCurve::new_curve(vec![(0.0, 0.0), (1.0, 360.0)]).unwrap(),
                    )],
                },
                StochasticSampler {
                    base_sampler: Sampler::Constant(0.0),
                    prob_curves: vec![Some(
                        ProbabilityCurve::new_curve(vec![(0.0, 0.0), (1.0, 360.0)]).unwrap(),
                    )],
                },
            ],
            emit_rotation_axes: vec![Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 1.0, 0.0)],
        }),
        flex_shape_definition: Some(ConfigVfxFlexShapeDefinition {
            scale_birth_scale_by_bound_object_size: Some(0.005),
            scale_birth_scale_by_bound_object_height: None,
            scale_birth_scale_by_bound_object_radius: None,
            scale_birth_translation_by_bound_object_size: None,
            scale_emit_offset_by_bound_object_height: None,
            scale_emit_offset_by_bound_object_radius: None,
            scale_emit_offset_by_bound_object_size: None,
        }),
        alpha_erosion_definition: Some(ConfigVfxAlphaErosionDefinition {
            erosion_drive_curve: curve_sampler(vec![(0.0, 0.0), (0.3, 0.0), (1.0, 0.7)]),
            erosion_drive_source: None,
            erosion_feather_in: None,
            erosion_feather_out: None,
            erosion_map: Some(VfxTexture {
                path: "ASSETS/Characters/Riven/Skins/Base/Particles/Riven_Base_Z_Erosion.png"
                    .into(),
                handle: asset_server.load_with_settings(
                    "ASSETS/Characters/Riven/Skins/Base/Particles/Riven_Base_Z_Erosion.png",
                    |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
                ),
            }),
            erosion_map_address_mode: None,
            erosion_map_channel_mixer: const_sampler(Vec4::new(1.0, 0.0, 0.0, 0.0)),
            erosion_slice_width: None,
            linger_erosion_drive_curve: None,
            use_linger_erosion_drive_curve: None,
        }),
        color_look_up_type_y: Some(3),
        color_render_flags: None,
        soft_particle_definition: None,
        palette_definition: None,
        reflection_definition: None,
    };

    let test_vfx_hash = league_utils::hash_bin("test_vfx_system_x100");
    let test_vfx_handle = HashKey::<ConfigVfxSystemDefinition>::from(test_vfx_hash);

    let system_def = ConfigVfxSystemDefinition {
        particle_name: "test_particle_x100".into(),
        particle_path: "".into(),
        complex_emitter_definition_data: Some(vec![emitter_def]),
        simple_emitter_definition_data: None,
        sound_on_create_default: None,
        sound_persistent_default: None,
    };

    vfx_system_assets.add_hash(test_vfx_hash, system_def);

    let emitter_entity = commands
        .spawn((
            Transform::from_xyz(100.0, 0.0, 0.0),
            GlobalTransform::from_xyz(100.0, 0.0, 0.0),
        ))
        .id();

    commands
        .entity(emitter_entity)
        .trigger(move |entity| CommandParticleSpawn {
            entity,
            vfx_handle: test_vfx_handle,
            rotation: None,
        });
}
