use binrw::binrw;

#[binrw]
#[br(little)]
#[br(magic = b"PROP")]
#[derive(Debug)]
pub struct PropFile {
    pub version: u32,

    #[br(if(version >= 2))]
    pub type_info_v2: Option<TypeInfoV2>,

    #[br(dbg)]
    pub types_count: u32,

    #[br(count = types_count * 4)]
    pub types_data: Vec<u8>,

    #[br(count = types_count)]
    pub entries: Vec<EntryData>,
}

#[binrw]
#[br(little)]
#[derive(Debug)]
pub struct TypeInfoV2 {
    pub count: u32,

    #[br(count = count)]
    pub entries: Vec<EntryData>,
}

#[binrw]
#[br(little)]
#[derive(Debug)]
pub struct EntryData {
    pub len: u32,

    #[br(count = len)]
    pub data: Vec<u8>,
}
