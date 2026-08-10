//! 启动器页渲染辅助：槽位卡片 / 阵营列 / 各区块 UI。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt};
use lol_web_protocol::agent::Agent;
use lol_web_protocol::spawn_preset::SpawnPreset as ProtoSpawnPreset;
use lol_web_protocol::GameConfig;

use super::logic::{build_all_agents, spawn_load_scenario, spawn_save_scenario};
use super::types::{update_state, with_state, LauncherSlot, LauncherView};
use crate::components::sidebar::AppSidebar;
use crate::services::provider::process_service;
use crate::types::RunningGameInfo;

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
                m = m.item(PopupMenuItem::new(name.clone()).checked(checked).on_click(
                    move |_, _, cx| {
                        update_state(|s| {
                            if slot_team == "Order" {
                                s.blue_slots[slot_idx].hero_name = name.clone();
                            } else {
                                s.red_slots[slot_idx].hero_name = name.clone();
                            }
                        });
                        weak.update(cx, |_, cx| cx.notify()).ok();
                    },
                ));
            }
            m
        });

    // 出生点下拉
    let spawns_owned = spawns.to_vec();
    let spawn_selected = slot.spawn_name.clone();
    let spawn_menu_weak = weak.clone();
    let spawn_dropdown =
        Button::new(format!("{team}-{index}-spawn"))
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
                    m = m.item(PopupMenuItem::new(label).checked(checked).on_click(
                        move |_, _, cx| {
                            update_state(|s| {
                                if slot_team == "Order" {
                                    s.blue_slots[slot_idx].spawn_name = sp_name.clone();
                                } else {
                                    s.red_slots[slot_idx].spawn_name = sp_name.clone();
                                }
                            });
                            weak.update(cx, |_, cx| cx.notify()).ok();
                        },
                    ));
                }
                m
            });

    let delete_btn = Button::new(format!("{team}-{index}-del"))
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
        }));

    let mut card = v_flex()
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
                        .child(div().text_xs().font_bold().child(format!("#{}", index + 1)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("槽位"),
                        ),
                )
                .child(delete_btn),
        )
        .child(hero_dropdown)
        .child(spawn_dropdown);

    if !subtitle.is_empty() {
        card = card.child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(subtitle),
        );
    }

    card.into_any_element()
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
    let header = h_flex()
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
        );

    let slots_list = v_flex().gap_2().p_2().children(
        slots
            .iter()
            .enumerate()
            .map(|(i, slot)| slot_card(cx, team, i, slot, agents, spawns)),
    );

    v_flex()
        .flex_1()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .overflow_hidden()
        .child(header)
        .child(slots_list)
        .into_any_element()
}

// ── 模块化子组件 ──

pub(super) fn render_header() -> AnyElement {
    h_flex()
        .items_center()
        .justify_between()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(IconName::Play)
                .child(div().font_bold().text_lg().child("启动器")),
        )
        .into_any_element()
}

pub(super) fn render_mode_and_champion(
    mode: &str,
    champ: &str,
    champions: &[String],
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let mode_section = v_flex()
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
        );

    let champ_section = v_flex()
        .gap_2()
        .child(div().font_bold().text_sm().child("英雄"))
        .child(
            h_flex()
                .gap_2()
                .flex_wrap()
                .children(champions.iter().map(|c| {
                    let selected = c == champ;
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
        );

    h_flex()
        .gap_6()
        .items_start()
        .child(mode_section)
        .child(champ_section)
        .into_any_element()
}

pub(super) fn render_load_dropdown(view: &LauncherView, cx: &mut Context<AppSidebar>) -> AnyElement {
    let weak = cx.entity().downgrade();
    let scenarios = view.scenarios.clone();
    Button::new("launcher-load-scenario")
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
                m = m.item(PopupMenuItem::new(sc_name).on_click(move |_, _, cx| {
                    update_state(|s| s.loading_scenario = true);
                    spawn_load_scenario(cx, weak.clone(), sc_id.clone());
                }));
            }
            m
        })
        .into_any_element()
}

pub(super) fn render_scene_section(
    view: &LauncherView,
    load_dropdown: AnyElement,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
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
        )
        .into_any_element()
}

pub(super) fn render_message_banners(
    view: &LauncherView,
    launch_error: Option<String>,
    cx: &mut Context<AppSidebar>,
) -> Vec<AnyElement> {
    let mut banners = Vec::new();

    if let Some(msg) = view.message.clone() {
        banners.push(
            div()
                .px_3()
                .py_2()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().accent)
                .bg(cx.theme().accent.opacity(0.1))
                .text_xs()
                .text_color(cx.theme().accent)
                .child(msg)
                .into_any_element(),
        );
    }

    if let Some(err) = view.error.clone() {
        banners.push(
            div()
                .px_3()
                .py_2()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().danger)
                .bg(cx.theme().danger.opacity(0.1))
                .text_xs()
                .text_color(cx.theme().danger)
                .child(err)
                .into_any_element(),
        );
    }

    if let Some(err) = launch_error {
        banners.push(
            div()
                .px_3()
                .py_2()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().danger)
                .bg(cx.theme().danger.opacity(0.1))
                .text_xs()
                .text_color(cx.theme().danger)
                .child(err)
                .into_any_element(),
        );
    }

    banners
}

pub(super) fn render_teams_section(view: &LauncherView, cx: &mut Context<AppSidebar>) -> AnyElement {
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
        ))
        .into_any_element()
}

pub(super) fn render_action_buttons(starting: bool, cx: &mut Context<AppSidebar>) -> AnyElement {
    let launch_game_btn = Button::new("launch-game-btn")
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
                move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
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
                                    entity.update(&mut cx, |sidebar, cx| {
                                        sidebar.is_starting_game = false;
                                        sidebar.current_game_id = Some(game.id.clone());
                                        sidebar.running_games.push(RunningGameInfo {
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
                                    entity.update(&mut cx, |sidebar, cx| {
                                        sidebar.is_starting_game = false;
                                        sidebar.launch_error = Some(format!("启动失败: {}", e));
                                        cx.notify();
                                    });
                                }
                            }
                        }
                    }
                },
            )
            .detach();
        }));

    launch_game_btn.into_any_element()
}
