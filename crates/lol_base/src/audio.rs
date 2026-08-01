use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 皮肤音效配置，作为可正反序列化的 Asset 存放在 skins/skin{N}_audio.ron 中。
///
/// key 为去掉 `Play_sfx_{Champ}_` 前缀后的事件描述名（保留相位后缀），
/// 如 `"BasicAttack_OnCast"`, `"FioraQ_OnCast"`, `"FioraPassiveReadySound_OnBuffActivate"`。
/// value 为该事件解析到的 ogg 变体路径列表（运行时随机挑一个播放）。
#[derive(Clone, Debug, Default, Serialize, Deserialize, Asset, TypePath)]
pub struct ConfigAudio {
    pub events: BTreeMap<String, Vec<String>>,
}

/// 皮肤音效句柄组件——挂在角色实体上，随皮肤场景 skin{N}.ron 反射序列化写回主 World。
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component, Default)]
pub struct AudioBank(pub Handle<ConfigAudio>);
