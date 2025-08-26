mod classes;
mod deserializer;
mod prop;

pub use classes::*;
pub use deserializer::*;
pub use prop::*;

use serde::Deserialize;

pub fn from_entry<'de, T>(slice: &'de EntryData) -> T
where
    T: Deserialize<'de>,
{
    let mut deserializer = BinDeserializer::from_bytes(&slice.data, BinType::Entry);
    T::deserialize(&mut deserializer).unwrap()
}
