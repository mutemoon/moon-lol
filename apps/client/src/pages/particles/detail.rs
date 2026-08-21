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
use super::state::hash_hex;
use crate::components::sidebar::AppSidebar;

pub(super) fn render_system_detail(
    sidebar: &mut AppSidebar,
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
    let active_tab = sidebar.particles.active_tab;

    v_flex()
        .size_full()
        .overflow_hidden()
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
                                .on_click(cx.listener(|this, _, _, cx| reset_system(this, cx))),
                        )
                        .child(
                            Button::new("particle-stop-btn")
                                .icon(IconName::CircleX)
                                .label("停止")
                                .on_click(cx.listener(|this, _, _, cx| stop_playing(this, cx))),
                        )
                        .child(
                            Button::new("particle-play-btn")
                                .icon(IconName::Play)
                                .label("播放")
                                .on_click(cx.listener(|this, _, _, cx| play_working(this, cx))),
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
                                btn.on_click(cx.listener(move |this, _, _, cx| {
                                    this.particles.active_tab = idx;
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
                                    .child(render_emitter_editor(
                                        window, cx, &*sidebar, hash, active_tab, em,
                                    ))
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
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    ws_url: &str,
    connected: bool,
    auto_play: bool,
) -> AnyElement {
    let theme = cx.theme();
    let hero = sidebar.particles.selected_hero.clone();
    let system_name = sidebar
        .particles
        .selected_system
        .as_ref()
        .map(|x| x.name.clone());
    let hash = sidebar.particles.selected_system.as_ref().map(|x| x.hash);

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
                    btn.on_click(cx.listener(|this, _, _, cx| {
                        this.particles.auto_play = !this.particles.auto_play;
                        cx.notify();
                    }))
                })
                .child(particle_ws_status_badge(connected, ws_url, cx)),
        )
        .into_any_element()
}

// 粒子 WebSocket 自动连接状态指示灯
fn particle_ws_status_badge(
    connected: bool,
    ws_url: &str,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let theme = cx.theme();
    let (dot_color, label) = if connected {
        (theme.accent, "已连接")
    } else {
        (theme.muted_foreground, "自动重连中...")
    };
    h_flex()
        .gap_1p5()
        .items_center()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(theme.muted.opacity(0.3))
        .text_xs()
        .child(div().w_2().h_2().rounded_full().bg(dot_color))
        .child(
            div()
                .text_color(if connected {
                    theme.accent
                } else {
                    theme.muted_foreground
                })
                .font_bold()
                .child(label),
        )
        .child(
            div()
                .text_color(theme.muted_foreground)
                .child(format!("({ws_url})")),
        )
        .into_any_element()
}
