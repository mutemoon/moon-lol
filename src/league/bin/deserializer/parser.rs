use std::collections::HashMap;

use bevy::math::Vec2;

use crate::league::{BinDeserializerError, BinDeserializerResult, BinStructHeader, BinType};

#[derive(Clone)]
pub struct BinParser<'de> {
    pub input: &'de [u8],
}

impl<'de> BinParser<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        BinParser { input }
    }

    pub fn read_bytes(&mut self, len: usize) -> BinDeserializerResult<&'de [u8]> {
        if self.input.len() < len {
            return Err(BinDeserializerError::Message(format!(
                "读取字节失败，长度不足: {} < {}",
                self.input.len(),
                len
            )));
        }

        let (slice, rest) = self.input.split_at(len);

        self.input = rest;

        Ok(slice)
    }

    pub fn read_type(&mut self) -> BinDeserializerResult<BinType> {
        BinType::try_from(u8::from_le_bytes(
            self.read_bytes(1).unwrap().try_into().unwrap(),
        ))
    }

    pub fn read_string(&mut self) -> BinDeserializerResult<String> {
        let len = self.read_u16().unwrap() as usize;

        let s = String::from_utf8(self.read_bytes(len).unwrap().to_vec())
            .map_err(|e| BinDeserializerError::Message(e.to_string()))
            .unwrap();

        Ok(s.to_owned())
    }

    pub fn read_hash(&mut self) -> BinDeserializerResult<u32> {
        self.read_u32()
    }

    pub fn read_link(&mut self) -> BinDeserializerResult<u32> {
        self.read_u32()
    }

    pub fn read_bool(&mut self) -> BinDeserializerResult<bool> {
        Ok(self.read_bytes(1).unwrap()[0] != 0)
    }

    pub fn read_flag(&mut self) -> BinDeserializerResult<bool> {
        self.read_bool()
    }

    pub fn read_u8(&mut self) -> BinDeserializerResult<u8> {
        Ok(u8::from_le_bytes(
            self.read_bytes(1).unwrap().try_into().unwrap(),
        ))
    }

    pub fn read_i8(&mut self) -> BinDeserializerResult<i8> {
        Ok(i8::from_le_bytes(
            self.read_bytes(1).unwrap().try_into().unwrap(),
        ))
    }

    pub fn read_s16(&mut self) -> BinDeserializerResult<i16> {
        Ok(i16::from_le_bytes(
            self.read_bytes(2).unwrap().try_into().unwrap(),
        ))
    }

    pub fn read_u16(&mut self) -> BinDeserializerResult<u16> {
        Ok(u16::from_le_bytes(
            self.read_bytes(2).unwrap().try_into().unwrap(),
        ))
    }

    pub fn read_u32(&mut self) -> BinDeserializerResult<u32> {
        Ok(u32::from_le_bytes(
            self.read_bytes(4).unwrap().try_into().unwrap(),
        ))
    }

    pub fn read_usize(&mut self) -> BinDeserializerResult<usize> {
        Ok(self.read_u32().unwrap() as usize)
    }

    pub fn read_s32(&mut self) -> BinDeserializerResult<i32> {
        Ok(i32::from_le_bytes(
            self.read_bytes(4).unwrap().try_into().unwrap(),
        ))
    }

    pub fn read_f32(&mut self) -> BinDeserializerResult<f32> {
        Ok(f32::from_le_bytes(
            self.read_bytes(4).unwrap().try_into().unwrap(),
        ))
    }

    pub fn read_u64(&mut self) -> BinDeserializerResult<u64> {
        Ok(u64::from_le_bytes(
            self.read_bytes(8).unwrap().try_into().unwrap(),
        ))
    }

    pub fn read_s64(&mut self) -> BinDeserializerResult<i64> {
        Ok(i64::from_le_bytes(
            self.read_bytes(8).unwrap().try_into().unwrap(),
        ))
    }

    pub fn read_vec2(&mut self) -> BinDeserializerResult<Vec2> {
        Ok(Vec2::new(
            self.read_f32().unwrap(),
            self.read_f32().unwrap(),
        ))
    }

    pub fn read_struct_header(&mut self) -> BinDeserializerResult<Option<BinStructHeader>> {
        let class_hash = self.read_hash().unwrap();

        if class_hash == 0 {
            Ok(None)
        } else {
            Ok(Some(BinStructHeader {
                class_hash,
                bytes_count: self.read_usize().unwrap(),
            }))
        }
    }

    pub fn read_fields(&mut self) -> BinDeserializerResult<HashMap<u32, (BinType, &'de [u8])>> {
        let field_count = self.read_u16().unwrap() as usize;

        let mut fields = HashMap::new();

        for _ in 0..field_count {
            let hash = self.read_hash().unwrap();

            let vtype = self.read_type().unwrap();

            fields.insert(hash, (vtype, self.skip_value(vtype).unwrap()));
        }

        Ok(fields)
    }

    pub fn read_list(&mut self, vtype: BinType) -> BinDeserializerResult<Vec<&'de [u8]>> {
        let list_count = self.read_u32().unwrap();

        let mut list = Vec::new();

        for _ in 0..list_count {
            list.push(self.skip_value(vtype).unwrap());
        }

        Ok(list)
    }

    pub fn skip_value(&mut self, vtype: BinType) -> BinDeserializerResult<&'de [u8]> {
        // 核心逻辑：先计算总长度，再一次性切片
        let total_len = Self::calculate_value_len(self.input, vtype).unwrap();
        self.read_bytes(total_len)
    }

    pub fn calculate_value_len(input: &[u8], vtype: BinType) -> BinDeserializerResult<usize> {
        // 检查切片长度的辅助宏
        macro_rules! ensure_len {
            ($len:expr) => {
                if input.len() < $len {
                    return Err(BinDeserializerError::Message(format!(
                        "计算长度失败，类型 {:?} 需要 {} 字节，但只剩下 {}",
                        vtype,
                        $len,
                        input.len()
                    )));
                }
            };
        }

        // 将切片安全转换为数组的辅助函数
        fn slice_to_array<const N: usize>(slice: &[u8]) -> BinDeserializerResult<[u8; N]> {
            slice.try_into().map_err(|_| {
                BinDeserializerError::Message(format!(
                    "无法将长度为 {} 的切片转换为数组 [u8; {}]",
                    slice.len(),
                    N
                ))
            })
        }

        match vtype {
            // --- 固定长度类型 ---
            BinType::None => Ok(6),
            BinType::Bool | BinType::S8 | BinType::U8 | BinType::Flag => Ok(1),
            BinType::S16 | BinType::U16 => Ok(2),
            BinType::S32 | BinType::U32 | BinType::Float | BinType::Hash | BinType::Link => Ok(4),
            BinType::S64 | BinType::U64 | BinType::Path => Ok(8),
            BinType::Vec2 => Ok(size_of::<f32>() * 2),
            BinType::Vec3 => Ok(size_of::<f32>() * 3),
            BinType::Vec4 => Ok(size_of::<f32>() * 4),
            BinType::Color => Ok(4),
            BinType::Matrix => Ok(size_of::<f32>() * 16),

            // --- 动态长度类型 ---
            BinType::String => {
                ensure_len!(2);
                let len_bytes = slice_to_array(&input[..2]).unwrap();
                let data_len = u16::from_le_bytes(len_bytes) as usize;

                Ok(2 + data_len)
            }
            BinType::List | BinType::List2 => {
                // 结构: [type: 1] + [count: 4] + [data: count]
                ensure_len!(5);
                let len_bytes = slice_to_array(&input[1..5]).unwrap();
                let data_len = u32::from_le_bytes(len_bytes) as usize;

                Ok(1 + 4 + data_len)
            }
            BinType::Struct | BinType::Embed => {
                // 结构: [hash: 4] + (如果 hash != 0 => [count: 4] + [data: count])
                ensure_len!(4);
                let hash_bytes = slice_to_array(&input[..4]).unwrap();
                let class_hash = u32::from_le_bytes(hash_bytes);

                if class_hash == 0 {
                    Ok(4)
                } else {
                    ensure_len!(8);
                    let len_bytes = slice_to_array(&input[4..8]).unwrap();
                    let data_len = u32::from_le_bytes(len_bytes) as usize;

                    Ok(4 + 4 + data_len)
                }
            }

            // --- 递归/嵌套类型 ---
            BinType::Option => {
                // 结构: [inner_type: 1] + [is_some: 1] + (如果 is_some != 0 => [data])
                ensure_len!(2);
                let vtype_byte = input[0];
                let some_byte = input[1];

                if some_byte == 0 {
                    Ok(2) // None, 只有两个字节
                } else {
                    let inner_vtype = BinType::try_from(vtype_byte).unwrap();
                    // 在剩余的 input 上递归计算内部值的长度
                    let inner_len = Self::calculate_value_len(&input[2..], inner_vtype).unwrap();

                    Ok(2 + inner_len)
                }
            }
            BinType::Map => {
                // 结构: [key_type: 1] + [value_type: 1] + [count: 4] + [data: count]
                ensure_len!(6);
                let len_bytes = slice_to_array(&input[2..6]).unwrap();
                let data_len = u32::from_le_bytes(len_bytes) as usize;

                Ok(1 + 1 + 4 + data_len)
            }

            // Entry 不应在流中独立存在
            BinType::Entry => unreachable!(),
        }
    }
}
