use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

use crate::particles::{
    blend_mode_to_alpha_mode, UniformsPixel, UniformsVertexQuad, ATTRIBUTE_LIFETIME,
    ATTRIBUTE_UV_FRAME, ATTRIBUTE_WORLD_POSITION,
};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct QuadSliceMaterial {
    #[uniform(0)]
    pub uniforms_vertex: UniformsVertexQuad,
    #[uniform(1)]
    pub uniforms_pixel: UniformsPixel,
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
    pub is_local_orientation: bool,
    pub blend_mode: u8,
}

impl Material for QuadSliceMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/quad_slice.frag".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/quad.vert".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        blend_mode_to_alpha_mode(self.blend_mode)
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();

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

        Ok(())
    }
}
