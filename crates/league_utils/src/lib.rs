use std::collections::HashMap;
use std::hash::Hasher;
use std::marker::PhantomData;

use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use heck::{ToPascalCase, ToSnakeCase};
use nom::number::complete::le_f32;
use nom::IResult;
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

pub fn nom_parse_vec3(input: &[u8]) -> IResult<&[u8], Vec3> {
    let (i, x) = le_f32(input)?;
    let (i, y) = le_f32(i)?;
    let (i, z) = le_f32(i)?;
    Ok((i, Vec3::new(x, y, z)))
}

pub fn nom_parse_quat(input: &[u8]) -> IResult<&[u8], Quat> {
    let (i, x) = le_f32(input)?;
    let (i, y) = le_f32(i)?;
    let (i, z) = le_f32(i)?;
    let (i, w) = le_f32(i)?;
    Ok((i, Quat::from_xyzw(x, y, z, w)))
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, min) = nom_parse_vec3(input)?;
        let (i, max) = nom_parse_vec3(i)?;
        Ok((i, BoundingBox { min, max }))
    }
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

pub fn get_shader_uuid_by_hash(path: &str, hash: u64) -> Uuid {
    Uuid::from_u128(hash_shader(&format!("{path}#{hash}")) as u128)
}

pub fn get_shader_handle_by_hash(path: &str, hash: u64) -> Handle<Shader> {
    Handle::Uuid(get_shader_uuid_by_hash(path, hash), PhantomData)
}

pub fn get_shader_handle(path: &str, defs: &Vec<String>) -> Handle<Shader> {
    get_shader_handle_by_hash(path, hash_shader_spec(defs))
}

pub fn get_extension_by_bytes(bytes: &[u8]) -> &str {
    if bytes.len() >= 8 {
        match &bytes[..8] {
            b"r3d2Mesh" => return "scb",
            b"r3d2sklt" => return "skl",
            b"r3d2anmd" => return "anm",
            b"r3d2canm" => return "anm",
            _ => {}
        };

        if &bytes[..4] == b"r3d2" {
            if u32::from_le_bytes(bytes[4..8].try_into().unwrap()) == 1 {
                return "wpk";
            }
        }

        if u32::from_le_bytes(bytes[4..8].try_into().unwrap()) == 0x22FD4FC3 {
            return "skl";
        }
    }

    if bytes.len() >= 7 {
        if &bytes[..7] == b"PreLoad" {
            return "preload";
        }
    }

    if bytes.len() >= 5 {
        if &bytes[1..5] == b"LuaQ" {
            return "luaobj";
        }
    }

    if bytes.len() >= 4 {
        match &bytes[..4] {
            b"DDS " => return "dds",
            b"PROP" => return "bin",
            b"BKHD" => return "bnk",
            b"WGEO" => return "wgeo",
            b"OEGM" => return "mapgeo",
            b"[Obj" => return "sco",
            b"PTCH" => return "bin",
            b"TEX\0" => return "tex",
            _ => {}
        }

        if &bytes[1..4] == b"PNG" {
            return "png";
        }

        let magic = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        match magic {
            0x00112233 => return "skn",
            0x3 => return "dat",
            _ => {}
        }
    }

    if bytes.len() >= 3 {
        if &bytes[..3] == b"RST" {
            return "stringtable";
        }
        if bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
            return "jpg";
        }
    }

    "unk"
}
