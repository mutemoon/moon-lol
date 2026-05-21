use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::entities::champion::Champion;
use crate::entities::minion::Minion;
use crate::life::EventDead;

/// 英雄统计数据组件，跟踪补刀与 KDA 统计
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, Default)]
#[reflect(Component)]
pub struct ChampionStats {
    /// 击杀英雄数
    pub kills: u32,
    /// 死亡数
    pub deaths: u32,
    /// 助攻数
    pub assists: u32,
    /// 击杀小兵数 (Creep Score / 补刀数)
    pub minion_kills: u32,
}

/// 英雄统计插件
#[derive(Default)]
pub struct PluginChampionStats;

impl Plugin for PluginChampionStats {
    fn build(&self, app: &mut App) {
        app.register_type::<ChampionStats>();
        app.add_observer(on_event_dead);
    }
}

/// 监听死亡事件，更新英雄统计数据
pub fn on_event_dead(
    event: On<EventDead>,
    q_minion: Query<&Minion>,
    q_champion: Query<&Champion>,
    mut q_stats: Query<&mut ChampionStats>,
) {
    let dead_entity = event.entity;

    // 若死亡实体是英雄，其死亡数加一
    if q_champion.get(dead_entity).is_ok() {
        if let Ok(mut dead_stats) = q_stats.get_mut(dead_entity) {
            dead_stats.deaths += 1;
            debug!(
                "{:?} 死亡数加一，当前死亡数: {}",
                dead_entity, dead_stats.deaths
            );
        }
    }

    // 必须有击杀者才做进一步统计
    let Some(killer_entity) = event.killer else {
        return;
    };

    // 只有击杀者是英雄/带统计组件的实体，才结算击杀和补刀
    let Ok(mut killer_stats) = q_stats.get_mut(killer_entity) else {
        return;
    };

    if q_minion.get(dead_entity).is_ok() {
        killer_stats.minion_kills += 1;
        debug!(
            "{:?} 击杀了小兵 {:?}，当前补刀数: {}",
            killer_entity, dead_entity, killer_stats.minion_kills
        );
        return;
    }

    if q_champion.get(dead_entity).is_ok() {
        killer_stats.kills += 1;
        debug!(
            "{:?} 击杀了英雄 {:?}，当前击杀数: {}",
            killer_entity, dead_entity, killer_stats.kills
        );
    }
}
