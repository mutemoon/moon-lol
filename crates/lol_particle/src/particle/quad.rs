use std::f32::consts::PI;

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use crate::{ATTRIBUTE_LIFETIME, ATTRIBUTE_UV_FRAME, ATTRIBUTE_UV_MULT, ATTRIBUTE_WORLD_POSITION};

// ---------------------------------------------------------------------------
// 网格构建
// ---------------------------------------------------------------------------

/// Quad 粒子网格构建器：生成带世界坐标、UV 帧、UV 乘子、生命周期与顶点色属性的
/// 单位平面（静态材质已删除，渲染走动态材质 ParticleMaterialDynamic）
#[derive(Default)]
pub struct ParticleMeshQuad {
    pub frame: f32,
}

impl From<ParticleMeshQuad> for Mesh {
    fn from(value: ParticleMeshQuad) -> Self {
        let mut mesh: Mesh = Plane3d::new(Vec3::NEG_Z, Vec2::splat(1.0)).into();

        let transform = Transform::from_rotation(Quat::from_rotation_z(PI / 2.));

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
                .map(|v| [v[0], v[1], value.frame as f32])
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
