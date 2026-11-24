pub mod accessor;
pub mod cycle;
pub mod deserializer;
pub mod extract;
pub mod parser;
pub mod prop;
pub mod types;

use serde::Deserialize;

/// 反序列化一个顶层 entry，`class_hash` 为该 entry 所属的类。
pub fn from_entry<'de, T>(class_hash: u32, slice: &'de prop::EntryData) -> Result<T, types::Error>
where
    T: Deserialize<'de>,
{
    let mut deserializer = deserializer::BinDeserializer::from_entry_bytes(&slice.data, class_hash);
    T::deserialize(&mut deserializer)
}
