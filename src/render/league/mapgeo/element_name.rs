use binrw::binread;

#[binread]
#[derive(Debug, Clone)]
#[br(little, repr = u32)]
pub enum ElementName {
    Unknown = -1,
    Position,
    BlendWeight,
    Normal,
    FogCoordinate,
    PrimaryColor,
    SecondaryColor,
    BlendIndex,
    Texcoord0,
    Texcoord1,
    Texcoord2,
    Texcoord3,
    Texcoord4,
    Texcoord5,
    Texcoord6,
    Texcoord7,
    Tangent,
}
