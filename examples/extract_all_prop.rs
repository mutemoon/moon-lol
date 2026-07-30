use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use league_loader::game::LeagueLoader;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_property::extract::get_hashes;
use league_property::parser::BinParser;
use league_property::prop::PropFile;
use league_property::types::{BinDeserializerResult, BinType, Error};
use rayon::prelude::*;

const OUTPUT_DIR: &str = "assets/props";

fn main() {
    let root_dir = r"D:\WeGameApps\英雄联盟\Game";

    let start = Instant::now();

    // 加载游戏目录下所有 wad 文件
    let loader = LeagueLoader::full(root_dir).unwrap();

    println!("加载 {} 个 wad 耗时: {:?}", loader.wads.len(), start.elapsed());

    let start = Instant::now();

    // bin 内部 hash 表（u32，FNV-1a），用于类名/字段名/entry 名
    let hashes = get_hashes(&[
        "assets/CommunityDragon-Data/hashes/lol/hashes.binentries.txt",
        "assets/CommunityDragon-Data/hashes/lol/hashes.binfields.txt",
        "assets/CommunityDragon-Data/hashes/lol/hashes.binhashes.txt",
        "assets/CommunityDragon-Data/hashes/lol/hashes.bintypes.txt",
    ]);

    // wad 路径 hash 表（u64，xxhash64），用于还原 prop 文件原始路径
    let game_hash_paths: Vec<String> = (0..9)
        .map(|i| format!("assets/CommunityDragon-Data/hashes/lol/hashes.game.txt.{}", i))
        .collect();
    let game_hashes = get_game_hashes(&game_hash_paths);

    println!(
        "加载 hash 表耗时: {:?} (bin: {}, game: {})",
        start.elapsed(),
        hashes.len(),
        game_hashes.len()
    );

    let start = Instant::now();

    // 收集所有 (wad_index, entry_hash) 任务，按 entry hash 去重（多个 wad 可能包含同一文件）
    let mut seen = HashSet::new();
    let tasks: Vec<_> = loader
        .wads
        .iter()
        .enumerate()
        .flat_map(|(wad_index, wad)| {
            wad.wad
                .entries
                .keys()
                .copied()
                .map(move |hash| (wad_index, hash))
        })
        .filter(|(_, hash)| seen.insert(*hash))
        .collect();

    let total_tasks = tasks.len();
    let processed_count = AtomicUsize::new(0);
    let extracted_count = AtomicUsize::new(0);
    let unresolved_count = AtomicUsize::new(0);
    let failed_count = AtomicUsize::new(0);

    // 静默解析失败的 panic 输出，避免刷屏
    std::panic::set_hook(Box::new(|_| {}));

    tasks.par_iter().for_each(|(wad_index, hash)| {
        let wad = &loader.wads[*wad_index];

        // 无论成功与否都增加计数
        let current = processed_count.fetch_add(1, Ordering::Relaxed) + 1;
        if current % 50000 == 0 || current == total_tasks {
            println!("已处理 {} / {} 个 entry", current, total_tasks);
        }

        let Ok(buffer) = wad.get_wad_entry_buffer_by_hash(*hash) else {
            return;
        };

        // 只处理 PROP 文件（PTCH 文件内嵌 PROP，跳过 12 字节头）
        let prop_bytes: &[u8] = if buffer.starts_with(b"PROP") {
            &buffer
        } else if buffer.starts_with(b"PTCH") && buffer.len() > 12 && buffer[12..].starts_with(b"PROP") {
            &buffer[12..]
        } else {
            return;
        };

        // 解析器内部存在 unwrap，用 catch_unwind 兜底跳过损坏文件
        let result = catch_unwind(AssertUnwindSafe(|| -> Option<String> {
            let (_, prop) = PropFile::parse(prop_bytes).ok()?;
            let mut writer = PropTextWriter {
                hashes: &hashes,
                game_hashes: &game_hashes,
                out: String::new(),
            };
            writer.write_prop(&prop).ok()?;
            Some(writer.out)
        }));

        let Ok(Some(text)) = result else {
            failed_count.fetch_add(1, Ordering::Relaxed);
            return;
        };

        // 查表成功用原始路径，失败按 {wad名}/{hash 16进制}.prop 回退
        let out_path = match game_hashes.get(hash) {
            Some(path) => Path::new(OUTPUT_DIR).join(Path::new(path).with_extension("prop")),
            None => {
                unresolved_count.fetch_add(1, Ordering::Relaxed);
                Path::new(OUTPUT_DIR)
                    .join(wad_name(&wad.relative_path))
                    .join(format!("{:016x}.prop", hash))
            }
        };

        if let Some(parent) = out_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if fs::write(&out_path, text).is_ok() {
            extracted_count.fetch_add(1, Ordering::Relaxed);
        }
    });

    // 恢复默认 panic hook
    let _ = std::panic::take_hook();

    println!(
        "提取完成，耗时: {:?}，共写出 {} 个 prop（路径未命中 {} 个，解析失败 {} 个），输出目录: {}",
        start.elapsed(),
        extracted_count.load(Ordering::Relaxed),
        unresolved_count.load(Ordering::Relaxed),
        failed_count.load(Ordering::Relaxed),
        OUTPUT_DIR
    );
}

/// 解析 hashes.game.txt.* 分片，得到 u64 -> 原始路径 表
fn get_game_hashes(paths: &[String]) -> HashMap<u64, String> {
    let maps: Vec<HashMap<u64, String>> = paths
        .par_iter()
        .map(|path| {
            let mut map = HashMap::new();
            if let Ok(content) = fs::read_to_string(path) {
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

    let mut merged = HashMap::new();
    for map in maps {
        merged.extend(map);
    }
    merged
}

/// 从 wad 相对路径取名，如 "DATA/FINAL/Champions/Fiora.wad.client" -> "Fiora"
fn wad_name(relative_path: &str) -> String {
    Path::new(relative_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .trim_end_matches(".wad.client")
        .to_string()
}

/// BinType -> ritobin 文本类型名
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

/// 字段声明的完整类型名，容器类型需从值切片中读取内部类型
fn type_decl(vtype: BinType, slice: &[u8]) -> String {
    let inner = |b: u8| {
        BinType::try_from(b)
            .map(bin_type_name)
            .unwrap_or("unknown")
    };
    match vtype {
        BinType::List if !slice.is_empty() => format!("list[{}]", inner(slice[0])),
        BinType::List2 if !slice.is_empty() => format!("list2[{}]", inner(slice[0])),
        BinType::Option if !slice.is_empty() => format!("option[{}]", inner(slice[0])),
        BinType::Map if slice.len() >= 2 => format!("map[{},{}]", inner(slice[0]), inner(slice[1])),
        _ => bin_type_name(vtype).to_string(),
    }
}

fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

struct PropTextWriter<'a> {
    hashes: &'a HashMap<u32, String>,
    game_hashes: &'a HashMap<u64, String>,
    out: String,
}

impl PropTextWriter<'_> {
    fn push_indent(&mut self, indent: usize) {
        for _ in 0..indent {
            self.out.push_str("    ");
        }
    }

    /// hash 值：查到名字用带引号字符串，否则 16 进制
    fn bin_hash_str(&self, hash: u32) -> String {
        match self.hashes.get(&hash) {
            Some(name) => format!("\"{}\"", escape_str(name)),
            None => format!("0x{:08x}", hash),
        }
    }

    /// 类名：查到用原名，否则 16 进制
    fn class_name_str(&self, hash: u32) -> String {
        match self.hashes.get(&hash) {
            Some(name) => name.clone(),
            None => format!("0x{:08x}", hash),
        }
    }

    /// 字段名：查到用原名，否则 16 进制
    fn field_name_str(&self, hash: u32) -> String {
        match self.hashes.get(&hash) {
            Some(name) => name.clone(),
            None => format!("0x{:08x}", hash),
        }
    }

    /// 文件路径 hash（u64）：查到用带引号路径，否则 16 进制
    fn game_hash_str(&self, hash: u64) -> String {
        match self.game_hashes.get(&hash) {
            Some(name) => format!("\"{}\"", escape_str(name)),
            None => format!("0x{:016x}", hash),
        }
    }

    fn write_prop(&mut self, prop: &PropFile) -> BinDeserializerResult<()> {
        self.out.push_str("#PROP_text\n");
        self.out.push_str("type: string = \"PROP\"\n");
        writeln!(self.out, "version: u32 = {}", prop.version).unwrap();

        if prop.links.is_empty() {
            self.out.push_str("linked: list[string] = {}\n");
        } else {
            self.out.push_str("linked: list[string] = {\n");
            for link in &prop.links {
                writeln!(self.out, "    \"{}\"", escape_str(&link.text)).unwrap();
            }
            self.out.push_str("}\n");
        }

        if prop.entries.is_empty() {
            self.out.push_str("entries: map[hash,embed] = {}\n");
        } else {
            self.out.push_str("entries: map[hash,embed] = {\n");
            for (class_hash, entry) in prop.iter_class_hash_and_entry() {
                self.push_indent(1);
                write!(
                    self.out,
                    "{} = {} {{\n",
                    self.bin_hash_str(entry.hash),
                    self.class_name_str(class_hash)
                )
                .unwrap();
                self.write_fields(&entry.data, 2)?;
                self.push_indent(1);
                self.out.push_str("}\n");
            }
            self.out.push_str("}\n");
        }

        Ok(())
    }

    /// 按文件顺序写出一段字段区（u16 数量 + 若干字段）
    fn write_fields(&mut self, data: &[u8], indent: usize) -> BinDeserializerResult<()> {
        let mut parser = BinParser::from_bytes(data);
        let field_count = parser.read_u16()? as usize;

        for _ in 0..field_count {
            let hash = parser.read_hash()?;
            let vtype = parser.read_type()?;
            let slice = parser.skip_value(vtype)?;

            self.push_indent(indent);
            write!(
                self.out,
                "{}: {} = ",
                self.field_name_str(hash),
                type_decl(vtype, slice)
            )
            .unwrap();
            self.write_value(vtype, slice, indent)?;
            self.out.push('\n');
        }

        Ok(())
    }

    fn write_value(
        &mut self,
        vtype: BinType,
        slice: &[u8],
        indent: usize,
    ) -> BinDeserializerResult<()> {
        let mut parser = BinParser::from_bytes(slice);

        match vtype {
            BinType::None => self.out.push_str("null"),
            BinType::Bool | BinType::Flag => {
                let v = parser.read_bool()?;
                write!(self.out, "{}", v).unwrap();
            }
            BinType::S8 => write!(self.out, "{}", parser.read_i8()?).unwrap(),
            BinType::U8 => write!(self.out, "{}", parser.read_u8()?).unwrap(),
            BinType::S16 => write!(self.out, "{}", parser.read_s16()?).unwrap(),
            BinType::U16 => write!(self.out, "{}", parser.read_u16()?).unwrap(),
            BinType::S32 => write!(self.out, "{}", parser.read_s32()?).unwrap(),
            BinType::U32 => write!(self.out, "{}", parser.read_u32()?).unwrap(),
            BinType::S64 => write!(self.out, "{}", parser.read_s64()?).unwrap(),
            BinType::U64 => write!(self.out, "{}", parser.read_u64()?).unwrap(),
            BinType::Float => write!(self.out, "{}", parser.read_f32()?).unwrap(),
            BinType::Vec2 | BinType::Vec3 | BinType::Vec4 | BinType::Matrix => {
                let count = match vtype {
                    BinType::Vec2 => 2,
                    BinType::Vec3 => 3,
                    BinType::Vec4 => 4,
                    _ => 16,
                };
                let values = parser.read_f32_many(count)?;
                let joined = values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(self.out, "{{ {} }}", joined).unwrap();
            }
            BinType::Color => {
                let values = parser.read_u8_many(4)?;
                let joined = values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(self.out, "{{ {} }}", joined).unwrap();
            }
            BinType::String => {
                let s = parser.read_string()?;
                write!(self.out, "\"{}\"", escape_str(&s)).unwrap();
            }
            BinType::Hash | BinType::Link => {
                let hash = parser.read_u32()?;
                let s = self.bin_hash_str(hash);
                self.out.push_str(&s);
            }
            BinType::Path => {
                let hash = parser.read_u64()?;
                let s = self.game_hash_str(hash);
                self.out.push_str(&s);
            }
            BinType::Struct | BinType::Embed => match parser.read_struct_header()? {
                None => self.out.push_str("null"),
                Some(header) => {
                    let name = self.class_name_str(header.class_hash);
                    let fields = parser.input;
                    // 空结构体直接写 {}
                    if fields.len() >= 2 && u16::from_le_bytes([fields[0], fields[1]]) == 0 {
                        write!(self.out, "{} {{}}", name).unwrap();
                    } else {
                        write!(self.out, "{} {{\n", name).unwrap();
                        self.write_fields(fields, indent + 1)?;
                        self.push_indent(indent);
                        self.out.push('}');
                    }
                }
            },
            BinType::List | BinType::List2 => {
                let inner = parser.read_type()?;
                let _bytes_count = parser.read_u32()?;
                let items = parser.read_list(inner)?;

                if items.is_empty() {
                    self.out.push_str("{}");
                } else {
                    self.out.push_str("{\n");
                    for item in items {
                        self.push_indent(indent + 1);
                        self.write_value(inner, item, indent + 1)?;
                        self.out.push('\n');
                    }
                    self.push_indent(indent);
                    self.out.push('}');
                }
            }
            BinType::Option => {
                let inner = parser.read_type()?;
                let some = parser.read_bool()?;

                if !some {
                    self.out.push_str("{}");
                } else {
                    let value_slice = parser.skip_value(inner)?;
                    self.out.push_str("{\n");
                    self.push_indent(indent + 1);
                    self.write_value(inner, value_slice, indent + 1)?;
                    self.out.push('\n');
                    self.push_indent(indent);
                    self.out.push('}');
                }
            }
            BinType::Map => {
                let ktype = parser.read_type()?;
                let vtype = parser.read_type()?;
                let _bytes_count = parser.read_u32()?;
                let count = parser.read_u32()?;

                if count == 0 {
                    self.out.push_str("{}");
                } else {
                    self.out.push_str("{\n");
                    for _ in 0..count {
                        let key_slice = parser.skip_value(ktype)?;
                        let value_slice = parser.skip_value(vtype)?;
                        self.push_indent(indent + 1);
                        self.write_value(ktype, key_slice, indent + 1)?;
                        self.out.push_str(" = ");
                        self.write_value(vtype, value_slice, indent + 1)?;
                        self.out.push('\n');
                    }
                    self.push_indent(indent);
                    self.out.push('}');
                }
            }
            BinType::Entry => {
                return Err(Error::Message("Entry 类型不应出现在值流中".to_string()));
            }
        }

        Ok(())
    }
}
