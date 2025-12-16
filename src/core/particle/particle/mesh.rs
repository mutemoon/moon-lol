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
pub struct UniformsVertexMesh {
    pub fog_of_war_params: Vec4,
    pub fog_of_war_always_below_y: Vec4,
    pub fow_height_fade: Vec4,
    pub m_world: Mat4,
    pub particle_depth_push_pull: f32,
    pub v_fresnel: Vec4,
    pub v_particle_uvtransform: [Vec3; 4],
    pub v_particle_uvtransform_mult: [Vec3; 4],
    pub k_color_factor: Vec4,
}

impl Default for UniformsVertexMesh {
    fn default() -> Self {
        Self {
            fog_of_war_params: Vec4::ZERO,
            fog_of_war_always_below_y: Vec4::ZERO,
            fow_height_fade: Vec4::ZERO,
            m_world: Default::default(),
            particle_depth_push_pull: Default::default(),
            v_fresnel: Vec4::W,
            v_particle_uvtransform: [Vec3::X, Vec3::Y, Vec3::ZERO, Vec3::ZERO],
            v_particle_uvtransform_mult: Default::default(),
            k_color_factor: Vec4::ONE,
        }
    }
}

#[derive(Clone, ShaderType, Debug)]
pub struct UniformsPixelMesh {
    pub fow_edge_control: Vec4,
    pub color_lookup_uv: Vec2,
}

impl Default for UniformsPixelMesh {
    fn default() -> Self {
        Self {
            fow_edge_control: Vec4::ONE,
            color_lookup_uv: Vec2::ONE,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
#[bind_group_data(ParticleMaterialKeyMesh)]
pub struct ParticleMaterialMesh {
    #[uniform(0)]
    pub uniforms_vertex: UniformsVertexMesh,
    #[uniform(1)]
    pub uniforms_pixel: UniformsPixelMesh,
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
    pub cmb_tex_fow_map_smp_clamp_no_mip: Option<Handle<Image>>,
    pub blend_mode: u8,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ParticleMaterialKeyMesh {
    blend_mode: u8,
}

// 2. 为 Key 实现 From Trait
impl From<&ParticleMaterialMesh> for ParticleMaterialKeyMesh {
    fn from(material: &ParticleMaterialMesh) -> Self {
        Self {
            blend_mode: material.blend_mode,
        }
    }
}

impl MaterialPath for ParticleMaterialMesh {
    const FRAG_PATH: &str = "assets/shaders/hlsl/particlesystem/mesh_ps.ps.glsl";
    const VERT_PATH: &str = "assets/shaders/hlsl/particlesystem/mesh_vs.vs.glsl";
}

impl Material for ParticleMaterialMesh {
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
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent::OVER,
            });
        }

        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(2),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(8),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}
