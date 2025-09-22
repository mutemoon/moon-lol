use bevy::{
    color::palettes::tailwind,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, ShaderType,
            SpecializedMeshPipelineError,
        },
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<ConditionalMaterial>::default())
        .add_systems(Startup, setup)
        .run();
}

#[derive(Clone, Default, Debug, ShaderType)]
struct ConditionalUniforms {
    base_color: Vec4,

    highlight_color: Vec4,
    special_effect_color: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(ConditionalMaterialKey)]
pub struct ConditionalMaterial {
    #[uniform(0)]
    pub uniforms: ConditionalUniforms,

    pub use_highlight: bool,
    pub use_special_effect: bool,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct ConditionalMaterialKey {
    use_highlight: bool,
    use_special_effect: bool,
}

// 2. 为 Key 实现 From Trait
impl From<&ConditionalMaterial> for ConditionalMaterialKey {
    fn from(material: &ConditionalMaterial) -> Self {
        Self {
            use_highlight: material.use_highlight,
            use_special_effect: material.use_special_effect,
        }
    }
}

impl Material for ConditionalMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ps.frag".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/vs.vert".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();

        let mut shader_defs = Vec::new();

        if key.bind_group_data.use_highlight {
            shader_defs.push("USE_HIGHLIGHT".into());
        }
        if key.bind_group_data.use_special_effect {
            shader_defs.push("USE_SPECIAL_EFFECT".into());
        }

        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.shader_defs = shader_defs;
        }
        Ok(())
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ConditionalMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
    ));

    let mesh = Mesh::from(Plane3d::new(vec3(0.0, 1.0, 0.0), Vec2::splat(20.0)));

    let mesh = meshes.add(mesh);

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(materials.add(ConditionalMaterial {
            uniforms: ConditionalUniforms {
                base_color: tailwind::BLUE_500.to_vec4(),
                highlight_color: tailwind::GREEN_500.to_vec4(),
                special_effect_color: tailwind::RED_500.to_vec4(),
            },
            use_highlight: false,
            use_special_effect: true,
        })),
        Transform::from_translation(vec3(10., 0., 0.)),
    ));
}
