use std::f32::consts::PI;

use bevy::asset::uuid::Uuid;
use bevy::mesh::{
    MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexAttributeValues, VertexFormat,
};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor,
    ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

// 原文中的顶点属性
const ATTRIBUTE_WORLD_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("ATTRIBUTE_WORLD_POSITION", 2020, VertexFormat::Float32x3);

const ATTRIBUTE_UV_FRAME: MeshVertexAttribute =
    MeshVertexAttribute::new("ATTRIBUTE_UV_FRAME", 2022, VertexFormat::Float32x3);

const ATTRIBUTE_LIFETIME: MeshVertexAttribute =
    MeshVertexAttribute::new("ATTRIBUTE_LIFETIME", 2023, VertexFormat::Float32x2);

const ATTRIBUTE_UV_MULT: MeshVertexAttribute =
    MeshVertexAttribute::new("ATTRIBUTE_UV_MULT", 2024, VertexFormat::Float32x2);

// 为 Shaders 分配固定的 UUID 以生成静态 Handle
const VERT_SHADER_UUID: Uuid = Uuid::from_u128(0x712a1709254077376921000000000001);
const FRAG_SHADER_UUID: Uuid = Uuid::from_u128(0x712a1709254077376921000000000002);

#[derive(Clone, ShaderType, Debug)]
pub struct MyVertexParams {
    pub val0: f32,   // Offset 0
    pub val1_x: f32, // Offset 4
    pub val1_y: f32, // Offset 8
    pub val2: f32,   // Offset 12
    pub val3: Vec4,  // Offset 16
}

impl Default for MyVertexParams {
    fn default() -> Self {
        Self {
            val0: 0.0,
            val1_x: 1.0,
            val1_y: 1.0,
            val2: 0.0,
            val3: Vec4::ZERO,
        }
    }
}

#[derive(Clone, ShaderType, Debug)]
pub struct MyMatrixParams {
    pub row0: Vec4,       // Offset 0
    pub row1: Vec4,       // Offset 16
    pub row2: Vec4,       // Offset 32
    pub row3: Vec4,       // Offset 48
    pub camera_pos: Vec3, // Offset 64
    pub padding: f32,     // Offset 76
}

impl Default for MyMatrixParams {
    fn default() -> Self {
        Self {
            row0: Vec4::new(1.0, 0.0, 0.0, 0.0),
            row1: Vec4::new(0.0, 1.0, 0.0, 0.0),
            row2: Vec4::new(0.0, 0.0, 1.0, 0.5),
            row3: Vec4::new(0.0, 0.0, 0.0, 1.0),
            camera_pos: Vec3::ZERO,
            padding: 0.0,
        }
    }
}

#[derive(Asset, AsBindGroup, TypePath, Clone, Debug, Default)]
#[bind_group_data(ConditionalMaterialKeyQuad)]
pub struct ParticleMaterialQuad {
    #[uniform(0, visibility(vertex, fragment))]
    pub uniforms_vertex: MyVertexParams,

    #[uniform(1, visibility(vertex, fragment))]
    pub uniforms_pixel: MyMatrixParams,

    #[sampler(4)]
    #[texture(5)]
    pub t0: Option<Handle<Image>>,

    #[sampler(2)]
    #[texture(6)]
    pub t1: Option<Handle<Image>>,

    #[sampler(3)]
    #[texture(7)]
    pub t2: Option<Handle<Image>>,

    pub blend_mode: u8,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct ConditionalMaterialKeyQuad {
    blend_mode: u8,
}

impl From<&ParticleMaterialQuad> for ConditionalMaterialKeyQuad {
    fn from(material: &ParticleMaterialQuad) -> Self {
        Self {
            blend_mode: material.blend_mode,
        }
    }
}

impl Material for ParticleMaterialQuad {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(Handle::Uuid(VERT_SHADER_UUID, std::marker::PhantomData))
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(Handle::Uuid(FRAG_SHADER_UUID, std::marker::PhantomData))
    }

    fn alpha_mode(&self) -> AlphaMode {
        match self.blend_mode {
            1 | 4 => AlphaMode::Blend,
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
            ATTRIBUTE_WORLD_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(1),
            ATTRIBUTE_UV_FRAME.at_shader_location(2),
            ATTRIBUTE_LIFETIME.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}

fn auto_exit(mut counter: bevy::prelude::Local<u32>) {
    *counter += 1;
    if *counter > 250 {
        std::process::exit(0);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<ParticleMaterialQuad>::default())
        .add_systems(Startup, (setup_shaders, setup))
        .add_systems(Update, auto_exit)
        .run();
}

fn setup_shaders(mut shaders: ResMut<Assets<Shader>>) {
    let vert_spv =
        std::fs::read("assets/shaders/hlsl/particlesystem/quad/vs/shader_0000.spv").unwrap();
    let frag_spv =
        std::fs::read("assets/shaders/hlsl/particlesystem/quad/ps/shader_0000.spv").unwrap();

    let vert_shader = Shader::from_spirv(vert_spv, "hlsl/particlesystem/quad/vs/shader_0000.spv");
    let frag_shader = Shader::from_spirv(frag_spv, "hlsl/particlesystem/quad/ps/shader_0000.spv");

    let _ = shaders.insert(
        Handle::<Shader>::Uuid(VERT_SHADER_UUID, std::marker::PhantomData).id(),
        vert_shader,
    );
    let _ = shaders.insert(
        Handle::<Shader>::Uuid(FRAG_SHADER_UUID, std::marker::PhantomData).id(),
        frag_shader,
    );
}

fn create_quad_mesh() -> Mesh {
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
            .map(|v| [1. - v[0], 1. - v[1], 0.0])
            .collect::<Vec<_>>();

        mesh.insert_attribute(ATTRIBUTE_UV_FRAME, values);
    }

    let values = Vec::from([[0.0; 2]; 4]);
    mesh.insert_attribute(ATTRIBUTE_LIFETIME, values);

    let values = Vec::from([[1.0; 4]; 4]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, values);

    mesh
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParticleMaterialQuad>>,
    mut images: ResMut<Assets<Image>>,
) {
    // 摄像机
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 制作纯白纹理用于 t1, t2
    let white_image = Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &[
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        ],
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    let white_handle = images.add(white_image);

    // 制作纯绿纹理用于 t0
    let green_image = Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &[
            0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255, 0, 255,
        ],
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    let green_handle = images.add(green_image);

    let mesh_handle = meshes.add(create_quad_mesh());

    let material_handle = materials.add(ParticleMaterialQuad {
        uniforms_vertex: MyVertexParams::default(),
        uniforms_pixel: MyMatrixParams::default(),
        t0: Some(green_handle),
        t1: Some(white_handle.clone()),
        t2: Some(white_handle),
        blend_mode: 0,
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
