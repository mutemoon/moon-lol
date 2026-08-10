use bevy::prelude::Vec3;
use bevy::reflect::Reflect;
use nom::IResult;
use nom::number::complete::le_f32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, min) = nom_parse_vec3(input)?;
        let (i, max) = nom_parse_vec3(i)?;
        Ok((i, BoundingBox { min, max }))
    }
}

fn nom_parse_vec3(input: &[u8]) -> IResult<&[u8], Vec3> {
    let (i, x) = le_f32(input)?;
    let (i, y) = le_f32(i)?;
    let (i, z) = le_f32(i)?;
    Ok((i, Vec3::new(x, y, z)))
}
