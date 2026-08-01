use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 单个触发点的音效线索：一次触发时从中随机挑一个变体播放。
///
/// `on_cast` 对应施放/前摇瞬间，`on_hit` 对应命中/出手瞬间；每项存放可播放的
/// ogg 资源路径（相对 `assets/`）。允许为空（该英雄没有对应音效）。
#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct AudioCue {
    #[serde(default)]
    pub on_cast: Vec<String>,
    #[serde(default)]
    pub on_hit: Vec<String>,
}

impl AudioCue {
    pub fn is_empty(&self) -> bool {
        self.on_cast.is_empty() && self.on_hit.is_empty()
    }
}

/// 皮肤音效配置，作为可正反序列化的 Asset 存放在 skins/skin{N}_audio.ron 中。
///
/// 因为运行时需按“攻击者实体”查表播放，所以该配置由挂在角色实体上的
/// [`AudioBank`] 组件持有其句柄（区别于 vfx 的全局 Resource 注入方式）。
#[derive(Clone, Debug, Default, Serialize, Deserialize, Asset, TypePath)]
pub struct ConfigAudio {
    /// 普通攻击音效（含暴击等变体已合并）。
    pub basic_attack: AudioCue,
    /// 技能音效，key 为技能对象名（如 `AatroxQ`），value 为其 cast/hit 音效。
    pub spells: BTreeMap<String, AudioCue>,
}

/// 皮肤音效句柄组件——挂在角色实体上，随皮肤场景 skin{N}.ron 反射序列化写回主 World。
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component, Default)]
pub struct AudioBank(pub Handle<ConfigAudio>);
