use serde::de::{self, Visitor};

use crate::league::{
    BinDeserializerError, BinDeserializerResult, BinParser, BinType, EnumReader, HashMapReader,
    LeagueLoader, MapReader, SeqReader,
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
    type Error = BinDeserializerError;

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        struct_fields: &'static [&'static str],
        visitor: V,
    ) -> BinDeserializerResult<V::Value> {
        println!("🚀 正在处理 rust 结构体: {:?}", name);

        if self.value_type != BinType::Entry {
            self.parser.read_struct_header()?;
        }

        let data_map = self.parser.read_fields()?;

        visitor.visit_map(MapReader {
            data_map,
            struct_fields: struct_fields.iter(),
            next_value: None,
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
            BinType::None => {
                todo!()
                // self.parser.read_bytes(6)?;
                // visitor.visit_unit()
            }
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
            BinType::Vec2 => visitor.visit_seq(SeqReader {
                de: &mut BinDeserializer::from_bytes(self.parser.input, BinType::Float),
                count: 2,
            }),
            BinType::Vec3 => visitor.visit_seq(SeqReader {
                de: &mut BinDeserializer::from_bytes(self.parser.input, BinType::Float),
                count: 3,
            }),
            BinType::Vec4 => visitor.visit_seq(SeqReader {
                de: &mut BinDeserializer::from_bytes(self.parser.input, BinType::Float),
                count: 4,
            }),
            BinType::Matrix => visitor.visit_seq(SeqReader {
                de: &mut BinDeserializer::from_bytes(self.parser.input, BinType::Float),
                count: 16,
            }),
            BinType::Color => todo!(),
            BinType::String => self.deserialize_string(visitor),
            BinType::Hash => visitor.visit_u32(self.parser.read_hash()?),
            BinType::Path => {
                // Path 通常是 u64
                visitor.visit_u64(self.parser.read_u64()?)
            }
            BinType::List | BinType::List2 => {
                let value_bin_type = self.parser.read_type()?;
                let _padding = self.parser.read_u32()?;
                let count = self.parser.read_u32()? as usize;
                println!("📕 获取线性信息: {:?} 共 {} 个", value_bin_type, count);

                visitor.visit_seq(SeqReader {
                    de: &mut BinDeserializer::from_bytes(self.parser.input, value_bin_type),
                    count,
                })
            }
            BinType::Struct | BinType::Embed => {
                unreachable!()
            }
            BinType::Link => {
                // Hash 和 Link 通常是 u32 或 u64 的包装，这里假设为 u32
                visitor.visit_u32(self.parser.read_link()?)
            }
            BinType::Option => todo!(),
            BinType::Map => {
                let ktype = self.parser.read_type()?;
                let vtype = self.parser.read_type()?;

                let _bytes_count = self.parser.read_u32()?;
                let count = self.parser.read_u32()?;
                println!("🐕 获取 Map 信息: {:?} 共 {} 个", (ktype, vtype), count);

                visitor.visit_map(HashMapReader {
                    de: &mut BinDeserializer::from_bytes(self.parser.input, ktype),
                    ktype,
                    vtype,
                    count,
                })
            }
            BinType::Flag => visitor.visit_bool(self.parser.read_flag()?),
            BinType::Entry => todo!(),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> BinDeserializerResult<V::Value>
    where
        V: Visitor<'de>,
    {
        println!("🎁 正在反序列化 Option，判定为 Some(...)");
        visitor.visit_some(self)
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
        println!("👻 开始反序列化 Enum 长度: {}", self.parser.input.len());
        let variant_hash = u32::from_le_bytes(self.parser.input[0..4].try_into().unwrap());
        println!(
            "👻 准备反序列化 Enum，偷看到的类型哈希为: {:x}",
            variant_hash
        );

        let (variant_index, _variant_name) = variants
            .iter()
            .enumerate()
            .find(|(_i, name)| {
                if name.starts_with("Unk") {
                    println!("👻 跳过 Unk 类型: {}", name);
                    return u32::from_str_radix(&name[5..], 16).unwrap() == variant_hash;
                }
                println!(
                    "👻 class hash: {:x}  variant name : {} hash: {:x}",
                    variant_hash,
                    name,
                    LeagueLoader::hash_bin(name)
                );

                LeagueLoader::hash_bin(name) == variant_hash
            })
            .ok_or_else(|| {
                BinDeserializerError::UnknownVariant(format!(
                    "未知的 Enum 变体哈希: 0x{:x}, 已有的变体: {:?}",
                    variant_hash, variants
                ))
            })?;

        println!(
            "🐕 获取变体索引: {}，变体名称: {}",
            variant_index, variants[variant_index]
        );

        visitor.visit_enum(EnumReader {
            de: self,
            variant_index: variant_index as u32,
        })
    }

    serde::forward_to_deserialize_any! {

      bool i8 i16 i32 f32 u16 i64 u8 u32 u64 f64 char str bytes

      byte_buf unit unit_struct tuple

      tuple_struct map identifier ignored_any seq

    }
}
