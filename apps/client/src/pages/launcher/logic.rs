//! 启动器页数据逻辑：阵容组装（toBackend / expandSlot）+ 场景保存/载入异步动作。

use gpui::prelude::*;
use gpui::*;
use lol_web_protocol::agent::Agent;
use lol_web_protocol::scenario::{CreateScenarioDto, UpdateScenarioDto};
use lol_web_protocol::FrontAgentConfig;

use super::types::{LauncherPageState, LauncherSlot};
use crate::components::sidebar::AppSidebar;
use crate::services::provider::cloud_client;

// ── 组装逻辑（对应前端 useSlotConfig.toBackend / expandSlot）──

fn default_spawn(team: &str) -> Vec<f32> {
    if team == "Order" {
        vec![1981.0, 11441.0]
    } else {
        vec![3318.0, 12875.0]
    }
}

/// 把单个阵营槽位展开为后端的 FrontAgentConfig。
fn build_team_agents(
    s: &LauncherPageState,
    team: &str,
    slots: &[LauncherSlot],
) -> Vec<FrontAgentConfig> {
    slots
        .iter()
        .filter_map(|slot| {
            if slot.hero_name.is_empty() {
                return None;
            }
            let hero = s.agents.iter().find(|a| a.name == slot.hero_name)?;
            if hero.champion.is_empty() {
                return None;
            }
            let spawn = s.spawns.iter().find(|sp| sp.name == slot.spawn_name);
            let spawn_point = spawn
                .map(|sp| vec![sp.x, sp.z])
                .unwrap_or_else(|| default_spawn(team));
            let config_json = hero.config_json.clone();
            let provider_id = config_json
                .as_ref()
                .and_then(|v| v.get("provider_id"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            Some(FrontAgentConfig {
                id: None,
                champion: hero.champion.clone(),
                team: team.to_string(),
                prompt: hero.prompt.clone(),
                spawn_point,
                agent_type: hero.agent_type.as_str().to_string(),
                model: hero.model.clone(),
                provider_id,
                config_json,
            })
        })
        .collect()
}

pub(super) fn build_all_agents(s: &LauncherPageState) -> Vec<FrontAgentConfig> {
    let mut agents = build_team_agents(s, "Order", &s.blue_slots);
    agents.extend(build_team_agents(s, "Chaos", &s.red_slots));
    agents
}

/// 反向匹配：从场景 agents 中识别选手预设名（对齐 useSlotConfig.matchHeroPreset）。
fn match_preset(agents: &[Agent], champion: &str, prompt: &str, agent_type: &str) -> String {
    for a in agents {
        if a.champion == champion && a.prompt == prompt && a.agent_type.as_str() == agent_type {
            return a.name.clone();
        }
    }
    agents
        .iter()
        .find(|a| a.champion == champion)
        .map(|a| a.name.clone())
        .unwrap_or_default()
}

// ── 异步动作 ──

/// 首次加载：拉取英雄预设 / 出生点预设 / 场景列表。
pub(super) fn spawn_initial_load(cx: &mut Context<AppSidebar>) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let (agents, spawns) = tokio::join!(
                    async { cloud_client().list_agents().await.unwrap_or_default() },
                    async {
                        cloud_client()
                            .list_spawn_presets()
                            .await
                            .unwrap_or_default()
                    },
                );
                let scenarios = cloud_client().list_scenarios().await.unwrap_or_default();
                weak.update(&mut cx, |this, cx| {
                    this.launcher.agents = agents;
                    this.launcher.spawns = spawns;
                    this.launcher.scenarios = scenarios;
                    this.launcher.loaded = true;
                    this.launcher.error = None;
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

/// 保存当前阵容为场景：同名存在则更新，否则新建。
pub(super) fn spawn_save_scenario(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let (scene_name, agents_json, existing_id) = {
        let s = &sidebar.launcher;
        let name = s.scene_name.trim().to_string();
        let agents = build_all_agents(s);
        let agents_json = serde_json::to_value(&agents).unwrap_or(serde_json::Value::Null);
        let existing_id = s
            .scenarios
            .iter()
            .find(|sc| sc.name == name)
            .map(|sc| sc.id.to_string());
        (name, agents_json, existing_id)
    };
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let empty = agents_json.as_array().map_or(true, |a| a.is_empty());
                if scene_name.is_empty() {
                    weak.update(&mut cx, |this, cx| {
                        this.launcher.saving = false;
                        this.launcher.error = Some("请输入场景名称".into());
                        cx.notify();
                    })
                    .ok();
                    return;
                }
                if empty {
                    weak.update(&mut cx, |this, cx| {
                        this.launcher.saving = false;
                        this.launcher.error = Some("请至少选择一个英雄预设".into());
                        cx.notify();
                    })
                    .ok();
                    return;
                }
                weak.update(&mut cx, |this, _| {
                    this.launcher.saving = true;
                    this.launcher.error = None;
                    this.launcher.message = None;
                })
                .ok();
                let result = match existing_id {
                    Some(id) => cloud_client()
                        .update_scenario(
                            &id,
                            &UpdateScenarioDto {
                                name: None,
                                agents: Some(agents_json),
                            },
                        )
                        .await
                        .map(|_| ()),
                    None => cloud_client()
                        .create_scenario(&CreateScenarioDto {
                            name: scene_name.clone(),
                            agents: agents_json,
                        })
                        .await
                        .map(|_| ()),
                };
                match result {
                    Ok(()) => {
                        let scenarios = cloud_client().list_scenarios().await.unwrap_or_default();
                        weak.update(&mut cx, |this, cx| {
                            this.launcher.scenarios = scenarios;
                            this.launcher.saving = false;
                            this.launcher.message = Some(format!("场景「{}」已保存", scene_name));
                            this.launcher.error = None;
                            cx.notify();
                        })
                        .ok();
                    }
                    Err(e) => {
                        weak.update(&mut cx, |this, cx| {
                            this.launcher.saving = false;
                            this.launcher.error = Some(format!("保存失败: {}", e));
                            cx.notify();
                        })
                        .ok();
                    }
                }
            }
        },
    )
    .detach();
}

/// 按 id 载入场景并回填蓝/红槽位。
pub(super) fn spawn_load_scenario(cx: &mut App, weak: WeakEntity<AppSidebar>, id: String) {
    cx.spawn(move |cx: &mut gpui::AsyncApp| {
        let mut cx = cx.clone();
        async move {
            match cloud_client().get_scenario(&id).await {
                Ok(sc) => {
                    let agents_json = sc.agents.clone();
                    let scene_name = sc.name.clone();
                    weak.update(&mut cx, |this, cx| {
                        let s = &mut this.launcher;
                        s.loading_scenario = false;
                        s.error = None;
                        s.message = None;
                        let arr = agents_json.as_array().cloned().unwrap_or_default();
                        let mut blue = Vec::new();
                        let mut red = Vec::new();
                        for a in arr {
                            let team = a
                                .get("team")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let champion = a
                                .get("champion")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let prompt = a
                                .get("prompt")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let agent_type = a
                                .get("agent_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("llm")
                                .to_string();
                            let hero_name =
                                match_preset(&s.agents, &champion, &prompt, &agent_type);
                            let (x, z) = a
                                .get("spawn_point")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    let x =
                                        arr.get(0).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                    let z =
                                        arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                                    (x, z)
                                })
                                .unwrap_or((0.0, 0.0));
                            let spawn_name = s
                                .spawns
                                .iter()
                                .find(|sp| (sp.x - x).abs() < 1.0 && (sp.z - z).abs() < 1.0)
                                .map(|sp| sp.name.clone())
                                .unwrap_or_default();
                            let slot = LauncherSlot {
                                hero_name,
                                spawn_name,
                            };
                            if team == "Order" {
                                blue.push(slot);
                            } else {
                                red.push(slot);
                            }
                        }
                        if blue.is_empty() {
                            blue.push(LauncherSlot::default());
                        }
                        if red.is_empty() {
                            red.push(LauncherSlot::default());
                        }
                        s.blue_slots = blue;
                        s.red_slots = red;
                        s.scene_name = scene_name.clone();
                        s.message = Some(format!("已载入场景「{}」", scene_name));
                        cx.notify();
                    })
                    .ok();
                }
                Err(e) => {
                    weak.update(&mut cx, |this, cx| {
                        this.launcher.loading_scenario = false;
                        this.launcher.error = Some(format!("载入失败: {}", e));
                        cx.notify();
                    })
                    .ok();
                }
            }
        }
    })
    .detach();
}
