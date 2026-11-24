//! 基于 gpui_component `Input` 的统一文本输入框封装。
//!
//! `InputState` 惰性创建（需 `window`）并跨渲染持有；`cx.subscribe` 监听
//! `Change`（实时写回）与 `PressEnter`（Enter 提交）。替代原先散落在
//! community / auth_dialog / mock / logs_archive / rooms / heroes / particles /
//! settings 等 8 处的手写输入框。

use std::cell::RefCell;
use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState, Textarea, TextareaState};

use crate::components::sidebar::AppSidebar;

/// 输入框渲染选项。
#[derive(Default)]
pub struct EditOptions {
    /// 多行输入（Enter 换行；与 `submit_on_enter` 互斥）。
    pub multiline: bool,
    /// 掩码显示（密码等）。
    pub masked: bool,
    /// Enter 提交（多行时 Enter 不换行，改用提交）。
    pub submit_on_enter: bool,
    /// Enter 提交回调，参数为输入框当前文本 + 可写 sidebar（页面状态直访）。
    pub on_enter: Option<Box<dyn Fn(String, &mut AppSidebar, &mut Context<AppSidebar>) + 'static>>,
}

/// 单行用 `InputState`，多行用 `TextareaState`（0.11 起多行是独立状态类型）。
#[derive(Clone)]
enum EditState {
    Single(Entity<InputState>),
    Multi(Entity<TextareaState>),
}

thread_local! {
    static STATES: RefCell<HashMap<String, EditState>> = RefCell::new(HashMap::new());
    /// 事件订阅需跨渲染存活（drop 会取消订阅）。
    static SUBS: RefCell<HashMap<String, Subscription>> = RefCell::new(HashMap::new());
}

/// 可聚焦、可键盘编辑的文本输入框（gpui_component `Input`）。
///
/// sidebar 来自调用方 render 持有的实体状态（render 期间实体被 gpui lease，
/// 不能再 `cx.entity().read(cx)`）。get_value 读实时值，set_value 在 Change 时写回；
/// 焦点、光标、未提交文本由 `InputState` 跨渲染保持。
pub fn render_edit_input(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    sidebar: &AppSidebar,
    id: &str,
    placeholder: &str,
    opts: EditOptions,
    get_value: impl Fn(&AppSidebar) -> String + 'static,
    set_value: impl Fn(&mut AppSidebar, String) + 'static,
) -> AnyElement {
    let state = STATES
        .with(|s| s.borrow().get(id).cloned())
        .unwrap_or_else(|| {
            let init = get_value(sidebar);
            let state = if opts.multiline {
                EditState::Multi(create_textarea(window, cx, placeholder, &init, &opts))
            } else {
                EditState::Single(create_input(window, cx, placeholder, &init, &opts))
            };
            let sub = match &state {
                EditState::Single(ed) => {
                    subscribe_edit(ed, |s| s.value().to_string(), set_value, opts.on_enter, cx)
                }
                EditState::Multi(ed) => {
                    subscribe_edit(ed, |s| s.value().to_string(), set_value, opts.on_enter, cx)
                }
            };
            STATES.with(|s| s.borrow_mut().insert(id.to_string(), state.clone()));
            SUBS.with(|s| s.borrow_mut().insert(id.to_string(), sub));
            state
        });

    // 外部值 → 输入状态同步（外部清空/加载时保持一致；输入中二者相等则跳过）
    let external = get_value(sidebar);
    match &state {
        EditState::Single(ed) => {
            if ed.read(cx).value().to_string() != external {
                cx.update_entity(ed, |s, cx| s.set_value(external.clone(), window, cx));
            }
            Input::new(ed).into_any_element()
        }
        EditState::Multi(ed) => {
            if ed.read(cx).value().to_string() != external {
                cx.update_entity(ed, |s, cx| s.set_value(external.clone(), window, cx));
            }
            Textarea::new(ed).into_any_element()
        }
    }
}

fn create_input(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    placeholder: &str,
    init: &str,
    opts: &EditOptions,
) -> Entity<InputState> {
    cx.new(|cx| {
        let st = InputState::new(window, cx)
            .placeholder(placeholder)
            .masked(opts.masked)
            .submit_on_enter(opts.submit_on_enter);
        st.default_value(init)
    })
}

fn create_textarea(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    placeholder: &str,
    init: &str,
    opts: &EditOptions,
) -> Entity<TextareaState> {
    cx.new(|cx| {
        TextareaState::new(window, cx)
            .placeholder(placeholder)
            .submit_on_enter(opts.submit_on_enter)
            .default_value(init)
    })
}

/// 创建时订阅一次：Change 实时写回，PressEnter 触发提交。
fn subscribe_edit<S: EventEmitter<InputEvent> + 'static>(
    ed: &Entity<S>,
    value_of: fn(&S) -> String,
    set_value: impl Fn(&mut AppSidebar, String) + 'static,
    on_enter: Option<Box<dyn Fn(String, &mut AppSidebar, &mut Context<AppSidebar>) + 'static>>,
    cx: &mut Context<AppSidebar>,
) -> Subscription {
    cx.subscribe(
        ed,
        move |this, state_entity, event: &InputEvent, cx| match event {
            InputEvent::Change => {
                let text = value_of(state_entity.read(cx));
                set_value(this, text);
                cx.notify();
            }
            InputEvent::PressEnter { .. } => {
                if let Some(f) = on_enter.as_ref() {
                    let text = value_of(state_entity.read(cx));
                    f(text, this, cx);
                }
            }
            _ => {}
        },
    )
}
