//! 音效播放插件：监听普攻/技能事件，从施法者实体上的 [`AudioBank`] 读取
//! 提取阶段转码好的 ogg 列表（见 league_to_lol::extract::audio），随机挑一个
//! 用 bevy_audio 播放（临时实体 + `PlaybackSettings::DESPAWN`，播完自动清理）。

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::prelude::*;
use lol_base::audio::{AudioBank, ConfigAudio};
use lol_base::render_cmd::CommandSkinSoundPlay;
use lol_base::spell::Spell;
use lol_core::attack::{Attack, EventAttackEnd, EventAttackStart};
use lol_core::skill::{EventSkillCast, Skill};

use crate::loaders::audio::LoaderConfigAudioLoader;

#[derive(Default)]
pub struct PluginAudio;

impl Plugin for PluginAudio {
    fn build(&self, app: &mut App) {
        app.init_asset::<ConfigAudio>();
        app.init_asset_loader::<LoaderConfigAudioLoader>();

        app.add_observer(on_attack_cast_play_sound);
        app.add_observer(on_attack_hit_play_sound);
        app.add_observer(on_skill_cast_play_sound);
        app.add_observer(on_skin_sound_play);
    }
}

fn strip_play_sfx_prefix(name: &str) -> &str {
    if let Some(rest) = name.strip_prefix("Play_sfx_") {
        if let Some((_champ, real_name)) = rest.split_once('_') {
            real_name
        } else {
            rest
        }
    } else {
        name
    }
}

/// 按候选后缀顺序匹配事件 key，命中第一个即返回其 ogg 列表，避免误匹配和无关音效混合。
fn find_event_by_candidates<'a>(
    config: &'a ConfigAudio,
    base_name: &str,
    suffixes: &[&str],
) -> Option<&'a Vec<String>> {
    let cleaned = strip_play_sfx_prefix(base_name);
    let mut name_options = vec![base_name];
    if cleaned != base_name {
        name_options.push(cleaned);
    }

    // 1. 精确匹配：name + suffix
    for name in &name_options {
        for suffix in suffixes {
            let candidate = format!("{}{}", name, suffix);
            if let Some(paths) = config.events.get(&candidate) {
                if !paths.is_empty() {
                    return Some(paths);
                }
            }
        }
    }

    // 2. 大小写不敏感匹配：name + suffix
    for name in &name_options {
        let lower_base = name.to_lowercase();
        for suffix in suffixes {
            let lower_candidate = format!("{}{}", lower_base, suffix.to_lowercase());
            if let Some((_, paths)) = config
                .events
                .iter()
                .find(|(k, paths)| !paths.is_empty() && k.to_lowercase() == lower_candidate)
            {
                return Some(paths);
            }
        }
    }

    // 3. 剥离 base_name 已有后缀后再次尝试匹配（处理传入 key 后缀与实际 ron 中后缀不一致的情况）
    for name in &name_options {
        let known_suffixes = [
            "_OnHit",
            "_Hit",
            "_hit",
            "_OnCast",
            "_Cast",
            "_cast",
            "_OnBuffCast",
            "_OnBuffActivate",
            "_buffcast",
            "_buffactivate",
            "_OnMissileLaunch",
            "_OnMissileHit",
        ];
        let mut stripped_name = *name;
        for s in known_suffixes {
            if let Some(prefix) = name.strip_suffix(s) {
                stripped_name = prefix;
                break;
            }
        }
        if stripped_name != *name {
            let fallback_suffixes = [
                "",
                "_OnBuffCast",
                "_OnBuffActivate",
                "_OnHit",
                "_Hit",
                "_hit",
                "_OnCast",
                "_Cast",
                "_cast",
                "_buffactivate",
                "_buffcast",
            ];
            for suffix in fallback_suffixes {
                let candidate = format!("{}{}", stripped_name, suffix);
                if let Some(paths) = config.events.get(&candidate) {
                    if !paths.is_empty() {
                        return Some(paths);
                    }
                }
                let lower_candidate = candidate.to_lowercase();
                if let Some((_, paths)) = config
                    .events
                    .iter()
                    .find(|(k, paths)| !paths.is_empty() && k.to_lowercase() == lower_candidate)
                {
                    return Some(paths);
                }
            }
        }
    }

    None
}

/// 通用普攻（出手/命中）音效查找：汇总所有匹配的普攻变体音效
fn find_attack_sounds<'a>(
    config: &'a ConfigAudio,
    spell_name: Option<&str>,
    is_hit: bool,
) -> Vec<&'a String> {
    let suffixes: &[&str] = if is_hit {
        &["_onhit", "_hit", "_onbuffcast", "_onbuffactivate"]
    } else {
        &["_oncast", "_cast", "_onbuffcast", "_onbuffactivate"]
    };

    let mut matched_paths = Vec::new();

    // 1. 如果有指定的普攻技能名（如 "FioraBasicAttack"）
    if let Some(raw_name) = spell_name {
        let cleaned = strip_play_sfx_prefix(raw_name).to_lowercase();
        let base = cleaned.trim_end_matches(|c: char| c.is_ascii_digit());

        for (key, paths) in &config.events {
            let lower_key = key.to_lowercase();
            if lower_key.contains(base) && suffixes.iter().any(|s| lower_key.ends_with(s)) {
                matched_paths.extend(paths.iter());
            }
        }
    }

    // 2. 如果指定技能名未命中或未提供，按 "basicattack" 匹配所有普攻音效
    if matched_paths.is_empty() {
        for (key, paths) in &config.events {
            let lower_key = key.to_lowercase();
            if lower_key.contains("basicattack") && suffixes.iter().any(|s| lower_key.ends_with(s))
            {
                matched_paths.extend(paths.iter());
            }
        }
    }

    // 3. 如果依然未找到，放宽至 "critattack" 或 "attack"
    if matched_paths.is_empty() {
        for (key, paths) in &config.events {
            let lower_key = key.to_lowercase();
            if (lower_key.contains("critattack") || lower_key.contains("attack"))
                && suffixes.iter().any(|s| lower_key.ends_with(s))
            {
                matched_paths.extend(paths.iter());
            }
        }
    }

    matched_paths
}

/// 技能/被动命中或触发音效：由英雄代码在对应时机显式触发（通过 CommandSkinSoundPlay）。
fn on_skin_sound_play(
    trigger: On<CommandSkinSoundPlay>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_bank: Query<&AudioBank>,
    res_audio: Res<Assets<ConfigAudio>>,
) {
    let entity = trigger.event_target();
    let Some(config) = config_of(&q_bank, &res_audio, entity) else {
        return;
    };
    let suffixes = [
        "",
        "_OnBuffCast",
        "_OnBuffActivate",
        "_OnHit",
        "_Hit",
        "_hit",
        "_OnCast",
        "_Cast",
        "_cast",
        "_buffactivate",
        "_buffcast",
    ];
    if let Some(paths) = find_event_by_candidates(config, &trigger.key, &suffixes) {
        play_random(&mut commands, &asset_server, paths);
    } else {
        info!("未找到匹配音效: {}", trigger.key);
    }
}

/// 普攻出手：播放包含 BasicAttack 的 Cast 音效
fn on_attack_cast_play_sound(
    trigger: On<EventAttackStart>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_bank: Query<&AudioBank>,
    q_attack: Query<&Attack>,
    res_audio: Res<Assets<ConfigAudio>>,
) {
    let attacker = trigger.event_target();
    let Some(config) = config_of(&q_bank, &res_audio, attacker) else {
        return;
    };
    let spell_name = q_attack
        .get(attacker)
        .ok()
        .and_then(|atk| spell_name_from_handle(&asset_server, &atk.spell));

    let sounds = find_attack_sounds(config, spell_name.as_deref(), false);
    if !sounds.is_empty() {
        play_random_from_refs(&mut commands, &asset_server, &sounds);
    }
}

/// 普攻命中：播放包含 BasicAttack 的 Hit 音效
fn on_attack_hit_play_sound(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_bank: Query<&AudioBank>,
    q_attack: Query<&Attack>,
    res_audio: Res<Assets<ConfigAudio>>,
) {
    let attacker = trigger.event_target();
    let Some(config) = config_of(&q_bank, &res_audio, attacker) else {
        return;
    };
    let spell_name = q_attack
        .get(attacker)
        .ok()
        .and_then(|atk| spell_name_from_handle(&asset_server, &atk.spell));

    let sounds = find_attack_sounds(config, spell_name.as_deref(), true);
    if !sounds.is_empty() {
        play_random_from_refs(&mut commands, &asset_server, &sounds);
    }
}

/// 技能施放：按技能对象名（如 AatroxQ）查找并播放对应 OnCast 音效
fn on_skill_cast_play_sound(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_bank: Query<&AudioBank>,
    q_skill: Query<&Skill>,
    res_audio: Res<Assets<ConfigAudio>>,
) {
    let caster = trigger.event_target();
    let Some(config) = config_of(&q_bank, &res_audio, caster) else {
        return;
    };
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    let Some(name) = spell_name_from_handle(&asset_server, &skill.spell) else {
        return;
    };

    let suffixes = [
        "_OnCast",
        "_Cast",
        "_OnBuffActivate",
        "_OnBuffCast",
        "_buffactivate",
        "_buffcast",
    ];
    if let Some(paths) = find_event_by_candidates(config, &name, &suffixes) {
        play_random(&mut commands, &asset_server, paths);
    }
}

/// 从实体读取 AudioBank 并拿到已加载的 ConfigAudio
fn config_of<'a>(
    q_bank: &'a Query<&AudioBank>,
    res_audio: &'a Res<Assets<ConfigAudio>>,
    entity: Entity,
) -> Option<&'a ConfigAudio> {
    let bank = q_bank.get(entity).ok()?;
    res_audio.get(&bank.0)
}

/// 从 Handle<Spell> 的资产路径提取技能对象名（`.../spells/{name}.ron` -> `{name}`）
fn spell_name_from_handle(asset_server: &AssetServer, handle: &Handle<Spell>) -> Option<String> {
    let path = asset_server.get_path(handle.id())?;
    let file = path.path().file_stem()?.to_str()?;
    if file.is_empty() {
        None
    } else {
        Some(file.to_string())
    }
}

/// 随机播放 `paths` 中的一个 ogg：spawn 临时实体，播放结束自动销毁
fn play_random(commands: &mut Commands, asset_server: &AssetServer, paths: &[String]) {
    if paths.is_empty() {
        return;
    }
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0)
        % paths.len();
    let handle: Handle<AudioSource> = asset_server.load(&paths[idx]);
    commands.spawn((AudioPlayer(handle), PlaybackSettings::DESPAWN));
}

/// 随机播放 `paths` 引用列表中的一个 ogg
fn play_random_from_refs(commands: &mut Commands, asset_server: &AssetServer, paths: &[&String]) {
    if paths.is_empty() {
        return;
    }
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0)
        % paths.len();
    let handle: Handle<AudioSource> = asset_server.load(paths[idx].clone());
    commands.spawn((AudioPlayer(handle), PlaybackSettings::DESPAWN));
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn mock_fiora_audio() -> ConfigAudio {
        let mut events = BTreeMap::new();
        events.insert(
            "FioraBasicAttack_OnCast".to_string(),
            vec!["cast_1.ogg".to_string(), "cast_2.ogg".to_string()],
        );
        events.insert(
            "FioraBasicAttack2_OnCast".to_string(),
            vec!["cast2_1.ogg".to_string()],
        );
        events.insert(
            "FioraBasicAttack_OnHit".to_string(),
            vec!["hit_1.ogg".to_string(), "hit_2.ogg".to_string()],
        );
        events.insert(
            "FioraBasicAttack2_OnHit".to_string(),
            vec!["hit2_1.ogg".to_string()],
        );
        events.insert(
            "FioraPassiveHitSound_OnBuffCast".to_string(),
            vec!["vital_hit.ogg".to_string()],
        );
        events.insert("FioraWSlow_hit".to_string(), vec!["w_slow.ogg".to_string()]);
        ConfigAudio { events }
    }

    #[test]
    fn test_find_attack_hit_sounds_by_spell_name() {
        let audio = mock_fiora_audio();
        let hit_sounds = find_attack_sounds(&audio, Some("FioraBasicAttack"), true);
        assert_eq!(hit_sounds.len(), 3); // 包含 FioraBasicAttack_OnHit (2) 和 FioraBasicAttack2_OnHit (1)
    }

    #[test]
    fn test_find_attack_cast_sounds_generic() {
        let audio = mock_fiora_audio();
        let cast_sounds = find_attack_sounds(&audio, None, false);
        assert_eq!(cast_sounds.len(), 3); // 包含 FioraBasicAttack_OnCast (2) 和 FioraBasicAttack2_OnCast (1)
    }

    #[test]
    fn test_find_event_by_candidates_fallback() {
        let audio = mock_fiora_audio();
        let suffixes = [
            "",
            "_OnBuffCast",
            "_OnBuffActivate",
            "_OnHit",
            "_Hit",
            "_hit",
            "_OnCast",
            "_Cast",
            "_cast",
        ];
        // 即使传入了 FioraPassiveHitSound_OnHit，也能容错匹配到 FioraPassiveHitSound_OnBuffCast
        let found = find_event_by_candidates(&audio, "FioraPassiveHitSound_OnHit", &suffixes);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &vec!["vital_hit.ogg".to_string()]);

        // 精确传入 FioraPassiveHitSound_OnBuffCast
        let found2 = find_event_by_candidates(&audio, "FioraPassiveHitSound_OnBuffCast", &suffixes);
        assert!(found2.is_some());
        assert_eq!(found2.unwrap(), &vec!["vital_hit.ogg".to_string()]);

        // FioraWSlow_OnHit 容错匹配 FioraWSlow_hit
        let found3 = find_event_by_candidates(&audio, "FioraWSlow_OnHit", &suffixes);
        assert!(found3.is_some());
        assert_eq!(found3.unwrap(), &vec!["w_slow.ogg".to_string()]);
    }
}
