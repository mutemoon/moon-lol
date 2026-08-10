use std::collections::HashMap;
use std::sync::OnceLock;

use league_property::extract::get_hashes;
use league_property::parser::BinParser;
use league_property::prop::PropFile;
use league_property::types::BinType;
use serde::{Deserialize, Serialize};

static HASH_MAP: OnceLock<HashMap<u32, String>> = OnceLock::new();

pub fn get_global_hashes() -> &'static HashMap<u32, String> {
    HASH_MAP.get_or_init(|| {
        get_hashes(&[
            "assets/CommunityDragon-Data/hashes/lol/hashes.binentries.txt",
            "assets/CommunityDragon-Data/hashes/lol/hashes.binfields.txt",
            "assets/CommunityDragon-Data/hashes/lol/hashes.binhashes.txt",
            "assets/CommunityDragon-Data/hashes/lol/hashes.bintypes.txt",
        ])
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropRonDocument {
    pub file_type: String,
    pub version: u32,
    pub links: Vec<String>,
    pub entries: Vec<PropRonEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropRonEntry {
    pub entry_hash: String,
    pub class_name: String,
    pub fields: Vec<PropRonField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropRonField {
    pub name: String,
    pub type_decl: String,
    pub value: PropRonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropRonValue {
    Null,
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    Float(f32),
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Vec4(f32, f32, f32, f32),
    Matrix(Vec<f32>),
    Color(u8, u8, u8, u8),
    String(String),
    Hash(String),
    Path(String),
    Struct {
        class_name: String,
        fields: Vec<PropRonField>,
    },
    List(Vec<PropRonValue>),
    Option(Option<Box<PropRonValue>>),
    Map(Vec<(PropRonValue, PropRonValue)>),
}

fn bin_hash_to_str(hash: u32, hashes: &HashMap<u32, String>) -> String {
    hashes
        .get(&hash)
        .cloned()
        .unwrap_or_else(|| format!("0x{:08x}", hash))
}

fn game_hash_to_str(hash: u64) -> String {
    format!("0x{:016x}", hash)
}

fn bin_type_name(vtype: BinType) -> &'static str {
    match vtype {
        BinType::None => "none",
        BinType::Bool => "bool",
        BinType::S8 => "i8",
        BinType::U8 => "u8",
        BinType::S16 => "i16",
        BinType::U16 => "u16",
        BinType::S32 => "i32",
        BinType::U32 => "u32",
        BinType::S64 => "i64",
        BinType::U64 => "u64",
        BinType::Float => "f32",
        BinType::Vec2 => "vec2",
        BinType::Vec3 => "vec3",
        BinType::Vec4 => "vec4",
        BinType::Matrix => "mtx44",
        BinType::Color => "rgba",
        BinType::String => "string",
        BinType::Hash => "hash",
        BinType::Path => "file",
        BinType::List => "list",
        BinType::List2 => "list2",
        BinType::Struct => "pointer",
        BinType::Embed => "embed",
        BinType::Link => "link",
        BinType::Option => "option",
        BinType::Map => "map",
        BinType::Flag => "flag",
        BinType::Entry => "entry",
    }
}

fn type_decl(vtype: BinType, slice: &[u8]) -> String {
    let inner = |b: u8| BinType::try_from(b).map(bin_type_name).unwrap_or("unknown");
    match vtype {
        BinType::List if !slice.is_empty() => format!("list[{}]", inner(slice[0])),
        BinType::List2 if !slice.is_empty() => format!("list2[{}]", inner(slice[0])),
        BinType::Option if !slice.is_empty() => format!("option[{}]", inner(slice[0])),
        BinType::Map if slice.len() >= 2 => format!("map[{},{}]", inner(slice[0]), inner(slice[1])),
        _ => bin_type_name(vtype).to_string(),
    }
}

/// 解析 PROP 二进制字节或文本并转译为标准 RON 字符串
pub fn convert_prop_bytes_to_ron(bytes: &[u8]) -> Result<String, String> {
    let prop_bytes: &[u8] = if bytes.starts_with(b"PROP") {
        bytes
    } else if bytes.starts_with(b"PTCH") && bytes.len() > 12 && bytes[12..].starts_with(b"PROP") {
        &bytes[12..]
    } else {
        // 如果原本就是文本格式
        if let Ok(text) = std::str::from_utf8(bytes) {
            if text.starts_with("#PROP_text") || text.contains("type: string = \"PROP\"") {
                return Ok(text.to_string());
            }
        }
        return Err("传入的文件不是有效 PROP/PTCH 二进制或 PROP 文本文件".to_string());
    };

    let (_, prop) = PropFile::parse(prop_bytes).map_err(|e| format!("PROP 解析失败: {:?}", e))?;

    let hashes = get_global_hashes();

    let mut entries = Vec::new();
    for (class_hash, entry) in prop.iter_class_hash_and_entry() {
        let entry_hash = bin_hash_to_str(entry.hash, hashes);
        let class_name = bin_hash_to_str(class_hash, hashes);
        let fields = parse_fields(&entry.data, hashes)?;
        entries.push(PropRonEntry {
            entry_hash,
            class_name,
            fields,
        });
    }

    let doc = PropRonDocument {
        file_type: "PROP".to_string(),
        version: prop.version,
        links: prop.links.iter().map(|l| l.text.clone()).collect(),
        entries,
    };

    let pretty_config = ron::ser::PrettyConfig::default()
        .depth_limit(10)
        .separate_tuple_members(true)
        .enumerate_arrays(false);

    ron::ser::to_string_pretty(&doc, pretty_config).map_err(|e| format!("RON 序列化失败: {}", e))
}

pub fn get_global_game_hashes() -> &'static HashMap<u64, String> {
    static GAME_HASHES: OnceLock<HashMap<u64, String>> = OnceLock::new();
    GAME_HASHES.get_or_init(|| {
        let game_hash_paths: Vec<String> = (0..9)
            .map(|i| {
                format!(
                    "assets/CommunityDragon-Data/hashes/lol/hashes.game.txt.{}",
                    i
                )
            })
            .collect();
        let maps: Vec<HashMap<u64, String>> = game_hash_paths
            .into_iter()
            .map(|path| {
                let mut map = HashMap::new();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        if let Some((hash, name)) = line.split_once(' ') {
                            if let Ok(hash) = u64::from_str_radix(hash, 16) {
                                map.insert(hash, name.to_string());
                            }
                        }
                    }
                }
                map
            })
            .collect();
        let mut total = HashMap::new();
        for map in maps {
            total.extend(map);
        }
        total
    })
}

fn parse_fields(data: &[u8], hashes: &HashMap<u32, String>) -> Result<Vec<PropRonField>, String> {
    let mut parser = BinParser::from_bytes(data);
    let field_count = parser
        .read_u16()
        .map_err(|e| format!("读取字段数失败: {:?}", e))? as usize;

    let mut fields = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        let hash = parser
            .read_hash()
            .map_err(|e| format!("读取 hash 失败: {:?}", e))?;
        let vtype = parser
            .read_type()
            .map_err(|e| format!("读取 type 失败: {:?}", e))?;
        let slice = parser
            .skip_value(vtype)
            .map_err(|e| format!("读取 value slice 失败: {:?}", e))?;

        let name = bin_hash_to_str(hash, hashes);
        let t_decl = type_decl(vtype, slice);
        let value = parse_val(vtype, slice, hashes)?;

        fields.push(PropRonField {
            name,
            type_decl: t_decl,
            value,
        });
    }

    Ok(fields)
}

fn parse_val(
    vtype: BinType,
    slice: &[u8],
    hashes: &HashMap<u32, String>,
) -> Result<PropRonValue, String> {
    let mut parser = BinParser::from_bytes(slice);
    let val = match vtype {
        BinType::None => PropRonValue::Null,
        BinType::Bool | BinType::Flag => {
            PropRonValue::Bool(parser.read_bool().map_err(|e| format!("{:?}", e))?)
        }
        BinType::S8 => PropRonValue::I8(parser.read_i8().map_err(|e| format!("{:?}", e))?),
        BinType::U8 => PropRonValue::U8(parser.read_u8().map_err(|e| format!("{:?}", e))?),
        BinType::S16 => PropRonValue::I16(parser.read_s16().map_err(|e| format!("{:?}", e))?),
        BinType::U16 => PropRonValue::U16(parser.read_u16().map_err(|e| format!("{:?}", e))?),
        BinType::S32 => PropRonValue::I32(parser.read_s32().map_err(|e| format!("{:?}", e))?),
        BinType::U32 => PropRonValue::U32(parser.read_u32().map_err(|e| format!("{:?}", e))?),
        BinType::S64 => PropRonValue::I64(parser.read_s64().map_err(|e| format!("{:?}", e))?),
        BinType::U64 => PropRonValue::U64(parser.read_u64().map_err(|e| format!("{:?}", e))?),
        BinType::Float => PropRonValue::Float(parser.read_f32().map_err(|e| format!("{:?}", e))?),
        BinType::Vec2 => {
            let v = parser.read_f32_many(2).map_err(|e| format!("{:?}", e))?;
            PropRonValue::Vec2(v[0], v[1])
        }
        BinType::Vec3 => {
            let v = parser.read_f32_many(3).map_err(|e| format!("{:?}", e))?;
            PropRonValue::Vec3(v[0], v[1], v[2])
        }
        BinType::Vec4 => {
            let v = parser.read_f32_many(4).map_err(|e| format!("{:?}", e))?;
            PropRonValue::Vec4(v[0], v[1], v[2], v[3])
        }
        BinType::Matrix => {
            let v = parser.read_f32_many(16).map_err(|e| format!("{:?}", e))?;
            PropRonValue::Matrix(v)
        }
        BinType::Color => {
            let v = parser.read_u8_many(4).map_err(|e| format!("{:?}", e))?;
            PropRonValue::Color(v[0], v[1], v[2], v[3])
        }
        BinType::String => {
            PropRonValue::String(parser.read_string().map_err(|e| format!("{:?}", e))?)
        }
        BinType::Hash | BinType::Link => {
            let hash = parser.read_u32().map_err(|e| format!("{:?}", e))?;
            PropRonValue::Hash(bin_hash_to_str(hash, hashes))
        }
        BinType::Path => {
            let hash = parser.read_u64().map_err(|e| format!("{:?}", e))?;
            PropRonValue::Path(game_hash_to_str(hash))
        }
        BinType::Struct | BinType::Embed => match parser
            .read_struct_header()
            .map_err(|e| format!("{:?}", e))?
        {
            None => PropRonValue::Null,
            Some(header) => {
                let name = bin_hash_to_str(header.class_hash, hashes);
                let fields = parse_fields(parser.input, hashes)?;
                PropRonValue::Struct {
                    class_name: name,
                    fields,
                }
            }
        },
        BinType::List | BinType::List2 => {
            let inner = parser.read_type().map_err(|e| format!("{:?}", e))?;
            let _bytes_count = parser.read_u32().map_err(|e| format!("{:?}", e))?;
            let items = parser.read_list(inner).map_err(|e| format!("{:?}", e))?;
            let mut list_vals = Vec::with_capacity(items.len());
            for item in items {
                list_vals.push(parse_val(inner, item, hashes)?);
            }
            PropRonValue::List(list_vals)
        }
        BinType::Option => {
            let inner = parser.read_type().map_err(|e| format!("{:?}", e))?;
            let some = parser.read_bool().map_err(|e| format!("{:?}", e))?;
            if !some {
                PropRonValue::Option(None)
            } else {
                let val_slice = parser.skip_value(inner).map_err(|e| format!("{:?}", e))?;
                let val = parse_val(inner, val_slice, hashes)?;
                PropRonValue::Option(Some(Box::new(val)))
            }
        }
        BinType::Map => {
            let ktype = parser.read_type().map_err(|e| format!("{:?}", e))?;
            let vtype = parser.read_type().map_err(|e| format!("{:?}", e))?;
            let _bytes_count = parser.read_u32().map_err(|e| format!("{:?}", e))?;
            let count = parser.read_u32().map_err(|e| format!("{:?}", e))?;
            let mut map_vals = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let k_slice = parser.skip_value(ktype).map_err(|e| format!("{:?}", e))?;
                let v_slice = parser.skip_value(vtype).map_err(|e| format!("{:?}", e))?;
                let k_val = parse_val(ktype, k_slice, hashes)?;
                let v_val = parse_val(vtype, v_slice, hashes)?;
                map_vals.push((k_val, v_val));
            }
            PropRonValue::Map(map_vals)
        }
        BinType::Entry => return Err("Entry 类型不应在值流中".to_string()),
    };
    Ok(val)
}
