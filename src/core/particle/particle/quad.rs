use std::f32::consts::PI;
use std::fmt::Debug;
use std::marker::PhantomData;

use bevy::asset::uuid::Uuid;
use bevy::mesh::{MeshVertexBufferLayoutRef, VertexAttributeValues};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor,
    ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use league_utils::hash_shader_spec;

use crate::{ATTRIBUTE_LIFETIME, ATTRIBUTE_UV_FRAME, ATTRIBUTE_UV_MULT, ATTRIBUTE_WORLD_POSITION};

#[derive(Clone, ShaderType, Debug)]
pub struct UniformsVertexQuad {
    pub fog_of_war_params: Vec4,
    pub fog_of_war_always_below_y: Vec4,
    pub fow_height_fade: Vec4,
    pub nav_grid_xform: Vec4,
    pub particle_depth_push_pull: f32,
    pub texture_info: Vec4,
    pub texture_info_2: Vec4,
}

impl Default for UniformsVertexQuad {
    fn default() -> Self {
        Self {
            fog_of_war_params: Vec4::ZERO,
            fog_of_war_always_below_y: Vec4::ZERO,
            fow_height_fade: Vec4::ZERO,
            nav_grid_xform: Vec4::ZERO,
            particle_depth_push_pull: 0.0,
            texture_info: Vec4::ONE,
            texture_info_2: Vec4::ONE,
        }
    }
}

#[derive(Default)]
pub struct ParticleMeshQuad {
    pub frame: f32,
}

impl From<ParticleMeshQuad> for Mesh {
    fn from(value: ParticleMeshQuad) -> Self {
        let mut mesh: Mesh = Plane3d::new(Vec3::NEG_Z, Vec2::splat(1.0)).into();

        let transform = Transform::from_rotation(Quat::from_rotation_z(-PI / 2.));

        if let VertexAttributeValues::Float32x3(values) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        {
            let values = values
                .into_iter()
                .map(|v| transform.transform_point(Vec3::from_array(*v)))
                .collect::<Vec<_>>();

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, values.clone());

            mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION, values.clone());
        }

        if let VertexAttributeValues::Float32x2(values) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap().clone()
        {
            mesh.insert_attribute(ATTRIBUTE_UV_MULT, values.clone());

            let values = values
                .into_iter()
                .map(|v| [1. - v[0], 1. - v[1], value.frame as f32, 0.0])
                .collect::<Vec<_>>();

            mesh.insert_attribute(ATTRIBUTE_UV_FRAME, values);
        }

        let values = Vec::from([[0.0; 2]; 4]);
        mesh.insert_attribute(ATTRIBUTE_LIFETIME, values);

        let values = Vec::from([[1.0; 4]; 4]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, values);

        mesh
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
#[bind_group_data(ConditionalMaterialKey)]
pub struct ParticleMaterialQuad {
    #[uniform(0)]
    pub uniforms_vertex: UniformsVertexQuad,
    #[texture(2)]
    #[sampler(3)]
    pub texture: Option<Handle<Image>>,
    #[texture(4)]
    #[sampler(5)]
    pub particle_color_texture: Option<Handle<Image>>,
    #[texture(6)]
    #[sampler(7)]
    pub cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Option<Handle<Image>>,
    #[texture(8)]
    #[sampler(9)]
    pub sampler_fow: Option<Handle<Image>>,
    #[texture(12)]
    #[sampler(13)]
    pub texturemult: Option<Handle<Image>>,
    pub blend_mode: u8,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct ConditionalMaterialKey {
    blend_mode: u8,
    shader_frag: Handle<Shader>,
}

// 2. 为 Key 实现 From Trait
impl From<&ParticleMaterialQuad> for ConditionalMaterialKey {
    fn from(material: &ParticleMaterialQuad) -> Self {
        let mut shader_frag_defs = vec![];
        if material.texturemult.is_some() {
            shader_frag_defs.push("MULT_PASS".to_string());
        }

        Self {
            blend_mode: material.blend_mode,
            shader_frag: get_shader_handle(&shader_frag_defs),
        }
    }
}

impl Material for ParticleMaterialQuad {
    fn fragment_shader() -> ShaderRef {
        "shaders/quad.frag".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/quad.vert".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        match self.blend_mode {
            1 => AlphaMode::Blend,
            4 => AlphaMode::Blend,
            _ => AlphaMode::Opaque,
        }
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = Some("main".into());
        descriptor.fragment.as_mut().unwrap().entry_point = Some("main".into());

        let fragment = descriptor.fragment.as_mut().unwrap();
        let target = fragment.targets.get_mut(0).unwrap().as_mut().unwrap();
        if key.bind_group_data.blend_mode == 4 {
            target.blend = Some(BlendState {
                color: BlendComponent {
                    // 源颜色乘以它自己的 alpha 值
                    src_factor: BlendFactor::SrcAlpha,
                    // 目标颜色乘以 1
                    dst_factor: BlendFactor::One,
                    // 操作：源 + 目标
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent {
                    // 通常在加法混合中，我们不想修改目标 Alpha
                    // 源 Alpha * 0
                    src_factor: BlendFactor::Zero,
                    // 目标 Alpha * 1
                    dst_factor: BlendFactor::One,
                    // 操作：(S.alpha * 0) + (D.alpha * 1) = D.alpha
                    operation: BlendOperation::Add,
                },
            });
        }

        fragment.shader = key.bind_group_data.shader_frag;

        let vertex_layout = layout.0.get_layout(&[
            ATTRIBUTE_WORLD_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(3),
            ATTRIBUTE_UV_FRAME.at_shader_location(8),
            ATTRIBUTE_LIFETIME.at_shader_location(9),
            // ATTRIBUTE_UV_MULT.at_shader_location(9),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}

pub fn get_shader_handle(defs: &Vec<String>) -> Handle<Shader> {
    Handle::Uuid(Uuid::from_u128(hash_shader_spec(defs) as u128), PhantomData)
}
