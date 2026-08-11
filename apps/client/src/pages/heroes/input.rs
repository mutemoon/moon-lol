//! 可编辑文本输入框（焦点/光标跨渲染保持，复用共享组件）。

use gpui::prelude::*;
use gpui::*;
use gpui_component::{v_flex, StyledExt};

use crate::components::sidebar::AppSidebar;
use crate::components::text_input::{self, EditOptions};

/// 可聚焦、可键盘编辑的文本输入框。get_value 读 live 值，set_value 写回 sidebar 字段。
pub(super) fn render_edit_input(
    sidebar: &AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    id: &str,
    placeholder: &str,
    multiline: bool,
    get_value: impl Fn(&AppSidebar) -> String + 'static,
    set_value: impl Fn(&mut AppSidebar, String) + 'static,
) -> AnyElement {
    text_input::render_edit_input(
        window,
        cx,
        sidebar,
        id,
        placeholder,
        EditOptions {
            multiline,
            ..Default::default()
        },
        get_value,
        set_value,
    )
}

/// 带标签的编辑区包装。
pub(super) fn edit_field(label: &str, input: AnyElement) -> AnyElement {
    v_flex()
        .gap_1()
        .child(div().text_sm().font_bold().child(label.to_string()))
        .child(input)
        .into_any_element()
}
