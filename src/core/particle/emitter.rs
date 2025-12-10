use bevy::animation::AnimationTarget;
use bevy::math::bounding::{Aabb3d, IntersectsVolume};
use bevy::mesh::skinning::SkinnedMesh;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use league_core::{
    EnumVfxPrimitive, EnumVfxShape, Unk0xee39916f, VfxEmitterDefinitionData, VfxPrimitiveMesh,
    VfxPrimitivePlanarProjection, VfxShapeBox, VfxShapeCylinder, VfxShapeLegacy,
    VfxSystemDefinitionData,
};
use lol_config::LeagueProperties;

use crate::{
    create_black_pixel_texture, spawn_shadow_skin_entity, Lifetime, MapGeometry, ParticleId,
    ParticleMaterialMesh, ParticleMaterialQuad, ParticleMaterialQuadSlice,
    ParticleMaterialSkinnedMeshParticle, ParticleMaterialUnlitDecal, ParticleMeshQuad,
    ParticleState, ResourceCache, StochasticSampler, UniformsPixelMesh, UniformsPixelQuadSlice,
    UniformsPixelSkinnedMeshParticle, UniformsPixelUnlitDecal, UniformsVertexMesh,
    UniformsVertexQuad, UniformsVertexSkinnedMeshParticle, UniformsVertexUnlitDecal,
};

#[derive(Component)]
#[require(Visibility)]
pub struct ParticleEmitterState {
    pub birth_acceleration: StochasticSampler<Vec3>,
    pub birth_color: StochasticSampler<Vec4>,
    pub birth_rotation0: StochasticSampler<Vec3>,
    pub birth_scale0: StochasticSampler<Vec3>,
    pub birth_uv_offset: StochasticSampler<Vec2>,
    pub birth_uv_scroll_rate: StochasticSampler<Vec2>,
    pub birth_velocity: StochasticSampler<Vec3>,
    pub bind_weight: StochasticSampler<f32>,
    pub color: StochasticSampler<Vec4>,
    pub scale0: StochasticSampler<Vec3>,
    pub emission_debt: f32,
    pub particle_lifetime: StochasticSampler<f32>,
    pub rate: StochasticSampler<f32>,
    pub emitter_position: StochasticSampler<Vec3>,
    pub global_transform: Transform,
}

#[derive(Component, Debug)]
#[relationship(relationship_target = Emitters)]
pub struct EmitterOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = EmitterOf, linked_spawn)]
pub struct Emitters(Vec<Entity>);

pub fn update_emitter_position(
    mut query: Query<(
        &mut Transform,
        &EmitterOf,
        &Lifetime,
        &ParticleEmitterState,
        &ParticleId,
    )>,
    res_league_properties: Res<LeagueProperties>,
    res_assets_vfx_system_definition_data: Res<Assets<VfxSystemDefinitionData>>,
    q_global_transform: Query<&GlobalTransform>,
) {
    for (mut transform, emitter_of, lifetime, emitter, particle_id) in query.iter_mut() {
        let vfx_emitter_definition_data = particle_id.get_def(
            &res_league_properties,
            &res_assets_vfx_system_definition_data,
        );

        let is_local_orientation = vfx_emitter_definition_data
            .is_local_orientation
            .unwrap_or(true);

        let progress = lifetime.progress();

        let emitter_position = emitter.emitter_position.sample_clamped(progress);

        let bind_weight = emitter.bind_weight.sample_clamped(progress);

        if bind_weight == 0. {
            continue;
        }

        let mut character_global_transform = q_global_transform
            .get(emitter_of.0)
            .unwrap()
            .compute_transform();

        if is_local_orientation {
            character_global_transform.translation += emitter_position;
            *transform = character_global_transform;
        } else {
            *transform = Transform::from_matrix(Mat4::from_scale_rotation_translation(
                character_global_transform.scale,
                Quat::default(),
                character_global_transform.translation + emitter_position,
            ));
        }
    }
}

struct ParticleBirthParams {
    birth_color: Vec4,
    birth_velocity: Vec3,
    birth_acceleration: Vec3,
    birth_rotation0: Vec3,
    birth_scale0: Vec3,
    birth_uv_offset: Vec2,
    birth_uv_scroll_rate: Vec2,
}

impl ParticleBirthParams {
    fn sample(emitter: &mut ParticleEmitterState, progress: f32) -> Self {
        Self {
            birth_color: emitter.birth_color.sample_clamped(progress),
            birth_velocity: emitter.birth_velocity.sample_clamped(progress),
            birth_acceleration: emitter.birth_acceleration.sample_clamped(progress),
            birth_rotation0: emitter.birth_rotation0.sample_clamped(progress),
            birth_scale0: emitter.birth_scale0.sample_clamped(progress),
            birth_uv_offset: emitter.birth_uv_offset.sample_clamped(progress),
            birth_uv_scroll_rate: emitter.birth_uv_scroll_rate.sample_clamped(progress),
        }
    }
}

fn calculate_particle_transform_frame(
    birth_params: &ParticleBirthParams,
    is_uniform_scale: bool,
    vfx_emitter_definition_data: &VfxEmitterDefinitionData,
    primitive: &EnumVfxPrimitive,
    progress: f32,
) -> (Transform, Vec3, f32) {
    let mut birth_scale0 = if is_uniform_scale {
        Vec3::splat(birth_params.birth_scale0.x)
    } else {
        birth_params.birth_scale0
    };

    let translation = vfx_emitter_definition_data
        .spawn_shape
        .clone()
        .and_then(|v| match v {
            EnumVfxShape::Unk0xee39916f(Unk0xee39916f { emit_offset }) => emit_offset,
            EnumVfxShape::VfxShapeLegacy(VfxShapeLegacy { emit_offset, .. }) => emit_offset
                .and_then(|v| Some(StochasticSampler::<Vec3>::from(v).sample_clamped(progress))),
            EnumVfxShape::VfxShapeBox(VfxShapeBox { .. }) => Some(Vec3::ZERO),
            EnumVfxShape::VfxShapeCylinder(VfxShapeCylinder { .. }) => Some(Vec3::ZERO),
            _ => todo!(),
        })
        .unwrap_or(Vec3::ZERO);

    let rotation_quat = Quat::from_euler(
        EulerRot::XYZEx,
        birth_params.birth_rotation0.x.to_radians(),
        (birth_params.birth_rotation0.y - birth_params.birth_rotation0.z).to_radians(),
        0.,
    );

    if let EnumVfxPrimitive::VfxPrimitivePlanarProjection(VfxPrimitivePlanarProjection {
        ref m_projection,
    }) = primitive
    {
        birth_scale0.x = birth_scale0.x * 2.;
        birth_scale0.y = m_projection.as_ref().unwrap().m_y_range.unwrap();
        birth_scale0.z = birth_scale0.z * 2.;
    }

    let transform = Transform::from_rotation(rotation_quat)
        .with_translation(translation)
        .with_scale(birth_scale0);

    let num_frames = vfx_emitter_definition_data.num_frames.unwrap_or(0) as f32;
    let frame = if vfx_emitter_definition_data
        .is_random_start_frame
        .unwrap_or(false)
    {
        (num_frames * rand::random::<f32>()).floor()
    } else {
        (num_frames * progress).floor()
    };

    (transform, birth_scale0, frame)
}

fn spawn_particle_entity(
    commands: &mut Commands,
    particle_id: &ParticleId,
    emitter_entity: Entity,
    particle_lifetime: f32,
    transform: Transform,
    frame: f32,
    birth_params: &ParticleBirthParams,
    adjusted_birth_scale0: Vec3,
) -> Entity {
    let particle_state = ParticleState {
        birth_uv_offset: birth_params.birth_uv_offset,
        birth_uv_scroll_rate: birth_params.birth_uv_scroll_rate,
        birth_color: birth_params.birth_color,
        birth_scale0: adjusted_birth_scale0,
        velocity: birth_params.birth_velocity,
        acceleration: birth_params.birth_acceleration,
        frame,
    };

    commands
        .spawn((
            particle_id.clone(),
            particle_state,
            Lifetime::new_timer(particle_lifetime),
            transform,
            Pickable::IGNORE,
            ChildOf(emitter_entity),
        ))
        .id()
}

fn attach_particle_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    vfx_emitter_definition_data: &VfxEmitterDefinitionData,
    primitive: &EnumVfxPrimitive,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    texture_mult: Option<Handle<Image>>,
    blend_mode: u8,
    frame: f32,
    res_mesh: &mut ResMut<Assets<Mesh>>,
    res_image: &mut ResMut<Assets<Image>>,
    res_quad_material: &mut ResMut<Assets<ParticleMaterialQuad>>,
    res_quad_slice_material: &mut ResMut<Assets<ParticleMaterialQuadSlice>>,
    res_unlit_decal_material: &mut ResMut<Assets<ParticleMaterialUnlitDecal>>,
    res_particle_material_mesh: &mut ResMut<Assets<ParticleMaterialMesh>>,
    res_resource_cache: &mut ResMut<ResourceCache>,
    res_asset_server: &Res<AssetServer>,
) {
    match primitive {
        EnumVfxPrimitive::VfxPrimitiveArbitraryQuad
        | EnumVfxPrimitive::VfxPrimitiveCameraUnitQuad => {
            let mesh = res_mesh.add(ParticleMeshQuad { frame });
            commands.entity(particle_entity).insert(Mesh3d(mesh));

            let black_pixel_texture = res_image.add(create_black_pixel_texture());
            let uniforms_vertex = UniformsVertexQuad {
                texture_info: match vfx_emitter_definition_data.tex_div {
                    Some(tex_div) => vec4(tex_div.x, 1.0 / tex_div.x, 1.0 / tex_div.y, 0.),
                    None => Vec4::ONE,
                },
                ..default()
            };

            if let Some(range) = vfx_emitter_definition_data.slice_technique_range {
                commands.entity(particle_entity).insert(MeshMaterial3d(
                    res_quad_slice_material.add(ParticleMaterialQuadSlice {
                        uniforms_vertex,
                        uniforms_pixel: UniformsPixelQuadSlice {
                            slice_range: vec2(range, 1.0 / (range * range)),
                            ..default()
                        },
                        particle_color_texture: particle_color_texture.clone(),
                        texture: texture.clone(),
                        cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Some(black_pixel_texture),
                        sampler_fow: None,
                        blend_mode,
                    }),
                ));
            } else {
                commands
                    .entity(particle_entity)
                    .insert(MeshMaterial3d(res_quad_material.add(
                        ParticleMaterialQuad {
                            uniforms_vertex,
                            particle_color_texture: particle_color_texture.clone(),
                            texture: texture.clone(),
                            cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Some(
                                black_pixel_texture,
                            ),
                            sampler_fow: None,
                            texturemult: texture_mult.clone(),
                            blend_mode,
                        },
                    )));
            };
        }
        EnumVfxPrimitive::VfxPrimitivePlanarProjection(VfxPrimitivePlanarProjection {
            ref m_projection,
        }) => {
            let material_handle = res_unlit_decal_material.add(ParticleMaterialUnlitDecal {
                uniforms_vertex: UniformsVertexUnlitDecal {
                    decal_projection_y_range: Vec4::splat(
                        m_projection.as_ref().unwrap().m_y_range.unwrap(),
                    ),
                    ..default()
                },
                uniforms_pixel: UniformsPixelUnlitDecal::default(),
                diffuse_map: texture.clone(),
                particle_color_texture: particle_color_texture.clone(),
                cmb_tex_fow_map_smp_clamp_no_mip: None,
                blend_mode,
            });

            commands
                .entity(particle_entity)
                .insert((ParticleDecal::default(), MeshMaterial3d(material_handle)));
        }
        EnumVfxPrimitive::VfxPrimitiveMesh(VfxPrimitiveMesh { ref m_mesh, .. }) => {
            let Some(m_mesh) = m_mesh else {
                println!("VfxPrimitiveMesh: m_mesh is None");
                return;
            };
            let Some(mesh_name) = &m_mesh.m_simple_mesh_name else {
                println!("VfxPrimitiveMesh: m_simple_mesh_name is None");
                return;
            };

            let mesh = res_resource_cache.get_mesh(res_asset_server, mesh_name);
            let black_pixel_texture = res_image.add(create_black_pixel_texture());

            commands.entity(particle_entity).insert((
                Mesh3d(mesh),
                MeshMaterial3d(res_particle_material_mesh.add(ParticleMaterialMesh {
                    uniforms_vertex: UniformsVertexMesh::default(),
                    uniforms_pixel: UniformsPixelMesh::default(),
                    texture: texture.clone(),
                    particle_color_texture: particle_color_texture.clone(),
                    cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Some(black_pixel_texture),
                    cmb_tex_fow_map_smp_clamp_no_mip: None,
                    blend_mode,
                })),
            ));
        }
        _ => {}
    }
}

pub fn update_emitter(
    mut commands: Commands,
    mut res_mesh: ResMut<Assets<Mesh>>,
    res_assets_vfx_system_definition_data: Res<Assets<VfxSystemDefinitionData>>,
    res_asset_server: Res<AssetServer>,
    mut res_resource_cache: ResMut<ResourceCache>,
    mut res_image: ResMut<Assets<Image>>,
    mut res_quad_material: ResMut<Assets<ParticleMaterialQuad>>,
    mut res_quad_slice_material: ResMut<Assets<ParticleMaterialQuadSlice>>,
    mut res_unlit_decal_material: ResMut<Assets<ParticleMaterialUnlitDecal>>,
    mut res_particle_material_mesh: ResMut<Assets<ParticleMaterialMesh>>,
    res_league_properties: Res<LeagueProperties>,
    mut query: Query<(
        Entity,
        &mut Lifetime,
        &mut ParticleEmitterState,
        &ParticleId,
    )>,
    time: Res<Time>,
) {
    for (emitter_entity, mut lifetime, mut emitter, particle_id) in query.iter_mut() {
        let vfx_emitter_definition_data = particle_id.get_def(
            &res_league_properties,
            &res_assets_vfx_system_definition_data,
        );

        let primitive = vfx_emitter_definition_data
            .primitive
            .clone()
            .unwrap_or(EnumVfxPrimitive::VfxPrimitiveCameraUnitQuad);

        if matches!(primitive, EnumVfxPrimitive::VfxPrimitiveAttachedMesh { .. }) {
            continue;
        }

        if lifetime.is_dead() {
            continue;
        }

        let progress = lifetime.progress();
        let rate = emitter.rate.sample_clamped(progress);

        let is_single_particle = vfx_emitter_definition_data
            .is_single_particle
            .unwrap_or(false);

        let particles_to_spawn_f32 = rate * time.delta_secs() + emitter.emission_debt;

        let particles_to_spawn = if is_single_particle {
            lifetime.dead();
            rate as u32
        } else {
            particles_to_spawn_f32.floor() as u32
        };

        emitter.emission_debt = particles_to_spawn_f32.fract();

        if particles_to_spawn == 0 {
            continue;
        }

        let is_uniform_scale = vfx_emitter_definition_data
            .is_uniform_scale
            .unwrap_or(false);

        let texture = vfx_emitter_definition_data
            .texture
            .as_ref()
            .map(|v| res_resource_cache.get_image(&res_asset_server, v));

        let particle_color_texture = vfx_emitter_definition_data
            .particle_color_texture
            .as_ref()
            .map(|v| res_resource_cache.get_image(&res_asset_server, v));

        let texture_mult = vfx_emitter_definition_data
            .texture_mult
            .as_ref()
            .and_then(|v| v.texture_mult.as_ref())
            .map(|v| res_resource_cache.get_image(&res_asset_server, v));

        let blend_mode = vfx_emitter_definition_data.blend_mode.unwrap_or(4);

        for _ in 0..particles_to_spawn {
            let particle_lifetime = emitter.particle_lifetime.sample_clamped(progress);
            let particle_lifetime = if particle_lifetime < 0. {
                0.
            } else {
                particle_lifetime
            };

            let birth_params = ParticleBirthParams::sample(&mut emitter, progress);

            let (transform, adjusted_birth_scale0, frame) = calculate_particle_transform_frame(
                &birth_params,
                is_uniform_scale,
                vfx_emitter_definition_data,
                &primitive,
                progress,
            );

            let particle_entity = spawn_particle_entity(
                &mut commands,
                particle_id,
                emitter_entity,
                particle_lifetime,
                transform,
                frame,
                &birth_params,
                adjusted_birth_scale0,
            );

            attach_particle_visuals(
                &mut commands,
                particle_entity,
                vfx_emitter_definition_data,
                &primitive,
                texture.clone(),
                particle_color_texture.clone(),
                texture_mult.clone(),
                blend_mode,
                frame,
                &mut res_mesh,
                &mut res_image,
                &mut res_quad_material,
                &mut res_quad_slice_material,
                &mut res_unlit_decal_material,
                &mut res_particle_material_mesh,
                &mut res_resource_cache,
                &res_asset_server,
            );
        }
    }
}

pub fn update_emitter_attached(
    mut commands: Commands,
    res_assets_vfx_system_definition_data: Res<Assets<VfxSystemDefinitionData>>,
    res_asset_server: Res<AssetServer>,
    mut res_resource_cache: ResMut<ResourceCache>,
    mut res_image: ResMut<Assets<Image>>,
    mut res_particle_material_skinned_mesh_particle: ResMut<
        Assets<ParticleMaterialSkinnedMeshParticle>,
    >,
    res_league_properties: Res<LeagueProperties>,
    mut query: Query<(
        Entity,
        &EmitterOf,
        &mut Lifetime,
        &mut ParticleEmitterState,
        &ParticleId,
    )>,
    q_mesh3d: Query<&Mesh3d>,
    q_skinned_mesh: Query<&SkinnedMesh>,
    q_children: Query<&Children>,
    q_animation_target: Query<(Entity, &Transform, &AnimationTarget)>,
    time: Res<Time>,
) {
    for (emitter_entity, emitter_of, mut lifetime, mut emitter, particle_id) in query.iter_mut() {
        let vfx_emitter_definition_data = particle_id.get_def(
            &res_league_properties,
            &res_assets_vfx_system_definition_data,
        );

        let primitive = vfx_emitter_definition_data
            .primitive
            .clone()
            .unwrap_or(EnumVfxPrimitive::VfxPrimitiveCameraUnitQuad);

        if !matches!(primitive, EnumVfxPrimitive::VfxPrimitiveAttachedMesh { .. }) {
            continue;
        }

        if lifetime.is_dead() {
            continue;
        }

        let progress = lifetime.progress();
        let rate = emitter.rate.sample_clamped(progress);

        let is_single_particle = vfx_emitter_definition_data
            .is_single_particle
            .unwrap_or(false);

        let particles_to_spawn_f32 = rate * time.delta_secs() + emitter.emission_debt;

        let particles_to_spawn = if is_single_particle {
            lifetime.dead();
            rate as u32
        } else {
            particles_to_spawn_f32.floor() as u32
        };

        emitter.emission_debt = particles_to_spawn_f32.fract();

        if particles_to_spawn == 0 {
            continue;
        }

        let is_uniform_scale = vfx_emitter_definition_data
            .is_uniform_scale
            .unwrap_or(false);

        let mut texture = vfx_emitter_definition_data
            .texture
            .as_ref()
            .map(|v| res_resource_cache.get_image(&res_asset_server, v));

        let particle_color_texture = vfx_emitter_definition_data
            .particle_color_texture
            .as_ref()
            .map(|v| res_resource_cache.get_image(&res_asset_server, v));

        let blend_mode = vfx_emitter_definition_data.blend_mode.unwrap_or(4);

        for _ in 0..particles_to_spawn {
            let particle_lifetime = emitter.particle_lifetime.sample_clamped(progress);
            let particle_lifetime = if particle_lifetime < 0. {
                0.
            } else {
                particle_lifetime
            };

            let birth_params = ParticleBirthParams::sample(&mut emitter, progress);

            let (transform, adjusted_birth_scale0, frame) = calculate_particle_transform_frame(
                &birth_params,
                is_uniform_scale,
                vfx_emitter_definition_data,
                &primitive,
                progress,
            );

            let particle_entity = spawn_particle_entity(
                &mut commands,
                particle_id,
                emitter_entity,
                particle_lifetime,
                transform,
                frame,
                &birth_params,
                adjusted_birth_scale0,
            );

            let black_pixel_texture = res_image.add(create_black_pixel_texture());
            if let Some(material_override_definitions) =
                &vfx_emitter_definition_data.material_override_definitions
            {
                for material_override_definition in material_override_definitions {
                    if let Some(base_texture) = &material_override_definition.base_texture {
                        texture = Some(res_asset_server.load(base_texture));
                    }
                }
            }

            let material = MeshMaterial3d(res_particle_material_skinned_mesh_particle.add(
                ParticleMaterialSkinnedMeshParticle {
                    uniforms_vertex: UniformsVertexSkinnedMeshParticle::default(),
                    uniforms_pixel: UniformsPixelSkinnedMeshParticle::default(),
                    texture: texture.clone(),
                    particle_color_texture: particle_color_texture.clone(),
                    cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Some(black_pixel_texture),
                    cmb_tex_fow_map_smp_clamp_no_mip: None,
                    blend_mode,
                },
            ));

            spawn_shadow_skin_entity(
                &mut commands,
                particle_entity,
                emitter_of.0,
                material,
                q_mesh3d,
                q_skinned_mesh,
                q_children,
                q_animation_target,
            );
        }
    }
}

#[derive(Component, Default)]
pub struct ParticleDecal {
    visible: HashSet<Entity>,
}

#[derive(Component)]
pub struct ParticleDecalGeometry(pub Entity);

pub fn update_decal_intersections(
    mut commands: Commands,
    mut q_decals: Query<(
        Entity,
        &MeshMaterial3d<ParticleMaterialUnlitDecal>,
        &mut ParticleDecal,
    )>,
    q_map_geo: Query<(Entity, &Mesh3d, &MapGeometry)>,
    q_particle_decal_geometry: Query<(Entity, &ParticleDecalGeometry)>,
    q_global_transform: Query<&GlobalTransform>,
) {
    for (particle_decal_entity, material, mut particle_decal) in q_decals.iter_mut() {
        let Ok(particle_decal_global_transform) = q_global_transform.get(particle_decal_entity)
        else {
            continue;
        };

        let current_bounding_box = Aabb3d::new(
            particle_decal_global_transform.translation(),
            particle_decal_global_transform.scale(),
        );

        for (geometry_entity, mesh3d, map_geometry) in q_map_geo.iter() {
            if current_bounding_box.intersects(&map_geometry.bounding_box) {
                if !particle_decal.visible.contains(&geometry_entity) {
                    commands.spawn((
                        mesh3d.clone(),
                        material.clone(),
                        Pickable::IGNORE,
                        ParticleDecalGeometry(particle_decal_entity),
                    ));
                    particle_decal.visible.insert(geometry_entity);
                }
            } else {
                particle_decal.visible.remove(&geometry_entity);
            }
        }
    }

    for (decal_entity, decal_geometry) in q_particle_decal_geometry.iter() {
        if q_decals.get(decal_geometry.0).is_err() {
            commands.entity(decal_entity).despawn();
        }
    }
}
