mod accessor;
mod cycle;
mod deserializer;
mod extract;
mod parser;
pub mod prop;
mod types;

use serde::Deserialize;

use crate::deserializer::BinDeserializer;
use crate::types::{BinType, Error};

pub fn from_entry_unwrap<'de, T>(slice: &'de prop::EntryData) -> T
where
    T: Deserialize<'de>,
{
    let mut deserializer = BinDeserializer::from_bytes(&slice.data, BinType::Entry);
    T::deserialize(&mut deserializer).unwrap()
}

pub fn from_entry<'de, T>(slice: &'de prop::EntryData) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let mut deserializer = BinDeserializer::from_bytes(&slice.data, BinType::Entry);
    T::deserialize(&mut deserializer)
}
