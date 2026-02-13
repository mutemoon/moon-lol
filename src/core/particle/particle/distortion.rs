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

use crate::{
    MaterialPath, ATTRIBUTE_LIFETIME, ATTRIBUTE_UV_FRAME, ATTRIBUTE_UV_MULT,
    ATTRIBUTE_WORLD_POSITION_VEC4,
};

#[derive(Clone, ShaderType, Debug)]
pub struct UniformsVertexDistortion {
    pub particle_depth_push_pull: f32,
    pub texture_info: Vec4,
}

impl Default for UniformsVertexDistortion {
    fn default() -> Self {
        Self {
            particle_depth_push_pull: 0.0,
            texture_info: Vec4::ZERO,
        }
    }
}

#[derive(Clone, ShaderType, Debug)]
pub struct UniformsPixelDistortion {
    pub alpha_test_reference_value: f32,
    pub distortion_power: f32,
    pub apply_team_color_correction: Vec4,
}

impl Default for UniformsPixelDistortion {
    fn default() -> Self {
        Self {
            alpha_test_reference_value: 0.0,
            distortion_power: 0.0,
            apply_team_color_correction: Vec4::ZERO,
        }
    }
}

#[derive(Default)]
pub struct ParticleMeshDistortion {
    pub frame: f32,
}

impl From<ParticleMeshDistortion> for Mesh {
    fn from(value: ParticleMeshDistortion) -> Self {
        let mut mesh: Mesh = Plane3d::new(Vec3::NEG_Z, Vec2::splat(1.0)).into();

        let transform = Transform::from_rotation(Quat::from_rotation_z(-PI / 2.));

        if let VertexAttributeValues::Float32x3(values) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        {
            let values = values
                .into_iter()
                .map(|v| transform.transform_point(Vec3::from_array(*v)).extend(0.0))
                .collect::<Vec<_>>();

            mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION_VEC4, values.clone());
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
#[bind_group_data(ConditionalMaterialKeyDistortion)]
pub struct ParticleMaterialDistortion {
    #[uniform(0)]
    pub uniforms_vertex: UniformsVertexDistortion,

    #[uniform(1)]
    pub uniforms_pixel: UniformsPixelDistortion,

    #[texture(2)]
    #[sampler(3)]
    pub texture: Option<Handle<Image>>,

    #[texture(4)]
    #[sampler(5)]
    pub particle_color_texture: Option<Handle<Image>>,

    #[texture(6)]
    #[sampler(7)]
    pub normal_map: Option<Handle<Image>>,

    #[texture(8)]
    #[sampler(9)]
    pub cmb_tex_sampler_back_buffer_copy_smp_clamp_no_mip: Option<Handle<Image>>,

    pub blend_mode: u8,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct ConditionalMaterialKeyDistortion {
    blend_mode: u8,
    shader_frag: Handle<Shader>,
}

// 2. 为 Key 实现 From Trait
impl From<&ParticleMaterialDistortion> for ConditionalMaterialKeyDistortion {
    fn from(material: &ParticleMaterialDistortion) -> Self {
        let mut shader_frag_defs = vec![];

        let shader_frag =
            get_shader_handle(ParticleMaterialDistortion::FRAG_PATH, &shader_frag_defs);

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

impl MaterialPath for ParticleMaterialDistortion {
    const FRAG_PATH: &str = "assets/shaders/hlsl/particlesystem/distortion_ps.ps.glsl";
    const VERT_PATH: &str = "assets/shaders/hlsl/particlesystem/distortion_vs.vs.glsl";
}

impl Material for ParticleMaterialDistortion {
    fn fragment_shader() -> ShaderRef {
        // get_shader_handle(Self::FRAG_PATH, &vec![]).into()
        "shaders/distortion.frag".into()
    }

    fn vertex_shader() -> ShaderRef {
        // get_shader_handle(Self::VERT_PATH, &vec![]).into()
        "shaders/distortion.vert".into()
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

        // fragment.shader = key.bind_group_data.shader_frag;

        let vertex_layout = layout.0.get_layout(&[
            ATTRIBUTE_WORLD_POSITION_VEC4.at_shader_location(0),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(3),
            ATTRIBUTE_LIFETIME.at_shader_location(8),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(9),
            // ATTRIBUTE_UV_FRAME.at_shader_location(9),
            // ATTRIBUTE_UV_MULT.at_shader_location(9),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}
