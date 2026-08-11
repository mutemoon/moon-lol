//! 输入控件：搜索 / 数字 / 文本输入（复用共享输入框）+ 采样器下拉 / 标志开关。

use std::cell::RefCell;
use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::checkbox::Checkbox;
use gpui_component::menu::{DropdownMenu, PopupMenuItem};

use super::edit::{set_flag_idx, set_sampler_mode_idx, FlagField, SamplerKind};
use super::play::replay_after_edit;
use super::state::format_number;
use crate::components::sidebar::AppSidebar;
use crate::components::text_input::{self, EditOptions};

// ── 输入缓冲：未提交文本跨渲染保持（Enter 提交到 STATE） ──

thread_local! {
    static BUFS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

pub(super) fn input_buffer(id: &str) -> Option<String> {
    BUFS.with(|b| b.borrow().get(id).cloned())
}

pub(super) fn set_input_buffer(id: &str, val: String) {
    BUFS.with(|b| {
        b.borrow_mut().insert(id.to_string(), val);
    })
}

pub(super) fn clear_input_buffer(id: &str) {
    BUFS.with(|b| {
        b.borrow_mut().remove(id);
    })
}

pub(super) fn clear_all_input_buffers() {
    BUFS.with(|b| b.borrow_mut().clear());
}

// ── 输入控件 ──

pub(super) fn render_search_input(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    sidebar: &AppSidebar,
) -> AnyElement {
    text_input::render_edit_input(
        window,
        cx,
        sidebar,
        "particle-search",
        "搜索英雄 / 粒子",
        EditOptions::default(),
        |s: &AppSidebar| s.particles.search_query.clone(),
        |s: &mut AppSidebar, v| s.particles.search_query = v,
    )
}

/// 手写数字输入框：回车提交（Enter）→ commit(v)；非法输入回车则回退。
pub(super) fn render_number_input(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    sidebar: &AppSidebar,
    id: String,
    value: f32,
    commit: impl Fn(&mut AppSidebar, f32) + 'static,
) -> AnyElement {
    let id_enter = id.clone();
    let on_enter = Box::new(
        move |text: String, sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>| match text
            .trim()
            .parse::<f32>()
        {
            Ok(v) => {
                commit(sidebar, v);
                clear_input_buffer(&id_enter);
                replay_after_edit(sidebar, cx);
            }
            Err(_) => {
                clear_input_buffer(&id_enter);
                cx.notify();
            }
        },
    );
    let id_get = id.clone();
    let id_set = id.clone();
    text_input::render_edit_input(
        window,
        cx,
        sidebar,
        &id,
        "0",
        EditOptions {
            on_enter: Some(on_enter),
            ..Default::default()
        },
        move |_s| input_buffer(&id_get).unwrap_or_else(|| format_number(value)),
        move |_s, v| set_input_buffer(&id_set, v),
    )
}

/// 手写文本输入框：回车提交（Enter）→ commit(text)。
pub(super) fn render_text_input(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    sidebar: &AppSidebar,
    id: String,
    value: String,
    placeholder: &str,
    commit: impl Fn(&mut AppSidebar, String) + 'static,
) -> AnyElement {
    let id_enter = id.clone();
    let on_enter = Box::new(
        move |text: String, sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>| {
            commit(sidebar, text.clone());
            clear_input_buffer(&id_enter);
            replay_after_edit(sidebar, cx);
        },
    );
    let id_get = id.clone();
    let value_get = value.clone();
    let id_set = id.clone();
    let placeholder_owned = placeholder.to_string();
    text_input::render_edit_input(
        window,
        cx,
        sidebar,
        &id,
        &placeholder_owned,
        EditOptions {
            on_enter: Some(on_enter),
            ..Default::default()
        },
        move |_s| input_buffer(&id_get).unwrap_or_else(|| value_get.clone()),
        move |_s, v| set_input_buffer(&id_set, v),
    )
}

/// 常量/曲线预设下拉。
pub(super) fn render_sampler_mode_dropdown(
    cx: &mut Context<AppSidebar>,
    id: String,
    idx: usize,
    kind: SamplerKind,
    is_curve: bool,
) -> AnyElement {
    let weak = cx.entity().downgrade();
    Button::new(id)
        .label(if is_curve { "曲线" } else { "常量" })
        .outline()
        .dropdown_menu(move |menu, _window, _cx| {
            let w1 = weak.clone();
            let w2 = weak.clone();
            menu.item(
                PopupMenuItem::new("常量")
                    .checked(!is_curve)
                    .on_click(move |_, _, cx| {
                        let _ = w1.update(cx, |sidebar, cx| {
                            set_sampler_mode_idx(sidebar, idx, kind, false);
                            replay_after_edit(sidebar, cx);
                        });
                    }),
            )
            .item(
                PopupMenuItem::new("曲线")
                    .checked(is_curve)
                    .on_click(move |_, _, cx| {
                        let _ = w2.update(cx, |sidebar, cx| {
                            set_sampler_mode_idx(sidebar, idx, kind, true);
                            replay_after_edit(sidebar, cx);
                        });
                    }),
            )
        })
        .into_any_element()
}

/// 布尔开关（渲染标志）。
pub(super) fn render_flag_toggle(
    cx: &mut Context<AppSidebar>,
    id: String,
    idx: usize,
    flag: FlagField,
    label: &str,
    checked: bool,
) -> AnyElement {
    let weak = cx.entity().downgrade();
    Checkbox::new(id)
        .checked(checked)
        .label(label)
        .on_click(move |new_checked, _, cx| {
            let _ = weak.update(cx, |sidebar, cx| {
                set_flag_idx(sidebar, idx, flag, *new_checked);
                replay_after_edit(sidebar, cx);
            });
        })
        .into_any_element()
}
