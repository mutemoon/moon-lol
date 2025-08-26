use std::fmt::Display;

use serde::de;
use thiserror::Error;

pub type BinLink = u32;

pub type BinHash = u32;

#[derive(Debug)]
pub struct BinStructHeader {
    pub class_hash: u32,
    pub bytes_count: usize,
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum BinDeserializerError {
    #[error("{0}")]
    Message(String),

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Invalid BIN type byte: {0}")]
    InvalidBinType(u8),

    #[error("Unknown variant: {0}")]
    UnknownVariant(String),
}

impl de::Error for BinDeserializerError {
    fn custom<T: Display>(msg: T) -> Self {
        BinDeserializerError::Message(msg.to_string())
    }
}

pub type BinDeserializerResult<T> = Result<T, BinDeserializerError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BinType {
    None = 0,
    Bool = 1,
    S8 = 2,
    U8 = 3,
    S16 = 4,
    U16 = 5,
    S32 = 6,
    U32 = 7,
    S64 = 8,
    U64 = 9,
    Float = 10,
    Vec2 = 11,
    Vec3 = 12,
    Vec4 = 13,
    Matrix = 14,
    Color = 15,
    String = 16,
    Hash = 17,
    Path = 18,
    List = 19,
    List2 = 20,
    Struct = 21,
    Embed = 22,
    Link = 23,
    Option = 24,
    Map = 25,
    Flag = 26,
    Entry,
}

impl TryFrom<u8> for BinType {
    type Error = BinDeserializerError;

    fn try_from(mut value: u8) -> Result<Self, Self::Error> {
        if value >= 0x80 {
            value = value - 0x80 + (BinType::List as u8);
        }
        Ok(match value {
            0 => BinType::None,
            1 => BinType::Bool,
            2 => BinType::S8,
            3 => BinType::U8,
            4 => BinType::S16,
            5 => BinType::U16,
            6 => BinType::S32,
            7 => BinType::U32,
            8 => BinType::S64,
            9 => BinType::U64,
            10 => BinType::Float,
            11 => BinType::Vec2,
            12 => BinType::Vec3,
            13 => BinType::Vec4,
            14 => BinType::Matrix,
            15 => BinType::Color,
            16 => BinType::String,
            17 => BinType::Hash,
            18 => BinType::Path,
            19 => BinType::List,
            20 => BinType::List2,
            21 => BinType::Struct,
            22 => BinType::Embed,
            23 => BinType::Link,
            24 => BinType::Option,
            25 => BinType::Map,
            26 => BinType::Flag,
            _ => return Err(BinDeserializerError::InvalidBinType(value)),
        })
    }
}
