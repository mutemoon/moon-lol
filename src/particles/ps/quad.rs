use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
            RenderPipelineDescriptor, ShaderRef, ShaderType, SpecializedMeshPipelineError,
        },
    },
};

use crate::particles::{ATTRIBUTE_LIFETIME, ATTRIBUTE_UV_FRAME, ATTRIBUTE_WORLD_POSITION};

#[derive(Clone, ShaderType)]
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

#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[bind_group_data(ConditionalMaterialKey)]
pub struct QuadMaterial {
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
    pub is_local_orientation: bool,
    pub blend_mode: u8,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct ConditionalMaterialKey {
    blend_mode: u8,
}

// 2. 为 Key 实现 From Trait
impl From<&QuadMaterial> for ConditionalMaterialKey {
    fn from(material: &QuadMaterial) -> Self {
        Self {
            blend_mode: material.blend_mode,
        }
    }
}

impl Material for QuadMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/quad.frag".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/quad.vert".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        match self.blend_mode {
            1 => AlphaMode::Blend,
            4 => AlphaMode::Add,
            _ => AlphaMode::Opaque,
        }
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();

        if key.bind_group_data.blend_mode == 4 {
            let fragment = descriptor.fragment.as_mut().unwrap();
            let target = fragment.targets.get_mut(0).unwrap().as_mut().unwrap();
            target.blend = Some(BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
            });
        }

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

#[derive(Clone, ShaderType)]
pub struct UniformsPixel {
    pub alpha_test_reference_value: f32,
    pub slice_range: Vec2,
    pub apply_team_color_correction: Vec4,
}
