//! 手写输入框（复用共享组件，跨渲染保持焦点/光标）。

use gpui::prelude::*;
use gpui::*;

use crate::components::sidebar::AppSidebar;
use crate::components::text_input::{self, EditOptions};

/// 可聚焦、可键盘编辑的文本输入框，读写 `sidebar.logs_archive`。
pub(super) fn render_text_input(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    id: &str,
    placeholder: &str,
    get_value: impl Fn(&AppSidebar) -> String + 'static,
    set_value: impl Fn(&mut AppSidebar, String) + 'static,
) -> AnyElement {
    text_input::render_edit_input(
        window,
        cx,
        id,
        placeholder,
        EditOptions::default(),
        get_value,
        set_value,
    )
}
