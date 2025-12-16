use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use league_utils::get_shader_handle;

use crate::{
    MaterialPath, UniformsVertexQuad, ATTRIBUTE_LIFETIME, ATTRIBUTE_UV_FRAME,
    ATTRIBUTE_WORLD_POSITION,
};

#[derive(Clone, ShaderType, Default)]
pub struct UniformsPixelQuadSlice {
    pub alpha_test_reference_value: f32,
    pub slice_range: Vec2,
    pub apply_team_color_correction: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct ParticleMaterialQuadSlice {
    #[uniform(0)]
    pub uniforms_vertex: UniformsVertexQuad,
    #[uniform(1)]
    pub uniforms_pixel: UniformsPixelQuadSlice,
    #[texture(2)]
    #[sampler(3)]
    pub particle_color_texture: Option<Handle<Image>>,
    #[texture(4)]
    #[sampler(5)]
    pub texture: Option<Handle<Image>>,
    #[texture(6)]
    #[sampler(7)]
    pub cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Option<Handle<Image>>,
    #[texture(8)]
    #[sampler(9)]
    pub sampler_fow: Option<Handle<Image>>,
    pub blend_mode: u8,
}

impl MaterialPath for ParticleMaterialQuadSlice {
    const FRAG_PATH: &str = "assets/shaders/hlsl/particlesystem/quad_ps_slice.ps.glsl";
    const VERT_PATH: &str = "assets/shaders/hlsl/particlesystem/quad_vs.vs.glsl";
}

impl Material for ParticleMaterialQuadSlice {
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
            _ => todo!(),
        }
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = Some("main".into());
        descriptor.fragment.as_mut().unwrap().entry_point = Some("main".into());

        // let fragment = descriptor.fragment.as_mut().unwrap();
        // let target = fragment.targets.get_mut(0).unwrap().as_mut().unwrap();
        // target.blend = Some(BlendState {
        //     color: BlendComponent {
        //         src_factor: BlendFactor::One,
        //         dst_factor: BlendFactor::One,
        //         operation: BlendOperation::Add,
        //     },
        //     alpha: BlendComponent {
        //         src_factor: BlendFactor::One,
        //         dst_factor: BlendFactor::One,
        //         operation: BlendOperation::Add,
        //     },
        // });

        let vertex_layout = layout.0.get_layout(&[
            ATTRIBUTE_WORLD_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(3),
            ATTRIBUTE_UV_FRAME.at_shader_location(8),
            ATTRIBUTE_LIFETIME.at_shader_location(9),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}
