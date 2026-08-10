//! 可编辑文本输入框（跨渲染保持焦点/光标，复用共享组件）。

use gpui::prelude::*;
use gpui::*;
use gpui_component::{v_flex, StyledExt};

use crate::components::sidebar::AppSidebar;
use crate::components::text_input::{self, EditOptions};

pub(super) fn render_edit_field(
    id: &str,
    label: impl Into<SharedString>,
    placeholder: &str,
    _sidebar: &AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    get_value: impl Fn(&AppSidebar) -> String + 'static,
    set_value: impl Fn(&mut AppSidebar, String) + 'static,
) -> AnyElement {
    v_flex()
        .gap_1()
        .child(div().text_xs().font_bold().child(label.into()))
        .child(text_input::render_edit_input(
            window,
            cx,
            id,
            placeholder,
            EditOptions::default(),
            get_value,
            set_value,
        ))
        .into_any_element()
}
