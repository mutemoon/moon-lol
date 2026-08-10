//! 手写输入框（焦点/光标/文本缓冲跨渲染保持）+ 各输入控件。

use std::cell::RefCell;
use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::checkbox::Checkbox;
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::{h_flex, ActiveTheme};

use super::edit::{set_flag_idx, set_sampler_mode_idx, FlagField, SamplerKind};
use super::play::replay_after_edit;
use super::state::{format_number, update_state, with_state};
use crate::components::sidebar::AppSidebar;

// ── 手写输入框：焦点 / 光标 / 文本缓冲（跨渲染保持） ──

#[derive(Clone)]
struct EditMeta {
    cursor: usize,
    focus: FocusHandle,
}

thread_local! {
    static EDITS: RefCell<HashMap<String, EditMeta>> = RefCell::new(HashMap::new());
    static BUFS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

fn edit_meta(id: &str, cx: &App) -> EditMeta {
    EDITS.with(|m| {
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

pub(super) fn edit_cursor(id: &str) -> usize {
    EDITS.with(|m| m.borrow().get(id).map_or(0, |e| e.cursor))
}

pub(super) fn set_edit_cursor(id: &str, cursor: usize) {
    EDITS.with(|m| {
        if let Some(e) = m.borrow_mut().get_mut(id) {
            e.cursor = cursor;
        }
    })
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

/// 处理单个按键，返回（新文本，新光标）。无变化返回 None。
fn apply_key(value: &str, cursor: usize, event: &KeyDownEvent) -> Option<(String, usize)> {
    let ks = &event.keystroke;
    let mods = &ks.modifiers;
    let mut chars: Vec<char> = value.chars().collect();
    let cursor = cursor.min(chars.len());

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

// ── 输入控件 ──

pub(super) fn render_search_input(cx: &mut Context<AppSidebar>, value: String) -> AnyElement {
    let id = "particle-search".to_string();
    let meta = edit_meta(&id, cx);
    let focus_handle = meta.focus.clone();
    let empty = value.is_empty();
    let chars: Vec<char> = value.chars().collect();
    let cursor = meta.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let id2 = id.clone();

    let listener = cx.listener(move |_this, event: &KeyDownEvent, _window, cx| {
        let live = with_state(|s| s.search_query.clone());
        let cur = edit_cursor(&id2);
        if let Some((nv, nc)) = apply_key(&live, cur, event) {
            update_state(|s| s.search_query = nv);
            set_edit_cursor(&id2, nc);
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
                .when(empty, |d| d.text_color(muted).child("搜索英雄 / 粒子"))
                .when(!empty, |d| {
                    d.child(before)
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(after)
                }),
        )
        .into_any_element()
}

/// 手写数字输入框：回车提交（Enter）→ commit(v)；非法输入回车则回退。
pub(super) fn render_number_input(
    cx: &mut Context<AppSidebar>,
    id: String,
    value: f32,
    commit: impl Fn(f32) + 'static,
) -> AnyElement {
    let meta = edit_meta(&id, cx);
    let focus_handle = meta.focus.clone();
    let buf = input_buffer(&id);
    let display = buf.unwrap_or_else(|| format_number(value));
    let empty = display.is_empty();
    let chars: Vec<char> = display.chars().collect();
    let cursor = meta.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let id2 = id.clone();

    let listener = cx.listener(move |_this, event: &KeyDownEvent, _window, cx| {
        let live = input_buffer(&id2).unwrap_or_else(|| format_number(value));
        let cur = edit_cursor(&id2);
        if let Some((nv, nc)) = apply_key(&live, cur, event) {
            set_input_buffer(&id2, nv);
            set_edit_cursor(&id2, nc);
            cx.notify();
        } else if event.keystroke.key == "enter" {
            match live.trim().parse::<f32>() {
                Ok(v) => {
                    commit(v);
                    clear_input_buffer(&id2);
                    set_edit_cursor(&id2, 0);
                    replay_after_edit(cx);
                }
                Err(_) => {
                    clear_input_buffer(&id2);
                    set_edit_cursor(&id2, 0);
                    cx.notify();
                }
            }
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
                .when(empty, |d| d.text_color(muted).child("0"))
                .when(!empty, |d| {
                    d.child(before)
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(after)
                }),
        )
        .into_any_element()
}

/// 手写文本输入框：回车提交（Enter）→ commit(text)。
pub(super) fn render_text_input(
    cx: &mut Context<AppSidebar>,
    id: String,
    value: String,
    placeholder: &str,
    commit: impl Fn(String) + 'static,
) -> AnyElement {
    let meta = edit_meta(&id, cx);
    let focus_handle = meta.focus.clone();
    let buf = input_buffer(&id);
    let display = buf.unwrap_or_else(|| value.clone());
    let empty = display.is_empty();
    let chars: Vec<char> = display.chars().collect();
    let cursor = meta.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let id2 = id.clone();
    let placeholder_owned = placeholder.to_string();

    let listener = cx.listener(move |_this, event: &KeyDownEvent, _window, cx| {
        let live = input_buffer(&id2).unwrap_or_else(|| value.clone());
        let cur = edit_cursor(&id2);
        if let Some((nv, nc)) = apply_key(&live, cur, event) {
            set_input_buffer(&id2, nv);
            set_edit_cursor(&id2, nc);
            cx.notify();
        } else if event.keystroke.key == "enter" {
            commit(live.clone());
            clear_input_buffer(&id2);
            set_edit_cursor(&id2, 0);
            replay_after_edit(cx);
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
                .when(empty, |d| d.text_color(muted).child(placeholder_owned))
                .when(!empty, |d| {
                    d.child(before)
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(after)
                }),
        )
        .into_any_element()
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
                        set_sampler_mode_idx(idx, kind, false);
                        let _ = w1.update(cx, |_, cx| replay_after_edit(cx));
                    }),
            )
            .item(
                PopupMenuItem::new("曲线")
                    .checked(is_curve)
                    .on_click(move |_, _, cx| {
                        set_sampler_mode_idx(idx, kind, true);
                        let _ = w2.update(cx, |_, cx| replay_after_edit(cx));
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
            set_flag_idx(idx, flag, *new_checked);
            let _ = weak.update(cx, |_, cx| replay_after_edit(cx));
        })
        .into_any_element()
}
