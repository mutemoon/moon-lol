//! 粒子系统编辑器：Rayon 树状文件侧边栏 → 选中系统 → 发射器参数编辑 → 自动重播。

mod detail;
mod edit;
mod emitter;
mod input;
mod play;
mod state;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable, StyledExt};
pub use state::ParticlesPageState;

use self::detail::{render_page_header, render_system_detail};
use self::input::{clear_all_input_buffers, clear_input_buffer, render_search_input};
use self::play::spawn_play_ron;
use self::state::{
    collect_flat_visible_nodes, scan_particles_rayon, start_async_rescan, toggle_node_expansion,
};
use crate::components::sidebar::AppSidebar;

/// 粒子系统编辑器：Rayon 树状文件侧边栏 → 选中系统 → 发射器参数编辑 → 自动重播。
pub fn render_particles(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let should_start_scan = {
        if !sidebar.particles.is_initialized && !sidebar.particles.is_scanning {
            sidebar.particles.is_initialized = true;
            sidebar.particles.is_scanning = true;
            true
        } else {
            false
        }
    };

    // 首次进入时，在后台 Rayon 多线程异步构建英雄粒子树
    if should_start_scan {
        let weak_entity = cx.entity().downgrade();
        cx.spawn(|_this, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let res = crate::services::runtime::tokio_runtime()
                    .spawn_blocking(scan_particles_rayon)
                    .await;
                let (roots, systems_map) = res.unwrap_or_default();

                let _ = weak_entity.update(&mut cx, |this, cx| {
                    this.particles.tree_roots = roots;
                    this.particles.hero_systems = systems_map;
                    this.particles.is_scanning = false;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    let query_lower = sidebar.particles.search_query.trim().to_lowercase();
    let mut flat_visible_nodes = Vec::new();
    collect_flat_visible_nodes(
        &sidebar.particles.tree_roots,
        0,
        &query_lower,
        &mut flat_visible_nodes,
    );
    let total_heroes = sidebar.particles.tree_roots.len();
    let total_particles: usize = sidebar
        .particles
        .tree_roots
        .iter()
        .map(|n| n.children.len())
        .sum();

    let connected = sidebar.particles.connected;
    let error = sidebar.particles.error.clone();
    let ws_url = sidebar.particles.ws_url.clone();
    let query = sidebar.particles.search_query.clone();
    let auto_play = sidebar.particles.auto_play;
    let is_scanning = sidebar.particles.is_scanning;
    let selected_system = sidebar.particles.selected_system.clone();
    let hero = sidebar.particles.selected_hero.clone();
    let name = sidebar
        .particles
        .selected_system
        .as_ref()
        .map(|x| x.name.clone());
    let hash = sidebar.particles.selected_system.as_ref().map(|x| x.hash);
    let wd = sidebar.particles.working_def.clone();

    let theme = cx.theme();
    let sidebar_bg = theme.sidebar;
    let border_color = theme.border;
    let bg_color = theme.background;
    let accent_color = theme.accent;
    let accent_fg = theme.accent_foreground;
    let muted_fg = theme.muted_foreground;
    let danger_color = theme.danger;

    // 右侧详情面板
    let right_panel = match (hero, name, hash, wd) {
        (Some(h), Some(n), Some(hh), Some(w)) => {
            render_system_detail(sidebar, window, cx, &h, &n, hh, &w)
        }
        _ => h_flex()
            .flex_1()
            .overflow_hidden()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Palette)
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted_fg)
                            .child("请在左侧展开英雄并选择粒子系统"),
                    ),
            )
            .into_any_element(),
    };

    let search_input_elem = render_search_input(window, cx, &*sidebar);
    let page_header_elem = render_page_header(sidebar, cx, &ws_url, connected, auto_play);

    h_flex()
        .flex_1()
        .overflow_hidden()
        .w_full()
        .py_2()
        .gap_3()
        .child(
            // ── 左侧英雄粒子树侧边栏（完全对齐 wad_browser 结构与视觉风格） ──
            v_flex()
                .w_80()
                .h_full()
                .p_3()
                .gap_2()
                .bg(sidebar_bg)
                .border_1()
                .border_color(border_color)
                .rounded_md()
                .child(
                    h_flex()
                        .w_full()
                        .justify_between()
                        .items_center()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(div().font_bold().text_base().child("英雄粒子树"))
                                .when(is_scanning, |this| {
                                    this.child(div().text_xs().opacity(0.6).child("(扫描中...)"))
                                }),
                        )
                        .child(
                            Button::new("refresh-particles-tree")
                                .icon(IconName::Redo)
                                .ghost()
                                .small()
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    start_async_rescan(this, cx);
                                })),
                        ),
                )
                // 搜索过滤框
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .w_full()
                                .px_2()
                                .py_1()
                                .bg(bg_color)
                                .border_1()
                                .border_color(border_color)
                                .rounded_md()
                                .items_center()
                                .gap_2()
                                .child(IconName::Search)
                                .child(div().flex_1().child(search_input_elem))
                                .when(!query.is_empty(), |this| {
                                    this.child(
                                        Button::new("particle-search-clear")
                                            .ghost()
                                            .small()
                                            .icon(IconName::Close)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.particles.search_query.clear();
                                                clear_input_buffer("particle-search");
                                                cx.notify();
                                            })),
                                    )
                                }),
                        )
                        .child(div().px_1().text_xs().text_color(muted_fg).child(
                            if query.is_empty() {
                                format!("共 {} 个英雄 · {} 个粒子", total_heroes, total_particles)
                            } else {
                                let matched_count =
                                    flat_visible_nodes.iter().filter(|n| !n.is_dir).count();
                                format!("匹配 {} 个粒子", matched_count)
                            },
                        )),
                )
                // 树状列表区域（严格只挂载当前展开显示的节点 DOM）
                .child(
                    v_flex()
                        .flex_1()
                        .w_full()
                        .overflow_y_scrollbar()
                        .gap_0p5()
                        .when(is_scanning, |this| {
                            this.child(
                                div()
                                    .w_full()
                                    .py_4()
                                    .flex()
                                    .justify_center()
                                    .text_sm()
                                    .opacity(0.6)
                                    .child("Rayon 多线程扫描英雄粒子中..."),
                            )
                        })
                        .when(!is_scanning && flat_visible_nodes.is_empty(), |this| {
                            this.child(
                                div()
                                    .w_full()
                                    .py_4()
                                    .flex()
                                    .justify_center()
                                    .text_sm()
                                    .opacity(0.6)
                                    .child("无匹配粒子节点"),
                            )
                        })
                        .children(flat_visible_nodes.into_iter().map(|node| {
                            let is_dir = node.is_dir;
                            let is_selected = !is_dir
                                && selected_system
                                    .as_ref()
                                    .map(|s| Some(s.hash) == node.hash)
                                    .unwrap_or(false);
                            let padding_left_px = (node.depth * 14 + 6) as f32;
                            let hero_name = node.hero_name.clone();
                            let node_hash = node.hash;

                            div()
                                .w_full()
                                .py_1()
                                .rounded_md()
                                .cursor_pointer()
                                .pl(px(padding_left_px))
                                .when(is_selected, |this| {
                                    this.bg(accent_color).text_color(accent_fg)
                                })
                                .when(!is_selected, |this| {
                                    this.hover(|style| style.bg(accent_color.opacity(0.1)))
                                })
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _window, cx| {
                                        if is_dir {
                                            let h = hero_name.clone();
                                            toggle_node_expansion(
                                                &mut this.particles.tree_roots,
                                                &h,
                                            );
                                            cx.notify();
                                        } else if let Some(target_hash) = node_hash {
                                            let h = hero_name.clone();
                                            let mut found_system = None;
                                            if let Some(systems) =
                                                this.particles.hero_systems.get(&h)
                                            {
                                                if let Some(sys) =
                                                    systems.iter().find(|s| s.hash == target_hash)
                                                {
                                                    this.particles.selected_hero = Some(h.clone());
                                                    this.particles.selected_system =
                                                        Some(sys.clone());
                                                    this.particles.active_tab = 0;
                                                    this.particles.working_def =
                                                        Some(sys.def.clone());
                                                    this.particles.initial_def_backup =
                                                        Some(sys.def.clone());
                                                    found_system = Some(sys.clone());
                                                }
                                            }
                                            clear_all_input_buffers();
                                            cx.notify();

                                            if let Some(sys) = found_system {
                                                let auto = this.particles.auto_play;
                                                if auto {
                                                    let ron = sys.def_ron;
                                                    spawn_play_ron(this, cx, ron);
                                                }
                                            }
                                        }
                                    }),
                                )
                                .child(
                                    h_flex()
                                        .gap_1p5()
                                        .items_center()
                                        .child(if node.is_dir {
                                            if node.is_expanded {
                                                IconName::FolderOpen
                                            } else {
                                                IconName::Folder
                                            }
                                        } else {
                                            IconName::Play
                                        })
                                        .child(
                                            div()
                                                .font_medium()
                                                .text_sm()
                                                .truncate()
                                                .child(node.name.clone()),
                                        )
                                        .when(node.is_dir, |this| {
                                            this.child(
                                                div()
                                                    .text_xs()
                                                    .opacity(0.5)
                                                    .child(format!("({})", node.children_count)),
                                            )
                                        }),
                                )
                        })),
                ),
        )
        .child(
            // ── 右侧工作区 ──
            v_flex()
                .flex_1()
                .overflow_hidden()
                .h_full()
                .gap_2()
                // 页头状态栏
                .child(page_header_elem)
                // 错误提示
                .when_some(error.as_ref(), |d, err| {
                    d.child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(danger_color.opacity(0.1))
                            .text_color(danger_color)
                            .text_xs()
                            .child(err.clone()),
                    )
                })
                // 主工作区面板
                .child(
                    v_flex()
                        .flex_1()
                        .overflow_hidden()
                        .w_full()
                        .bg(bg_color)
                        .border_1()
                        .border_color(border_color)
                        .rounded_md()
                        .child(right_panel),
                ),
        )
        .into_any_element()
}
