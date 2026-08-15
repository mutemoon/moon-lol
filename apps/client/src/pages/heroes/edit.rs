//! Edit：编辑视图 + Config Tab（类型选择 / 提示词 / 模型 / RL 配置 / JSON）。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::separator::Separator;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable, StyledExt};
use lol_web_protocol::agent::AgentType;
use lol_web_protocol::model_provider::ModelProvider;

use super::input::{edit_field, render_edit_input};
use super::publish::{render_publish_tab, render_status_bar};
use super::types::{HeroesMode, HeroesTab, PLATFORM_PROVIDER_ID, RL_REWARD_KEYS};
use super::utils::champion_display;
use super::{
    ensure_providers_loaded, handle_export_json, handle_import_json, handle_save, open_delete_modal,
};
use crate::components::sidebar::AppSidebar;

pub(super) fn render_edit(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let editing = matches!(
        sidebar.heroes.mode,
        HeroesMode::Edit {
            editing_id: Some(_)
        }
    );
    let draft_name = sidebar.heroes.draft_name.clone();
    let selected_tab = sidebar.heroes.selected_tab;

    let tab_config = make_tab_btn(
        "tab-config",
        "配置",
        selected_tab == HeroesTab::Config,
        cx,
        |this, cx| {
            this.heroes.selected_tab = HeroesTab::Config;
            cx.notify();
        },
    );
    let tab_publish = make_tab_btn(
        "tab-publish",
        "发布与快照",
        selected_tab == HeroesTab::Publish,
        cx,
        |this, cx| {
            this.heroes.selected_tab = HeroesTab::Publish;
            cx.notify();
        },
    );

    let tab_content = if selected_tab == HeroesTab::Config {
        render_config_tab(sidebar, window, cx)
    } else {
        render_publish_tab(sidebar, cx)
    };

    let main = v_flex()
        .size_full()
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .p_4()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Button::new("back-btn")
                                .ghost()
                                .icon(IconName::ChevronLeft)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.heroes.mode = HeroesMode::Browse;
                                    this.heroes.error_msg.clear();
                                    this.heroes.success_msg.clear();
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(if editing {
                                    "编辑选手"
                                } else {
                                    "新建选手"
                                }),
                        )
                        .child(div().text_lg().font_bold().child(if draft_name.is_empty() {
                            "未命名选手".to_string()
                        } else {
                            draft_name.clone()
                        })),
                )
                .child(
                    h_flex().gap_2().child(
                        Button::new("save-btn")
                            .primary()
                            .label("保存配置")
                            .on_click(cx.listener(|this, _, _, cx| {
                                handle_save(this, cx);
                            })),
                    ),
                ),
        )
        .child(Separator::horizontal())
        .child(
            h_flex()
                .gap_1()
                .px_4()
                .py_2()
                .child(tab_config)
                .child(tab_publish),
        )
        .child(Separator::horizontal())
        .child(
            div()
                .flex_1()
                .w_full()
                .overflow_y_scrollbar()
                .px_6()
                .py_4()
                .child(tab_content),
        )
        .child(render_status_bar(sidebar, cx));

    if sidebar.heroes.show_delete_confirm {
        sidebar.heroes.show_delete_confirm = false;
        open_delete_modal(window, cx);
    }

    main.into_any_element()
}

fn make_tab_btn(
    id: &str,
    label: impl Into<SharedString>,
    active: bool,
    cx: &mut Context<AppSidebar>,
    on_click: impl Fn(&mut AppSidebar, &mut Context<AppSidebar>) + 'static,
) -> AnyElement {
    let el_id: SharedString = id.to_string().into();
    let btn = Button::new(el_id).label(label);
    let btn = if active { btn.primary() } else { btn.ghost() };
    btn.on_click(cx.listener(move |this, _, _, cx| on_click(this, cx)))
        .into_any_element()
}

fn render_config_tab(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    ensure_providers_loaded(sidebar, cx);

    let agent_type = sidebar.heroes.draft_agent_type;

    let type_buttons: Vec<AnyElement> = [AgentType::Llm, AgentType::Rl, AgentType::Script]
        .iter()
        .map(|&at| {
            let active = agent_type == at;
            let label = match at {
                AgentType::Llm => "LLM（语言模型）",
                AgentType::Rl => "RL（强化学习）",
                AgentType::Script => "Script（脚本）",
            };
            let btn = Button::new(format!("type-{:?}", at)).label(label);
            let btn = if active { btn.primary() } else { btn.outline() };
            btn.on_click(cx.listener(move |this, _, _, cx| {
                this.heroes.draft_agent_type = at;
                cx.notify();
            }))
            .into_any_element()
        })
        .collect();

    let type_specific = match agent_type {
        AgentType::Llm => v_flex()
            .gap_4()
            .child(edit_field(
                "系统提示词",
                render_edit_input(
                    sidebar,
                    window,
                    cx,
                    "heroes-prompt",
                    "描述该选手的行为策略、对线风格、连招意图…",
                    true,
                    |s| s.heroes.draft_prompt.clone(),
                    |s, v| s.heroes.draft_prompt = v,
                ),
            ))
            .child(
                h_flex()
                    .gap_4()
                    .child(div().flex_1().child(edit_field("模型供应商", render_provider_select(sidebar, cx))))
                    .child(div().flex_1().child(edit_field("模型（留空用全局默认）", render_model_select(sidebar, window, cx)))),
            )
            .child(render_think_depth(sidebar, cx))
            .into_any_element(),
        AgentType::Rl => v_flex()
            .gap_4()
            .child(edit_field(
                "模型权重路径 (.pth)",
                render_edit_input(
                    sidebar,
                    window,
                    cx,
                    "heroes-rl-path",
                    "如 checkpoints/ppo_riven.pth",
                    false,
                    |s| s.heroes.draft_rl_model_path.clone(),
                    |s, v| s.heroes.draft_rl_model_path = v,
                ),
            ))
            .child(edit_field(
                "推理端点 URL（可选）",
                render_edit_input(
                    sidebar,
                    window,
                    cx,
                    "heroes-rl-endpoint",
                    "如 ws://127.0.0.1:8765",
                    false,
                    |s| s.heroes.draft_rl_endpoint.clone(),
                    |s, v| s.heroes.draft_rl_endpoint = v,
                ),
            ))
            .child(reward_grid(sidebar, window, cx))
            .into_any_element(),
        AgentType::Script => v_flex()
            .gap_4()
            .child(edit_field(
                "脚本",
                render_edit_input(
                    sidebar,
                    window,
                    cx,
                    "heroes-script",
                    "// 在此编写宿主 API 脚本…",
                    true,
                    |s| s.heroes.draft_script.clone(),
                    |s, v| s.heroes.draft_script = v,
                ),
            ))
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("简化版脚本编辑器（无语法高亮 / 断点 / 热重载），保存时写入 config_json.script。"),
            )
            .into_any_element(),
    };

    v_flex()
        .gap_5()
        .child(edit_field(
            "选手名称",
            render_edit_input(
                sidebar,
                window,
                cx,
                "heroes-name",
                "如：锐雯 · 激进压制",
                false,
                |s| s.heroes.draft_name.clone(),
                |s, v| s.heroes.draft_name = v,
            ),
        ))
        .child(
            h_flex()
                .gap_4()
                .child(
                    div()
                        .flex_1()
                        .child(edit_field("英雄", render_champion_select(sidebar, cx))),
                )
                .child(div().flex_1().child(edit_field(
                    "决策驱动类型",
                    h_flex().gap_2().children(type_buttons).into_any_element(),
                ))),
        )
        .child(type_specific)
        .child(Separator::horizontal())
        .child(render_json_section(sidebar, window, cx))
        .into_any_element()
}

/// LLM 思考深度滑块（1-5，每格可点击）。
fn render_think_depth(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let td = sidebar.heroes.draft_thinking_depth as i32;
    let pips: Vec<AnyElement> = (1..=5)
        .map(|d| {
            let active = d <= td;
            let target = d as u32;
            div()
                .w(px(40.))
                .h(px(16.))
                .rounded_sm()
                .bg(if active {
                    cx.theme().primary
                } else {
                    cx.theme().muted
                })
                .cursor_pointer()
                .id(format!("depth-pip-{}", d))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.heroes.draft_thinking_depth = target;
                    cx.notify();
                }))
                .into_any_element()
        })
        .collect();

    v_flex()
        .gap_2()
        .child(
            h_flex()
                .justify_between()
                .child(div().text_sm().font_bold().child("思考深度"))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{} / 5", td)),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "更高的思考深度让 LLM 在决策前展开更长的推理链，更稳但更慢、消耗更多 token。",
                ),
        )
        .child(h_flex().gap_2().children(pips))
        .into_any_element()
}

/// 英雄下拉（从 sidebar.champions_list 选择）。
fn render_champion_select(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let list: Vec<String> = if sidebar.champions_list.is_empty() {
        vec!["Riven".to_string(), "Fiora".to_string()]
    } else {
        sidebar.champions_list.clone()
    };
    let current = sidebar.heroes.draft_champion.clone();
    let weak = cx.entity().downgrade();

    Button::new("heroes-champion")
        .outline()
        .w_full()
        .label(champion_display(&current))
        .dropdown_menu(move |menu, _w, _cx| {
            let mut m = menu;
            for c in &list {
                let cname = c.clone();
                let weak = weak.clone();
                let checked = current == cname;
                m = m.item(
                    PopupMenuItem::new(champion_display(&cname))
                        .checked(checked)
                        .on_click(move |_, _, cx| {
                            if let Some(app) = weak.upgrade() {
                                let _ = app.update(cx, |this, cx| {
                                    this.heroes.draft_champion = cname.clone();
                                    cx.notify();
                                });
                            }
                        }),
                );
            }
            m
        })
        .into_any_element()
}

/// provider 下拉：平台模型 + 启用的自带供应商（BYO）。
fn render_provider_select(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let providers = sidebar.heroes.model_providers.clone();
    let current = sidebar.heroes.draft_provider_id.clone();
    let enabled: Vec<ModelProvider> = providers.iter().filter(|p| p.enabled).cloned().collect();
    let current_label = if current.is_empty() || current == PLATFORM_PROVIDER_ID {
        "平台模型".to_string()
    } else {
        providers
            .iter()
            .find(|p| p.id.to_string() == current)
            .map_or("未知供应商".to_string(), |p| p.name.clone())
    };
    let weak = cx.entity().downgrade();

    Button::new("heroes-provider")
        .outline()
        .w_full()
        .label(current_label)
        .dropdown_menu(move |menu, _w, _cx| {
            let mut m = menu;
            let checked_platform = current == PLATFORM_PROVIDER_ID;
            let weak_platform = weak.clone();
            m = m.item(
                PopupMenuItem::new("平台模型")
                    .checked(checked_platform)
                    .on_click(move |_, _, cx| {
                        if let Some(app) = weak_platform.upgrade() {
                            let _ = app.update(cx, |this, cx| {
                                this.heroes.draft_provider_id = PLATFORM_PROVIDER_ID.to_string();
                                this.heroes.draft_manual_model = false;
                                cx.notify();
                            });
                        }
                    }),
            );
            if enabled.is_empty() {
                m = m.item(PopupMenuItem::new("暂无启用供应商").disabled(true));
            }
            for p in &enabled {
                let pid = p.id.to_string();
                let pname = p.name.clone();
                let weak = weak.clone();
                let checked = current == pid;
                m = m.item(
                    PopupMenuItem::new(pname)
                        .checked(checked)
                        .on_click(move |_, _, cx| {
                            if let Some(app) = weak.upgrade() {
                                let _ = app.update(cx, |this, cx| {
                                    this.heroes.draft_provider_id = pid.clone();
                                    this.heroes.draft_manual_model = false;
                                    cx.notify();
                                });
                            }
                        }),
                );
            }
            m
        })
        .into_any_element()
}

/// 模型选择：平台模型走管理员清单；BYO 供应商走其 models 列表；可切「手填」。
fn render_model_select(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let is_platform = sidebar.heroes.draft_provider_id.is_empty()
        || sidebar.heroes.draft_provider_id == PLATFORM_PROVIDER_ID;
    let manual = sidebar.heroes.draft_manual_model;
    let model = sidebar.heroes.draft_model.clone();
    let platform_models = sidebar.heroes.platform_models.clone();
    let providers = sidebar.heroes.model_providers.clone();
    let pid = sidebar.heroes.draft_provider_id.clone();

    let provider_models: Vec<String> = providers
        .iter()
        .find(|p| p.id.to_string() == pid)
        .map(|p| p.models.iter().map(|m| m.name.clone()).collect())
        .unwrap_or_default();
    let model_options: Vec<String> = if is_platform {
        platform_models.clone()
    } else {
        provider_models
    };
    let model_not_in_list = !model.is_empty() && !model_options.contains(&model);
    let show_manual = !is_platform && (manual || model_not_in_list || model_options.is_empty());

    let weak = cx.entity().downgrade();

    let selector: AnyElement = if show_manual {
        render_edit_input(
            sidebar,
            window,
            cx,
            "heroes-model-manual",
            "手动输入模型名…",
            false,
            |s| s.heroes.draft_model.clone(),
            |s, v| s.heroes.draft_model = v,
        )
    } else {
        let current_label = if model.is_empty() {
            "选择模型…".to_string()
        } else {
            model.clone()
        };
        let options = model_options.clone();
        let current = model.clone();
        Button::new("heroes-model")
            .outline()
            .w_full()
            .label(current_label)
            .dropdown_menu(move |menu, _w, _cx| {
                let mut m = menu;
                if options.is_empty() {
                    m = m.item(PopupMenuItem::new("暂无可选模型").disabled(true));
                }
                for opt in &options {
                    let opt = opt.clone();
                    let weak = weak.clone();
                    let checked = current == opt;
                    m = m.item(PopupMenuItem::new(opt.clone()).checked(checked).on_click(
                        move |_, _, cx| {
                            if let Some(app) = weak.upgrade() {
                                let _ = app.update(cx, |this, cx| {
                                    this.heroes.draft_model = opt.clone();
                                    this.heroes.draft_manual_model = false;
                                    cx.notify();
                                });
                            }
                        },
                    ));
                }
                m
            })
            .into_any_element()
    };

    v_flex()
        .gap_2()
        .child(selector)
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(if !is_platform {
                    Button::new("heroes-model-manual-toggle")
                        .ghost()
                        .xsmall()
                        .label(if manual { "使用列表" } else { "手填" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.heroes.draft_manual_model = !this.heroes.draft_manual_model;
                            cx.notify();
                        }))
                        .into_any_element()
                } else {
                    div().into_any_element()
                })
                .child(if is_platform && platform_models.is_empty() {
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("管理员暂未配置平台模型，可改选自带供应商。")
                        .into_any_element()
                } else {
                    div().into_any_element()
                }),
        )
        .into_any_element()
}

/// RL Reward Shaper 权重输入网格。
fn reward_grid(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    v_flex()
        .gap_2()
        .child(div().text_sm().font_bold().child("Reward Shaper 权重"))
        .child(
            div()
                .grid()
                .grid_cols(3)
                .gap_2()
                .children(RL_REWARD_KEYS.iter().map(|key| {
                    let kid = format!("heroes-rl-{}", key);
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(key.to_string()),
                        )
                        .child(render_edit_input(
                            sidebar,
                            window,
                            cx,
                            &kid,
                            "0",
                            false,
                            move |s: &AppSidebar| {
                                s.heroes
                                    .draft_rl_rewards
                                    .get(*key)
                                    .map_or_else(|| "0".to_string(), |v| format!("{}", v))
                            },
                            move |s: &mut AppSidebar, v: String| {
                                let parsed = v.trim().parse::<f64>().unwrap_or(0.0);
                                s.heroes.draft_rl_rewards.insert(key.to_string(), parsed);
                            },
                        ))
                        .into_any_element()
                })),
        )
        .into_any_element()
}

/// JSON 导入 / 导出区块。
fn render_json_section(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    v_flex()
        .gap_2()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .font_bold()
                        .child("配置 JSON（导入 / 导出）"),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("heroes-json-export")
                                .outline()
                                .label("导出")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    handle_export_json(this, cx);
                                })),
                        )
                        .child(
                            Button::new("heroes-json-import")
                                .outline()
                                .label("应用")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    handle_import_json(this, cx);
                                })),
                        ),
                ),
        )
        .child(render_edit_input(
            sidebar,
            window,
            cx,
            "heroes-json",
            "{}",
            true,
            |s| s.heroes.draft_config_json_str.clone(),
            |s, v| s.heroes.draft_config_json_str = v,
        ))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "「导出」序列化当前表单并复制到剪贴板；粘贴 JSON 到上方后点「应用」填充表单。",
                ),
        )
        .into_any_element()
}
