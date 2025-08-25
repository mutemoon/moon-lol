use std::collections::HashMap;

use serde::de::{self, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::Deserializer;

use crate::league::{
    BinDeserializer, BinDeserializerError, BinDeserializerResult, BinType, LeagueLoader,
};

pub struct SeqReader<'a, 'de: 'a> {
    pub de: &'a mut BinDeserializer<'de>,
    pub count: usize,
}

impl<'de, 'a> SeqAccess<'de> for SeqReader<'a, 'de> {
    type Error = BinDeserializerError;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> BinDeserializerResult<Option<T::Value>> {
        if self.count == 0 {
            return Ok(None);
        }

        self.count -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

pub struct MapReader<'de> {
    pub data_map: HashMap<u32, (BinType, &'de [u8])>,
    pub struct_fields: std::slice::Iter<'static, &'static str>,
    pub next_value: Option<(BinType, &'de [u8])>,
}

impl<'de> MapAccess<'de> for MapReader<'de> {
    type Error = BinDeserializerError;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> BinDeserializerResult<Option<K::Value>> {
        while let Some(field_name) = self.struct_fields.next() {
            let hash = if field_name.starts_with("unk") {
                u32::from_str_radix(&field_name[5..], 16).unwrap()
            } else {
                LeagueLoader::hash_bin(field_name)
            };

            if let Some((vtype, value_slice)) = self.data_map.remove(&hash) {
                self.next_value = Some((vtype, value_slice));

                return seed.deserialize(field_name.into_deserializer()).map(Some);
            };
        }

        Ok(None)
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> BinDeserializerResult<V::Value> {
        let (vtype, value_slice) = self.next_value.unwrap();

        let mut value_de = BinDeserializer::from_bytes(value_slice, vtype);

        seed.deserialize(&mut value_de)
    }
}

pub struct HashMapReader<'a, 'de: 'a> {
    pub de: &'a mut BinDeserializer<'de>,
    pub ktype: BinType,
    pub vtype: BinType,
    pub count: u32,
}

impl<'de, 'a> MapAccess<'de> for HashMapReader<'a, 'de> {
    type Error = BinDeserializerError;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> BinDeserializerResult<Option<K::Value>> {
        // 如果 count 为 0，说明 map 的所有条目都已读取完毕
        if self.count == 0 {
            return Ok(None);
        }

        // 使用 seed 来反序列化 key
        self.de.value_type = self.ktype;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> BinDeserializerResult<V::Value> {
        // 临时设置 deserializer 要解析的类型为 value 的类型
        self.de.value_type = self.vtype;

        let value = seed.deserialize(&mut *self.de)?;

        // 一个完整的键值对已经读取完毕，将计数器减 1
        self.count -= 1;

        Ok(value)
    }
}

pub struct EnumReader<'a, 'de: 'a> {
    pub de: &'a mut BinDeserializer<'de>,
    pub variant_index: u32,
}

impl<'de, 'a> EnumAccess<'de> for EnumReader<'a, 'de> {
    type Error = BinDeserializerError;
    type Variant = VariantReader<'a, 'de>;

    fn variant_seed<V>(self, seed: V) -> BinDeserializerResult<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(self.variant_index.into_deserializer())?;

        Ok((variant, VariantReader { de: self.de }))
    }
}

pub struct VariantReader<'a, 'de: 'a> {
    de: &'a mut BinDeserializer<'de>,
}

impl<'de, 'a> VariantAccess<'de> for VariantReader<'a, 'de> {
    type Error = BinDeserializerError;

    fn unit_variant(self) -> BinDeserializerResult<()> {
        self.de.parser.skip_value(BinType::Struct)?;

        return Ok(());
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> BinDeserializerResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de.deserialize_struct("", fields, visitor)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> BinDeserializerResult<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> BinDeserializerResult<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(BinDeserializerError::Message("不支持 Tuple 变体".into()))
    }
}
