use binrw::BinRead;
use moon_lol::league::PropFile;
use serde::de::{self, IntoDeserializer, MapAccess, SeqAccess, Visitor};

use serde::Deserialize;

use std::collections::HashMap;

use std::fmt::{self, Display};

use std::fs::File;
use std::io::BufReader;

// --- 目标数据结构 (保持不变) ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]

pub struct VfxSystemDefinitionData {
    pub complex_emitter_definition_data: Vec<VfxEmitterDefinitionData>,
    pub particle_name: String,
    pub particle_path: String,
    pub flags: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]

pub struct VfxEmitterDefinitionData {
    pub emitter_name: String,
    pub primitive: VfxPrimitive,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VfxPrimitive {
    #[serde(rename_all = "camelCase")]
    VfxPrimitiveMesh {
        m_mesh: VfxMeshDefinitionData,
    },
    VfxPrimitiveArbitraryQuad,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VfxMeshDefinitionData {
    pub m_simple_mesh_name: String,
}

// --- main 函数 (演示如何使用) ---

fn main() {
    let path = "assets/fiora_skins_skin41_skins_skin42_skins_skin43_skins_skin44_skins_skin45_skins_skin46_skins_skin47_skins_skin48_skins_skin49.bin";

    println!("尝试读取文件: {}", path);

    let file = File::open(path).unwrap();

    let prop_file = PropFile::read(&mut BufReader::new(file)).unwrap();

    let vfx_data =
        bin_deserializer::from_slice::<VfxSystemDefinitionData>(&prop_file.entries[0].data)
            .unwrap();

    println!("反序列化成功，结果: {:#?}", vfx_data);
}

// +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
// + 完全独立的二进制 Deserializer 模块
// +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

pub mod bin_deserializer {
    use super::*;

    use binrw::helpers::count;
    use cdragon_hashes::bin::compute_binhash;
    use serde::{
        de::{EnumAccess, VariantAccess},
        Deserializer,
    };

    /// 从完整的 .bin 文件字节流中反序列化第一个 Entry
    pub fn from_slice<'de, T>(slice: &'de [u8]) -> Result<T, Error>
    where
        T: Deserialize<'de>,
    {
        let mut deserializer = BinDeserializer::from_bytes(slice, true);
        T::deserialize(&mut deserializer)
    }

    #[derive(Clone, Debug, PartialEq)]

    pub enum Error {
        Message(String),
        Eof,
        MissingField(String),
        InvalidBinType(u8),
    }

    impl de::Error for Error {
        fn custom<T: Display>(msg: T) -> Self {
            Error::Message(msg.to_string())
        }
    }

    impl Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::Message(msg) => write!(f, "{}", msg),
                Error::Eof => write!(f, "Unexpected end of input"),
                Error::MissingField(name) => write!(f, "Missing field: {}", name),
                Error::InvalidBinType(byte) => write!(f, "Invalid BIN type byte: {}", byte),
            }
        }
    }

    impl std::error::Error for Error {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]

    enum BinType {
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

    // --- 核心 Deserializer ---

    pub struct BinDeserializer<'de> {
        input: &'de [u8],

        is_top_level: bool, // <--- 新增状态标志
    }

    impl<'de> BinDeserializer<'de> {
        fn from_bytes(input: &'de [u8], is_top_level: bool) -> Self {
            BinDeserializer {
                input,
                is_top_level,
            }
        }

        fn read_bytes(&mut self, len: usize) -> Result<&'de [u8], Error> {
            if self.input.len() < len {
                return Err(Error::Eof);
            }

            let (slice, rest) = self.input.split_at(len);

            self.input = rest;

            Ok(slice)
        }

        fn read_bintype(&mut self) -> Result<BinType, Error> {
            BinType::try_from(u8::from_le_bytes(self.read_bytes(1)?.try_into().unwrap()))
        }
    }

    impl<'de, 'a> de::Deserializer<'de> for &'a mut BinDeserializer<'de> {
        type Error = Error;

        fn deserialize_any<V: Visitor<'de>>(self, _v: V) -> Result<V::Value, Self::Error> {
            Err(Error::Message("deserialize_any unsupported".into()))
        }

        fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            visitor.visit_u16(u16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap()))
        }

        fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            let len = u16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap()) as usize;

            let s = std::str::from_utf8(self.read_bytes(len)?)
                .map_err(|e| Error::Message(e.to_string()))?;
            println!("😫 解析字符串：{}", s);

            visitor.visit_string(s.to_owned())
        }

        fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            let vtype = self.read_bintype()?;
            let _padding = self.read_bytes(4)?;
            let count = u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap()) as usize;
            println!("📕 获取线性信息: {:?} 共 {} 个", vtype, count);

            visitor.visit_seq(SeqReader {
                de: self,
                vtype,
                count,
            })
        }

        fn deserialize_struct<V: Visitor<'de>>(
            self,
            _name: &'static str,
            struct_fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            let (hash, field_count) = if self.is_top_level {
                let hash = u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap());

                let field_count =
                    u16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap()) as usize;

                (hash, field_count)
            } else {
                let hash = u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap());

                let _fields_len = self.read_bytes(4)?;

                let field_count =
                    u16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap()) as usize;

                (hash, field_count)
            };
            println!("获取映射信息: 哈希值为 {:x} 总共 {} 个", hash, field_count);

            let mut data_map: HashMap<u32, (BinType, &'de [u8])> =
                HashMap::with_capacity(field_count);

            let fields_block_all = self.input;

            let mut temp_parser = BinDeserializer::from_bytes(fields_block_all, false);

            for i in 0..field_count {
                let hash = u32::from_le_bytes(temp_parser.read_bytes(4)?.try_into().unwrap());
                println!("获取映射信息: 第 {} 个 hash 为 {:x}", i, hash);

                let vtype = temp_parser.read_bintype()?;

                let value_start_offset = fields_block_all.len() - temp_parser.input.len();

                let before_len = temp_parser.input.len();

                temp_parser.skip_value(vtype)?;

                println!(
                    "尝试跳过类型: {:?}，总计: {}，剩余：{}",
                    vtype,
                    before_len - temp_parser.input.len(),
                    temp_parser.input.len()
                );

                let value_end_offset = fields_block_all.len() - temp_parser.input.len();

                let value_slice = &fields_block_all[value_start_offset..value_end_offset];

                data_map.insert(hash, (vtype, value_slice));
            }

            self.input = temp_parser.input;

            visitor.visit_map(MapReader {
                data_map,
                struct_fields: struct_fields.iter(),
                next_value: None,
            })
        }

        fn deserialize_newtype_struct<V: Visitor<'de>>(
            self,

            _name: &'static str,

            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            visitor.visit_newtype_struct(self)
        }

        /// 新增：实现 deserialize_enum 方法来处理我们的 VfxPrimitive enum
        fn deserialize_enum<V>(
            self,
            _name: &'static str,
            variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            // 在 .bin 格式中，一个嵌入的 struct/enum 的开头就是它的类型哈希 (class hash)。
            // 我们需要 "偷看" 一下这个哈希值，来判断它究竟是哪个 enum 变体。
            if self.input.len() < 4 {
                return Err(Error::Eof);
            }
            let class_hash = u32::from_le_bytes(self.input[0..4].try_into().unwrap());
            println!("👻 准备反序列化 Enum，偷看到的类型哈希为: {:x}", class_hash);

            // `serde` 的流程是，我们告诉它变体的名字，然后它会继续处理。
            // 我们通过计算 `variants` (serde 传给我们的所有变体名，如 "VfxPrimitiveMesh") 的哈希，
            // 来找到和 `class_hash` 匹配的那个。
            let (variant_index, _variant_name) = variants
                .iter()
                .enumerate()
                .find(|(_i, name)| compute_binhash(name) == class_hash)
                .ok_or_else(|| {
                    Error::Message(format!("未知的 Enum 变体哈希: 0x{:x}", class_hash))
                })?;

            // 使用 EnumReader 作为访问器，将变体索引传递给 visitor
            visitor.visit_enum(EnumReader {
                de: self,
                variant_index: variant_index as u32,
            })
        }

        serde::forward_to_deserialize_any! {

          bool i8 i16 i32 i64 u8 u32 u64 f32 f64 char str bytes

          byte_buf option unit unit_struct tuple

          tuple_struct map identifier ignored_any

        }
    }

    // --- Seq (Vec) 读取器 ---

    struct SeqReader<'a, 'de: 'a> {
        de: &'a mut BinDeserializer<'de>,

        vtype: BinType,

        count: usize,
    }

    impl<'de, 'a> SeqAccess<'de> for SeqReader<'a, 'de> {
        type Error = Error;

        fn next_element_seed<T: de::DeserializeSeed<'de>>(
            &mut self,

            seed: T,
        ) -> Result<Option<T::Value>, Self::Error> {
            if self.count == 0 {
                return Ok(None);
            }

            self.count -= 1;

            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    // --- Struct 读取器 ---
    struct MapReader<'de> {
        data_map: HashMap<u32, (BinType, &'de [u8])>,
        struct_fields: std::slice::Iter<'static, &'static str>,
        next_value: Option<(BinType, &'de [u8])>,
    }

    impl<'de> MapAccess<'de> for MapReader<'de> {
        type Error = Error;

        fn next_key_seed<K: de::DeserializeSeed<'de>>(
            &mut self,

            seed: K,
        ) -> Result<Option<K::Value>, Self::Error> {
            while let Some(field_name) = self.struct_fields.next() {
                let hash = compute_binhash(field_name);

                if let Some((vtype, value_slice)) = self.data_map.remove(&hash) {
                    self.next_value = Some((vtype, value_slice));
                    println!("🐕 获取映射键: {:?}", field_name);

                    return seed.deserialize(field_name.into_deserializer()).map(Some);
                }
                println!("🐎 没找着 {}", field_name);
            }

            Ok(None)
        }

        fn next_value_seed<V: de::DeserializeSeed<'de>>(
            &mut self,

            seed: V,
        ) -> Result<V::Value, Self::Error> {
            let (vtype, value_slice) = self
                .next_value
                .take()
                .expect("next_value_seed called without key");

            println!("🐕 获取映射值: {:?} 长度: {}", vtype, value_slice.len());

            let mut value_de = BinDeserializer::from_bytes(value_slice, false);

            // if !value_de.input.is_empty() {
            //     return Err(Error::Message(format!(
            //         "还有 {} 字节没被消耗",
            //         value_de.input.len()
            //     )));
            // }

            seed.deserialize(&mut value_de)
        }
    }

    /// 辅助结构体，用于实现 serde::de::EnumAccess
    struct EnumReader<'a, 'de: 'a> {
        de: &'a mut BinDeserializer<'de>,
        variant_index: u32,
    }

    impl<'de, 'a> EnumAccess<'de> for EnumReader<'a, 'de> {
        type Error = Error;
        type Variant = VariantReader<'a, 'de>; // 下一步的访问器

        fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where
            V: de::DeserializeSeed<'de>,
        {
            // 将我们之前找到的变体索引反序列化，这样 serde 就能知道是哪个变体了。
            let variant = seed.deserialize(self.variant_index.into_deserializer())?;
            // 返回变体的值和下一步的访问器
            Ok((variant, VariantReader { de: self.de }))
        }
    }

    /// 辅助结构体，用于实现 serde::de::VariantAccess
    struct VariantReader<'a, 'de: 'a> {
        de: &'a mut BinDeserializer<'de>,
    }
    impl<'de, 'a> VariantAccess<'de> for VariantReader<'a, 'de> {
        type Error = Error;

        /// 处理单元变体, 例如 `VfxPrimitiveArbitraryQuad`
        fn unit_variant(self) -> Result<(), Self::Error> {
            println!("📦 正在解析 Unit 变体 (例如 VfxPrimitiveArbitraryQuad)");
            // 在我们的二进制格式中, 一个单元变体对应一个包含 0 个字段的结构体。
            // 我们必须完整地消耗掉它的头部信息。
            let _class_hash = self.de.read_bytes(4)?;
            let _fields_len = self.de.read_bytes(4)?;
            let field_count = u16::from_le_bytes(self.de.read_bytes(2)?.try_into().unwrap());

            // 确认字段数确实为 0
            if field_count == 0 {
                Ok(())
            } else {
                Err(Error::Message(format!(
                    "期望 Unit 变体 (0 个字段)，但文件中记录了 {} 个字段",
                    field_count
                )))
            }
        }

        /// 处理结构体变体, 例如 `VfxPrimitiveMesh { ... }`
        fn struct_variant<V>(
            self,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            println!("🏗️ 正在解析 Struct 变体 (例如 VfxPrimitiveMesh)");
            // 对于结构体变体，serde 希望我们像解析一个普通 struct 那样进行处理。
            // 我们可以直接将这个请求转发给我们的 `deserialize_struct` 方法。
            // `deserialize_struct` 会读取类型哈希、字段数，并构建字段哈希图，
            // 然后 `visitor` 会正确地访问并填充 `m_mesh` 字段。
            self.de
                .deserialize_struct("VfxPrimitiveMesh", fields, visitor)
        }

        // 下面这两种变体我们没有用到，所以返回错误即可
        fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            Err(Error::Message("不支持 Newtype 变体".into()))
        }

        fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(Error::Message("不支持 Tuple 变体".into()))
        }
    }
    // --- 跳过数据的辅助函数 ---

    impl<'de> BinDeserializer<'de> {
        fn skip_value(&mut self, vtype: BinType) -> Result<(), Error> {
            use std::mem::size_of;

            match vtype {
                BinType::None => {
                    self.read_bytes(6)?;
                }
                BinType::Bool | BinType::S8 | BinType::U8 | BinType::Flag => {
                    self.read_bytes(1)?;
                }
                BinType::S16 | BinType::U16 => {
                    self.read_bytes(2)?;
                }
                BinType::S32 | BinType::U32 | BinType::Float | BinType::Hash | BinType::Link => {
                    self.read_bytes(4)?;
                }
                BinType::S64 | BinType::U64 | BinType::Path => {
                    self.read_bytes(8)?;
                }
                BinType::Vec2 => {
                    self.read_bytes(size_of::<f32>() * 2)?;
                }
                BinType::Vec3 => {
                    self.read_bytes(size_of::<f32>() * 3)?;
                }
                BinType::Vec4 => {
                    self.read_bytes(size_of::<f32>() * 4)?;
                }
                BinType::Color => {
                    self.read_bytes(4)?;
                }
                BinType::Matrix => {
                    self.read_bytes(size_of::<f32>() * 16)?;
                }
                BinType::String => {
                    let len = u16::from_le_bytes(self.read_bytes(2)?.try_into().unwrap());

                    let str = String::from_utf8_lossy(self.read_bytes(len as usize)?);
                }
                BinType::List | BinType::List2 => {
                    let el_vtype = self.read_bintype()?;

                    self.read_bytes(4)?; // padding

                    let count = u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap());

                    for _ in 0..count {
                        self.skip_value(el_vtype)?;
                    }
                } // ==================== 最终修正点 ====================
                BinType::Struct | BinType::Embed => {
                    // 1. 读取 4 字节的 class_hash
                    let class_hash = u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap());

                    // 2. 检查是否为 null struct (class_hash 为 0)
                    if class_hash != 0 {
                        // 3. 读取字段总长度 (这和 deserialize_struct 行为一致)
                        let fields_total_len =
                            u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap());

                        // 4. 读取并消耗掉 field_count (这和 deserialize_struct 行为一致)
                        let _field_count = self.read_bytes(2)?; // 消耗 u16

                        // 5. fields_total_len 包含了 field_count(2字节) 和 后续字段数据。
                        //    因为我们已经手动读取了 field_count，所以只需要跳过剩下的部分。
                        self.read_bytes((fields_total_len - 2) as usize)?;
                    }
                } // ====================================================
                BinType::Option => {
                    // 添加了 Option 的处理
                    let el_vtype = self.read_bintype()?;
                    let count = u8::from_le_bytes(self.read_bytes(1)?.try_into().unwrap());
                    if count == 1 {
                        self.skip_value(el_vtype)?;
                    }
                }
                BinType::Map => {
                    // 添加了 Map 的处理
                    let ktype = self.read_bintype()?;
                    let vtype = self.read_bintype()?;

                    self.read_bytes(4)?; // padding

                    let count = u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap());

                    for _ in 0..count {
                        self.skip_value(ktype)?;
                        self.skip_value(vtype)?;
                    }
                }
            }

            Ok(())
        }
    }
}
