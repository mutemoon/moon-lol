use std::collections::{HashMap, VecDeque};

use serde::de::{self, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::Deserializer;

use league_utils::hash_bin;

use crate::{BinDeserializer, BinDeserializerResult, BinType, Error};

pub struct SeqIntoReader<E> {
    pub values: VecDeque<E>,
}

impl<E> SeqIntoReader<E> {
    pub fn from_values(values: Vec<E>) -> Self {
        Self {
            values: VecDeque::from(values),
        }
    }
}

impl<'de, E: IntoDeserializer<'de, Error> + Copy> SeqAccess<'de> for SeqIntoReader<E> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> BinDeserializerResult<Option<T::Value>> {
        let Some(value) = self.values.pop_front() else {
            return Ok(None);
        };

        seed.deserialize(value.into_deserializer()).map(Some)
    }
}

pub struct SeqDerReader<'de> {
    pub values: VecDeque<&'de [u8]>,
    pub vtype: BinType,
    pub current_index: usize,
}

impl<'de> SeqDerReader<'de> {
    pub fn from_values(values: Vec<&'de [u8]>, vtype: BinType) -> Self {
        Self {
            values: VecDeque::from(values),
            vtype,
            current_index: 0,
        }
    }
}

impl<'de> SeqAccess<'de> for SeqDerReader<'de> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> BinDeserializerResult<Option<T::Value>> {
        let Some(value) = self.values.pop_front() else {
            return Ok(None);
        };

        let index = self.current_index;
        self.current_index += 1;

        seed.deserialize(&mut BinDeserializer::from_bytes(value, self.vtype))
            .map(Some)
            .map_err(|e| e.with_context(format!("列表[{}]", index)))
    }
}

pub struct MapReader<'de> {
    pub data_map: HashMap<u32, (BinType, &'de [u8])>,
    pub struct_fields: std::slice::Iter<'static, &'static str>,
    pub next_value: Option<(BinType, &'de [u8])>,
    pub current_field: Option<&'static str>,
}

impl<'de> MapAccess<'de> for MapReader<'de> {
    type Error = Error;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> BinDeserializerResult<Option<K::Value>> {
        while let Some(field_name) = self.struct_fields.next() {
            let hash = if field_name.starts_with("unk") {
                u32::from_str_radix(&field_name[5..], 16).unwrap()
            } else {
                hash_bin(field_name)
            };

            if let Some((vtype, value_slice)) = self.data_map.remove(&hash) {
                self.next_value = Some((vtype, value_slice));
                self.current_field = Some(field_name);

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
        let field_name = self.current_field.unwrap_or("<unknown>");

        let mut value_de = BinDeserializer::from_bytes(value_slice, vtype);

        seed.deserialize(&mut value_de)
            .map_err(|e| e.with_context(format!("字段 \"{}\" (类型: {:?})", field_name, vtype)))
    }
}

pub struct HashMapReader<'de> {
    pub map: VecDeque<(&'de [u8], &'de [u8])>,
    pub ktype: BinType,
    pub vtype: BinType,
}

impl<'de> MapAccess<'de> for HashMapReader<'de> {
    type Error = Error;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> BinDeserializerResult<Option<K::Value>> {
        let Some((key, _)) = self.map.get(0) else {
            return Ok(None);
        };

        seed.deserialize(&mut BinDeserializer::from_bytes(key, self.ktype))
            .map(Some)
            .map_err(|e| e.with_context("Map 键"))
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> BinDeserializerResult<V::Value> {
        let (_, value) = self.map.pop_front().unwrap();

        seed.deserialize(&mut BinDeserializer::from_bytes(value, self.vtype))
            .map_err(|e| e.with_context("Map 值"))
    }
}

pub struct EnumReader<'a, 'de: 'a> {
    pub de: &'a mut BinDeserializer<'de>,
    pub variant_index: u32,
}

impl<'de, 'a> EnumAccess<'de> for EnumReader<'a, 'de> {
    type Error = Error;
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
    type Error = Error;

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
        Err(Error::Message("不支持 Tuple 变体".into()))
    }
}
