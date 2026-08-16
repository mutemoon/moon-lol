//! Model Settings Tab：供应商侧栏 / 表单 / 模型增删改测对话框。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::{DialogAction, DialogClose, DialogFooter};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use lol_web_protocol::model_provider::ModelConfig;

use super::input::render_edit_field;
use super::logic::{
    api_key_placeholder, handle_delete_provider, handle_refresh_models, handle_save_provider,
    handle_test_model, is_new_key, select_provider,
};
use super::presets::{api_format_label, API_FORMATS, PROVIDER_PRESETS};
use super::types::{NEW_KEY, PLATFORM_KEY, PRESET_PREFIX};
use crate::components::dialog::open_form_dialog;
use crate::components::sidebar::AppSidebar;

pub(super) fn render_model_settings(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    // 连接测试结果由异步完成置位，渲染帧消费并弹出结果 Dialog
    if sidebar.settings.show_test_result {
        sidebar.settings.show_test_result = false;
        open_test_result_dialog(window, cx);
    }

    v_flex()
        .size_full()
        .gap_0()
        .overflow_hidden()
        .child(
            h_flex()
                .size_full()
                .gap_0()
                .overflow_hidden()
                .child(render_provider_sidebar(sidebar, cx))
                .child(
                    div()
                        .flex_1()
                        .id("settings-form-scroll")
                        .child(render_provider_form(sidebar, window, cx)),
                ),
        )
        .into_any_element()
}

fn render_provider_sidebar(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let providers = sidebar.settings.providers.clone();
    let selected = sidebar.settings.selected_key.clone();

    let preset_items: Vec<AnyElement> = PROVIDER_PRESETS
        .iter()
        .map(|p| {
            let key = format!("{}{}", PRESET_PREFIX, p.preset_type);
            let active = selected == key;
            make_sidebar_btn(&key, p.name.to_string(), active, cx)
        })
        .collect();

    let provider_items: Vec<AnyElement> = providers
        .iter()
        .map(|p| {
            let key = p.id.to_string();
            let active = selected == key;
            make_provider_btn(&key, p.name.clone(), p.enabled, active, cx)
        })
        .collect();

    let is_platform = selected == PLATFORM_KEY;
    let is_new = selected == NEW_KEY;

    v_flex()
        .w(px(220.))
        .h_full()
        .border_r_1()
        .border_color(cx.theme().border)
        .px_2()
        .py_3()
        .gap_3()
        .overflow_y_scrollbar()
        // ── 平台组：平台共享模型 + 预设供应商 ──
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .font_bold()
                        .child("平台模型"),
                )
                .child(make_sidebar_btn(
                    PLATFORM_KEY,
                    "平台共享模型".to_string(),
                    is_platform,
                    cx,
                ))
                .child(
                    div()
                        .px_2()
                        .pt_2()
                        .pb_1()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .font_bold()
                        .child("预设供应商"),
                )
                .children(preset_items),
        )
        // ── 我的组：用户已建供应商 ──
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .font_bold()
                        .child("我的供应商"),
                )
                .child(if providers.is_empty() {
                    div()
                        .px_2()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("暂无供应商，点击下方「添加供应商」开始配置。")
                        .into_any_element()
                } else {
                    v_flex()
                        .gap_0p5()
                        .children(provider_items)
                        .into_any_element()
                })
                .child(make_sidebar_btn(
                    NEW_KEY,
                    "添加供应商".to_string(),
                    is_new,
                    cx,
                )),
        )
        .into_any_element()
}

fn make_sidebar_btn(
    key: &str,
    label: String,
    active: bool,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let key = key.to_string();
    let mut btn = div()
        .id(key.clone())
        .py_1()
        .px_2()
        .rounded_md()
        .text_sm()
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            select_provider(this, &key, cx);
        }))
        .child(label);
    if active {
        btn = btn.bg(cx.theme().muted);
    }
    btn.into_any_element()
}

fn make_provider_btn(
    key: &str,
    label: String,
    enabled: bool,
    active: bool,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let key = key.to_string();
    let dot_color = if enabled {
        cx.theme().accent
    } else {
        cx.theme().muted_foreground
    };
    let mut btn = div()
        .id(key.clone())
        .py_1()
        .px_2()
        .rounded_md()
        .text_sm()
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            select_provider(this, &key, cx);
        }))
        .child(
            h_flex()
                .gap_1p5()
                .items_center()
                .child(div().size(px(8.)).rounded_full().bg(dot_color))
                .child(label),
        );
    if active {
        btn = btn.bg(cx.theme().muted);
    }
    btn.into_any_element()
}

fn render_provider_form(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    if sidebar.settings.selected_key == PLATFORM_KEY {
        return v_flex()
            .p_6()
            .gap_4()
            .child(div().text_lg().font_bold().child("平台共享模型"))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("走平台网关，由管理员在服务端 env 配置；按 Token 消耗以精粹结算。"),
            )
            .into_any_element();
    }

    let is_new = is_new_key(&sidebar.settings.selected_key);
    let api_format = sidebar.settings.form_api_format.clone();
    let models = sidebar.settings.form_models.clone();
    let error = sidebar.settings.error_msg.clone();
    let success = sidebar.settings.success_msg.clone();
    let saving = sidebar.settings.saving;
    let api_key_placeholder_text = api_key_placeholder(sidebar);

    let model_rows: Vec<AnyElement> = models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let name = m.name.clone();
            let max_tokens = m.max_tokens;
            h_flex()
                .gap_1()
                .items_center()
                .child(
                    div()
                        .flex_1()
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .bg(cx.theme().muted)
                        .text_sm()
                        .child(format!("{} ({} max tokens)", name, max_tokens)),
                )
                .child(
                    Button::new(format!("edit-model-{}", i))
                        .ghost()
                        .label("编辑")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            if let Some(m) = this.settings.form_models.get(i) {
                                this.settings.model_form_name = m.name.clone();
                                this.settings.model_form_max_tokens = m.max_tokens.to_string();
                            }
                            this.settings.editing_model_idx = Some(i);
                            cx.notify();
                            open_model_dialog(window, cx);
                        })),
                )
                .child(
                    Button::new(format!("test-model-{}", i))
                        .ghost()
                        .icon(IconName::Play)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            handle_test_model(this, i, cx);
                        })),
                )
                .child(
                    Button::new(format!("remove-model-{}", i))
                        .ghost()
                        .icon(IconName::Close)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.settings.form_models.remove(i);
                            cx.notify();
                        })),
                )
                .into_any_element()
        })
        .collect();

    v_flex()
        .p_6()
        .gap_4()
        .child(div().text_lg().font_bold().child(if is_new {
            "添加模型供应商"
        } else {
            "编辑供应商"
        }))
        .child(if is_new {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("从左侧选择预设可自动填入厂商参数，或完全手填。")
                .into_any_element()
        } else {
            div().into_any_element()
        })
        .child(render_edit_field(
            "form-name",
            "名称",
            "如：智谱 GLM",
            sidebar,
            window,
            cx,
            |t| t.settings.form_name.clone(),
            |t, v| t.settings.form_name = v,
        ))
        .child(render_edit_field(
            "form-base-url",
            "Base URL",
            "https://api.example.com/v1",
            sidebar,
            window,
            cx,
            |t| t.settings.form_base_url.clone(),
            |t, v| t.settings.form_base_url = v,
        ))
        .child(render_edit_field(
            "form-api-key",
            "API Key",
            &api_key_placeholder_text,
            sidebar,
            window,
            cx,
            |t| t.settings.form_api_key.clone(),
            |t, v| t.settings.form_api_key = v,
        ))
        .child(
            v_flex()
                .gap_1()
                .child(div().text_xs().font_bold().child("API 格式"))
                .child(render_api_format_field(&api_format, sidebar, cx)),
        )
        .child(
            v_flex()
                .gap_2()
                .child(div().text_sm().font_bold().child("模型列表"))
                .child(v_flex().gap_1().children(model_rows))
                .child(
                    Button::new("add-model-btn")
                        .outline()
                        .icon(IconName::Plus)
                        .label("添加模型")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.settings.editing_model_idx = None;
                            this.settings.model_form_name.clear();
                            this.settings.model_form_max_tokens = "200000".to_string();
                            cx.notify();
                            open_model_dialog(window, cx);
                        })),
                ),
        )
        .child(
            h_flex()
                .gap_2()
                .child(
                    Button::new("save-provider-btn")
                        .primary()
                        .label("保存")
                        .disabled(saving)
                        .on_click(cx.listener(|this, _, _, cx| {
                            handle_save_provider(this, cx);
                        })),
                )
                .child(
                    Button::new("refresh-models-btn")
                        .outline()
                        .icon(IconName::Loader)
                        .label("刷新远程模型")
                        .on_click(cx.listener(|this, _, _, cx| {
                            handle_refresh_models(this, cx);
                        })),
                )
                .child(if !is_new {
                    Button::new("delete-provider-btn")
                        .ghost()
                        .label("删除")
                        .on_click(cx.listener(|this, _, _, cx| {
                            handle_delete_provider(this, cx);
                        }))
                        .into_any_element()
                } else {
                    div().into_any_element()
                }),
        )
        .child(if !error.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().danger)
                .child(error)
                .into_any_element()
        } else if !success.is_empty() {
            div().text_sm().child(success).into_any_element()
        } else {
            div().into_any_element()
        })
        .into_any_element()
}

fn render_api_format_field(
    current: &str,
    _sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let current = current.to_string();
    let weak = cx.entity().downgrade();

    Button::new("api-format-dropdown")
        .label(api_format_label(&current))
        .dropdown_caret(true)
        .outline()
        .dropdown_menu(move |menu, _window, _cx| {
            let mut menu = menu;
            for &(val, label) in API_FORMATS {
                let weak = weak.clone();
                let val = val.to_string();
                menu = menu.item(PopupMenuItem::new(label).checked(current == val).on_click(
                    move |_, _, cx| {
                        let _ = weak.update(cx, |this, cx| {
                            this.settings.form_api_format = val.clone();
                            cx.notify();
                        });
                    },
                ));
            }
            menu
        })
        .into_any_element()
}

// ── 模型新增 / 编辑对话框 ──

fn build_model_form(
    sidebar: &AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let editing = sidebar.settings.editing_model_idx.is_some();

    v_flex()
        .gap_4()
        .child(div().font_bold().text_sm().child(if editing {
            "编辑模型"
        } else {
            "添加模型"
        }))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("请配置该模型的 ID / 名称以及最大上下文 Token 限制。"),
        )
        .child(
            v_flex()
                .gap_3()
                .child(render_edit_field(
                    "model-dialog-name",
                    "模型 ID",
                    "如 gpt-4o, claude-3-5-sonnet",
                    sidebar,
                    window,
                    cx,
                    |t| t.settings.model_form_name.clone(),
                    |t, v| t.settings.model_form_name = v,
                ))
                .child(render_edit_field(
                    "model-dialog-tokens",
                    "最大上下文 Token 数",
                    "200000",
                    sidebar,
                    window,
                    cx,
                    |t| t.settings.model_form_max_tokens.clone(),
                    |t, v| {
                        t.settings.model_form_max_tokens =
                            v.chars().filter(|c| c.is_ascii_digit()).collect();
                    },
                )),
        )
        .into_any_element()
}

fn open_model_dialog(window: &mut Window, cx: &mut Context<AppSidebar>) {
    let is_new = cx.entity().read(cx).settings.editing_model_idx.is_none();
    let title = if is_new { "添加模型" } else { "编辑模型" };
    let weak = cx.entity().downgrade();
    let save_weak = weak.clone();
    open_form_dialog(window, cx, weak, build_model_form, move |dialog, form| {
        let save_weak = save_weak.clone();
        dialog
            .title(title)
            .w(px(384.))
            .child(form)
            .footer(
                DialogFooter::new()
                    .child(
                        DialogClose::new()
                            .child(Button::new("model-dialog-cancel").outline().label("取消")),
                    )
                    .child(
                        DialogAction::new()
                            .child(Button::new("model-dialog-confirm").primary().label("确定")),
                    ),
            )
            .on_ok(move |_, _, cx| {
                let _ = save_weak.update(cx, |this, cx| handle_save_model(this, cx));
                true
            })
    });
}

fn handle_save_model(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let name = sidebar.settings.model_form_name.trim().to_string();
    if name.is_empty() {
        return;
    }
    let max_tokens = sidebar
        .settings
        .model_form_max_tokens
        .parse::<u32>()
        .unwrap_or(200000);

    match sidebar.settings.editing_model_idx {
        Some(i) => {
            if let Some(m) = sidebar.settings.form_models.get_mut(i) {
                m.name = name;
                m.max_tokens = max_tokens;
            }
        }
        None => {
            sidebar
                .settings
                .form_models
                .push(ModelConfig { name, max_tokens });
        }
    }
    cx.notify();
}

// ── 连接测试结果对话框 ──

fn build_test_result_form(
    sidebar: &AppSidebar,
    _window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let success = sidebar
        .settings
        .test_result
        .as_ref()
        .map(|r| r.success)
        .unwrap_or(false);
    let message = sidebar
        .settings
        .test_result
        .as_ref()
        .map(|r| r.message.clone())
        .unwrap_or_default();

    v_flex()
        .gap_4()
        .child(
            div()
                .font_bold()
                .text_sm()
                .text_color(if success {
                    cx.theme().accent
                } else {
                    cx.theme().danger
                })
                .child(if success {
                    "连接测试成功"
                } else {
                    "连接测试失败"
                }),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(if success {
                    "模型成功回复了消息："
                } else {
                    "测试未成功，详细错误信息如下："
                }),
        )
        .child(
            div()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().muted)
                .p_3()
                .text_xs()
                .max_h(rems(12.))
                .overflow_y_scrollbar()
                .child(message),
        )
        .into_any_element()
}

fn open_test_result_dialog(window: &mut Window, cx: &mut Context<AppSidebar>) {
    let weak = cx.entity().downgrade();
    open_form_dialog(window, cx, weak, build_test_result_form, |dialog, form| {
        dialog
            .title("连通性测试结果")
            .w(px(384.))
            .child(form)
            .footer(DialogFooter::new().child(
                DialogAction::new().child(Button::new("test-result-close").primary().label("确定")),
            ))
    });
}
