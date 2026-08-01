//! 皮肤音效提取：从 skin_audio_properties.bank_units 出发，解析 Wwise bnk/wpk，
//! 把普攻/技能事件解析成 wem，转码为 ogg 落盘，并生成 [`ConfigAudio`]。
//!
//! 数据链路：bankUnits → bankPath(*_audio.bnk/*.wpk 提供媒体, *_events.bnk 提供 HIRC)
//! + Events(事件名) → FNV-1 哈希 → HIRC 解析出 wem id → ww2ogg 转 ogg。

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Cursor;

use league_core::extract::SkinCharacterDataProperties;
use league_file::bnk::Bnk;
use league_file::wpk::WpkFile;
use league_loader::game::LeagueLoader;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_loader::wad::LeagueWadLoader;
use lol_base::audio::{AudioCue, ConfigAudio};
use ww2ogg::{CodebookLibrary, WwiseRiffVorbis};

use super::utils::write_to_file;

/// 触发相位。
#[derive(Clone, Copy, PartialEq, Debug)]
enum Phase {
    Cast,
    Hit,
}

/// 从事件名判断相位；除了 OnCast/OnHit，Wwise 里大量技能音效走 buff 通道
/// （`_OnBuffActivate`/`_OnBuffCast`/`_buffactivate` 等），R、被动、W 格挡等音效都在其中。
/// `Stop_*` 是停止命令，不参与归类；VO 等其余事件返回 None 被忽略。
fn phase_of(event: &str) -> Option<Phase> {
    let e = event.to_ascii_lowercase();
    if e.starts_with("stop_") {
        return None;
    }
    if e.contains("onhit") || e.contains("hitsound") || e.contains("_hit") {
        Some(Phase::Hit)
    } else if e.contains("attack") && e.contains("buffcast")
        || e.contains("attack") && e.contains("buffactivate")
    {
        // 攻击类音效走 buff 通道时是命中反馈（如 FioraQAttack_OnBuffCast 即 Q 的命中音）
        Some(Phase::Hit)
    } else if e.contains("oncast")
        || e.contains("onbuffactivate")
        || e.contains("onbuffcast")
        || e.contains("buffactivate")
        || e.contains("buffcast")
    {
        Some(Phase::Cast)
    } else {
        None
    }
}

/// 事件是否走 buff 通道（OnBuffActivate/OnBuffCast/_buffactivate/_buffcast）。
fn is_buff_channel_event(event: &str) -> bool {
    let e = event.to_ascii_lowercase();
    e.contains("onbuffactivate")
        || e.contains("onbuffcast")
        || e.contains("buffactivate")
        || e.contains("buffcast")
}

/// 从事件名提取描述名：去掉 `Play_sfx_<Champ>_` 前缀和相位后缀。
/// 如 `Play_sfx_Fiora_FioraPassiveReadySound_OnBuffActivate` -> `FioraPassiveReadySound`。
fn event_descriptor(event: &str, champ_name: &str) -> Option<String> {
    let prefix = format!("Play_sfx_{}_", champ_name);
    let rest = event.strip_prefix(&prefix)?;
    for suffix in [
        "_OnBuffActivate",
        "_OnBuffCast",
        "_OnHit",
        "_OnCast",
        "_buffactivate",
        "_buffcast",
        "_hit",
    ] {
        if let Some(s) = rest.strip_suffix(suffix) {
            return Some(s.to_string());
        }
    }
    None
}

/// 提取皮肤音效并返回 ConfigAudio；`all_spell_names` 为该英雄全部技能对象名，
/// `basic_attack_names` 为普攻/暴击的攻击名（用于把事件归类到普攻）。
pub fn export_audio_for_skin(
    loader: &LeagueLoader,
    champ_name: &str,
    skin_id: &str,
    skin_data: &SkinCharacterDataProperties,
    all_spell_names: &[String],
    basic_attack_names: &[String],
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

    // 2) 分类事件到 普攻 / 各技能
    // 技能候选：排除普攻名，按名字长度降序以便优先匹配更具体的技能名
    let basic_lower: Vec<String> = basic_attack_names.iter().map(|s| s.to_lowercase()).collect();
    let mut spell_candidates: Vec<String> = all_spell_names
        .iter()
        .filter(|name| {
            let lower = name.to_lowercase();
            !basic_lower.iter().any(|b| !b.is_empty() && lower == *b)
        })
        .cloned()
        .collect();
    spell_candidates.sort_by(|a, b| b.len().cmp(&a.len()));

    let sounds_dir = format!("characters/{}/sounds", champ_name);
    let mut ogg_cache: HashMap<u32, Option<String>> = HashMap::new();

    let mut basic = AudioCue::default();
    let mut spells: BTreeMap<String, AudioCue> = BTreeMap::new();

    println!("[AUDIO] 开始处理事件（技能候选 {} 个）", spell_candidates.len());
    let mut n_matched = 0usize;
    for (ei, event) in all_events.iter().enumerate() {
        let Some(phase) = phase_of(event) else {
            println!("[AUDIO]   [{}/{}] 跳过（非 cast/hit）: {}", ei + 1, all_events.len(), event);
            continue;
        };
        let lower = event.to_lowercase();

        // 归类：普攻优先（名字命中或含 basicattack/critattack 关键字），否则匹配技能名
        let is_basic = basic_lower.iter().any(|b| !b.is_empty() && lower.contains(b))
            || lower.contains("basicattack")
            || lower.contains("critattack");
        let bucket: Option<String> = if is_basic {
            None // 归入 basic
        } else {
            spell_candidates
                .iter()
                .find(|name| lower.contains(&name.to_lowercase()))
                .cloned()
        };
        // 若描述名不是技能名，则拆成独立 cue（如 FioraPassiveReadySound / FioraWSlow /
        // FioraRMark），避免 ReadySound/Speed/Stun 等不同时机的音效混进同一组随机播放。
        let cue_key: Option<String> = if let Some(b) = &bucket {
            if !is_basic {
                if let Some(desc) = event_descriptor(event, champ_name) {
                    if *b != desc {
                        Some(desc)
                    } else {
                        Some(b.clone())
                    }
                } else {
                    Some(b.clone())
                }
            } else {
                Some(b.clone())
            }
        } else {
            None
        };
        if !is_basic && cue_key.is_none() {
            println!("[AUDIO]   [{}/{}] 跳过（与普攻/技能无关）: {}", ei + 1, all_events.len(), event);
            continue;
        }

        // 解析事件 -> wem -> ogg 路径
        let mut wems: Vec<u32> = Vec::new();
        for bnk in &event_banks {
            wems.extend(bnk.resolve_event(event));
        }
        wems.sort_unstable();
        wems.dedup();
        println!(
            "[AUDIO]   [{}/{}] 事件 {} -> phase={:?} cue={:?} wems={:?}",
            ei + 1,
            all_events.len(),
            event,
            phase,
            cue_key,
            wems
        );
        if wems.is_empty() {
            println!("[AUDIO]     没有解析到任何 wem（事件未命中 HIRC）");
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

        let cue = match &cue_key {
            Some(name) => spells.entry(name.clone()).or_default(),
            None => &mut basic,
        };
        match phase {
            Phase::Cast => cue.on_cast.extend(oggs),
            Phase::Hit => cue.on_hit.extend(oggs),
        }
        n_matched += 1;
    }
    println!("[AUDIO] 事件处理完成：成功归类 {} 个事件", n_matched);

    // 减去与普攻共享的“通用层”音效：原版命中/施法音常由 通用层 + 专属层 两个 action 组成
    // （如破绽击破 = 普攻命中容器 + 专属破绽音），单文件随机播放模型下只保留专属层，
    // 避免破绽/技能命中随机抽到普通攻击音。
    let basic_cast_set: HashSet<&String> = basic.on_cast.iter().collect();
    let basic_hit_set: HashSet<&String> = basic.on_hit.iter().collect();
    for cue in spells.values_mut() {
        cue.on_cast.retain(|p| !basic_cast_set.contains(p));
        cue.on_hit.retain(|p| !basic_hit_set.contains(p));
    }

    // 去重每个 cue 的变体
    dedup_cue(&mut basic);
    for cue in spells.values_mut() {
        dedup_cue(cue);
    }
    spells.retain(|_, cue| !cue.is_empty());

    config.basic_attack = basic;
    config.spells = spells;

    let cast_n = config.basic_attack.on_cast.len();
    let hit_n = config.basic_attack.on_hit.len();
    println!(
        "[AUDIO] {} {}: 普攻 cast={} hit={}, 技能 {} 组",
        champ_name,
        skin_id,
        cast_n,
        hit_n,
        config.spells.len()
    );

    config
}

fn dedup_cue(cue: &mut AudioCue) {
    cue.on_cast.sort();
    cue.on_cast.dedup();
    cue.on_hit.sort();
    cue.on_hit.dedup();
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
