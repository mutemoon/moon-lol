//! 启动器页面：模式 + 场景 + 双阵营槽位编排 + 启动。

mod logic;
mod render;
mod types;

use gpui::prelude::*;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::v_flex;
pub use types::LauncherPageState;

use self::logic::spawn_initial_load;
use self::render::{
    render_action_buttons, render_header, render_load_dropdown, render_message_banners,
    render_mode_and_champion, render_scene_section, render_teams_section,
};
use crate::components::sidebar::AppSidebar;

// ── 页面入口 ──

/// 启动器页面：模式 + 场景 + 双阵营槽位编排 + 启动。
pub fn render_launcher(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let champ = sidebar.champion.clone();
    let mode = sidebar.game_mode.clone();
    let launch_error = sidebar.launch_error.clone();
    let champions = sidebar.champions_list.clone();

    if !sidebar.launcher.loaded {
        spawn_initial_load(cx);
    }

    let load_dropdown = render_load_dropdown(sidebar, cx);

    let mut container = v_flex().size_full().flex_1().gap_6().overflow_y_scrollbar();

    container = container.child(render_header());
    container = container.child(render_mode_and_champion(&mode, &champ, &champions, cx));
    container = container.child(render_scene_section(sidebar, load_dropdown, cx));

    for banner in render_message_banners(sidebar, launch_error, cx) {
        container = container.child(banner);
    }

    container = container.child(render_teams_section(sidebar, cx));
    container = container.child(render_action_buttons(sidebar, cx));

    container.into_any_element()
}
