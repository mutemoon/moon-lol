//! 右侧系统详情面板（发射器 tab 行 + 编辑器）+ 页头（标题 + WS 连接控制）。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use lol_share::{ConfigVfxEmitterDefinition, ConfigVfxSystemDefinition};

use super::edit::primary_list_ref;
use super::emitter::render_emitter_editor;
use super::input::clear_all_input_buffers;
use super::play::{play_working, reset_system, stop_playing};
use super::process_ws_event;
use super::state::{hash_hex, update_state, with_state};
use crate::components::sidebar::AppSidebar;
use crate::services::particle_service;

pub(super) fn render_system_detail(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    hero: &str,
    name: &str,
    hash: u32,
    wd: &ConfigVfxSystemDefinition,
) -> AnyElement {
    let emitters: Vec<ConfigVfxEmitterDefinition> =
        primary_list_ref(wd).map(|l| l.clone()).unwrap_or_default();
    let emitter_count = emitters.len();
    let active_tab = with_state(|s| s.active_tab);

    v_flex()
        .size_full()
        .child(
            // 面板顶栏
            h_flex()
                .px_4()
                .py_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .font_bold()
                                .child(format!("{} / {}  {} 个发射器", hero, name, emitter_count)),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .bg(cx.theme().accent.opacity(0.1))
                                .text_xs()
                                .child(hash_hex(hash)),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("particle-reset-sys-btn")
                                .ghost()
                                .icon(IconName::Redo)
                                .label("重置系统")
                                .on_click(cx.listener(|_, _, _, cx| reset_system(cx))),
                        )
                        .child(
                            Button::new("particle-stop-btn")
                                .icon(IconName::CircleX)
                                .label("停止")
                                .on_click(cx.listener(|_, _, _, cx| stop_playing(cx))),
                        )
                        .child(
                            Button::new("particle-play-btn")
                                .icon(IconName::Play)
                                .label("播放")
                                .on_click(cx.listener(|_, _, _, cx| play_working(cx))),
                        ),
                ),
        )
        .when(emitter_count == 0, |d| {
            d.child(
                div().flex_1().flex().items_center().justify_center().child(
                    v_flex().gap_2().items_center().child(IconName::File).child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("该粒子系统未包含发射器定义数据"),
                    ),
                ),
            )
        })
        .when(emitter_count > 0, |d| {
            d.child(
                v_flex()
                    .size_full()
                    // 发射器 tab 行
                    .child(
                        h_flex()
                            .px_2()
                            .py_1()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .gap_1()
                            .children(emitters.iter().enumerate().map(|(idx, em)| {
                                let is_active = active_tab == idx;
                                let label = em
                                    .emitter_name
                                    .clone()
                                    .unwrap_or_else(|| format!("发射器 #{}", idx + 1));
                                let btn = Button::new(format!("emitter-tab-{}", idx)).label(label);
                                let btn = if is_active { btn } else { btn.ghost() };
                                btn.on_click(cx.listener(move |_, _, _, cx| {
                                    update_state(|s| {
                                        s.active_tab = idx;
                                    });
                                    clear_all_input_buffers();
                                    cx.notify();
                                }))
                                .into_any_element()
                            })),
                    )
                    // 编辑器主体
                    .child(
                        emitters
                            .get(active_tab)
                            .map(|em| {
                                div()
                                    .flex_1()
                                    .overflow_y_scrollbar()
                                    .p_4()
                                    .child(render_emitter_editor(window, cx, hash, active_tab, em))
                                    .into_any_element()
                            })
                            .unwrap_or_else(|| div().flex_1().into_any_element()),
                    ),
            )
        })
        .into_any_element()
}

// ── 页头：标题 + WS 连接控制 ──

pub(super) fn render_page_header(
    cx: &mut Context<AppSidebar>,
    ws_url: &str,
    connected: bool,
    auto_play: bool,
) -> AnyElement {
    let theme = cx.theme();
    let (hero, system_name, hash) = with_state(|s| {
        (
            s.selected_hero.clone(),
            s.selected_system.as_ref().map(|x| x.name.clone()),
            s.selected_system.as_ref().map(|x| x.hash),
        )
    });

    h_flex()
        .w_full()
        .justify_between()
        .items_center()
        .px_3()
        .py_2()
        .bg(theme.background)
        .border_1()
        .border_color(theme.border)
        .rounded_md()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(IconName::Palette)
                .child(
                    div()
                        .font_bold()
                        .text_base()
                        .child(match (&hero, &system_name) {
                            (Some(h), Some(n)) => format!("{} / {}", h, n),
                            _ => "粒子系统编辑器".to_string(),
                        }),
                )
                .when_some(hash, |this, h| {
                    this.child(
                        div()
                            .px_2()
                            .py_0p5()
                            .bg(theme.accent.opacity(0.2))
                            .rounded_sm()
                            .text_xs()
                            .child(hash_hex(h)),
                    )
                }),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child({
                    let label = format!("自动播放: {}", if auto_play { "ON" } else { "OFF" });
                    let btn = Button::new("particle-auto-play-toggle").label(label);
                    let btn = if auto_play { btn } else { btn.ghost() };
                    btn.on_click(cx.listener(|_this, _, _, cx| {
                        update_state(|s| s.auto_play = !s.auto_play);
                        cx.notify();
                    }))
                })
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(theme.background)
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(format!("Server: {}", ws_url)),
                )
                .when(!connected, |d| {
                    d.child({
                        let url = ws_url.to_string();
                        Button::new("particle-connect-btn")
                            .label("连接服务器")
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                let target_url = url.clone();
                                update_state(|s| {
                                    s.error = None;
                                    s.ws_url = target_url.clone();
                                });
                                cx.notify();

                                let (handle, mut ev_rx) =
                                    particle_service::connect_to_particle_server(&target_url);
                                update_state(|s| {
                                    s.ws_handle = Some(handle);
                                });

                                let weak = cx.entity().downgrade();
                                cx.spawn(
                                    move |_: gpui::WeakEntity<AppSidebar>,
                                          cx: &mut gpui::AsyncApp| {
                                        let mut cx2 = cx.clone();
                                        async move {
                                            while let Some(event) = ev_rx.recv().await {
                                                let notify = process_ws_event(event);
                                                if notify {
                                                    if let Some(e) = weak.upgrade() {
                                                        let _ = e.update(
                                                            &mut cx2,
                                                            |_, cx| cx.notify(),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    },
                                )
                                .detach();
                            }))
                    })
                })
                .when(connected, |d| {
                    d.child(
                        Button::new("particle-disconnect-btn")
                            .icon(IconName::CircleX)
                            .label("断开")
                            .on_click(cx.listener(|_this, _, _, cx| {
                                update_state(|s| {
                                    if let Some(h) = &s.ws_handle {
                                        h.disconnect();
                                    }
                                    s.ws_handle = None;
                                    s.connected = false;
                                });
                                cx.notify();
                            })),
                    )
                })
                .when(connected, |d| {
                    d.child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .bg(theme.accent.opacity(0.15))
                            .text_color(theme.accent)
                            .text_xs()
                            .font_bold()
                            .child("已连接"),
                    )
                }),
        )
        .into_any_element()
}
