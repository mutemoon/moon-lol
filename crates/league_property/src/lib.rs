mod accessor;
mod cycle;
mod deserializer;
mod extract;
mod parser;
mod prop;
mod types;

pub use accessor::*;
pub use cycle::*;
pub use deserializer::*;
pub use extract::*;
pub use parser::*;
pub use prop::*;
pub use types::*;

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
