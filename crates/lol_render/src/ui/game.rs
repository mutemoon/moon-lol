use bevy::prelude::*;
use lol_base::ui::{LOLEnumUiMetric, LOLLolGameStateViewController, LOLUiElementTextData};
use lol_core::base::stats::ChampionStats;
use lol_core::team::Team;

use crate::controller::Controller;
use crate::ui::element::{UIElementEntity, UIState};
use crate::ui::text::UiTextState;

#[derive(Default)]
pub struct PluginUIGame;

impl Plugin for PluginUIGame {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_game_metrics.run_if(in_state(UIState::Loaded)),
        );
    }
}

fn update_game_metrics(
    res_game_state: Res<LOLLolGameStateViewController>,
    res_ui_element_entity: Res<UIElementEntity>,
    time: Res<Time>,
    q_stats: Query<(&ChampionStats, Option<&Team>)>,
    q_player_stats: Query<&ChampionStats, With<Controller>>,
    mut q_ui_text_state: Query<&mut UiTextState>,
) {
    // 格式化游戏时间 (分:秒)
    let elapsed = time.elapsed_secs() as u32;
    let time_str = format!("{:02}:{:02}", elapsed / 60, elapsed % 60);

    // 模拟或计算实时 FPS，限制在合理的心跳帧区间 [60, 240]
    let delta = time.delta_secs();
    let raw_fps = if delta > 0.0 {
        (1.0 / delta) as u32
    } else {
        144
    };
    let fps_str = format!("{}", raw_fps.clamp(60, 240));

    // 模拟略带微小抖动的网络延迟 (以 28ms 为基准波动)
    let latency_str = format!("{}", 28 + (elapsed % 3));

    // 获取玩家控制英雄的 KDA
    let (kills, deaths, assists) = if let Ok(stats) = q_player_stats.single() {
        (stats.kills, stats.deaths, stats.assists)
    } else {
        (0, 0, 0)
    };
    let kda_str = format!("{}/{}/{}", kills, deaths, assists);

    // 获取玩家控制英雄的补刀数 (Creep Score)
    let cs = if let Ok(stats) = q_player_stats.single() {
        stats.minion_kills
    } else {
        0
    };
    let cs_str = cs.to_string();

    // 统计红蓝两队的击杀总数
    let mut team1_kills = 0;
    let mut team2_kills = 0;
    for (stats, team) in &q_stats {
        let Some(t) = team else {
            continue;
        };
        match t {
            Team::Order => team1_kills += stats.kills,
            Team::Chaos => team2_kills += stats.kills,
            _ => {}
        }
    }
    let team1_kills_str = team1_kills.to_string();
    let team2_kills_str = team2_kills.to_string();

    // 遍历 UI 排版并更新各项指标
    for metric in &res_game_state.metrics {
        match metric {
            LOLEnumUiMetric::UiMetricFps(m) => {
                update_text_element(
                    &res_ui_element_entity,
                    &m.fps_text,
                    &fps_str,
                    &mut q_ui_text_state,
                );
            }
            LOLEnumUiMetric::UiMetricLatencyText(m) => {
                update_text_element(
                    &res_ui_element_entity,
                    &m.latency_text,
                    &latency_str,
                    &mut q_ui_text_state,
                );
            }
            LOLEnumUiMetric::UiMetricKda(m) => {
                update_text_element(
                    &res_ui_element_entity,
                    &m.text,
                    &kda_str,
                    &mut q_ui_text_state,
                );
            }
            LOLEnumUiMetric::UiMetricTeamKills(m) => {
                update_text_element(
                    &res_ui_element_entity,
                    &m.team1_kill_text,
                    &team1_kills_str,
                    &mut q_ui_text_state,
                );
                update_text_element(
                    &res_ui_element_entity,
                    &m.team2_kill_text,
                    &team2_kills_str,
                    &mut q_ui_text_state,
                );
            }
            LOLEnumUiMetric::UiMetricCreepScore(m) => {
                update_text_element(
                    &res_ui_element_entity,
                    &m.text,
                    &cs_str,
                    &mut q_ui_text_state,
                );
            }
            LOLEnumUiMetric::UiMetricGameTime(m) => {
                update_text_element(
                    &res_ui_element_entity,
                    &m.time_text,
                    &time_str,
                    &mut q_ui_text_state,
                );
            }
            _ => {}
        }
    }
}

/// 辅助函数：更新文本元素的内容
fn update_text_element(
    res_ui: &UIElementEntity,
    key: &lol_base::hash_key::HashKey<LOLUiElementTextData>,
    new_text: &str,
    q_ui_text_state: &mut Query<&mut UiTextState>,
) {
    let entity = res_ui.get_entity(key);
    let Ok(mut text_state) = q_ui_text_state.get_mut(entity) else {
        return;
    };
    text_state.text = new_text.to_string();
}
