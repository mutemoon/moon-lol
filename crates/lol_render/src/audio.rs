//! 音效播放插件：监听普攻/技能事件，从施法者实体上的 [`AudioBank`] 读取
//! 提取阶段转码好的 ogg 列表（见 league_to_lol::extract::audio），随机挑一个
//! 用 bevy_audio 播放（临时实体 + `PlaybackSettings::DESPAWN`，播完自动清理）。

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::prelude::*;
use lol_base::audio::{AudioBank, ConfigAudio};
use lol_base::render_cmd::CommandSkinSoundPlay;
use lol_base::spell::Spell;
use lol_core::attack::{EventAttackEnd, EventAttackStart};
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
    let name_options = if cleaned != base_name {
        vec![base_name, cleaned]
    } else {
        vec![base_name]
    };

    for name in &name_options {
        for suffix in suffixes {
            let candidate = format!("{}{}", name, suffix);
            if let Some(paths) = config.events.get(&candidate) {
                return Some(paths);
            }
        }
    }

    for name in &name_options {
        let lower_base = name.to_lowercase();
        for suffix in suffixes {
            let lower_candidate = format!("{}{}", lower_base, suffix.to_lowercase());
            if let Some((_, paths)) = config
                .events
                .iter()
                .find(|(k, _)| k.to_lowercase() == lower_candidate)
            {
                return Some(paths);
            }
        }
    }

    None
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
    let suffixes = [""];
    if let Some(paths) = find_event_by_candidates(config, &trigger.key, &suffixes) {
        play_random(&mut commands, &asset_server, paths);
    } else {
        info!("{}", trigger.key);
    }
}

/// 普攻出手：播放包含 BasicAttack 的 Cast 音效
fn on_attack_cast_play_sound(
    trigger: On<EventAttackStart>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_bank: Query<&AudioBank>,
    res_audio: Res<Assets<ConfigAudio>>,
) {
    let attacker = trigger.event_target();
    let Some(config) = config_of(&q_bank, &res_audio, attacker) else {
        return;
    };
    let suffixes = ["_OnCast", "_Cast"];
    if let Some(paths) = find_event_by_candidates(config, "BasicAttack", &suffixes) {
        play_random(&mut commands, &asset_server, paths);
    }
}

/// 普攻命中：播放包含 BasicAttack 的 Hit 音效
fn on_attack_hit_play_sound(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_bank: Query<&AudioBank>,
    res_audio: Res<Assets<ConfigAudio>>,
) {
    let attacker = trigger.event_target();
    let Some(config) = config_of(&q_bank, &res_audio, attacker) else {
        return;
    };
    let suffixes = ["_OnHit", "_Hit"];
    if let Some(paths) = find_event_by_candidates(config, "BasicAttack", &suffixes) {
        play_random(&mut commands, &asset_server, paths);
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
    // 音效变体较多时用系统时间低位做伪随机下标（避免引入 rand 版本冲突）
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0)
        % paths.len();
    let handle: Handle<AudioSource> = asset_server.load(&paths[idx]);
    commands.spawn((AudioPlayer(handle), PlaybackSettings::DESPAWN));
}
