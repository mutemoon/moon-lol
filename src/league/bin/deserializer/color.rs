use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

pub struct BinColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

struct ColorVisitor;

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = BinColor;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a u32 representing a Color variant")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let r = seq.next_element().unwrap().unwrap();
        let g = seq.next_element().unwrap().unwrap();
        let b = seq.next_element().unwrap().unwrap();
        let a = seq.next_element().unwrap().unwrap();

        Ok(BinColor { r, g, b, a })
    }
}

impl<'de> Deserialize<'de> for BinColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ColorVisitor)
    }
}
