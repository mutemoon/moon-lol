use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::{le_u16, le_u32};
use nom::{IResult, Parser};

pub struct PropFile {
    pub version: u32,
    pub links: Vec<SizedStringU16>,
    pub entry_classes: Vec<u32>,
    pub entries: Vec<EntryData>,
}

impl PropFile {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, _) = take(4usize)(input)?; // magic: PROP
        let (i, version) = le_u32(i)?;
        let (i, link_count) = le_u32(i)?;
        let (i, links) = count(SizedStringU16::parse, link_count as usize).parse(i)?;
        let (i, entry_length) = le_u32(i)?;
        let (i, entry_classes) = count(le_u32, entry_length as usize).parse(i)?;
        let (i, entries) = count(EntryData::parse, entry_length as usize).parse(i)?;

        Ok((
            i,
            PropFile {
                version,
                links,
                entry_classes,
                entries,
            },
        ))
    }

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

pub struct EntryData {
    pub len: u32,
    pub hash: u32,
    pub data: Vec<u8>,
}

impl EntryData {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, len) = le_u32(input)?;
        let (i, hash) = le_u32(i)?;
        let (i, data) = take((len - 4) as usize)(i)?;
        Ok((
            i,
            EntryData {
                len,
                hash,
                data: data.to_vec(),
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct SizedStringU16 {
    pub len: u16,
    pub text: String,
}

impl SizedStringU16 {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, len) = le_u16(input)?;
        let (i, bytes) = take(len as usize)(i)?;
        let text = String::from_utf8_lossy(bytes).to_string();
        Ok((i, SizedStringU16 { len, text }))
    }
}
