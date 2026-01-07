use std::fmt::Debug;

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor,
    ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use league_utils::get_shader_handle;

use crate::MaterialPath;

#[derive(Clone, ShaderType, Debug)]
pub struct UniformsVertexUnlitDecal {
    pub fog_of_war_params: Vec4,
    pub fog_of_war_always_below_y: Vec4,
    pub fow_height_fade: Vec4,
    pub decal_world_matrix: Mat4,
    pub decal_world_to_uv_matrix: Mat4,
    pub decal_projection_y_range: Vec4,
}

impl Default for UniformsVertexUnlitDecal {
    fn default() -> Self {
        Self {
            fog_of_war_params: Vec4::ZERO,
            fog_of_war_always_below_y: Vec4::ZERO,
            fow_height_fade: Vec4::ZERO,
            decal_world_matrix: Mat4::IDENTITY,
            decal_world_to_uv_matrix: Mat4::IDENTITY,
            decal_projection_y_range: Vec4::splat(100.0),
        }
    }
}

#[derive(Clone, ShaderType, Debug)]
pub struct UniformsPixelUnlitDecal {
    pub color_uv: Vec4,
    pub modulate_color: Vec4,
}

impl Default for UniformsPixelUnlitDecal {
    fn default() -> Self {
        Self {
            color_uv: Vec4::ONE,
            modulate_color: Vec4::ONE,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
#[bind_group_data(ParticleMaterialKeyUnlitDecal)]
pub struct ParticleMaterialUnlitDecal {
    #[uniform(0)]
    pub uniforms_vertex: UniformsVertexUnlitDecal,
    #[uniform(1)]
    pub uniforms_pixel: UniformsPixelUnlitDecal,
    #[texture(2)]
    #[sampler(3)]
    pub diffuse_map: Option<Handle<Image>>,
    #[texture(4)]
    #[sampler(5)]
    pub particle_color_texture: Option<Handle<Image>>,
    #[texture(6)]
    #[sampler(7)]
    pub cmb_tex_fow_map_smp_clamp_no_mip: Option<Handle<Image>>,
    pub blend_mode: u8,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ParticleMaterialKeyUnlitDecal {
    blend_mode: u8,
}

impl MaterialPath for ParticleMaterialUnlitDecal {
    const FRAG_PATH: &str = "assets/shaders/hlsl/environment/unlit_decal_ps.ps.glsl";
    const VERT_PATH: &str = "assets/shaders/hlsl/environment/unlit_decal_vs.vs.glsl";
}

// 2. 为 Key 实现 From Trait
impl From<&ParticleMaterialUnlitDecal> for ParticleMaterialKeyUnlitDecal {
    fn from(material: &ParticleMaterialUnlitDecal) -> Self {
        Self {
            blend_mode: material.blend_mode,
        }
    }
}

impl Material for ParticleMaterialUnlitDecal {
    fn fragment_shader() -> ShaderRef {
        get_shader_handle(Self::FRAG_PATH, &vec![]).into()
    }

    fn vertex_shader() -> ShaderRef {
        get_shader_handle(Self::VERT_PATH, &vec![]).into()
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
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = Some("main".into());
        descriptor.fragment.as_mut().unwrap().entry_point = Some("main".into());

        let fragment = descriptor.fragment.as_mut().unwrap();
        let target = fragment.targets.get_mut(0).unwrap().as_mut().unwrap();
        if key.bind_group_data.blend_mode == 4 {
            target.blend = Some(BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent::OVER,
            });
        }
        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}
