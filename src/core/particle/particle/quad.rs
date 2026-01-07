use std::f32::consts::PI;
use std::fmt::Debug;

use bevy::mesh::{MeshVertexBufferLayoutRef, VertexAttributeValues};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor,
    ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use league_utils::get_shader_handle;

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

#[derive(Clone, ShaderType, Debug)]
pub struct UniformsPixel {
    pub c_depth_conversion_params: Vec4,
    pub c_soft_particle_params: Vec4,
    pub c_soft_particle_control: Vec4,
    pub c_palette_select_main: Vec4,
    pub c_palette_src_mixer_main: Vec4,
    pub c_alpha_erosion_params: Vec4,
    pub c_alpha_erosion_texture_mixer: Vec4,
    pub alpha_test_reference_value: f32,
    pub apply_team_color_correction: Vec4,
}

impl Default for UniformsPixel {
    fn default() -> Self {
        Self {
            c_depth_conversion_params: Vec4::new(10.0, -9.999, 0.0, 0.0),
            c_soft_particle_params: Vec4::new(0.0, 0.0, 1.0 / 100.0, 1.0 / 100.0),
            c_soft_particle_control: Vec4::new(2.0, 1.0, 0.5, 0.5),
            c_palette_select_main: Vec4::ZERO,
            c_palette_src_mixer_main: Vec4::ZERO,
            c_alpha_erosion_params: Vec4::ZERO,
            c_alpha_erosion_texture_mixer: Vec4::ZERO,
            alpha_test_reference_value: 0.0,
            apply_team_color_correction: Vec4::ZERO,
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

#[derive(Asset, AsBindGroup, TypePath, Clone, Debug, Default)]
#[bind_group_data(ConditionalMaterialKey)]
pub struct ParticleMaterialQuad {
    #[uniform(0)]
    pub uniforms_vertex: UniformsVertexQuad,

    #[uniform(1)]
    pub uniforms_pixel: UniformsPixel,

    #[texture(2)]
    #[sampler(3)]
    pub texture: Option<Handle<Image>>,

    #[texture(4)]
    #[sampler(5)]
    pub s_palettes_texture: Option<Handle<Image>>,

    #[texture(6)]
    #[sampler(7)]
    pub texturemult: Option<Handle<Image>>,

    #[texture(8)]
    #[sampler(9)]
    pub particle_color_texture: Option<Handle<Image>>,

    #[texture(10)]
    #[sampler(11)]
    pub cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Option<Handle<Image>>,

    #[texture(12)]
    #[sampler(13)]
    pub cmb_tex_fow_map_smp_clamp_no_mip: Option<Handle<Image>>,

    #[texture(14)]
    #[sampler(15)]
    pub s_depth_texture: Option<Handle<Image>>,

    #[texture(16)]
    #[sampler(17)]
    pub navmesh_mask_texture: Option<Handle<Image>>,

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

        if material.blend_mode == 4 {
            shader_frag_defs.push("SOFT_PARTICLES".to_string());
        }

        if material.texturemult.is_some() {
            shader_frag_defs.push("MULT_PASS".to_string());
        }

        let shader_frag = get_shader_handle(ParticleMaterialQuad::FRAG_PATH, &shader_frag_defs);

        debug!("shader_frag_defs: {:?}", shader_frag_defs);
        match shader_frag {
            Handle::Uuid(uuid, ..) => {
                debug!("shader {:x}", uuid.as_u128() as u64);
            }
            _ => {}
        }

        Self {
            blend_mode: material.blend_mode,
            shader_frag,
        }
    }
}

pub trait MaterialPath {
    const FRAG_PATH: &str;
    const VERT_PATH: &str;
}

impl MaterialPath for ParticleMaterialQuad {
    const FRAG_PATH: &str = "assets/shaders/hlsl/particlesystem/quad_ps.ps.glsl";
    const VERT_PATH: &str = "assets/shaders/hlsl/particlesystem/quad_vs.vs.glsl";
}

impl Material for ParticleMaterialQuad {
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
