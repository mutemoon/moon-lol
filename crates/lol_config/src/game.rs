use std::fmt::Formatter;

use serde::Deserialize;

use bevy::{prelude::*, reflect::TypeRegistry, scene::serde::SceneMapDeserializer};
use serde::{
    de::{DeserializeSeed, Visitor},
    Deserializer,
};

#[derive(Resource, Default)]
pub struct ConfigGame {
    pub legends: Vec<(Entity, String)>,
}

type ConfigLegend = (ConfigCharacter, Vec<Box<dyn PartialReflect>>);

#[derive(Deserialize, Debug)]
pub struct ConfigCharacter {
    pub skin_path: String,
}

pub struct CharacterConfigsDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for CharacterConfigsDeserializer<'a> {
    type Value = Vec<ConfigLegend>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(CharacterConfigsVisitor {
            type_registry: self.type_registry,
        })
    }
}

struct CharacterConfigsVisitor<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for CharacterConfigsVisitor<'a> {
    type Value = Vec<ConfigLegend>;

    fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
        formatter.write_str("list of character configs")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut configs = Vec::new();

        // 循环读取序列中的所有元素
        while let Some(intermediate) = seq.next_element::<ConfigCharacter>()? {
            // 读取 config 后，必须紧跟着一个组件 map
            let components = seq
                .next_element_seed(SceneMapDeserializer {
                    registry: self.type_registry,
                })?
                .ok_or_else(|| serde::de::Error::custom("expected component map after config"))?;

            configs.push((intermediate, components));
        }

        Ok(configs)
    }
}
