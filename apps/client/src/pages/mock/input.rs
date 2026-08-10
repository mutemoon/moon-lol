//! 输入框（复用共享组件）：可聚焦、可键盘编辑，Enter 触发提交。

use gpui::prelude::*;
use gpui::*;

use crate::components::sidebar::AppSidebar;
use crate::components::text_input::{self, EditOptions};

/// 可聚焦、可键盘编辑的文本输入框。get_value / set_value 读写页面 STATE，
/// Enter 触发 submit（注入消息）。
pub(super) fn render_edit_input(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    id: &'static str,
    placeholder: &'static str,
    get_value: impl Fn() -> String + 'static,
    set_value: impl Fn(String) + 'static,
    submit: Option<Box<dyn Fn(&mut Context<AppSidebar>) + 'static>>,
) -> AnyElement {
    text_input::render_edit_input(
        window,
        cx,
        id,
        placeholder,
        EditOptions {
            on_enter: submit.map(|f| {
                Box::new(move |_text: String, cx: &mut Context<AppSidebar>| f(cx))
                    as Box<dyn Fn(String, &mut Context<AppSidebar>) + 'static>
            }),
            ..Default::default()
        },
        move |_s| get_value(),
        move |_s, v| set_value(v),
    )
}
