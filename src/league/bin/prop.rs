use binrw::binread;

use crate::league::SizedStringU16;

#[binread]
#[br(little)]
#[br(magic = b"PROP")]
pub struct PropFile {
    pub version: u32,

    pub link_count: u32,

    #[br(count = link_count)]
    pub links: Vec<SizedStringU16>,

    pub entry_length: u32,

    #[br(count = entry_length)]
    pub entry_classes: Vec<u32>,

    #[br(count = entry_length)]
    pub entries: Vec<EntryData>,
}

#[binread]
#[br(little)]
pub struct EntryData {
    pub len: u32,

    pub hash: u32,

    #[br(count = len - 4)]
    pub data: Vec<u8>,
}

impl PropFile {
    pub fn get_entry(&self, hash: u32) -> &EntryData {
        self.entries.iter().find(|v| v.hash == hash).unwrap()
    }

    pub fn iter_entry_by_class(&self, hash: u32) -> impl Iterator<Item = &EntryData> {
        self.entries
            .iter()
            .enumerate()
            .filter(move |(i, _v)| self.entry_classes[*i] == hash)
            .map(|v| v.1)
    }

    pub fn iter_class_hash_and_entry(&self) -> impl Iterator<Item = (u32, &EntryData)> {
        self.entries
            .iter()
            .enumerate()
            .map(|(i, v)| (self.entry_classes[i], v))
    }
}
