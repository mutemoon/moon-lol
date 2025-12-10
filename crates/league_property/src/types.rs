use std::fmt::Display;

use serde::de;
use thiserror::Error;

#[derive(Debug)]
pub struct BinStructHeader {
    pub class_hash: u32,
    pub bytes_count: usize,
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Invalid BIN type byte: {0}")]
    InvalidBinType(u8),

    #[error("Unknown variant: {0}")]
    UnknownVariant(String),

    #[error("{message}\n  -> {context}")]
    WithContext {
        context: String,
        #[source]
        source: Box<Error>,
        message: String,
    },
}

impl Error {
    /// 为错误添加上下文，形成类似堆栈的错误链
    pub fn with_context(self, context: impl Into<String>) -> Self {
        let context = context.into();
        let message = self.format_full_chain();
        Error::WithContext {
            context,
            source: Box::new(self),
            message,
        }
    }

    /// 格式化完整的错误链，生成类似堆栈的输出
    fn format_full_chain(&self) -> String {
        let mut parts = Vec::new();
        self.collect_chain(&mut parts);
        parts.join("\n")
    }

    fn collect_chain(&self, parts: &mut Vec<String>) {
        match self {
            Error::WithContext {
                context, source, ..
            } => {
                parts.push(format!("  -> {}", context));
                source.collect_chain(parts);
            }
            other => {
                parts.push(format!("  !! {}", other.root_message()));
            }
        }
    }

    fn root_message(&self) -> String {
        match self {
            Error::Message(msg) => msg.clone(),
            Error::MissingField(field) => format!("缺少字段: {}", field),
            Error::InvalidBinType(byte) => format!("无效的类型字节: {}", byte),
            Error::UnknownVariant(msg) => msg.clone(),
            Error::WithContext { source, .. } => source.root_message(),
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

pub type BinDeserializerResult<T> = Result<T, Error>;

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
    type Error = Error;

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
            _ => return Err(Error::InvalidBinType(value)),
        })
    }
}
