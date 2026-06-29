//! 对局事件产出插件：将游戏内的击杀、推塔、补刀里程碑转化为结构化事件，
//! 供 web server 侧的 match supervisor 订阅并套用 SOLO 胜负规则。
//!
//! 设计要点：
//! - 胜负判定不在 Bevy 进程内（见 docs/product/match/arch.md）。本插件只**产出**
//!   事件，不做终局判定。
//! - 复用已有数据：`EventDead`（life.rs）覆盖英雄/防御塔死亡；`ChampionStats`
//!   （base/stats.rs）已跟踪 kills / minion_kills。
//! - 一血 / 一塔由 web server 判定"首个"（跨事件状态），本插件只上报"发生了
//!   击杀/推塔"及其所属阵营，避免在引擎侧维护全局先后顺序。
//! - 补刀阈值（默认 100）一旦达到即上报 cs_threshold；上限事件只发一次（用
//!   [`CsThresholdReached`] 标记组件去重）。
//! - 事件经 [`MatchEventChannel`]（async_channel）对外暴露，由 lol_server 的
//!   poll 系统读取并转发到 WS 客户端。本 crate 不依赖 lol_server，避免环依赖。

use async_channel::Sender;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::base::stats::ChampionStats;
use crate::entities::champion::Champion;
use crate::entities::turret::Turret;
use crate::life::{Death, EventDead};
use crate::team::Team;

/// 补刀获胜阈值（与 docs/product/match/product.md §3.C 一致）。
pub const CS_TARGET: u32 = 100;

/// 引擎产出的对局事件。由 lol_server 转发到 WS，供 match supervisor 消费。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MatchEventOut {
    /// 英雄被击杀。`killer_team` 为击杀者阵营（拿人头方）。
    /// 一血判定由 web server 做（首个 champion_kill 即一血）。
    ChampionKill { killer_team: Team },
    /// 防御塔被摧毁。`killer_team` 为推塔方。
    /// 一塔判定由 web server 做（首个 turret_destroyed 即一塔）。
    TurretDestroyed { killer_team: Team },
    /// 某方补刀达到阈值（CS_TARGET）。每方只发一次。
    CsThreshold { team: Team, cs: u32 },
    /// 对局时间推进（秒）。由 supervisor 累积用于 15 分钟超时判定。
    /// 周期性上报，频率较低（每秒一次）。
    TimeProgress { elapsed_secs: f64 },
}

/// 对局事件输出通道（Resource）。由 lol_server 在启动时注入 Sender。
/// lol_core 只负责 send，不持有 Receiver。
#[derive(Resource)]
pub struct MatchEventChannel {
    pub tx: Sender<MatchEventOut>,
}

/// 标记某英雄已上报过补刀阈值事件，避免重复发送。
#[derive(Component, Default)]
pub struct CsThresholdReached;

/// 对局事件产出插件。
/// 注意：需要外部（lol_server）在启动后插入 [`MatchEventChannel`] 资源，
/// 否则事件将被静默丢弃（插件对缺失通道安全降级）。
#[derive(Default)]
pub struct PluginMatchEvents;

impl Plugin for PluginMatchEvents {
    fn build(&self, app: &mut App) {
        app.add_observer(on_event_dead);
        app.add_systems(FixedUpdate, check_cs_threshold);
        app.init_resource::<TimeProgressAccumulator>();
        app.add_systems(FixedUpdate, report_time_progress);
    }
}

/// 把事件送入通道；通道缺失时静默丢弃。
fn emit(ch: &MatchEventChannel, event: MatchEventOut) {
    let _ = ch.tx.try_send(event);
}

/// 监听死亡事件，产出英雄击杀 / 推塔事件。
fn on_event_dead(
    trigger: On<EventDead>,
    q_champion: Query<&Team, With<Champion>>,
    q_turret: Query<&Team, With<Turret>>,
    channel: Option<Res<MatchEventChannel>>,
) {
    let Some(ch) = channel else { return };
    let dead_entity = trigger.entity;

    // 英雄被击杀：上报击杀者阵营。
    if q_champion.get(dead_entity).is_ok() {
        if let Some(killer_entity) = trigger.killer {
            if let Ok(&killer_team) = q_champion.get(killer_entity) {
                emit(&ch, MatchEventOut::ChampionKill { killer_team });
            }
        }
        return;
    }

    // 防御塔被摧毁：上报推塔方阵营。
    if q_turret.get(dead_entity).is_ok() {
        // 击杀者可能是英雄；若拿不到阵营（泉水/小兵击杀），用被毁塔的对方阵营。
        let killer_team = trigger
            .killer
            .and_then(|k| q_champion.get(k).ok().copied())
            .or_else(|| {
                q_turret.get(dead_entity).ok().copied().map(|t| match t {
                    Team::Order => Team::Chaos,
                    Team::Chaos => Team::Order,
                    Team::Neutral => Team::Neutral,
                })
            });
        if let Some(killer_team) = killer_team {
            emit(&ch, MatchEventOut::TurretDestroyed { killer_team });
        }
    }
    // 小兵死亡不计入对局事件（补刀里程碑由 check_cs_threshold 统一处理）。
}

/// 检查每方补刀是否达到阈值，首次达到时上报（用 CsThresholdReached 去重）。
fn check_cs_threshold(
    mut q: Query<(Entity, &ChampionStats, &Team, Option<&CsThresholdReached>), Without<Death>>,
    mut commands: Commands,
    channel: Option<Res<MatchEventChannel>>,
) {
    let Some(ch) = channel else { return };
    for (entity, stats, team, reached) in q.iter_mut() {
        if reached.is_some() {
            continue;
        }
        if stats.minion_kills >= CS_TARGET {
            let _ = ch.tx.try_send(MatchEventOut::CsThreshold {
                team: *team,
                cs: stats.minion_kills,
            });
            commands.entity(entity).insert(CsThresholdReached);
        }
    }
}

#[derive(Resource, Default)]
struct TimeProgressAccumulator(f32);

/// 周期性上报对局时间，供 supervisor 判 15 分钟超时。
fn report_time_progress(
    time: Res<Time<Fixed>>,
    mut acc: ResMut<TimeProgressAccumulator>,
    channel: Option<Res<MatchEventChannel>>,
) {
    let Some(ch) = channel else { return };
    acc.0 += time.delta_secs();
    if acc.0 >= 1.0 {
        let _ = ch.tx.try_send(MatchEventOut::TimeProgress {
            elapsed_secs: time.elapsed_secs() as f64,
        });
        acc.0 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use async_channel::{Receiver, Sender, unbounded};

    use super::*;

    /// 收集通道中所有待读事件。
    fn drain(rx: &Receiver<MatchEventOut>) -> Vec<MatchEventOut> {
        let mut out = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            out.push(ev);
        }
        out
    }

    /// 构建一个挂载了 PluginMatchEvents + MatchEventChannel 的 app，返回 (app, rx)。
    fn setup_app() -> (App, Receiver<MatchEventOut>) {
        let (tx, rx) = unbounded::<MatchEventOut>();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginMatchEvents);
        app.insert_resource(MatchEventChannel { tx });
        (app, rx)
    }

    #[test]
    fn champion_kill_emits_event() {
        let (mut app, rx) = setup_app();

        let killer = app.world_mut().spawn((Champion, Team::Order)).id();
        let victim = app.world_mut().spawn((Champion, Team::Chaos)).id();

        app.world_mut().trigger(EventDead {
            entity: victim,
            killer: Some(killer),
        });
        app.update();

        let events = drain(&rx);
        assert!(events.iter().any(|e| matches!(
            e,
            MatchEventOut::ChampionKill {
                killer_team: Team::Order
            }
        )));
    }

    #[test]
    fn turret_destroyed_emits_event() {
        let (mut app, rx) = setup_app();

        let killer = app.world_mut().spawn((Champion, Team::Chaos)).id();
        let turret = app.world_mut().spawn((Turret, Team::Order)).id();

        app.world_mut().trigger(EventDead {
            entity: turret,
            killer: Some(killer),
        });
        app.update();

        let events = drain(&rx);
        assert!(events.iter().any(|e| matches!(
            e,
            MatchEventOut::TurretDestroyed {
                killer_team: Team::Chaos
            }
        )));
    }

    #[test]
    fn cs_threshold_emits_once() {
        use bevy::ecs::system::RunSystemOnce;
        let (mut app, rx) = setup_app();
        let _champ = app.world_mut().spawn((
            Champion,
            Team::Order,
            ChampionStats {
                minion_kills: CS_TARGET,
                ..default()
            },
        ));

        // 直接运行系统一次，避免对 FixedUpdate 调度的依赖。
        app.world_mut().run_system_once(check_cs_threshold);
        let count = drain(&rx)
            .iter()
            .filter(|e| matches!(e, MatchEventOut::CsThreshold { .. }))
            .count();
        assert_eq!(count, 1);

        // 第二次不应再发（已标记 CsThresholdReached）。
        app.world_mut().run_system_once(check_cs_threshold);
        let count2 = drain(&rx)
            .iter()
            .filter(|e| matches!(e, MatchEventOut::CsThreshold { .. }))
            .count();
        assert_eq!(count2, 0);
    }

    #[test]
    fn cs_below_threshold_no_event() {
        use bevy::ecs::system::RunSystemOnce;
        let (mut app, rx) = setup_app();
        app.world_mut().spawn((
            Champion,
            Team::Order,
            ChampionStats {
                minion_kills: CS_TARGET - 1,
                ..default()
            },
        ));

        app.world_mut().run_system_once(check_cs_threshold);
        let count = drain(&rx)
            .iter()
            .filter(|e| matches!(e, MatchEventOut::CsThreshold { .. }))
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn missing_channel_silently_drops() {
        // 不插入 MatchEventChannel：系统应安全降级，不 panic。
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginMatchEvents);
        app.world_mut().spawn((
            Champion,
            Team::Order,
            ChampionStats {
                minion_kills: CS_TARGET,
                ..default()
            },
        ));
        app.update(); // 不应 panic
    }

    // 抑制未使用导入告警（Sender 在签名中用到）。
    #[allow(dead_code)]
    fn _ensure_sender_used(_s: Sender<MatchEventOut>) {}
}
