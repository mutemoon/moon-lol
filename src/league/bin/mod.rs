mod character;
mod deserializer;
mod materials;
mod prop;

pub use character::*;
pub use deserializer::*;
pub use materials::*;
pub use prop::*;

use serde::Deserialize;

pub fn from_entry<'de, T>(slice: &'de EntryData) -> T
where
    T: Deserialize<'de>,
{
    let mut deserializer = BinDeserializer::from_bytes(&slice.data, BinType::Entry);
    T::deserialize(&mut deserializer).unwrap()
}
