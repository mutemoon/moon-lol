use bevy::{prelude::*, render::mesh::VertexAttributeValues};

use crate::core::particle::{ATTRIBUTE_LIFETIME, ATTRIBUTE_UV_FRAME, ATTRIBUTE_WORLD_POSITION};

#[derive(Default)]
pub struct ParticleQuad {}

impl From<ParticleQuad> for Mesh {
    fn from(_value: ParticleQuad) -> Self {
        let mut mesh = Mesh::from(Plane3d::new(vec3(0.0, 1.0, 0.0), Vec2::splat(100.0)));

        let VertexAttributeValues::Float32x3(values) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        else {
            panic!();
        };

        mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION, values.clone());

        let VertexAttributeValues::Float32x2(uv_values) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!();
        };

        let values = uv_values
            .into_iter()
            .map(|&v| [v[0], v[1], 0.0, 0.0])
            .collect::<Vec<_>>();

        mesh.insert_attribute(ATTRIBUTE_UV_FRAME, values);

        let values = Vec::from([[0.0; 2]; 4]);
        mesh.insert_attribute(ATTRIBUTE_LIFETIME, values);

        let values = Vec::from([[1.0; 4]; 4]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, values);

        mesh
    }
}
