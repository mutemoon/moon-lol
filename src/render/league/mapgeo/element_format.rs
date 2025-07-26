use binrw::binread;

#[binread]
#[derive(Debug, Clone)]
#[br(little, repr = u32)]
pub enum ElementFormat {
    Unknown = -1,
    XFloat32,
    XyFloat32,
    XyzFloat32,
    XyzwFloat32,
    BgraPacked8888,
    ZyxwPacked8888,
    RgbaPacked8888,
    XyPacked1616,
    XyzPacked161616,
    XyzwPacked16161616,
    XyPacked88,
    XyzPacked888,
    XyzwPacked8888,
}

impl ElementFormat {
    pub fn get_size(&self) -> usize {
        match self {
            ElementFormat::XFloat32 => 4,
            ElementFormat::XyFloat32 => 8,
            ElementFormat::XyzFloat32 => 12,
            ElementFormat::XyzwFloat32 => 16,
            ElementFormat::BgraPacked8888
            | ElementFormat::ZyxwPacked8888
            | ElementFormat::RgbaPacked8888 => 4,
            ElementFormat::XyPacked1616 => 4,
            ElementFormat::XyzPacked161616 => 8,
            ElementFormat::XyzwPacked16161616 => 8,
            ElementFormat::XyPacked88 => 2,
            ElementFormat::XyzPacked888 => 3,
            ElementFormat::XyzwPacked8888 => 4,
            ElementFormat::Unknown => 0,
        }
    }
}
