//! 皮肤音效提取：从 skin_audio_properties.bank_units 出发，解析 Wwise bnk/wpk，
//! 将事件名映射为 ogg 音频文件列表，并生成 [`ConfigAudio`]。
//!
//! 数据链路：bankUnits → bankPath(*_audio.bnk/*.wpk 提供媒体, *_events.bnk 提供 HIRC)
//! + Events(事件名) → FNV-1 哈希 → HIRC 解析出 wem id → ww2ogg 转 ogg。

use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;

use league_core::extract::SkinCharacterDataProperties;
use league_file::bnk::Bnk;
use league_file::wpk::WpkFile;
use league_loader::game::LeagueLoader;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_loader::wad::LeagueWadLoader;
use lol_base::audio::ConfigAudio;
use ww2ogg::{CodebookLibrary, WwiseRiffVorbis};

use super::utils::write_to_file;

/// 从事件名提取 key：去掉 `Play_sfx_{Champ}_` 或 `Play_sfx_` 前缀。
fn clean_event_key<'a>(event: &'a str, champ_name: &str) -> &'a str {
    let champ_prefix = format!("Play_sfx_{}_", champ_name);
    if let Some(rest) = event.strip_prefix(&champ_prefix) {
        rest
    } else if let Some(rest) = event.strip_prefix("Play_sfx_") {
        rest
    } else {
        event
    }
}

/// 提取皮肤音效并返回 ConfigAudio；解析 Wwise bnk/wpk，将事件映射到 ogg 资源路径。
pub fn export_audio_for_skin(
    loader: &LeagueLoader,
    champ_name: &str,
    skin_id: &str,
    skin_data: &SkinCharacterDataProperties,
) -> ConfigAudio {
    let mut config = ConfigAudio::default();

    let Some(audio_props) = &skin_data.skin_audio_properties else {
        return config;
    };
    let Some(bank_units) = &audio_props.bank_units else {
        return config;
    };

    // 1) 收集媒体（wem id -> 字节）与事件 bank（含 HIRC）
    let mut media: HashMap<u32, Vec<u8>> = HashMap::new();
    let mut event_banks: Vec<Bnk> = Vec::new();
    let mut all_events: Vec<String> = Vec::new();
    // VO 等本地化资源在 Champions/<champ>.<locale>.wad.client 里，
    // 主 loader 只加载基础 WAD，首次未命中时懒加载语言包 WAD 兜底。
    let mut locale_wads: Vec<LeagueWadLoader> = Vec::new();

    println!("[AUDIO] {} {}: 共 {} 个 bank_unit", champ_name, skin_id, bank_units.len());
    for (ui, unit) in bank_units.iter().enumerate() {
        let n_events = unit.events.as_ref().map(|e| e.len()).unwrap_or(0);
        let n_paths = unit.bank_path.as_ref().map(|p| p.len()).unwrap_or(0);
        println!(
            "[AUDIO]   unit[{}/{}] name={} events={} paths={}",
            ui + 1,
            bank_units.len(),
            unit.name,
            n_events,
            n_paths
        );
        if let Some(events) = &unit.events {
            all_events.extend(events.iter().cloned());
        }
        let Some(paths) = &unit.bank_path else {
            continue;
        };
        for path in paths {
            let t0 = std::time::Instant::now();
            let buf = match loader.get_wad_entry_buffer_by_path(path) {
                Ok(buf) => Some(buf),
                Err(_) => {
                    if locale_wads.is_empty() {
                        for locale in ["en_US", "zh_CN"] {
                            let rel = format!("DATA/FINAL/Champions/{}.{}.wad.client", champ_name, locale);
                            if let Ok(wad) = LeagueWadLoader::from_relative_path(&loader.root_dir, &rel) {
                                locale_wads.push(wad);
                            }
                        }
                    }
                    locale_wads
                        .iter()
                        .find_map(|wad| wad.get_wad_entry_buffer_by_path(path).ok())
                }
            };
            let Some(buf) = buf else {
                println!("[AUDIO]     bank 加载失败: {}", path);
                continue;
            };
            println!(
                "[AUDIO]     bank 加载成功: {} ({} bytes, {:?})",
                path,
                buf.len(),
                t0.elapsed()
            );
            if buf.len() >= 4 && &buf[0..4] == b"BKHD" {
                if let Some(bnk) = Bnk::parse(&buf) {
                    println!(
                        "[AUDIO]       bnk: version=0x{:x} media={} events={} has_hirc={}",
                        bnk.version,
                        bnk.media.len(),
                        bnk.event_count(),
                        bnk.has_events()
                    );
                    for (id, data) in &bnk.media {
                        media.entry(*id).or_insert_with(|| data.clone());
                    }
                    if bnk.has_events() {
                        event_banks.push(bnk);
                    }
                } else {
                    println!("[AUDIO]       bnk 解析失败: {}", path);
                }
            } else if buf.len() >= 4 && &buf[0..4] == *b"r3d2" {
                if let Ok((_, wpk)) = WpkFile::parse(&buf) {
                    println!("[AUDIO]       wpk: entries={}", wpk.entries.len());
                    for entry in wpk.entries {
                        media.entry(entry.id).or_insert(entry.data);
                    }
                } else {
                    println!("[AUDIO]       wpk 解析失败: {}", path);
                }
            } else {
                println!("[AUDIO]       ！未知文件类型: {} (前 4 字节 {:?})", path, &buf[..4.min(buf.len())]);
            }
        }
    }

    println!(
        "[AUDIO] 汇总: 事件总数={} (去重后 {})，媒体 wem 数={}，事件 bank 数={}",
        all_events.len(),
        {
            let mut dedup = all_events.clone();
            dedup.sort();
            dedup.dedup();
            dedup.len()
        },
        media.len(),
        event_banks.len()
    );

    if all_events.is_empty() || event_banks.is_empty() {
        println!("[AUDIO] 提前退出：事件或事件 bank 为空");
        return config;
    }

    let sounds_dir = format!("characters/{}/sounds", champ_name);
    let mut ogg_cache: HashMap<u32, Option<String>> = HashMap::new();
    let mut events_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    println!("[AUDIO] 开始处理事件");
    let mut n_matched = 0usize;
    for (ei, event) in all_events.iter().enumerate() {
        if event.to_ascii_lowercase().starts_with("stop_") {
            continue;
        }

        let event_key = clean_event_key(event, champ_name).to_string();

        // 解析事件 -> wem -> ogg 路径
        let mut wems: Vec<u32> = Vec::new();
        for bnk in &event_banks {
            wems.extend(bnk.resolve_event(event));
        }
        wems.sort_unstable();
        wems.dedup();
        if wems.is_empty() {
            println!("[AUDIO]     [{}/{}] 没有解析到任何 wem: {}", ei + 1, all_events.len(), event);
            continue;
        }

        let mut oggs: Vec<String> = Vec::new();
        for wem_id in wems {
            if let Some(path) = transcode_wem(&media, wem_id, &sounds_dir, &mut ogg_cache) {
                oggs.push(path);
            } else {
                println!("[AUDIO]     wem {} 转码失败或媒体缺失", wem_id);
            }
        }
        if oggs.is_empty() {
            println!("[AUDIO]     没有可用的 ogg，跳过该事件");
            continue;
        }

        events_map.entry(event_key).or_default().extend(oggs);
        n_matched += 1;
    }
    println!("[AUDIO] 事件处理完成：成功解析 {} 个事件", n_matched);

    // 去重每个 key 的 ogg 变体
    for oggs in events_map.values_mut() {
        oggs.sort();
        oggs.dedup();
    }
    events_map.retain(|_, oggs| !oggs.is_empty());

    config.events = events_map;

    println!(
        "[AUDIO] {} {}: 提取完成，共包含 {} 个音效事件",
        champ_name,
        skin_id,
        config.events.len()
    );

    config
}

/// 把一个 wem 转码为 ogg 落盘，返回相对 `assets/` 的路径；带缓存避免重复转码。
fn transcode_wem(
    media: &HashMap<u32, Vec<u8>>,
    wem_id: u32,
    sounds_dir: &str,
    cache: &mut HashMap<u32, Option<String>>,
) -> Option<String> {
    if let Some(cached) = cache.get(&wem_id) {
        return cached.clone();
    }

    let result = (|| {
        let data = media.get(&wem_id)?;
        let rel_path = format!("{}/{}.ogg", sounds_dir, wem_id);
        let abs = std::path::Path::new("assets").join(&rel_path);
        if abs.exists() {
            println!("[AUDIO]     wem {} 已存在 ogg，复用", wem_id);
        } else {
            let t0 = std::time::Instant::now();
            println!("[AUDIO]     wem {} 开始转码 ({} bytes)...", wem_id, data.len());
            // LoL 的 Wwise 2016 wem 使用 aoTuV 6.03 码本；默认码本会解码成噪声
            // （ww2ogg::validate 会报 "likely wrong codebook"）。
            let codebooks = CodebookLibrary::aotuv_codebooks().ok()?;
            let cursor = Cursor::new(data.clone());
            let mut converter = WwiseRiffVorbis::new(cursor, codebooks).ok()?;
            let mut out: Vec<u8> = Vec::new();
            converter.generate_ogg(&mut out).ok()?;
            println!(
                "[AUDIO]     wem {} 转码完成 -> {} bytes, {:?}",
                wem_id,
                out.len(),
                t0.elapsed()
            );
            write_to_file(&rel_path, &out);
        }
        Some(rel_path)
    })();

    cache.insert(wem_id, result.clone());
    result
}
