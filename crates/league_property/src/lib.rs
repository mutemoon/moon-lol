pub mod accessor;
pub mod cycle;
pub mod deserializer;
pub mod extract;
pub mod parser;
pub mod prop;
pub mod types;
use serde::Deserialize;

pub fn from_entry_unwrap<'de, T>(slice: &'de EntryData) -> T
where
    T: Deserialize<'de>,
{
    let mut deserializer = BinDeserializer::from_bytes(&slice.data, BinType::Entry);
    T::deserialize(&mut deserializer).unwrap()
}

pub fn from_entry<'de, T>(slice: &'de EntryData) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let mut deserializer = BinDeserializer::from_bytes(&slice.data, BinType::Entry);
    T::deserialize(&mut deserializer)
}
