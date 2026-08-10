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

use self::detail::{render_page_header, render_system_detail};
use self::input::{
    clear_all_input_buffers, clear_input_buffer, render_search_input,
};
use self::play::spawn_play_ron;
use self::state::{
    collect_flat_visible_nodes, hash_hex, scan_particles_rayon, start_async_rescan,
    toggle_node_expansion, update_state, update_state_returns, with_state,
};
use crate::components::sidebar::AppSidebar;
use crate::services::particle_service::ParticleWsEvent;

/// 粒子系统编辑器：Rayon 树状文件侧边栏 → 选中系统 → 发射器参数编辑 → 自动重播。
pub fn render_particles(
    _sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let should_start_scan = update_state_returns(|s| {
        if !s.is_initialized && !s.is_scanning {
            s.is_initialized = true;
            s.is_scanning = true;
            true
        } else {
            false
        }
    });

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

                let _ = weak_entity.update(&mut cx, |_, cx| {
                    update_state(|state| {
                        state.tree_roots = roots;
                        state.hero_systems = systems_map;
                        state.is_scanning = false;
                    });
                    cx.notify();
                });
            }
        })
        .detach();
    }

    let (
        flat_visible_nodes,
        total_heroes,
        total_particles,
        connected,
        error,
        ws_url,
        query,
        auto_play,
        is_scanning,
        selected_system,
        hero,
        name,
        hash,
        wd,
    ) = with_state(|s| {
        let mut flat = Vec::new();
        let query_lower = s.search_query.trim().to_lowercase();
        collect_flat_visible_nodes(&s.tree_roots, 0, &query_lower, &mut flat);
        let total_heroes = s.tree_roots.len();
        let total_particles: usize = s.tree_roots.iter().map(|n| n.children.len()).sum();

        (
            flat,
            total_heroes,
            total_particles,
            s.connected,
            s.error.clone(),
            s.ws_url.clone(),
            s.search_query.clone(),
            s.auto_play,
            s.is_scanning,
            s.selected_system.clone(),
            s.selected_hero.clone(),
            s.selected_system.as_ref().map(|x| x.name.clone()),
            s.selected_system.as_ref().map(|x| x.hash),
            s.working_def.clone(),
        )
    });

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
        (Some(h), Some(n), Some(hh), Some(w)) => render_system_detail(window, cx, &h, &n, hh, &w),
        _ => div()
            .flex_1()
            .flex()
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

    let search_input_elem = render_search_input(window, cx);
    let page_header_elem = render_page_header(cx, &ws_url, connected, auto_play);

    h_flex()
        .w_full()
        .h_full()
        .gap_3()
        .child(
            // ── 左侧英雄粒子树侧边栏（完全对齐 wad_browser 结构与视觉风格） ──
            v_flex()
                .w_80()
                .h_full()
                .p_3()
                .gap_2()
                .bg(sidebar_bg)
                .border_r_1()
                .border_color(border_color)
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
                                .on_click(cx.listener(|_, _, _window, cx| {
                                    start_async_rescan(cx);
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
                                            .on_click(cx.listener(|_, _, _, cx| {
                                                update_state(|s| {
                                                    s.search_query.clear();
                                                });
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
                                    cx.listener(move |_, _, _window, cx| {
                                        if is_dir {
                                            let h = hero_name.clone();
                                            update_state(|s| {
                                                toggle_node_expansion(&mut s.tree_roots, &h);
                                            });
                                            cx.notify();
                                        } else if let Some(target_hash) = node_hash {
                                            let h = hero_name.clone();
                                            let mut found_system = None;
                                            update_state(|s| {
                                                if let Some(systems) = s.hero_systems.get(&h) {
                                                    if let Some(sys) = systems
                                                        .iter()
                                                        .find(|s| s.hash == target_hash)
                                                    {
                                                        s.selected_hero = Some(h.clone());
                                                        s.selected_system = Some(sys.clone());
                                                        s.active_tab = 0;
                                                        s.working_def = Some(sys.def.clone());
                                                        s.initial_def_backup =
                                                            Some(sys.def.clone());
                                                        found_system = Some(sys.clone());
                                                    }
                                                }
                                            });
                                            clear_all_input_buffers();
                                            cx.notify();

                                            if let Some(sys) = found_system {
                                                let auto = with_state(|s| s.auto_play);
                                                if auto {
                                                    let ron = sys.def_ron;
                                                    spawn_play_ron(cx, ron);
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
                                        })
                                        .when_some(node.hash, |this, h| {
                                            this.child(
                                                div()
                                                    .ml_auto()
                                                    .mr_2()
                                                    .px_1()
                                                    .py_0p5()
                                                    .rounded_sm()
                                                    .bg(if is_selected {
                                                        accent_fg.opacity(0.2)
                                                    } else {
                                                        accent_color.opacity(0.15)
                                                    })
                                                    .text_xs()
                                                    .child(hash_hex(h)),
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
                .h_full()
                .p_3()
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
                        .w_full()
                        .bg(bg_color)
                        .border_1()
                        .border_color(border_color)
                        .rounded_md()
                        .overflow_hidden()
                        .child(right_panel),
                ),
        )
        .into_any_element()
}

// ── WS 事件处理 ──

pub(super) fn process_ws_event(event: ParticleWsEvent) -> bool {
    match event {
        ParticleWsEvent::Connected => {
            update_state(|s| s.connected = true);
            true
        }
        ParticleWsEvent::Disconnected { error } => {
            update_state(|s| {
                s.connected = false;
                s.ws_handle = None;
                if let Some(e) = error {
                    s.error = Some(e);
                }
            });
            true
        }
    }
}
