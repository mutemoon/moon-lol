use binrw::{binread, BinRead, BinResult, Endian};
use bitflags::bitflags;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InibinFlags: u16 {
        const INT32_LIST = 1;
        const FLOAT32_LIST = 1 << 1;
        const FIXED_POINT_FLOAT_LIST = 1 << 2;
        const INT16_LIST = 1 << 3;
        const INT8_LIST = 1 << 4;
        const BIT_LIST = 1 << 5;
        const FIXED_POINT_FLOAT_LIST_VEC3 = 1 << 6;
        const FLOAT32_LIST_VEC3 = 1 << 7;
        const FIXED_POINT_FLOAT_LIST_VEC2 = 1 << 8;
        const FLOAT32_LIST_VEC2 = 1 << 9;
        const FIXED_POINT_FLOAT_LIST_VEC4 = 1 << 10;
        const FLOAT32_LIST_VEC4 = 1 << 11;
        const STRING_LIST = 1 << 12;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InibinValue {
    Int32(i32),
    Float32(f32),
    FixedPointFloat(f32),
    Int16(i16),
    Int8(u8),
    Bool(bool),
    Vec3Fixed([u8; 3]),
    Vec3Float([f32; 3]),
    Vec2Fixed([u8; 2]),
    Vec2Float([f32; 2]),
    Vec4Fixed([u8; 4]),
    Vec4Float([f32; 4]),
    String(String),
}

#[derive(Debug, Clone)]
pub struct InibinSet {
    pub type_: InibinFlags,
    pub values: HashMap<u32, InibinValue>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct InibinFile {
    pub version: u8,

    #[br(parse_with = parse_inibin_sets, args(version))]
    pub sets: HashMap<InibinFlags, InibinSet>,
}

fn parse_inibin_sets<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    args: (u8,),
) -> BinResult<HashMap<InibinFlags, InibinSet>> {
    let (version,) = args;
    let mut sets = HashMap::new();

    if version == 1 {
        let _padding: [u8; 3] = BinRead::read(reader)?;
        let value_count: u32 = <_>::read_options(reader, endian, ())?;
        let string_data_length: u32 = <_>::read_options(reader, endian, ())?;

        let current_pos = reader.stream_position()?;
        let file_len = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(current_pos))?;

        let string_offset = file_len - string_data_length as u64;

        let set = parse_inibin_set_v1(
            reader,
            endian,
            InibinFlags::STRING_LIST,
            string_offset,
            value_count,
        )?;
        sets.insert(InibinFlags::STRING_LIST, set);
    } else if version == 2 {
        let string_data_length: u16 = <_>::read_options(reader, endian, ())?;
        let flags_val: u16 = <_>::read_options(reader, endian, ())?;
        let flags = InibinFlags::from_bits_truncate(flags_val);

        let current_pos = reader.stream_position()?;
        let file_len = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(current_pos))?;

        let string_offset = file_len - string_data_length as u64;

        let flag_order = [
            InibinFlags::INT32_LIST,
            InibinFlags::FLOAT32_LIST,
            InibinFlags::FIXED_POINT_FLOAT_LIST,
            InibinFlags::INT16_LIST,
            InibinFlags::INT8_LIST,
            InibinFlags::BIT_LIST,
            InibinFlags::FIXED_POINT_FLOAT_LIST_VEC3,
            InibinFlags::FLOAT32_LIST_VEC3,
            InibinFlags::FIXED_POINT_FLOAT_LIST_VEC2,
            InibinFlags::FLOAT32_LIST_VEC2,
            InibinFlags::FIXED_POINT_FLOAT_LIST_VEC4,
            InibinFlags::FLOAT32_LIST_VEC4,
            InibinFlags::STRING_LIST,
        ];

        for flag in flag_order {
            if flags.contains(flag) {
                let set = parse_inibin_set(reader, endian, flag, string_offset)?;
                sets.insert(flag, set);
            }
        }
    } else {
        return Err(binrw::Error::Custom {
            pos: reader.stream_position()?,
            err: Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unsupported version",
            )),
        });
    }

    Ok(sets)
}

fn parse_inibin_set<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    type_: InibinFlags,
    string_offset: u64,
) -> BinResult<InibinSet> {
    let value_count: u16 = <_>::read_options(reader, endian, ())?;
    parse_inibin_set_internal(reader, endian, type_, string_offset, value_count as u32)
}

fn parse_inibin_set_v1<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    type_: InibinFlags,
    string_offset: u64,
    value_count: u32,
) -> BinResult<InibinSet> {
    parse_inibin_set_internal(reader, endian, type_, string_offset, value_count)
}

fn parse_inibin_set_internal<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    type_: InibinFlags,
    string_offset: u64,
    value_count: u32,
) -> BinResult<InibinSet> {
    let mut hashes = Vec::with_capacity(value_count as usize);
    for _ in 0..value_count {
        let hash: u32 = <_>::read_options(reader, endian, ())?;
        hashes.push(hash);
    }

    let mut values = HashMap::new();
    let mut boolean: u8 = 0;

    for (i, hash) in hashes.into_iter().enumerate() {
        let value = match type_ {
            InibinFlags::INT32_LIST => InibinValue::Int32(<_>::read_options(reader, endian, ())?),
            InibinFlags::FLOAT32_LIST => {
                InibinValue::Float32(<_>::read_options(reader, endian, ())?)
            }
            InibinFlags::FIXED_POINT_FLOAT_LIST => {
                let b: u8 = <_>::read_options(reader, endian, ())?;
                InibinValue::FixedPointFloat(b as f32 * 0.1)
            }
            InibinFlags::INT16_LIST => InibinValue::Int16(<_>::read_options(reader, endian, ())?),
            InibinFlags::INT8_LIST => InibinValue::Int8(<_>::read_options(reader, endian, ())?),
            InibinFlags::BIT_LIST => {
                if i % 8 == 0 {
                    boolean = <_>::read_options(reader, endian, ())?;
                } else {
                    boolean >>= 1;
                }
                InibinValue::Bool((boolean & 0x1) != 0)
            }
            InibinFlags::FIXED_POINT_FLOAT_LIST_VEC3 => {
                let v: [u8; 3] = <_>::read_options(reader, endian, ())?;
                InibinValue::Vec3Fixed(v)
            }
            InibinFlags::FLOAT32_LIST_VEC3 => {
                let v: [f32; 3] = <_>::read_options(reader, endian, ())?;
                InibinValue::Vec3Float(v)
            }
            InibinFlags::FIXED_POINT_FLOAT_LIST_VEC2 => {
                let v: [u8; 2] = <_>::read_options(reader, endian, ())?;
                InibinValue::Vec2Fixed(v)
            }
            InibinFlags::FLOAT32_LIST_VEC2 => {
                let v: [f32; 2] = <_>::read_options(reader, endian, ())?;
                InibinValue::Vec2Float(v)
            }
            InibinFlags::FIXED_POINT_FLOAT_LIST_VEC4 => {
                let v: [u8; 4] = <_>::read_options(reader, endian, ())?;
                InibinValue::Vec4Fixed(v)
            }
            InibinFlags::FLOAT32_LIST_VEC4 => {
                let v: [f32; 4] = <_>::read_options(reader, endian, ())?;
                InibinValue::Vec4Float(v)
            }
            InibinFlags::STRING_LIST => {
                let offset: u16 = <_>::read_options(reader, endian, ())?;
                let old_pos = reader.stream_position()?;
                reader.seek(SeekFrom::Start(string_offset + offset as u64))?;

                let mut bytes = Vec::new();
                loop {
                    let b: u8 = BinRead::read(reader)?;
                    if b == 0 {
                        break;
                    }
                    bytes.push(b);
                }
                let s = String::from_utf8_lossy(&bytes).to_string();
                reader.seek(SeekFrom::Start(old_pos))?;
                InibinValue::String(s)
            }
            _ => {
                return Err(binrw::Error::Custom {
                    pos: reader.stream_position()?,
                    err: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Unsupported InibinFlags: {:?}", type_),
                    )),
                });
            }
        };
        values.insert(hash, value);
    }

    Ok(InibinSet { type_, values })
}
