use std::collections::VecDeque;

use serde::de::{self, Visitor};

use league_utils::hash_bin;

use crate::{
    BinDeserializerResult, BinParser, BinType, EnumReader, Error, HashMapReader, MapReader,
    SeqDerReader, SeqIntoReader,
};

pub struct BinDeserializer<'de> {
    pub parser: BinParser<'de>,
    pub value_type: BinType,
}

impl<'de> BinDeserializer<'de> {
    pub fn from_bytes(input: &'de [u8], value_type: BinType) -> Self {
        BinDeserializer {
            parser: BinParser::from_bytes(input),
            value_type,
        }
    }

    pub fn from_parser(parser: BinParser<'de>, value_type: BinType) -> Self {
        BinDeserializer { parser, value_type }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut BinDeserializer<'de> {
    type Error = Error;

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        struct_fields: &'static [&'static str],
        visitor: V,
    ) -> BinDeserializerResult<V::Value> {
        if self.value_type != BinType::Entry {
            self.parser.read_struct_header()?;
        }

        let data_map = self.parser.read_fields()?;

        visitor
            .visit_map(MapReader {
                data_map,
                struct_fields: struct_fields.iter(),
                next_value: None,
                current_field: None,
            })
            .map_err(|e| {
                if _name.is_empty() {
                    e
                } else {
                    e.with_context(format!("结构体 \"{}\"", _name))
                }
            })
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.parser.read_string()?)
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> BinDeserializerResult<V::Value> {
        match self.value_type {
            BinType::Bool => visitor.visit_bool(self.parser.read_bool()?),
            BinType::S8 => visitor.visit_i8(self.parser.read_i8()?),
            BinType::U8 => visitor.visit_u8(self.parser.read_u8()?),
            BinType::S16 => visitor.visit_i16(self.parser.read_s16()?),
            BinType::U16 => visitor.visit_u16(self.parser.read_u16()?),
            BinType::S32 => visitor.visit_i32(self.parser.read_s32()?),
            BinType::U32 => visitor.visit_u32(self.parser.read_u32()?),
            BinType::S64 => visitor.visit_i64(self.parser.read_s64()?),
            BinType::U64 => visitor.visit_u64(self.parser.read_u64()?),
            BinType::Float => visitor.visit_f32(self.parser.read_f32()?),
            BinType::Vec2 => {
                visitor.visit_seq(SeqIntoReader::from_values(self.parser.read_f32_many(2)?))
            }
            BinType::Vec3 => {
                visitor.visit_seq(SeqIntoReader::from_values(self.parser.read_f32_many(3)?))
            }
            BinType::Vec4 => {
                visitor.visit_seq(SeqIntoReader::from_values(self.parser.read_f32_many(4)?))
            }
            BinType::Matrix => {
                visitor.visit_seq(SeqIntoReader::from_values(self.parser.read_f32_many(16)?))
            }
            BinType::Color => {
                visitor.visit_seq(SeqIntoReader::from_values(self.parser.read_u8_many(4)?))
            }
            BinType::String => self.deserialize_string(visitor),
            BinType::Hash => visitor.visit_u32(self.parser.read_hash()?),
            BinType::Path => {
                // Path 通常是 u64
                visitor.visit_u64(self.parser.read_u64()?)
            }
            BinType::List | BinType::List2 => {
                let vtype = self.parser.read_type()?;
                let _bytes_count = self.parser.read_u32()?;
                let count = self.parser.read_u32()? as usize;

                let mut values = Vec::new();
                for _ in 0..count {
                    if vtype == BinType::Struct && self.parser.test_null_struct()? {
                        self.parser.skip_value(vtype)?;
                    } else {
                        values.push(self.parser.skip_value(vtype)?);
                    }
                }

                visitor.visit_seq(SeqDerReader::from_values(values, vtype))
            }
            BinType::Link => visitor.visit_u32(self.parser.read_link()?),
            BinType::Map => {
                let ktype = self.parser.read_type()?;
                let vtype = self.parser.read_type()?;

                let _bytes_count = self.parser.read_u32()?;
                let count = self.parser.read_u32()?;

                let mut map = VecDeque::new();
                for _ in 0..count {
                    let key = self.parser.skip_value(ktype)?;
                    if vtype == BinType::Struct && self.parser.test_null_struct()? {
                        self.parser.skip_value(vtype)?;
                    } else {
                        map.push_back((key, self.parser.skip_value(vtype)?));
                    }
                }

                visitor.visit_map(HashMapReader { map, ktype, vtype })
            }
            BinType::Flag => visitor.visit_bool(self.parser.read_flag()?),
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> BinDeserializerResult<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.value_type == BinType::Option {
            self.value_type = self.parser.read_type().unwrap();
            let some = self.parser.read_bool().unwrap();
            if some {
                visitor.visit_some(self)
            } else {
                visitor.visit_none()
            }
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> BinDeserializerResult<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> BinDeserializerResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let variant_hash = u32::from_le_bytes(self.parser.input[0..4].try_into().unwrap());

        let (variant_index, _variant_name) = variants
            .iter()
            .enumerate()
            .find(|(_i, &name)| {
                if name.starts_with("Unk") {
                    return u32::from_str_radix(&name[5..], 16).unwrap() == variant_hash;
                }
                if name == "MySelf" {
                    hash_bin("Self") == variant_hash
                } else {
                    hash_bin(name) == variant_hash
                }
            })
            .ok_or_else(|| {
                Error::UnknownVariant(format!(
                    "未知的 Enum 变体哈希: 0x{:x}, 已有的变体: {:?}",
                    variant_hash, variants
                ))
            })?;

        visitor
            .visit_enum(EnumReader {
                de: self,
                variant_index: variant_index as u32,
            })
            .map_err(|e| e.with_context(format!("枚举 \"{}\" (变体: {})", _name, _variant_name)))
    }

    serde::forward_to_deserialize_any! {

      bool i8 i16 i32 f32 u16 i64 u8 u32 u64 f64 char str bytes

      byte_buf unit unit_struct tuple

      tuple_struct map identifier ignored_any seq

    }
}
