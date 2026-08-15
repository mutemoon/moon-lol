//! 启动器页渲染辅助：槽位卡片 / 阵营列 / 各区块 UI。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt};
use lol_web_protocol::agent::Agent;
use lol_web_protocol::spawn_preset::SpawnPreset as ProtoSpawnPreset;
use lol_web_protocol::GameConfig;

use super::logic::{build_all_agents, spawn_launch_game, spawn_load_scenario, spawn_save_scenario};
use super::types::{LauncherSlot, LauncherView};
use crate::components::sidebar::AppSidebar;

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
                        weak.update(cx, |this, cx| {
                            if slot_team == "Order" {
                                this.launcher.blue_slots[slot_idx].hero_name = name.clone();
                            } else {
                                this.launcher.red_slots[slot_idx].hero_name = name.clone();
                            }
                            cx.notify();
                        })
                        .ok();
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
                            weak.update(cx, |this, cx| {
                                if slot_team == "Order" {
                                    this.launcher.blue_slots[slot_idx].spawn_name = sp_name.clone();
                                } else {
                                    this.launcher.red_slots[slot_idx].spawn_name = sp_name.clone();
                                }
                                cx.notify();
                            })
                            .ok();
                        },
                    ));
                }
                m
            });

    let delete_btn = Button::new(format!("{team}-{index}-del"))
        .ghost()
        .xsmall()
        .icon(IconName::Delete)
        .on_click(cx.listener(move |this, _, _, cx| {
            if team == "Order" {
                this.launcher.blue_slots.remove(index);
            } else {
                this.launcher.red_slots.remove(index);
            }
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
                        .bg(cx.theme().muted)
                        .text_xs()
                        .font_bold()
                        .text_color(cx.theme().foreground)
                        .child(format!("{}", slots.len())),
                ),
        )
        .child(
            Button::new(format!("add-{team}-slot"))
                .outline()
                .xsmall()
                .icon(IconName::Plus)
                .label("添加槽位")
                .on_click(cx.listener(move |this, _, _, cx| {
                    let slot = LauncherSlot::default();
                    if team == "Order" {
                        this.launcher.blue_slots.push(slot);
                    } else {
                        this.launcher.red_slots.push(slot);
                    }
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

// ── 模式卡片选择页 ──

/// 单张模式卡片：图标 + 标题 + 描述，点击触发回调。
fn mode_card(
    cx: &mut Context<AppSidebar>,
    icon: IconName,
    title: &str,
    desc: &str,
    on_click: impl Fn(&mut AppSidebar, &mut Context<AppSidebar>) + 'static,
) -> AnyElement {
    let theme = cx.theme();
    div()
        .flex_1()
        .rounded_lg()
        .border_1()
        .border_color(theme.border)
        .p_5()
        .flex()
        .flex_col()
        .justify_between()
        .h_32()
        .hover(|s| s.bg(theme.accent.opacity(0.05)))
        .cursor_pointer()
        .on_any_mouse_down(cx.listener(move |this, _event, _window, cx| on_click(this, cx)))
        .child(
            h_flex()
                .items_start()
                .justify_between()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().font_bold().text_sm().child(title.to_string()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(desc.to_string()),
                        ),
                )
                .child(
                    div()
                        .p_2()
                        .rounded_md()
                        .bg(theme.accent.opacity(0.1))
                        .child(icon),
                ),
        )
        .into_any_element()
}

/// 模式卡片页（顶层）：默认 / 自定义，点击自定义进入编排页。
pub(super) fn render_mode_cards(sidebar: &AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let default_card = mode_card(
        cx,
        IconName::LayoutDashboard,
        "默认",
        "一键启动默认对局（沙盒 · 瑞雯）",
        |this, cx| {
            if this.is_starting_game {
                return;
            }
            this.is_starting_game = true;
            this.launcher.message = Some("正在启动默认对局…".into());
            this.launcher.error = None;
            cx.notify();
            // 与 `cargo run` 默认一致：沙盒 + Riven，省略 scene 由游戏进程用默认 classic.ron，无 AI
            let weak = cx.entity().downgrade();
            let config = GameConfig {
                mode: "sandbox".into(),
                champion: "Riven".into(),
                scene_name: None,
                agents: None,
                providers: None,
            };
            spawn_launch_game(weak, cx, config, "默认对局已启动".into());
        },
    );
    let custom_card = mode_card(
        cx,
        IconName::Settings,
        "自定义",
        "编排场景、英雄预设与出生点后启动",
        |this, cx| {
            this.launcher.view = LauncherView::Custom;
            this.launcher.message = None;
            cx.notify();
        },
    );

    let theme = cx.theme();
    let muted = theme.muted_foreground;

    v_flex()
        .size_full()
        .flex_1()
        .gap_6()
        .overflow_y_scrollbar()
        .child(
            v_flex()
                .gap_1()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::Play)
                        .child(div().font_bold().text_lg().child("启动游戏")),
                )
                .child(div().text_xs().text_color(muted).child("选择对局模式")),
        )
        .when_some(sidebar.launcher.message.clone(), |d, msg| {
            d.child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.muted)
                    .text_xs()
                    .text_color(theme.foreground)
                    .child(msg),
            )
        })
        .child(
            h_flex()
                .gap_4()
                .items_start()
                .child(default_card)
                .child(custom_card),
        )
        .into_any_element()
}

// ── 模块化子组件 ──

pub(super) fn render_header(cx: &mut Context<AppSidebar>) -> AnyElement {
    h_flex()
        .items_center()
        .justify_between()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(IconName::Play)
                .child(div().font_bold().text_lg().child("启动游戏")),
        )
        .child(
            Button::new("launcher-back-to-modes")
                .ghost()
                .icon(IconName::ArrowLeft)
                .label("返回模式选择")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.launcher.view = LauncherView::Modes;
                    this.launcher.message = None;
                    cx.notify();
                })),
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

pub(super) fn render_load_dropdown(
    sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let weak = cx.entity().downgrade();
    let scenarios = sidebar.launcher.scenarios.clone();
    Button::new("launcher-load-scenario")
        .outline()
        .xsmall()
        .icon(IconName::Folder)
        .label("载入")
        .disabled(sidebar.launcher.loading_scenario)
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
                    weak.update(cx, |this, cx| {
                        this.launcher.loading_scenario = true;
                        cx.notify();
                    })
                    .ok();
                    spawn_load_scenario(cx, weak.clone(), sc_id.clone());
                }));
            }
            m
        })
        .into_any_element()
}

pub(super) fn render_scene_section(
    sidebar: &AppSidebar,
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
                        .child(sidebar.launcher.scene_name.clone()),
                )
                .child(load_dropdown)
                .child(
                    Button::new("launcher-save-scenario")
                        .outline()
                        .xsmall()
                        .icon(IconName::Check)
                        .label("保存")
                        .disabled(sidebar.launcher.saving)
                        .on_click(cx.listener(|this, _, _, cx| {
                            spawn_save_scenario(this, cx);
                        })),
                )
                .child(
                    Button::new("launcher-new-scenario")
                        .outline()
                        .xsmall()
                        .icon(IconName::Plus)
                        .label("新建")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.launcher.scene_name = format!("custom_agents_{}", unix_ts());
                            this.launcher.blue_slots = vec![LauncherSlot::default()];
                            this.launcher.red_slots = vec![LauncherSlot::default()];
                            this.launcher.error = None;
                            this.launcher.message = None;
                            cx.notify();
                        })),
                ),
        )
        .into_any_element()
}

pub(super) fn render_message_banners(
    sidebar: &AppSidebar,
    launch_error: Option<String>,
    cx: &mut Context<AppSidebar>,
) -> Vec<AnyElement> {
    let mut banners = Vec::new();

    if let Some(msg) = sidebar.launcher.message.clone() {
        banners.push(
            div()
                .px_3()
                .py_2()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().muted)
                .text_xs()
                .text_color(cx.theme().foreground)
                .child(msg)
                .into_any_element(),
        );
    }

    if let Some(err) = sidebar.launcher.error.clone() {
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

pub(super) fn render_teams_section(
    sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    h_flex()
        .gap_4()
        .items_start()
        .child(team_column(
            cx,
            "Order",
            "蓝色方 (Order)",
            gpui::hsla(0.6, 0.7, 0.5, 1.0),
            &sidebar.launcher.blue_slots,
            &sidebar.launcher.agents,
            &sidebar.launcher.spawns,
        ))
        .child(team_column(
            cx,
            "Chaos",
            "红色方 (Chaos)",
            gpui::hsla(0.0, 0.7, 0.5, 1.0),
            &sidebar.launcher.red_slots,
            &sidebar.launcher.agents,
            &sidebar.launcher.spawns,
        ))
        .into_any_element()
}

pub(super) fn render_action_buttons(
    sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let starting = sidebar.is_starting_game;
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
            let (scene_name, agents) = (
                this.launcher.scene_name.trim().to_string(),
                build_all_agents(&this.launcher),
            );
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
            let weak = cx.entity().downgrade();
            let config = GameConfig {
                mode: mode.clone(),
                champion: champ.clone(),
                scene_name: Some(scene_name.clone()),
                agents: Some(agents.clone()),
                providers: None,
            };
            spawn_launch_game(weak, cx, config, "对局已启动".into());
        }));

    launch_game_btn.into_any_element()
}
