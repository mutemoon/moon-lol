pub mod accessor;
pub mod cycle;
pub mod deserializer;
pub mod extract;
pub mod parser;
pub mod prop;
pub mod types;

use serde::Deserialize;

pub fn from_entry_unwrap<'de, T>(slice: &'de prop::EntryData) -> T
where
    T: Deserialize<'de>,
{
    let mut deserializer =
        deserializer::BinDeserializer::from_bytes(&slice.data, types::BinType::Entry);
    T::deserialize(&mut deserializer).unwrap()
}

pub fn from_entry<'de, T>(slice: &'de prop::EntryData) -> Result<T, types::Error>
where
    T: Deserialize<'de>,
{
    let mut deserializer =
        deserializer::BinDeserializer::from_bytes(&slice.data, types::BinType::Entry);
    T::deserialize(&mut deserializer)
}
