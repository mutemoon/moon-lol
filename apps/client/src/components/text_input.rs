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
use gpui_component::input::{Input, InputEvent, InputState};

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

thread_local! {
    static STATES: RefCell<HashMap<String, Entity<InputState>>> = RefCell::new(HashMap::new());
    /// 事件订阅需跨渲染存活（drop 会取消订阅）。
    static SUBS: RefCell<HashMap<String, Subscription>> = RefCell::new(HashMap::new());
}

/// 可聚焦、可键盘编辑的文本输入框（gpui_component `Input`）。
///
/// get_value 读实时值（从 sidebar 或页面状态），set_value 在 Change 时写回；
/// 焦点、光标、未提交文本由 `InputState` 跨渲染保持。
pub fn render_edit_input(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    id: &str,
    placeholder: &str,
    opts: EditOptions,
    get_value: impl Fn(&AppSidebar) -> String + 'static,
    set_value: impl Fn(&mut AppSidebar, String) + 'static,
) -> AnyElement {
    let state = STATES
        .with(|s| s.borrow().get(id).cloned())
        .unwrap_or_else(|| {
            let init = {
                let entity = cx.entity();
                let sidebar = entity.read(cx);
                get_value(sidebar)
            };
            let ed = cx.new(|cx| {
                let mut st = InputState::new(window, cx).placeholder(placeholder);
                if opts.multiline {
                    st = st.multi_line(true);
                }
                if opts.masked {
                    st = st.masked(true);
                }
                if opts.submit_on_enter {
                    st = st.submit_on_enter(true);
                }
                st.default_value(init)
            });
            // 创建时订阅一次：Change 实时写回，PressEnter 触发提交
            let sub_entity = ed.clone();
            let on_enter = opts.on_enter;
            let sub = cx.subscribe(
                &sub_entity,
                move |this, state_entity, event: &InputEvent, cx| match event {
                    InputEvent::Change => {
                        let text = state_entity.read(cx).value().to_string();
                        set_value(this, text);
                        cx.notify();
                    }
                    InputEvent::PressEnter { .. } => {
                        if let Some(f) = on_enter.as_ref() {
                            let text = state_entity.read(cx).value().to_string();
                            f(text, this, cx);
                        }
                    }
                    _ => {}
                },
            );
            STATES.with(|s| s.borrow_mut().insert(id.to_string(), ed.clone()));
            SUBS.with(|s| s.borrow_mut().insert(id.to_string(), sub));
            ed
        });

    // 外部值 → InputState 同步（外部清空/加载时保持一致；输入中二者相等则跳过）
    let external = {
        let entity = cx.entity();
        let sidebar = entity.read(cx);
        get_value(sidebar)
    };
    if state.read(cx).value().to_string() != external {
        cx.update_entity(&state, |s, cx| s.set_value(external.clone(), window, cx));
    }

    Input::new(&state).into_any_element()
}
