use std::collections::HashMap;

use bitflags::bitflags;
use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i16, le_i32, le_u16, le_u32, le_u8};
use nom::{IResult, Parser};

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

#[derive(Debug)]
pub struct InibinFile {
    pub version: u8,
    pub sets: HashMap<InibinFlags, InibinSet>,
}

impl InibinFile {
    pub fn parse(full_input: &[u8]) -> IResult<&[u8], Self> {
        let (i, version) = le_u8(full_input)?;
        let mut sets = HashMap::new();

        if version == 1 {
            let (i, _padding) = take(3usize)(i)?;
            let (i, value_count) = le_u32(i)?;
            let (i, string_data_length) = le_u32(i)?;

            let string_offset = (full_input.len() as u32 - string_data_length) as usize;

            let (i, set) = parse_inibin_set_internal(
                i,
                full_input,
                InibinFlags::STRING_LIST,
                string_offset,
                value_count,
            )?;
            sets.insert(InibinFlags::STRING_LIST, set);
            Ok((i, InibinFile { version, sets }))
        } else if version == 2 {
            let (i, string_data_length) = le_u16(i)?;
            let (i, flags_val) = le_u16(i)?;
            let flags = InibinFlags::from_bits_truncate(flags_val);

            let string_offset = (full_input.len() as u16 - string_data_length) as usize;

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

            let mut current_i = i;
            for flag in flag_order {
                if flags.contains(flag) {
                    let (i_next, value_count) = le_u16(current_i)?;
                    let (i_next, set) = parse_inibin_set_internal(
                        i_next,
                        full_input,
                        flag,
                        string_offset,
                        value_count as u32,
                    )?;
                    sets.insert(flag, set);
                    current_i = i_next;
                }
            }
            Ok((current_i, InibinFile { version, sets }))
        } else {
            panic!("Unsupported version: {}", version);
        }
    }
}

fn parse_inibin_set_internal<'a>(
    input: &'a [u8],
    full_input: &'a [u8],
    type_: InibinFlags,
    string_offset: usize,
    value_count: u32,
) -> IResult<&'a [u8], InibinSet> {
    let (i, hashes) = count(le_u32, value_count as usize).parse(input)?;

    let mut values = HashMap::new();
    let mut boolean: u8 = 0;
    let mut current_i = i;

    for (idx, hash) in hashes.into_iter().enumerate() {
        let (i_next, value) = match type_ {
            InibinFlags::INT32_LIST => {
                le_i32(current_i).map(|(i, v)| (i, InibinValue::Int32(v)))?
            }
            InibinFlags::FLOAT32_LIST => {
                le_f32(current_i).map(|(i, v)| (i, InibinValue::Float32(v)))?
            }
            InibinFlags::FIXED_POINT_FLOAT_LIST => {
                le_u8(current_i).map(|(i, v)| (i, InibinValue::FixedPointFloat(v as f32 * 0.1)))?
            }
            InibinFlags::INT16_LIST => {
                le_i16(current_i).map(|(i, v)| (i, InibinValue::Int16(v)))?
            }
            InibinFlags::INT8_LIST => le_u8(current_i).map(|(i, v)| (i, InibinValue::Int8(v)))?,
            InibinFlags::BIT_LIST => {
                if idx % 8 == 0 {
                    let (i, b) = le_u8(current_i)?;
                    boolean = b;
                    (i, InibinValue::Bool((boolean & 0x1) != 0))
                } else {
                    boolean >>= 1;
                    (current_i, InibinValue::Bool((boolean & 0x1) != 0))
                }
            }
            InibinFlags::FIXED_POINT_FLOAT_LIST_VEC3 => {
                let (i, v) = take(3usize)(current_i)?;
                (i, InibinValue::Vec3Fixed([v[0], v[1], v[2]]))
            }
            InibinFlags::FLOAT32_LIST_VEC3 => {
                let (i, v0) = le_f32(current_i)?;
                let (i, v1) = le_f32(i)?;
                let (i, v2) = le_f32(i)?;
                (i, InibinValue::Vec3Float([v0, v1, v2]))
            }
            InibinFlags::FIXED_POINT_FLOAT_LIST_VEC2 => {
                let (i, v) = take(2usize)(current_i)?;
                (i, InibinValue::Vec2Fixed([v[0], v[1]]))
            }
            InibinFlags::FLOAT32_LIST_VEC2 => {
                let (i, v0) = le_f32(current_i)?;
                let (i, v1) = le_f32(i)?;
                (i, InibinValue::Vec2Float([v0, v1]))
            }
            InibinFlags::FIXED_POINT_FLOAT_LIST_VEC4 => {
                let (i, v) = take(4usize)(current_i)?;
                (i, InibinValue::Vec4Fixed([v[0], v[1], v[2], v[3]]))
            }
            InibinFlags::FLOAT32_LIST_VEC4 => {
                let (i, v0) = le_f32(current_i)?;
                let (i, v1) = le_f32(i)?;
                let (i, v2) = le_f32(i)?;
                let (i, v3) = le_f32(i)?;
                (i, InibinValue::Vec4Float([v0, v1, v2, v3]))
            }
            InibinFlags::STRING_LIST => {
                let (i, offset) = le_u16(current_i)?;
                let string_pos = string_offset + offset as usize;
                let mut bytes = Vec::new();
                let mut s_input = &full_input[string_pos..];
                loop {
                    let (s_i, b) = le_u8(s_input)?;
                    if b == 0 {
                        break;
                    }
                    bytes.push(b);
                    s_input = s_i;
                }
                let s = String::from_utf8_lossy(&bytes).to_string();
                (i, InibinValue::String(s))
            }
            _ => panic!("Unsupported InibinFlags: {:?}", type_),
        };
        values.insert(hash, value);
        current_i = i_next;
    }

    Ok((current_i, InibinSet { type_, values }))
}
