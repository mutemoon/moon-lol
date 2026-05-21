use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::entities::champion::Champion;
use crate::entities::minion::Minion;
use crate::life::{Death, EventDead};

/// 金币资产组件，支持 Bevy ECS 反射和序列化
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug)]
#[reflect(Component)]
pub struct Gold {
    /// 当前金币余额
    pub current: f32,
    /// 累计获得的总金币
    pub total: f32,
}

impl Default for Gold {
    fn default() -> Self {
        Self {
            current: 500.0, // 开局自带 500 金币
            total: 500.0,
        }
    }
}

/// 金币掉落组件，死者实体携带此组件，死亡时分配金币给最后一击施加者
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug)]
#[reflect(Component)]
pub struct GoldDrop {
    /// 死亡时给出的金币量
    pub gold_given_on_death: f32,
    /// 金币掉落分配范围半径
    pub gold_radius: f32,
}

impl Default for GoldDrop {
    fn default() -> Self {
        Self {
            gold_given_on_death: 35.0,
            gold_radius: 1400.0,
        }
    }
}

/// 游戏金币系统的 Bevy 插件
#[derive(Default)]
pub struct PluginGold;

impl Plugin for PluginGold {
    fn build(&self, app: &mut App) {
        app.register_type::<Gold>();
        app.register_type::<GoldDrop>();
        app.add_systems(FixedUpdate, update_passive_gold);
        app.add_observer(on_event_dead);
    }
}

/// 每秒给存活英雄增加 2.04 金币
pub fn update_passive_gold(mut q_gold: Query<&mut Gold, Without<Death>>, time: Res<Time<Fixed>>) {
    let gold_per_sec = 2.04;
    let amount = gold_per_sec * time.delta_secs();
    for mut gold in q_gold.iter_mut() {
        gold.current += amount;
        gold.total += amount;
    }
}

/// 监听单位死亡事件，若死者携带 GoldDrop 且存在击杀者，则结算金币奖励
pub fn on_event_dead(
    event: On<EventDead>,
    q_gold_drop: Query<&GoldDrop>,
    q_minion: Query<&Minion>,
    q_champion: Query<&Champion>,
    mut q_gold: Query<&mut Gold>,
) {
    let dead_entity = event.entity;
    let Some(killer_entity) = event.killer else {
        return;
    };

    // 只有当击杀者是带有金币组件的实体时，才发放金币奖励
    let Ok(mut killer_gold) = q_gold.get_mut(killer_entity) else {
        return;
    };

    let mut gold_gain = 0.0;

    // 优先读取 GoldDrop 掉落数据
    if let Ok(gold_drop) = q_gold_drop.get(dead_entity) {
        gold_gain = gold_drop.gold_given_on_death;
    } else if let Ok(minion) = q_minion.get(dead_entity) {
        // Fallback 到小兵的基础金币
        gold_gain = match minion {
            Minion::Melee => 21.0,
            Minion::Ranged => 14.0,
            Minion::Siege => 60.0,
            Minion::Super => 90.0,
        };
    } else if q_champion.get(dead_entity).is_ok() {
        // Fallback 到英雄的击杀金币
        gold_gain = 300.0;
    }

    if gold_gain > 0.0 {
        killer_gold.current += gold_gain;
        killer_gold.total += gold_gain;
        debug!(
            "{:?} 击杀了 {:?}，获得 {:.1} 金币！当前金币: {:.1}",
            killer_entity, dead_entity, gold_gain, killer_gold.current
        );
    }
}
