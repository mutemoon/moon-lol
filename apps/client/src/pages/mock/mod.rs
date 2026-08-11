//! 离线 mock 测试沙盒（对应 client `pages/mock/index.vue` + `pages/mock/chat.vue`）。
//!
//! 双态页面：列表态（index.vue 落地页）与会话态（chat.vue 调试床）。
//! 数据源来自 client 侧 `assets/mock.json` 的 AI 决策流示例，移植为本文件
//! 常量消息序列，渲染复用 `render_agent_chat_history`。
//! 全部为本地状态，无任何服务依赖；文案内联中文。

mod input;
mod logic;
mod types;
mod ui;

use gpui::prelude::*;
use gpui::*;
use gpui_component::v_flex;

pub use self::types::MockPageState;
use self::types::MockView;
use self::ui::{render_chat_view, render_list_view, render_page_header};
use crate::components::sidebar::AppSidebar;

// ── 公开入口 ──

/// 离线 mock 测试沙盒（对应 client `pages/mock/*.vue`）。
pub fn render_mock(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let view = sidebar.mock.view;
    v_flex()
        .size_full()
        .flex_1()
        .gap_4()
        .overflow_hidden()
        .child(render_page_header(cx, &view))
        .child(div().flex_1().w_full().overflow_hidden().child(match view {
            MockView::List => render_list_view(cx),
            MockView::Chat => render_chat_view(sidebar, window, cx),
        }))
        .into_any_element()
}
