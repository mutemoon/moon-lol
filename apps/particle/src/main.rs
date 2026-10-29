use bevy::asset::AssetMetaCheck;
use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use league_utils::hash_bin;
use lol_base::hash_key::{HashKey, LoadHashKeyTrait};
use lol_base_render::camera::{Focus, PluginCamera};
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
        .add_plugins(
            DefaultPlugins
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
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut vfx_system_assets: ResMut<Assets<ConfigVfxSystemDefinition>>,
) {
    // 1. 在原点放置一个默认的网格参考地面
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(500.0)))),
        MeshMaterial3d(std_materials.add(Color::srgb(0.2, 0.2, 0.25))),
        Transform::default().with_translation(Vec3::NEG_Y * 10.0),
    ));

    // 2. 构造与 fiora/riven 粒子对齐的 ConfigVfxEmitterDefinition
    let emitter_def = ConfigVfxEmitterDefinition {
        emitter_name: Some("Riven_Q_Smoke".into()),
        lifetime: Some(99999.0),
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
            base_sampler: Sampler::Constant(0.6),
            prob_curves: vec![Some(
                ProbabilityCurve::new_curve(vec![(0.0, 0.5), (1.0, 2.0)]).unwrap(),
            )],
        },
        rate: const_sampler(5.0),
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
            path: "ASSETS/Characters/Riven/Skins/Base/Particles/Riven_Base_Q_SmokeShapes.png"
                .into(),
            handle: asset_server
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| settings.is_srgb = false)
                .load("ASSETS/Characters/Riven/Skins/Base/Particles/Riven_Base_Q_SmokeShapes.png"),
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
                handle: asset_server
                    .load_builder()
                    .with_settings(|settings: &mut ImageLoaderSettings| settings.is_srgb = false)
                    .load("ASSETS/Characters/Riven/Skins/Base/Particles/Riven_Base_Z_Erosion.png"),
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

    let test_vfx_hash = hash_bin("test_vfx_system_x100");
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

    // 3. 生成粒子发射器实体并附带 Focus 聚焦视角
    let emitter_entity = commands
        .spawn((
            Focus,
            Transform::from_xyz(100.0, 0.0, 100.0),
            GlobalTransform::from_xyz(100.0, 0.0, 100.0),
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
