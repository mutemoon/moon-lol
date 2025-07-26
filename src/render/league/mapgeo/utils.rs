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
#[derive(Debug, Clone)]
#[br(little)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

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
    pub min: Vector3,
    pub max: Vector3,
}
