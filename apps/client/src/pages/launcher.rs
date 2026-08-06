use std::cell::RefCell;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt};
use lol_web_protocol::agent::Agent;
use lol_web_protocol::scenario::{CreateScenarioDto, UpdateScenarioDto, Scenario};
use lol_web_protocol::spawn_preset::SpawnPreset as ProtoSpawnPreset;
use lol_web_protocol::{FrontAgentConfig, GameConfig};

use crate::components::sidebar::AppSidebar;
use crate::services::provider::{cloud_client, process_service};
use crate::types::{HeroPreset, SpawnPreset};

// ── 页面本地状态 ──

/// 单个阵营槽位：选手预设名 + 出生点预设名，二者相互独立。
#[derive(Debug, Clone, Default)]
struct LauncherSlot {
    hero_name: String,
    spawn_name: String,
}

struct LauncherPageState {
    loaded: bool,
    agents: Vec<Agent>,
    spawns: Vec<ProtoSpawnPreset>,
    scenarios: Vec<Scenario>,
    blue_slots: Vec<LauncherSlot>,
    red_slots: Vec<LauncherSlot>,
    scene_name: String,
    saving: bool,
    loading_scenario: bool,
    error: Option<String>,
    message: Option<String>,
}

impl Default for LauncherPageState {
    fn default() -> Self {
        Self {
            loaded: false,
            agents: Vec::new(),
            spawns: Vec::new(),
            scenarios: Vec::new(),
            blue_slots: vec![LauncherSlot::default()],
            red_slots: vec![LauncherSlot::default()],
            scene_name: "default_scenario".into(),
            saving: false,
            loading_scenario: false,
            error: None,
            message: None,
        }
    }
}

thread_local! {
    static STATE: RefCell<LauncherPageState> = RefCell::new(LauncherPageState::default());
}

fn with_state<R>(f: impl FnOnce(&LauncherPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

fn update_state(f: impl FnOnce(&mut LauncherPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

/// 渲染时快照，避免 borrow 逃逸。
struct LauncherView {
    loaded: bool,
    scene_name: String,
    scenarios: Vec<Scenario>,
    blue_slots: Vec<LauncherSlot>,
    red_slots: Vec<LauncherSlot>,
    agents: Vec<Agent>,
    spawns: Vec<ProtoSpawnPreset>,
    saving: bool,
    loading_scenario: bool,
    error: Option<String>,
    message: Option<String>,
}

fn snapshot() -> LauncherView {
    with_state(|s| LauncherView {
        loaded: s.loaded,
        scene_name: s.scene_name.clone(),
        scenarios: s.scenarios.clone(),
        blue_slots: s.blue_slots.clone(),
        red_slots: s.red_slots.clone(),
        agents: s.agents.clone(),
        spawns: s.spawns.clone(),
        saving: s.saving,
        loading_scenario: s.loading_scenario,
        error: s.error.clone(),
        message: s.message.clone(),
    })
}

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

fn build_all_agents(s: &LauncherPageState) -> Vec<FrontAgentConfig> {
    let mut agents = build_team_agents(s, "Order", &s.blue_slots);
    agents.extend(build_team_agents(s, "Chaos", &s.red_slots));
    agents
}

/// 反向匹配：从场景 agents 中识别选手预设名（对齐 useSlotConfig.matchHeroPreset）。
fn match_preset(agents: &[Agent], champion: &str, prompt: &str, agent_type: &str) -> String {
    for a in agents {
        if a.champion == champion
            && a.prompt == prompt
            && a.agent_type.as_str() == agent_type
        {
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

/// 首次加载：拉取英雄预设 / 出生点预设 / 场景列表，并回填 sidebar 全局字段。
fn spawn_initial_load(cx: &mut Context<AppSidebar>) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let (agents, spawns) = tokio::join!(
                    async { cloud_client().list_agents().await.unwrap_or_default() },
                    async { cloud_client().list_spawn_presets().await.unwrap_or_default() },
                );
                let scenarios = cloud_client().list_scenarios().await.unwrap_or_default();
                update_state(|s| {
                    s.agents = agents.clone();
                    s.spawns = spawns.clone();
                    s.scenarios = scenarios.clone();
                    s.loaded = true;
                    s.error = None;
                });
                if let Some(entity) = weak.upgrade() {
                    entity
                        .update(&mut cx, |sidebar, cx| {
                            sidebar.hero_presets = agents
                                .iter()
                                .map(|a| HeroPreset {
                                    name: a.name.clone(),
                                    hero: a.champion.clone(),
                                    agent_type: a.agent_type.as_str().to_string(),
                                })
                                .collect();
                            sidebar.spawn_presets = spawns
                                .iter()
                                .map(|sp| SpawnPreset {
                                    name: sp.name.clone(),
                                    x: sp.x,
                                    z: sp.z,
                                    team: sp.team.as_str().to_string(),
                                })
                                .collect();
                            cx.notify();
                        });
                }
            }
        },
    )
    .detach();
}

/// 保存当前阵容为场景：同名存在则更新，否则新建。
fn spawn_save_scenario(cx: &mut Context<AppSidebar>) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let (scene_name, agents_json, existing_id) = with_state(|s| {
                    let name = s.scene_name.trim().to_string();
                    let agents = build_all_agents(s);
                    let agents_json =
                        serde_json::to_value(&agents).unwrap_or(serde_json::Value::Null);
                    let existing_id = s
                        .scenarios
                        .iter()
                        .find(|sc| sc.name == name)
                        .map(|sc| sc.id.to_string());
                    (name, agents_json, existing_id)
                });
                let empty = agents_json
                    .as_array()
                    .map_or(true, |a| a.is_empty());
                let finish = |cx: &mut AsyncApp| {
                    if let Some(entity) = weak.upgrade() {
                        entity.update(cx, |_, cx| cx.notify());
                    }
                };
                if scene_name.is_empty() {
                    update_state(|s| {
                        s.saving = false;
                        s.error = Some("请输入场景名称".into());
                    });
                    finish(&mut cx);
                    return;
                }
                if empty {
                    update_state(|s| {
                        s.saving = false;
                        s.error = Some("请至少选择一个英雄预设".into());
                    });
                    finish(&mut cx);
                    return;
                }
                update_state(|s| {
                    s.saving = true;
                    s.error = None;
                    s.message = None;
                });
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
                        let scenarios =
                            cloud_client().list_scenarios().await.unwrap_or_default();
                        update_state(|s| {
                            s.scenarios = scenarios;
                            s.saving = false;
                            s.message = Some(format!("场景「{}」已保存", scene_name));
                            s.error = None;
                        });
                    }
                    Err(e) => update_state(|s| {
                        s.saving = false;
                        s.error = Some(format!("保存失败: {}", e));
                    }),
                }
                finish(&mut cx);
            }
        },
    )
    .detach();
}

/// 按 id 载入场景并回填蓝/红槽位。
fn spawn_load_scenario(cx: &mut App, weak: WeakEntity<AppSidebar>, id: String) {
    cx.spawn(move |cx: &mut gpui::AsyncApp| {
        let mut cx = cx.clone();
        async move {
            match cloud_client().get_scenario(&id).await {
            Ok(sc) => {
                let agents_json = sc.agents.clone();
                let scene_name = sc.name.clone();
                update_state(|s| {
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
                                let x = arr
                                    .get(0)
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0) as f32;
                                let z = arr
                                    .get(1)
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0) as f32;
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
                });
            }
            Err(e) => update_state(|s| {
                s.loading_scenario = false;
                s.error = Some(format!("载入失败: {}", e));
            }),
        }
        if let Some(entity) = weak.upgrade() {
            entity.update(&mut cx, |_, cx| cx.notify());
        }
        }
    })
    .detach();
}

// ── 渲染辅助 ──

fn champion_display(key: &str) -> String {
    match key {
        "Riven" => "瑞雯".to_string(),
        "Fiora" => "菲奥娜".to_string(),
        other => other.to_string(),
    }
}

fn unix_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 单个槽位卡片：英雄预设下拉 + 出生点下拉 + 删除。
fn slot_card(
    cx: &mut Context<AppSidebar>,
    team: &'static str,
    index: usize,
    slot: &LauncherSlot,
    agents: &[Agent],
    spawns: &[ProtoSpawnPreset],
) -> AnyElement {
    let weak = cx.entity().downgrade();
    let hero_label = if slot.hero_name.is_empty() {
        "选择英雄预设".to_string()
    } else {
        slot.hero_name.clone()
    };
    let spawn_label = if slot.spawn_name.is_empty() {
        "选择出生点".to_string()
    } else {
        slot.spawn_name.clone()
    };
    let subtitle = agents
        .iter()
        .find(|a| a.name == slot.hero_name)
        .map(|a| format!("{} · {}", a.champion, a.agent_type.as_str().to_uppercase()))
        .unwrap_or_default();

    // 英雄预设下拉
    let hero_agents = agents.to_vec();
    let hero_selected = slot.hero_name.clone();
    let hero_menu_weak = weak.clone();
    let hero_dropdown = Button::new(format!("{team}-{index}-hero"))
        .outline()
        .xsmall()
        .label(hero_label)
        .dropdown_menu(move |menu, _window, _cx| {
            let mut m = menu;
            if hero_agents.is_empty() {
                m = m.item(PopupMenuItem::new("暂无英雄预设").disabled(true));
            }
            for a in &hero_agents {
                let name = a.name.clone();
                let checked = name == hero_selected;
                let slot_team = team;
                let slot_idx = index;
                let weak = hero_menu_weak.clone();
                m = m.item(
                    PopupMenuItem::new(name.clone())
                        .checked(checked)
                        .on_click(move |_, _, cx| {
                            update_state(|s| {
                                if slot_team == "Order" {
                                    s.blue_slots[slot_idx].hero_name = name.clone();
                                } else {
                                    s.red_slots[slot_idx].hero_name = name.clone();
                                }
                            });
                            weak.update(cx, |_, cx| cx.notify()).ok();
                        }),
                );
            }
            m
        });

    // 出生点下拉
    let spawns_owned = spawns.to_vec();
    let spawn_selected = slot.spawn_name.clone();
    let spawn_menu_weak = weak.clone();
    let spawn_dropdown = Button::new(format!("{team}-{index}-spawn"))
        .outline()
        .xsmall()
        .label(spawn_label)
        .dropdown_menu(move |menu, _window, _cx| {
            let mut m = menu;
            if spawns_owned.is_empty() {
                m = m.item(PopupMenuItem::new("暂无出生点").disabled(true));
            }
            for sp in &spawns_owned {
                let sp_name = sp.name.clone();
                let label = format!("{} ({:.0}, {:.0})", sp.name, sp.x, sp.z);
                let checked = sp_name == spawn_selected;
                let slot_team = team;
                let slot_idx = index;
                let weak = spawn_menu_weak.clone();
                m = m.item(
                    PopupMenuItem::new(label)
                        .checked(checked)
                        .on_click(move |_, _, cx| {
                            update_state(|s| {
                                if slot_team == "Order" {
                                    s.blue_slots[slot_idx].spawn_name = sp_name.clone();
                                } else {
                                    s.red_slots[slot_idx].spawn_name = sp_name.clone();
                                }
                            });
                            weak.update(cx, |_, cx| cx.notify()).ok();
                        }),
                );
            }
            m
        });

    v_flex()
        .gap_1p5()
        .p_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .font_bold()
                                .child(format!("#{}", index + 1)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("槽位"),
                        ),
                )
                .child(
                    Button::new(format!("{team}-{index}-del"))
                        .ghost()
                        .xsmall()
                        .icon(IconName::Delete)
                        .on_click(cx.listener(move |_, _, _, cx| {
                            update_state(|s| {
                                if team == "Order" {
                                    s.blue_slots.remove(index);
                                } else {
                                    s.red_slots.remove(index);
                                }
                            });
                            cx.notify();
                        })),
                ),
        )
        .child(hero_dropdown)
        .child(spawn_dropdown)
        .when(!subtitle.is_empty(), |d| {
            d.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(subtitle),
            )
        })
        .into_any_element()
}

/// 阵营列容器：标题/圆点/计数 + 槽位列表 + 新增槽位。
fn team_column(
    cx: &mut Context<AppSidebar>,
    team: &'static str,
    label: &str,
    color: Hsla,
    slots: &[LauncherSlot],
    agents: &[Agent],
    spawns: &[ProtoSpawnPreset],
) -> AnyElement {
    v_flex()
        .flex_1()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(div().w_2().h_2().rounded_full().bg(color))
                        .child(div().text_xs().font_bold().child(label.to_string()))
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .bg(cx.theme().accent.opacity(0.15))
                                .text_xs()
                                .font_bold()
                                .text_color(cx.theme().accent)
                                .child(format!("{}", slots.len())),
                        ),
                )
                .child(
                    Button::new(format!("add-{team}-slot"))
                        .outline()
                        .xsmall()
                        .icon(IconName::Plus)
                        .label("添加槽位")
                        .on_click(cx.listener(move |_, _, _, cx| {
                            update_state(|s| {
                                let slot = LauncherSlot::default();
                                if team == "Order" {
                                    s.blue_slots.push(slot);
                                } else {
                                    s.red_slots.push(slot);
                                }
                            });
                            cx.notify();
                        })),
                ),
        )
        .child(
            v_flex()
                .gap_2()
                .p_2()
                .children(
                    slots
                        .iter()
                        .enumerate()
                        .map(|(i, slot)| slot_card(cx, team, i, slot, agents, spawns)),
                ),
        )
        .into_any_element()
}

// ── 页面入口 ──

/// 启动器页面：模式 + 场景 + 双阵营槽位编排 + 启动。
pub fn render_launcher(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let champ = sidebar.champion.clone();
    let mode = sidebar.game_mode.clone();
    let launch_error = sidebar.launch_error.clone();
    let starting = sidebar.is_starting_game;
    let champions = sidebar.champions_list.clone();
    let view = snapshot();

    if !view.loaded {
        spawn_initial_load(cx);
    }

    // 载入场景下拉
    let weak = cx.entity().downgrade();
    let scenarios = view.scenarios.clone();
    let load_dropdown = Button::new("launcher-load-scenario")
        .outline()
        .xsmall()
        .icon(IconName::Folder)
        .label("载入")
        .disabled(view.loading_scenario)
        .dropdown_menu(move |menu, _window, _cx| {
            let mut m = menu;
            if scenarios.is_empty() {
                m = m.item(PopupMenuItem::new("暂无场景").disabled(true));
            }
            for sc in &scenarios {
                let sc_id = sc.id.to_string();
                let sc_name = sc.name.clone();
                let weak = weak.clone();
                m = m.item(
                    PopupMenuItem::new(sc_name).on_click(move |_, _, cx| {
                        update_state(|s| s.loading_scenario = true);
                        spawn_load_scenario(cx, weak.clone(), sc_id.clone());
                    }),
                );
            }
            m
        });

    v_flex()
        .size_full()
        .flex_1()
        .gap_6()
        .overflow_y_scrollbar()
        // ── 标题行 ──
        .child(
            h_flex().items_center().justify_between().child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Play)
                    .child(div().font_bold().text_lg().child("启动器")),
            ),
        )
        // ── 模式与英雄选择 ──
        .child(
            h_flex()
                .gap_6()
                .items_start()
                .child(
                    v_flex()
                        .gap_2()
                        .child(div().font_bold().text_sm().child("模式"))
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("mode-agent")
                                        .when(mode == "agent", |b| b.primary())
                                        .when(mode != "agent", |b| b.outline())
                                        .label("Agent 模式")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.game_mode = "agent".into();
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Button::new("mode-sandbox")
                                        .when(mode == "sandbox", |b| b.primary())
                                        .when(mode != "sandbox", |b| b.outline())
                                        .label("沙盒模式")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.game_mode = "sandbox".into();
                                            cx.notify();
                                        })),
                                ),
                        ),
                )
                .child(
                    v_flex()
                        .gap_2()
                        .child(div().font_bold().text_sm().child("英雄"))
                        .child(
                            h_flex()
                                .gap_2()
                                .flex_wrap()
                                .children(champions.iter().map(|c| {
                                    let selected = *c == champ;
                                    let champ_clone = c.clone();
                                    Button::new(format!("champ-{}", c))
                                        .when(selected, |b| b.primary())
                                        .when(!selected, |b| b.outline())
                                        .label(champion_display(c))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.champion = champ_clone.clone();
                                            cx.notify();
                                        }))
                                })),
                        ),
                ),
        )
        // ── 场景栏：名称 + 载入 / 保存 / 新建 ──
        .child(
            v_flex()
                .gap_2()
                .child(div().font_bold().text_sm().child("场景"))
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .flex_1()
                                .px_3()
                                .py_2()
                                .rounded_md()
                                .border_1()
                                .border_color(cx.theme().border)
                                .text_sm()
                                .child(view.scene_name.clone()),
                        )
                        .child(load_dropdown)
                        .child(
                            Button::new("launcher-save-scenario")
                                .outline()
                                .xsmall()
                                .icon(IconName::Check)
                                .label("保存")
                                .disabled(view.saving)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    spawn_save_scenario(cx);
                                })),
                        )
                        .child(
                            Button::new("launcher-new-scenario")
                                .outline()
                                .xsmall()
                                .icon(IconName::Plus)
                                .label("新建")
                                .on_click(cx.listener(|_, _, _, cx| {
                                    update_state(|s| {
                                        s.scene_name = format!("custom_agents_{}", unix_ts());
                                        s.blue_slots = vec![LauncherSlot::default()];
                                        s.red_slots = vec![LauncherSlot::default()];
                                        s.error = None;
                                        s.message = None;
                                    });
                                    cx.notify();
                                })),
                        ),
                ),
        )
        // ── 页面提示（载入/保存结果）──
        .when_some(view.message.clone(), |d, msg| {
            d.child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().accent)
                    .bg(cx.theme().accent.opacity(0.1))
                    .text_xs()
                    .text_color(cx.theme().accent)
                    .child(msg),
            )
        })
        .when_some(view.error.clone(), |d, err| {
            d.child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().danger)
                    .bg(cx.theme().danger.opacity(0.1))
                    .text_xs()
                    .text_color(cx.theme().danger)
                    .child(err),
            )
        })
        .when_some(launch_error, |d, err| {
            d.child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().danger)
                    .bg(cx.theme().danger.opacity(0.1))
                    .text_xs()
                    .text_color(cx.theme().danger)
                    .child(err),
            )
        })
        // ── 双阵营槽位 ──
        .child(
            h_flex()
                .gap_4()
                .items_start()
                .child(team_column(
                    cx,
                    "Order",
                    "蓝色方 (Order)",
                    gpui::hsla(0.6, 0.7, 0.5, 1.0),
                    &view.blue_slots,
                    &view.agents,
                    &view.spawns,
                ))
                .child(team_column(
                    cx,
                    "Chaos",
                    "红色方 (Chaos)",
                    gpui::hsla(0.0, 0.7, 0.5, 1.0),
                    &view.red_slots,
                    &view.agents,
                    &view.spawns,
                )),
        )
        // ── 启动按钮 ──
        .child(
            h_flex().gap_2().child(
                Button::new("launch-game-btn")
                    .primary()
                    .icon(if starting {
                        IconName::Loader
                    } else {
                        IconName::Play
                    })
                    .label(if starting {
                        "启动中…".to_string()
                    } else {
                        "启动对局".to_string()
                    })
                    .disabled(starting)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let mode = this.game_mode.clone();
                        let champ = this.champion.clone();
                        let (scene_name, agents) =
                            with_state(|s| (s.scene_name.trim().to_string(), build_all_agents(s)));
                        if agents.is_empty() {
                            this.launch_error = Some("请至少选择一个英雄预设".into());
                            cx.notify();
                            return;
                        }
                        let scene_name = if scene_name.is_empty() {
                            format!("custom_agents_{}", unix_ts())
                        } else {
                            scene_name
                        };
                        this.is_starting_game = true;
                        this.launch_error = None;
                        cx.notify();
                        cx.spawn(
                            move |weak: gpui::WeakEntity<AppSidebar>,
                                  cx: &mut gpui::AsyncApp| {
                                let mut cx = cx.clone();
                                let mode = mode.clone();
                                let champ = champ.clone();
                                async move {
                                    let config = GameConfig {
                                        mode: mode.clone(),
                                        champion: champ.clone(),
                                        scene_name: Some(scene_name.clone()),
                                        agents: Some(agents.clone()),
                                        providers: None,
                                    };
                                    match process_service().start(config).await {
                                        Ok(game) => {
                                            if let Some(entity) = weak.upgrade() {
                                                entity
                                                    .update(&mut cx, |sidebar, cx| {
                                                        sidebar.is_starting_game = false;
                                                        sidebar.current_game_id =
                                                            Some(game.id.clone());
                                                        sidebar
                                                            .running_games
                                                            .push(crate::types::RunningGameInfo {
                                                                id: game.id,
                                                                mode: mode.clone(),
                                                                champion: champ.clone(),
                                                                port: game.port as u16,
                                                            });
                                                        cx.notify();
                                                    });
                                            }
                                        }
                                        Err(e) => {
                                            if let Some(entity) = weak.upgrade() {
                                                entity
                                                    .update(&mut cx, |sidebar, cx| {
                                                        sidebar.is_starting_game = false;
                                                        sidebar.launch_error =
                                                            Some(format!("启动失败: {}", e));
                                                        cx.notify();
                                                    });
                                            }
                                        }
                                    }
                                }
                            },
                        )
                        .detach();
                    })),
            ),
        )
        .into_any_element()
}
