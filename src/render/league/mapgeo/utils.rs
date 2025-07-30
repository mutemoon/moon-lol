use bevy::math::{Quat, Vec3};
use binrw::binread;

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct SizedString {
    pub len: u32,
    #[br(count = len, try_map = |bytes: Vec<u8>| String::from_utf8(bytes))]
    pub text: String,
}

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[binread]
#[br(little)]
#[derive(Debug, Clone)]
pub struct BinVec3(#[br(map = |(x, y, z): (f32, f32, f32)| Vec3::new(x, y, z))] pub Vec3);

#[binread]
#[br(little)]
#[derive(Debug, Clone)]
pub struct BinQuat(
    #[br(map = |(x, y, z, w): (f32, f32, f32, f32)| Quat::from_xyzw(x, y, z, w))] pub Quat,
);

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct Matrix4x4 {
    #[br(count = 16)]
    pub data: Vec<f32>,
}

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct BoundingBox {
    pub min: BinVec3,
    pub max: BinVec3,
}
