//! 房间页输入控件：复用共享文本输入框（render_state_input）。

use gpui::prelude::*;
use gpui::*;

use crate::components::sidebar::AppSidebar;
use crate::components::text_input::{self, EditOptions};

pub(super) fn render_state_input(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    id: &str,
    placeholder: &str,
    get_value: impl Fn() -> String + 'static,
    set_value: impl Fn(String) + 'static,
    on_enter: Option<Box<dyn Fn(&mut Context<AppSidebar>) + 'static>>,
) -> AnyElement {
    text_input::render_edit_input(
        window,
        cx,
        id,
        placeholder,
        EditOptions {
            on_enter: on_enter.map(|f| {
                Box::new(move |_text: String, cx: &mut Context<AppSidebar>| f(cx))
                    as Box<dyn Fn(String, &mut Context<AppSidebar>) + 'static>
            }),
            ..Default::default()
        },
        move |_s| get_value(),
        move |_s, v| set_value(v),
    )
}
