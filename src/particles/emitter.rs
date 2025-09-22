use bevy::prelude::*;

use league_core::{
    Unk0xee39916f, VfxEmitterDefinitionData, VfxEmitterDefinitionDataPrimitive,
    VfxEmitterDefinitionDataSpawnShape,
};

use crate::particles::{
    create_black_pixel_texture, ParticleLifeState, ParticleQuad, QuadMaterial, QuadSliceMaterial,
    Sampler, UniformsPixel, UniformsVertexQuad,
};

#[derive(Component)]
pub struct ParticleEmitterState {
    pub timer: Timer,
    pub rate_sampler: Sampler,
    pub life_sampler: Sampler,
    pub emission_debt: f32,
}

pub fn update_emitter(
    mut commands: Commands,
    mut res_mesh: ResMut<Assets<Mesh>>,
    res_asset_server: Res<AssetServer>,
    mut res_image: ResMut<Assets<Image>>,
    mut res_quad_material: ResMut<Assets<QuadMaterial>>,
    mut res_quad_slice_material: ResMut<Assets<QuadSliceMaterial>>,
    mut query: Query<(
        Entity,
        &ChildOf,
        &mut ParticleEmitterState,
        &VfxEmitterDefinitionData,
    )>,
    time: Res<Time>,
) {
    for (entity, parent, mut emitter, vfx_emitter_definition_data) in query.iter_mut() {
        emitter.timer.tick(time.delta());

        if emitter.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let normalized_time = emitter.timer.elapsed_secs() / emitter.timer.duration().as_secs_f32();

        let current_rate = emitter.rate_sampler.sample_clamped(normalized_time);

        // 计算本帧应该发射的粒子数量
        // 加上一帧的 "欠账"，这使得在低速率下也能平滑发射
        let particles_to_spawn_f32 = current_rate * time.delta_secs() + emitter.emission_debt;

        // 向下取整，得到本帧实际生成的整数粒子数
        let particles_to_spawn = particles_to_spawn_f32.floor() as u32;

        // 更新 "欠账"，为下一帧做准备
        emitter.emission_debt = particles_to_spawn_f32.fract();

        let lifetime = emitter.life_sampler.sample_clamped(normalized_time);

        for _ in 0..particles_to_spawn {
            let mesh = vfx_emitter_definition_data
                .primitive
                .clone()
                .unwrap_or(VfxEmitterDefinitionDataPrimitive::VfxPrimitiveArbitraryQuad);

            let mesh = match mesh {
                VfxEmitterDefinitionDataPrimitive::VfxPrimitiveArbitraryQuad => {
                    ParticleQuad::default()
                }
                _ => todo!(),
            };

            let mesh = res_mesh.add(mesh);

            let texture = vfx_emitter_definition_data
                .texture
                .as_ref()
                .map(|v| res_asset_server.load(v))
                .unwrap();

            let particle_color_texture = vfx_emitter_definition_data
                .particle_color_texture
                .as_ref()
                .map(|v| res_asset_server.load(v));

            let offset = match vfx_emitter_definition_data.spawn_shape.clone().unwrap() {
                VfxEmitterDefinitionDataSpawnShape::Unk0xee39916f(Unk0xee39916f {
                    emit_offset,
                }) => emit_offset.unwrap(),
                _ => todo!(),
            };

            let mut target = commands.spawn((
                Mesh3d(mesh),
                Transform::from_translation(offset),
                ParticleLifeState {
                    timer: Timer::from_seconds(lifetime, TimerMode::Repeating),
                },
                Pickable::IGNORE,
            ));

            let target_id = target.id();

            let blend_mode = vfx_emitter_definition_data.blend_mode.unwrap_or(0);

            if let Some(range) = vfx_emitter_definition_data.slice_technique_range {
                target.insert(MeshMaterial3d(
                    res_quad_slice_material.add(QuadSliceMaterial {
                        uniforms_vertex: UniformsVertexQuad::default(),
                        uniforms_pixel: UniformsPixel {
                            alpha_test_reference_value: f32::default(),
                            slice_range: vec2(range, 1.0 / (range * range)),
                            apply_team_color_correction: Vec4::default(),
                        },
                        particle_color_texture,
                        texture: Some(texture),
                        cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Some(
                            res_image.add(create_black_pixel_texture()),
                        ),
                        sampler_fow: None,
                        is_local_orientation: vfx_emitter_definition_data
                            .is_local_orientation
                            .unwrap_or(false),
                        blend_mode,
                    }),
                ));
            } else {
                target.insert(MeshMaterial3d(
                    res_quad_material.add(QuadMaterial {
                        uniforms_vertex: UniformsVertexQuad::default(),
                        particle_color_texture: None,
                        texture: Some(texture),
                        cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Some(
                            res_image.add(create_black_pixel_texture()),
                        ),
                        sampler_fow: None,
                        is_local_orientation: vfx_emitter_definition_data
                            .is_local_orientation
                            .unwrap_or(false),
                        blend_mode,
                    }),
                ));
            };

            commands.entity(parent.0).add_child(target_id);
        }
    }
}
