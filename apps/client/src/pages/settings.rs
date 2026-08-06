//! 设置页 — 对应 apps/client/src/pages/settings.vue
//!
//! 包含「常规设置」（主题 / 语言）与「模型设置」（供应商侧栏 / 表单 / 模型增删改测）。
//! 供应商预设数据移植自 apps/client/src/config/providerPresets.ts。

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::separator::Separator;
use gpui_component::{
    h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt, Theme, ThemeMode,
};
use lol_web_protocol::model_provider::{
    ModelConfig, ModelProvider, ModelProviderInput, TestModelProviderResponse,
};
use uuid::Uuid;

use crate::components::sidebar::AppSidebar;

// ── 供应商预设（移植自 apps/client/src/config/providerPresets.ts） ──

struct ProviderPreset {
    name: &'static str,
    preset_type: &'static str,
    base_url: &'static str,
    api_format: &'static str,
    default_models: &'static [&'static str],
    website_url: &'static str,
    api_key_url: &'static str,
    icon: &'static str,
    icon_color: &'static str,
}

/// 15 个厂商预设：点选后预填表单（name / baseUrl / apiFormat / 默认模型 / api_key 申请地址）。
const PROVIDER_PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        name: "智谱 BigModel",
        preset_type: "zhipu",
        base_url: "https://open.bigmodel.cn/api/anthropic",
        api_format: "anthropic",
        default_models: &["glm-5.1"],
        website_url: "https://open.bigmodel.cn",
        api_key_url: "https://www.bigmodel.cn/claude-code",
        icon: "zhipu",
        icon_color: "#0F62FE",
    },
    ProviderPreset {
        name: "智谱 z.ai（海外）",
        preset_type: "zhipu_en",
        base_url: "https://api.z.ai/api/anthropic",
        api_format: "anthropic",
        default_models: &["glm-5.1"],
        website_url: "https://z.ai",
        api_key_url: "https://z.ai/subscribe",
        icon: "zhipu",
        icon_color: "#0F62FE",
    },
    ProviderPreset {
        name: "DeepSeek",
        preset_type: "deepseek",
        base_url: "https://api.deepseek.com/anthropic",
        api_format: "anthropic",
        default_models: &["deepseek-v4-pro"],
        website_url: "https://www.deepseek.com",
        api_key_url: "",
        icon: "deepseek",
        icon_color: "#4D6BFE",
    },
    ProviderPreset {
        name: "火山方舟 Agentplan",
        preset_type: "volcengine",
        base_url: "https://ark.cn-beijing.volces.com/api/coding",
        api_format: "anthropic",
        default_models: &["ark-code-latest"],
        website_url: "https://www.volcengine.com/product/ark",
        api_key_url: "",
        icon: "volcengine",
        icon_color: "#1664FF",
    },
    ProviderPreset {
        name: "豆包 Seed",
        preset_type: "doubao",
        base_url: "https://ark.cn-beijing.volces.com/api/compatible",
        api_format: "anthropic",
        default_models: &["doubao-seed-2-1-pro"],
        website_url: "https://www.volcengine.com/product/doubao",
        api_key_url: "",
        icon: "doubao",
        icon_color: "#1664FF",
    },
    ProviderPreset {
        name: "百度千帆 Coding",
        preset_type: "qianfan",
        base_url: "https://qianfan.baidubce.com/anthropic/coding",
        api_format: "anthropic",
        default_models: &["qianfan-code-latest"],
        website_url: "https://cloud.baidu.com/product/qianfan_modelbuilder",
        api_key_url: "",
        icon: "baidu",
        icon_color: "#2932E1",
    },
    ProviderPreset {
        name: "阿里百炼",
        preset_type: "bailian",
        base_url: "https://dashscope.aliyuncs.com/apps/anthropic",
        api_format: "anthropic",
        default_models: &[],
        website_url: "https://bailian.console.aliyun.com",
        api_key_url: "",
        icon: "bailian",
        icon_color: "#624AFF",
    },
    ProviderPreset {
        name: "阿里百炼 For Coding",
        preset_type: "bailian_coding",
        base_url: "https://coding.dashscope.aliyuncs.com/apps/anthropic",
        api_format: "anthropic",
        default_models: &[],
        website_url: "https://bailian.console.aliyun.com",
        api_key_url: "",
        icon: "bailian",
        icon_color: "#624AFF",
    },
    ProviderPreset {
        name: "Kimi",
        preset_type: "kimi",
        base_url: "https://api.moonshot.cn/anthropic",
        api_format: "anthropic",
        default_models: &["kimi-k2.7-code"],
        website_url: "https://platform.moonshot.cn",
        api_key_url: "",
        icon: "kimi",
        icon_color: "#1D1D1F",
    },
    ProviderPreset {
        name: "StepFun",
        preset_type: "stepfun",
        base_url: "https://api.stepfun.com/step_plan",
        api_format: "anthropic",
        default_models: &["step-3.5-flash-2603"],
        website_url: "https://platform.stepfun.com",
        api_key_url: "",
        icon: "stepfun",
        icon_color: "#0066FF",
    },
    ProviderPreset {
        name: "MiniMax",
        preset_type: "minimax",
        base_url: "https://api.minimaxi.com/anthropic",
        api_format: "anthropic",
        default_models: &["MiniMax-M2.7"],
        website_url: "https://platform.minimaxi.com",
        api_key_url: "",
        icon: "minimax",
        icon_color: "#FF6B00",
    },
    ProviderPreset {
        name: "Longcat",
        preset_type: "longcat",
        base_url: "https://api.longcat.chat/anthropic",
        api_format: "anthropic",
        default_models: &["LongCat-Flash-Chat"],
        website_url: "https://longcat.chat",
        api_key_url: "",
        icon: "longcat",
        icon_color: "#7C3AED",
    },
    ProviderPreset {
        name: "百灵 BaiLing",
        preset_type: "bailing",
        base_url: "https://api.tbox.cn/api/anthropic",
        api_format: "anthropic",
        default_models: &["Ling-2.5-1T"],
        website_url: "https://www.tbox.cn",
        api_key_url: "",
        icon: "bailing",
        icon_color: "#1A73E8",
    },
    ProviderPreset {
        name: "小米 MiMo",
        preset_type: "mimo",
        base_url: "https://api.xiaomimimo.com/anthropic",
        api_format: "anthropic",
        default_models: &["mimo-v2.5-pro"],
        website_url: "https://xiaomimimo.com",
        api_key_url: "",
        icon: "mimo",
        icon_color: "#FF6900",
    },
    ProviderPreset {
        name: "KAT-Coder",
        preset_type: "katcoder",
        base_url: "https://vanchin.streamlake.ai/api/gateway/v1/endpoints/claude-code-proxy",
        api_format: "anthropic",
        default_models: &["KAT-Coder-Pro V1"],
        website_url: "https://vanchin.streamlake.ai",
        api_key_url: "",
        icon: "katcoder",
        icon_color: "#111827",
    },
];

/// API 格式下拉选项（取值与 client settings.vue 的 API_FORMATS 一致）。
const API_FORMATS: &[(&str, &str)] = &[
    ("anthropic", "Anthropic Messages (/v1/messages)"),
    ("openai_chat", "OpenAI Chat Completions"),
    ("openai_responses", "OpenAI Responses"),
    ("gemini_native", "Gemini Native"),
];

fn api_format_label(fmt: &str) -> String {
    API_FORMATS
        .iter()
        .find(|(v, _)| *v == fmt)
        .map(|(_, l)| l.to_string())
        .unwrap_or_else(|| fmt.to_string())
}

// ── 页面状态类型（存储在 AppSidebar） ──

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsTab {
    General,
    ModelSettings,
}

const PLATFORM_KEY: &str = "__platform__";
const NEW_KEY: &str = "__new__";
const PRESET_PREFIX: &str = "__preset__:";

pub struct SettingsState {
    pub active_tab: SettingsTab,
    pub providers: Vec<ModelProvider>,
    pub loading: bool,
    pub error_msg: String,
    pub success_msg: String,

    pub selected_key: String,
    pub form_name: String,
    pub form_base_url: String,
    pub form_api_key: String,
    pub form_api_format: String,
    pub form_models: Vec<ModelConfig>,
    pub form_has_api_key: bool,
    pub form_category: String,
    pub form_preset_type: String,
    pub form_website_url: String,
    pub form_api_key_url: String,
    pub form_icon: String,
    pub form_icon_color: String,
    pub form_sort_order: i32,
    pub saving: bool,

    pub show_model_dialog: bool,
    pub editing_model_idx: Option<usize>,
    pub model_form_name: String,
    pub model_form_max_tokens: String,

    pub testing_model_idx: Option<usize>,
    pub test_result: Option<TestModelProviderResponse>,
    pub show_test_result: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            active_tab: SettingsTab::General,
            providers: Vec::new(),
            loading: false,
            error_msg: String::new(),
            success_msg: String::new(),
            selected_key: PLATFORM_KEY.to_string(),
            form_name: String::new(),
            form_base_url: String::new(),
            form_api_key: String::new(),
            form_api_format: "anthropic".to_string(),
            form_models: Vec::new(),
            form_has_api_key: false,
            form_category: "custom".to_string(),
            form_preset_type: String::new(),
            form_website_url: String::new(),
            form_api_key_url: String::new(),
            form_icon: String::new(),
            form_icon_color: String::new(),
            form_sort_order: 0,
            saving: false,
            show_model_dialog: false,
            editing_model_idx: None,
            model_form_name: String::new(),
            model_form_max_tokens: "200000".to_string(),
            testing_model_idx: None,
            test_result: None,
            show_test_result: false,
        }
    }
}

// ── 可编辑输入框（参照 community.rs 手写实现，避免依赖 &mut Window） ──

#[derive(Clone)]
struct EditMeta {
    cursor: usize,
    focus: FocusHandle,
}

thread_local! {
    static EDIT_STATE: RefCell<HashMap<String, EditMeta>> = RefCell::new(HashMap::new());
}

fn edit_meta(id: &str, cx: &App) -> EditMeta {
    EDIT_STATE.with(|m| {
        let mut m = m.borrow_mut();
        if let Some(meta) = m.get(id) {
            return meta.clone();
        }
        let meta = EditMeta {
            cursor: 0,
            focus: cx.focus_handle(),
        };
        m.insert(id.to_string(), meta.clone());
        meta
    })
}

fn edit_cursor(id: &str) -> usize {
    EDIT_STATE.with(|m| m.borrow().get(id).map_or(0, |e| e.cursor))
}

fn set_edit_cursor(id: &str, cursor: usize) {
    EDIT_STATE.with(|m| {
        if let Some(e) = m.borrow_mut().get_mut(id) {
            e.cursor = cursor;
        }
    })
}

/// 处理单个按键，返回（新文本，新光标）。无变化返回 None。
fn apply_key(value: &str, cursor: usize, event: &KeyDownEvent) -> Option<(String, usize)> {
    let ks = &event.keystroke;
    let mods = &ks.modifiers;
    let mut chars: Vec<char> = value.chars().collect();
    let cursor = cursor.min(chars.len());

    // ctrl / cmd 组合键不作为字符输入
    if mods.control || mods.platform {
        return None;
    }

    if let Some(ch) = ks.key_char.as_deref() {
        let insert_chars: Vec<char> = ch.chars().collect();
        if !mods.alt && !insert_chars.is_empty() && !insert_chars.iter().any(|c| c.is_control()) {
            for (i, c) in insert_chars.iter().enumerate() {
                chars.insert(cursor + i, *c);
            }
            return Some((chars.into_iter().collect(), cursor + insert_chars.len()));
        }
    }

    match ks.key.as_str() {
        "backspace" => {
            if cursor > 0 {
                chars.remove(cursor - 1);
                Some((chars.into_iter().collect(), cursor - 1))
            } else {
                None
            }
        }
        "delete" => {
            if cursor < chars.len() {
                chars.remove(cursor);
                Some((chars.into_iter().collect(), cursor))
            } else {
                None
            }
        }
        "left" => Some((value.to_string(), cursor.saturating_sub(1))),
        "right" => Some((value.to_string(), (cursor + 1).min(chars.len()))),
        "home" => Some((value.to_string(), 0)),
        "end" => Some((value.to_string(), chars.len())),
        "space" => {
            chars.insert(cursor, ' ');
            Some((chars.into_iter().collect(), cursor + 1))
        }
        _ => None,
    }
}

/// 可聚焦、可键盘编辑的文本输入框。get_value 读 live 值，set_value 写回 sidebar 字段。
fn render_edit_input(
    sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
    id: &str,
    placeholder: &str,
    get_value: impl Fn(&AppSidebar) -> String + 'static,
    set_value: impl Fn(&mut AppSidebar, String) + 'static,
) -> AnyElement {
    let value = get_value(sidebar);
    let meta = edit_meta(id, cx);
    let focus_handle = meta.focus.clone();
    let empty = value.is_empty();
    let chars: Vec<char> = value.chars().collect();
    let cursor = meta.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let id_owned = id.to_string();

    let listener = cx.listener(move |this, event: &KeyDownEvent, _window, cx| {
        let live = get_value(this);
        let cur = edit_cursor(&id_owned);
        if let Some((nv, nc)) = apply_key(&live, cur, event) {
            set_value(this, nv);
            set_edit_cursor(&id_owned, nc);
            cx.notify();
        }
    });

    div()
        .track_focus(&focus_handle)
        .px_2()
        .py_1()
        .w_full()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_md()
        .bg(cx.theme().background)
        .text_sm()
        .on_key_down(listener)
        .child(
            h_flex()
                .items_center()
                .when(empty, |d| d.text_color(muted).child(placeholder.to_string()))
                .when(!empty, |d| {
                    d.child(before)
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(after)
                }),
        )
        .into_any_element()
}

fn render_edit_field(
    id: &str,
    label: impl Into<SharedString>,
    placeholder: &str,
    sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
    get_value: impl Fn(&AppSidebar) -> String + 'static,
    set_value: impl Fn(&mut AppSidebar, String) + 'static,
) -> AnyElement {
    v_flex()
        .gap_1()
        .child(div().text_xs().font_bold().child(label.into()))
        .child(render_edit_input(sidebar, cx, id, placeholder, get_value, set_value))
        .into_any_element()
}

// ── 主渲染函数 ──

pub fn render_settings(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let tab = sidebar.settings.active_tab;

    let tab_general = make_tab(
        "tab-general",
        "常规设置",
        tab == SettingsTab::General,
        cx,
        |this, cx| {
            this.settings.active_tab = SettingsTab::General;
            cx.notify();
        },
    );
    let tab_models = make_tab(
        "tab-models",
        "模型设置",
        tab == SettingsTab::ModelSettings,
        cx,
        |this, cx| {
            this.settings.active_tab = SettingsTab::ModelSettings;
            cx.notify();
        },
    );

    let content = match tab {
        SettingsTab::General => render_general(sidebar, cx),
        SettingsTab::ModelSettings => render_model_settings(sidebar, cx),
    };

    v_flex()
        .size_full()
        .overflow_hidden()
        .child(
            h_flex()
                .gap_1()
                .px_4()
                .py_2()
                .child(tab_general)
                .child(tab_models),
        )
        .child(Separator::horizontal())
        .child(
            div()
                .flex_1()
                .w_full()
                .id("settings-scroll")
                .px_6()
                .py_6()
                .child(content),
        )
        .into_any_element()
}

fn make_tab(
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

// ── General Tab ──

fn render_general(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let locale = sidebar.locale.clone();
    let is_dark = cx.theme().is_dark();

    let theme_dark = {
        let btn = Button::new("theme-dark").label("深色");
        let btn = if is_dark { btn.primary() } else { btn.outline() };
        btn.on_click(cx.listener(|_, _, window, cx| {
            Theme::change(ThemeMode::Dark, Some(window), cx);
        }))
    };
    let theme_light = {
        let btn = Button::new("theme-light").label("浅色");
        let btn = if is_dark { btn.outline() } else { btn.primary() };
        btn.on_click(cx.listener(|_, _, window, cx| {
            Theme::change(ThemeMode::Light, Some(window), cx);
        }))
    };

    let lang_zh = {
        let btn = Button::new("lang-zh").label("简体中文");
        let btn = if locale == "zh-CN" {
            btn.primary()
        } else {
            btn.outline()
        };
        btn.on_click(cx.listener(|this, _, _, cx| {
            this.change_locale("zh-CN", cx);
        }))
    };
    let lang_en = {
        let btn = Button::new("lang-en").label("English");
        let btn = if locale == "en" {
            btn.primary()
        } else {
            btn.outline()
        };
        btn.on_click(cx.listener(|this, _, _, cx| {
            this.change_locale("en", cx);
        }))
    };

    v_flex()
        .gap_6()
        .child(v_flex().gap_2().child(
            div().text_xl().font_bold().child("常规设置"),
        ))
        .child(
            v_flex()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .p_5()
                .gap_4()
                .child(div().text_sm().font_bold().child("外观与语言"))
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("界面主题"),
                        )
                        .child(h_flex().gap_2().child(theme_dark).child(theme_light)),
                )
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("界面语言"),
                        )
                        .child(h_flex().gap_2().child(lang_zh).child(lang_en)),
                ),
        )
        .into_any_element()
}

// ── Model Settings Tab ──

fn render_model_settings(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let show_model_dialog = sidebar.settings.show_model_dialog;
    let show_test_result = sidebar.settings.show_test_result;

    let dialog = if show_model_dialog {
        render_model_dialog(sidebar, cx).into_any_element()
    } else if show_test_result {
        render_test_result_dialog(sidebar, cx).into_any_element()
    } else {
        div().into_any_element()
    };

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
                        .child(render_provider_form(sidebar, cx)),
                ),
        )
        .child(dialog)
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

fn render_provider_form(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
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
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(m) = this.settings.form_models.get(i) {
                                this.settings.model_form_name = m.name.clone();
                                this.settings.model_form_max_tokens = m.max_tokens.to_string();
                            }
                            this.settings.editing_model_idx = Some(i);
                            this.settings.show_model_dialog = true;
                            cx.notify();
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
            cx,
            |t| t.settings.form_name.clone(),
            |t, v| t.settings.form_name = v,
        ))
        .child(render_edit_field(
            "form-base-url",
            "Base URL",
            "https://api.example.com/v1",
            sidebar,
            cx,
            |t| t.settings.form_base_url.clone(),
            |t, v| t.settings.form_base_url = v,
        ))
        .child(render_edit_field(
            "form-api-key",
            "API Key",
            &api_key_placeholder_text,
            sidebar,
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
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.settings.show_model_dialog = true;
                            this.settings.editing_model_idx = None;
                            this.settings.model_form_name.clear();
                            this.settings.model_form_max_tokens = "200000".to_string();
                            cx.notify();
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
                menu = menu.item(
                    PopupMenuItem::new(label)
                        .checked(current == val)
                        .on_click(move |_, _, cx| {
                            let _ = weak.update(cx, |this, cx| {
                                this.settings.form_api_format = val.clone();
                                cx.notify();
                            });
                        }),
                );
            }
            menu
        })
        .into_any_element()
}

// ── 模型新增 / 编辑对话框 ──

fn render_model_dialog(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let editing = sidebar.settings.editing_model_idx.is_some();

    div()
        .absolute()
        .inset_0()
        .bg(gpui::black().opacity(0.4))
        .flex()
        .items_center()
        .justify_center()
        .on_any_mouse_down(cx.listener(|this, _, _, cx| {
            this.settings.show_model_dialog = false;
            cx.notify();
        }))
        .child(
            div()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .p_6()
                .w_96()
                .flex()
                .flex_col()
                .gap_4()
                .on_any_mouse_down(|_, _, _| {})
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(div().font_bold().text_sm().child(if editing {
                            "编辑模型"
                        } else {
                            "添加模型"
                        }))
                        .child(
                            Button::new("close-model-dialog")
                                .ghost()
                                .icon(IconName::Close)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.settings.show_model_dialog = false;
                                    cx.notify();
                                })),
                        ),
                )
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
                            cx,
                            |t| t.settings.model_form_name.clone(),
                            |t, v| t.settings.model_form_name = v,
                        ))
                        .child(render_edit_field(
                            "model-dialog-tokens",
                            "最大上下文 Token 数",
                            "200000",
                            sidebar,
                            cx,
                            |t| t.settings.model_form_max_tokens.clone(),
                            |t, v| {
                                t.settings.model_form_max_tokens =
                                    v.chars().filter(|c| c.is_ascii_digit()).collect();
                            },
                        )),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .justify_end()
                        .child(
                            Button::new("model-dialog-cancel")
                                .outline()
                                .label("取消")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.settings.show_model_dialog = false;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("model-dialog-confirm")
                                .primary()
                                .label("确定")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    handle_save_model(this, cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
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
    sidebar.settings.show_model_dialog = false;
    cx.notify();
}

// ── 连接测试结果对话框 ──

fn render_test_result_dialog(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
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

    div()
        .absolute()
        .inset_0()
        .bg(gpui::black().opacity(0.4))
        .flex()
        .items_center()
        .justify_center()
        .on_any_mouse_down(cx.listener(|this, _, _, cx| {
            this.settings.show_test_result = false;
            cx.notify();
        }))
        .child(
            div()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .p_6()
                .w_96()
                .flex()
                .flex_col()
                .gap_4()
                .on_any_mouse_down(|_, _, _| {})
                .child(
                    div()
                        .font_bold()
                        .text_sm()
                        .text_color(if success { cx.theme().accent } else { cx.theme().danger })
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
                .child(
                    h_flex()
                        .gap_2()
                        .justify_end()
                        .child(
                            Button::new("test-result-close")
                                .primary()
                                .label("确定")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.settings.show_test_result = false;
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .into_any_element()
}

// ── 操作逻辑 ──

fn is_new_key(key: &str) -> bool {
    key == NEW_KEY || key.starts_with(PRESET_PREFIX)
}

fn is_provider_key(key: &str) -> bool {
    key != PLATFORM_KEY && !is_new_key(key)
}

fn find_preset_by_key(key: &str) -> Option<&'static ProviderPreset> {
    let preset_key = key.strip_prefix(PRESET_PREFIX)?;
    PROVIDER_PRESETS
        .iter()
        .find(|p| p.preset_type == preset_key)
}

fn api_key_placeholder(sidebar: &AppSidebar) -> String {
    if sidebar.settings.form_has_api_key {
        "已设置，留空保持不变".to_string()
    } else if let Some(preset) = find_preset_by_key(&sidebar.settings.selected_key) {
        if preset.api_key_url.is_empty() {
            "输入 API Key".to_string()
        } else {
            format!("输入 API Key（可在 {} 申请）", preset.api_key_url)
        }
    } else {
        "输入 API Key".to_string()
    }
}

fn reset_form(sidebar: &mut AppSidebar) {
    sidebar.settings.form_name.clear();
    sidebar.settings.form_base_url.clear();
    sidebar.settings.form_api_key.clear();
    sidebar.settings.form_api_format = "anthropic".to_string();
    sidebar.settings.form_models.clear();
    sidebar.settings.form_has_api_key = false;
    sidebar.settings.form_category = "custom".to_string();
    sidebar.settings.form_preset_type.clear();
    sidebar.settings.form_website_url.clear();
    sidebar.settings.form_api_key_url.clear();
    sidebar.settings.form_icon.clear();
    sidebar.settings.form_icon_color.clear();
    sidebar.settings.form_sort_order = 0;
}

fn apply_provider_to_form(sidebar: &mut AppSidebar, p: &ModelProvider) {
    sidebar.settings.form_name = p.name.clone();
    sidebar.settings.form_base_url = p.base_url.clone();
    sidebar.settings.form_api_key.clear();
    sidebar.settings.form_api_format = p.api_format.clone();
    sidebar.settings.form_models = p.models.clone();
    sidebar.settings.form_has_api_key = p.has_api_key;
    sidebar.settings.form_category = p.category.clone();
    sidebar.settings.form_preset_type = p.preset_type.clone();
    sidebar.settings.form_website_url = p.website_url.clone().unwrap_or_default();
    sidebar.settings.form_api_key_url = p.api_key_url.clone().unwrap_or_default();
    sidebar.settings.form_icon = p.icon.clone().unwrap_or_default();
    sidebar.settings.form_icon_color = p.icon_color.clone().unwrap_or_default();
    sidebar.settings.form_sort_order = p.sort_order;
}

fn select_provider(sidebar: &mut AppSidebar, key: &str, cx: &mut Context<AppSidebar>) {
    sidebar.settings.selected_key = key.to_string();
    sidebar.settings.error_msg.clear();
    sidebar.settings.success_msg.clear();

    if key == PLATFORM_KEY {
        cx.notify();
        return;
    }

    // 预设项：以「新增」模式预填表单
    if let Some(preset) = find_preset_by_key(key) {
        reset_form(sidebar);
        sidebar.settings.form_name = preset.name.to_string();
        sidebar.settings.form_base_url = preset.base_url.to_string();
        sidebar.settings.form_api_format = preset.api_format.to_string();
        sidebar.settings.form_models = preset
            .default_models
            .iter()
            .map(|n| ModelConfig {
                name: n.to_string(),
                max_tokens: 200000,
            })
            .collect();
        sidebar.settings.form_category = "preset".to_string();
        sidebar.settings.form_preset_type = preset.preset_type.to_string();
        sidebar.settings.form_website_url = preset.website_url.to_string();
        sidebar.settings.form_api_key_url = preset.api_key_url.to_string();
        sidebar.settings.form_icon = preset.icon.to_string();
        sidebar.settings.form_icon_color = preset.icon_color.to_string();
        cx.notify();
        return;
    }

    if key == NEW_KEY {
        reset_form(sidebar);
        cx.notify();
        return;
    }

    if let Some(p) = sidebar
        .settings
        .providers
        .iter()
        .find(|p| p.id.to_string() == key)
        .cloned()
    {
        apply_provider_to_form(sidebar, &p);
    }
    cx.notify();
}

fn handle_save_provider(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let name = sidebar.settings.form_name.trim().to_string();
    if name.is_empty() {
        sidebar.settings.error_msg = "名称不能为空".to_string();
        cx.notify();
        return;
    }

    sidebar.settings.saving = true;
    let input = ModelProviderInput {
        name,
        category: sidebar.settings.form_category.clone(),
        preset_type: sidebar.settings.form_preset_type.clone(),
        base_url: sidebar.settings.form_base_url.trim().to_string(),
        api_key: sidebar.settings.form_api_key.clone(),
        api_format: sidebar.settings.form_api_format.clone(),
        models: sidebar.settings.form_models.clone(),
        enabled: true,
        website_url: sidebar.settings.form_website_url.clone(),
        api_key_url: sidebar.settings.form_api_key_url.clone(),
        icon: sidebar.settings.form_icon.clone(),
        icon_color: sidebar.settings.form_icon_color.clone(),
        sort_order: sidebar.settings.form_sort_order,
    };

    let cloud = sidebar.cloud.clone();
    let editing_id = if is_provider_key(&sidebar.settings.selected_key) {
        Uuid::parse_str(&sidebar.settings.selected_key).ok()
    } else {
        None
    };

    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let created = if let Some(id) = editing_id {
                    cloud
                        .update_model_provider(&id.to_string(), &input)
                        .await
                        .map(|_| None)
                } else {
                    cloud.create_model_provider(&input).await.map(Some)
                };

                // reload providers
                let c2 = match this.update(&mut cx, |t, _| t.cloud.clone()) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                if let Ok(pros) = c2.list_model_providers().await {
                    this.update(&mut cx, |t, cx| {
                        t.settings.providers = pros;
                        cx.notify();
                    })
                    .ok();
                }

                this.update(&mut cx, |this, cx| {
                    this.settings.saving = false;
                    match created {
                        Ok(Some(p)) => {
                            this.settings.selected_key = p.id.to_string();
                            apply_provider_to_form(this, &p);
                            this.settings.success_msg = "已保存".to_string();
                        }
                        Ok(None) => {
                            this.settings.success_msg = "已保存".to_string();
                        }
                        Err(e) => {
                            this.settings.error_msg = format!("{}", e);
                        }
                    }
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

fn handle_delete_provider(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let id = match Uuid::parse_str(&sidebar.settings.selected_key) {
        Ok(id) => id,
        Err(_) => return,
    };
    let cloud = sidebar.cloud.clone();
    let id_str = id.to_string();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let _ = cloud.delete_model_provider(&id_str).await;

                // reload providers
                let c2 = match this.update(&mut cx, |t, _| t.cloud.clone()) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                if let Ok(pros) = c2.list_model_providers().await {
                    this.update(&mut cx, |t, cx| {
                        t.settings.providers = pros;
                        cx.notify();
                    })
                    .ok();
                }

                this.update(&mut cx, |this, cx| {
                    this.settings.selected_key = PLATFORM_KEY.to_string();
                    reset_form(this);
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

fn handle_test_model(sidebar: &mut AppSidebar, idx: usize, cx: &mut Context<AppSidebar>) {
    let base_url = sidebar.settings.form_base_url.trim().to_string();
    if base_url.is_empty() {
        sidebar.settings.error_msg = "Base URL 不能为空".to_string();
        cx.notify();
        return;
    }

    let model = match sidebar.settings.form_models.get(idx) {
        Some(m) => m.clone(),
        None => return,
    };

    sidebar.settings.testing_model_idx = Some(idx);
    sidebar.settings.test_result = None;

    let cloud = sidebar.cloud.clone();
    let api_key = sidebar.settings.form_api_key.clone();
    let api_format = sidebar.settings.form_api_format.clone();

    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let input = lol_web_protocol::model_provider::TestModelProviderInput {
                    provider_id: None,
                    base_url,
                    api_key: if api_key.is_empty() {
                        None
                    } else {
                        Some(api_key)
                    },
                    api_format,
                    model: model.name,
                    max_tokens: Some(model.max_tokens),
                };
                let result = cloud.test_model_provider(&input).await;

                this.update(&mut cx, |this, cx| {
                    this.settings.testing_model_idx = None;
                    match result {
                        Ok(resp) => {
                            this.settings.test_result = Some(resp);
                            this.settings.show_test_result = true;
                        }
                        Err(e) => {
                            this.settings.test_result = Some(TestModelProviderResponse {
                                success: false,
                                message: e.to_string(),
                            });
                            this.settings.show_test_result = true;
                        }
                    }
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

// ── 远程模型刷新（可选能力）：GET {baseUrl}/v1/models，按名称去重合并进表单模型列表 ──

fn handle_refresh_models(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let base_url = sidebar.settings.form_base_url.trim().to_string();
    if base_url.is_empty() {
        sidebar.settings.error_msg = "请先填写 Base URL".to_string();
        cx.notify();
        return;
    }

    let api_key = sidebar.settings.form_api_key.clone();
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
    let existing: HashSet<String> = sidebar
        .settings
        .form_models
        .iter()
        .map(|m| m.name.clone())
        .collect();

    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = crate::services::runtime::run_on_tokio(move || async move {
                    let client = reqwest::Client::new();
                    let mut req = client.get(&url);
                    if !api_key.is_empty() {
                        req = req.header("Authorization", format!("Bearer {}", api_key));
                    }
                    let resp = req.send().await.map_err(|e| e.to_string())?;
                    let text = resp.text().await.map_err(|e| e.to_string())?;
                    let data: serde_json::Value =
                        serde_json::from_str(&text).map_err(|e| e.to_string())?;
                    Ok::<serde_json::Value, String>(data)
                })
                .await;

                match result {
                    Ok(data) => {
                        let mut new_models: Vec<String> = Vec::new();
                        let arr = data.get("data").or_else(|| data.get("models"));
                        if let Some(serde_json::Value::Array(items)) = arr {
                            for item in items {
                                let name = match item {
                                    serde_json::Value::String(s) => s.clone(),
                                    serde_json::Value::Object(o) => o
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_default(),
                                    _ => String::new(),
                                };
                                if !name.is_empty() && !existing.contains(&name) {
                                    new_models.push(name);
                                }
                            }
                        }
                        let count = new_models.len();
                        this.update(&mut cx, |this, cx| {
                            for n in new_models {
                                this.settings.form_models.push(ModelConfig {
                                    name: n,
                                    max_tokens: 200000,
                                });
                            }
                            this.settings.success_msg = format!("已合并 {} 个远程模型", count);
                            cx.notify();
                        })
                        .ok();
                    }
                    Err(e) => {
                        this.update(&mut cx, |this, cx| {
                            this.settings.error_msg = format!("刷新失败：{}", e);
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
