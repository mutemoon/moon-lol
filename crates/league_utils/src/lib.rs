use std::collections::HashMap;
use std::hash::Hasher;

use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use binrw::binread;
use heck::{ToPascalCase, ToSnakeCase};
use serde::{Deserialize, Serialize};
use twox_hash::XxHash64;

pub fn parse_vec3(v: [f32; 3]) -> Vec3 {
    Vec3::new(v[0], v[1], v[2])
}

pub fn parse_vec3_array(v: Vec<[f32; 3]>) -> Vec<Vec3> {
    v.into_iter().map(parse_vec3).collect()
}

pub fn parse_quat(v: [f32; 4]) -> Quat {
    Quat::from_xyzw(v[0], v[1], v[2], v[3])
}

pub fn parse_quat_array(v: Vec<[f32; 4]>) -> Vec<Quat> {
    v.into_iter().map(parse_quat).collect()
}

pub fn hash_wad(s: &str) -> u64 {
    let mut h = XxHash64::with_seed(0);
    h.write(s.to_ascii_lowercase().as_bytes());
    h.finish()
}

pub fn hash_shader(s: &str) -> u64 {
    let mut h = XxHash64::with_seed(0);
    h.write(s.as_bytes());
    h.finish()
}

pub fn hash_bin(s: &str) -> u32 {
    s.to_ascii_lowercase().bytes().fold(0x811c9dc5_u32, |h, b| {
        (h ^ b as u32).wrapping_mul(0x01000193)
    })
}

pub fn hash_joint(s: &str) -> u32 {
    let mut hash = 0u32;
    for b in s.to_ascii_lowercase().bytes() {
        hash = (hash << 4) + (b as u32);
        let high = hash & 0xf0000000;
        if high != 0 {
            hash ^= high >> 24;
        }
        hash &= !high;
    }
    hash
}

pub fn hash_shader_spec(defs: &Vec<String>) -> u64 {
    let mut defs = defs.clone();

    defs.sort();

    let define_string = defs
        .iter()
        .map(|v| format!("{v}=1"))
        .collect::<Vec<_>>()
        .join("");

    hash_shader(&define_string)
}

#[binread]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[br(little)]
pub struct BoundingBox {
    #[br(map = Vec3::from_array)]
    pub min: Vec3,
    #[br(map = Vec3::from_array)]
    pub max: Vec3,
}

pub fn get_padded_string_64(bytes: [u8; 64]) -> String {
    String::from_utf8_lossy(&bytes)
        .trim_end_matches('\0')
        .to_string()
}

pub fn get_padded_string_128(bytes: [u8; 128]) -> String {
    String::from_utf8_lossy(&bytes)
        .trim_end_matches('\0')
        .to_string()
}

pub fn get_asset_id_by_path<A: Asset>(path: &str) -> AssetId<A> {
    AssetId::Uuid {
        uuid: Uuid::from_u128(hash_bin(path) as u128),
    }
}

pub fn get_asset_id_by_hash<A: Asset>(hash: u32) -> AssetId<A> {
    AssetId::Uuid {
        uuid: Uuid::from_u128(hash as u128),
    }
}

pub fn hash_to_type_name(hash: &u32, hash_to_string: &HashMap<u32, String>) -> String {
    hash_to_string
        .get(hash)
        .map(|s| {
            let pascal = s.to_pascal_case();
            match pascal.as_str() {
                "Self" => "MySelf".to_string(),
                _ => pascal,
            }
        })
        .unwrap_or_else(|| format!("Unk0x{:x}", hash))
}

pub fn hash_to_field_name(hash: &u32, hash_to_string: &HashMap<u32, String>) -> String {
    hash_to_string
        .get(hash)
        .map(|s| {
            let snake = s.to_snake_case();
            match snake.as_str() {
                "type" => "r#type".to_string(),
                "move" => "r#move".to_string(),
                "loop" => "r#loop".to_string(),
                "trait" => "r#trait".to_string(),
                "box" => "r#box".to_string(),
                _ => snake,
            }
        })
        .unwrap_or_else(|| format!("unk_0x{:x}", hash))
}

pub fn type_name_to_hash(type_name: &str) -> u32 {
    if type_name.starts_with("Unk0x") {
        u32::from_str_radix(&type_name[5..], 16).unwrap()
    } else {
        hash_bin(&type_name)
    }
}
