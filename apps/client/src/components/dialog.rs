//! 基于 gpui_component `Dialog` 的统一弹窗封装。
//!
//! 走命令式 `window.open_dialog`：内容表单由 `Root::render_dialog_layer` 每帧重跑
//! builder 驱动重建，因此下拉标签、错误提示、按钮态等仍实时刷新。Esc / focus trap /
//! 遮罩点击关闭 / 层叠均由 Dialog 原生提供。

use gpui::{AnyElement, App, Context, WeakEntity, Window};
use gpui_component::dialog::Dialog;
use gpui_component::WindowExt as _;

use crate::components::sidebar::AppSidebar;

/// 打开一个内容为「每帧从 AppSidebar 重建」的 Dialog。
///
/// `build_form` 拿到 `&AppSidebar` + `&mut Window` + `&mut Context<AppSidebar>` 返回表单体；
/// `configure` 在拿到表单体后配置 Dialog（width / title / footer / on_ok 等）。
pub fn open_form_dialog<F, C>(
    window: &mut Window,
    cx: &mut App,
    weak: WeakEntity<AppSidebar>,
    build_form: F,
    configure: C,
) where
    F: Fn(&AppSidebar, &mut Window, &mut Context<AppSidebar>) -> AnyElement + 'static,
    C: Fn(Dialog, AnyElement) -> Dialog + 'static,
{
    window.open_dialog(cx, move |dialog, window, cx| {
        let form = weak.update(cx, |this, cx| build_form(this, window, cx));
        match form {
            Ok(form) => configure(dialog, form),
            Err(_) => dialog,
        }
    });
}
