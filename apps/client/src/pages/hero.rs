use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};

use crate::components::sidebar::AppSidebar;
use crate::types::ActiveView;

/// 英雄营销落地页（对应 client `pages/hero.vue`）。纯静态展示，无服务依赖。
pub fn render_hero(_sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    v_flex()
        .size_full()
        .flex_1()
        .gap_6()
        .overflow_y_scrollbar()
        .child(hero_banner(cx))
        .child(tech_strip(cx))
        .child(stack_section(cx))
        .child(logs_section(cx))
        .into_any_element()
}

/// Hero 大标题区：徽章 + 大标题 + 副标题 + CTA。
fn hero_banner(cx: &mut Context<AppSidebar>) -> AnyElement {
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let foreground = cx.theme().foreground;

    v_flex()
        .w_full()
        .items_center()
        .gap_4()
        .py_8()
        .child(
            div()
                .px_3()
                .py_1()
                .rounded_full()
                .border_1()
                .border_color(accent.opacity(0.5))
                .bg(accent.opacity(0.08))
                .text_xs()
                .text_color(accent)
                .child("高性能游戏环境 · 实时 AI 对战"),
        )
        .child(
            div()
                .text_size(rems(3.))
                .font_bold()
                .line_height(rems(3.2))
                .text_color(foreground)
                .child("MOONLOL"),
        )
        .child(
            div()
                .text_sm()
                .text_color(muted)
                .child("Rust 引擎驱动的英雄联盟 AI 对抗模拟平台"),
        )
        .child(
            h_flex().gap_3().child(
                Button::new("hero-cta")
                    .primary()
                    .icon(IconName::Play)
                    .label("开始逗蛐蛐 →")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.navigate_to(ActiveView::Launcher);
                        cx.notify();
                    })),
            ),
        )
        .into_any_element()
}

/// 技术栈条带（静态展示）。
fn tech_strip(cx: &mut Context<AppSidebar>) -> AnyElement {
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;

    let chips: Vec<AnyElement> = [
        "RUST", "BEVY", "ECS", "WEBGL", "WGPU", "VUE", "TAILWIND", "GPUI",
    ]
    .iter()
    .map(|name| {
        h_flex()
            .gap_4()
            .items_center()
            .whitespace_nowrap()
            .child(
                div()
                    .text_lg()
                    .font_bold()
                    .text_color(accent)
                    .child(name.to_string()),
            )
            .child(div().text_sm().text_color(muted).child("///"))
            .into_any_element()
    })
    .collect();

    div()
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(border)
        .px_6()
        .py_3()
        .overflow_hidden()
        .child(h_flex().gap_4().children(chips))
        .into_any_element()
}

/// 技术栈三卡片。
fn stack_section(cx: &mut Context<AppSidebar>) -> AnyElement {
    v_flex()
        .w_full()
        .gap_4()
        .child(
            div()
                .text_xl()
                .font_bold()
                .text_color(cx.theme().foreground)
                .child("技术栈"),
        )
        .child(
            h_flex()
                .w_full()
                .gap_4()
                .items_start()
                .child(stack_card(
                    cx,
                    "01.",
                    "Rust 语言",
                    "追求极致的性能与内存安全",
                    &["零成本抽象", "无 GC 高并发", "稳定可靠的内存管理"],
                ))
                .child(stack_card(
                    cx,
                    "02.",
                    "Bevy 引擎",
                    "Rust 编写的模块化 ECS 游戏引擎",
                    &["Bevy 0.17", "并行 ECS 架构", "WGPU 跨平台渲染"],
                ))
                .child(stack_card(
                    cx,
                    "03.",
                    "可视化前端",
                    "AI 对战过程的实时监控面板",
                    &["Vue 3 + TypeScript", "实时指标图表", "WebSocket 通信"],
                )),
        )
        .into_any_element()
}

/// 单张技术栈卡片。
fn stack_card(
    cx: &mut Context<AppSidebar>,
    index: &str,
    title: &str,
    desc: &str,
    items: &[&str],
) -> AnyElement {
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;

    v_flex()
        .flex_1()
        .rounded_lg()
        .border_1()
        .border_color(border)
        .p_5()
        .gap_3()
        .child(
            h_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(accent)
                        .font_bold()
                        .child(index.to_string()),
                )
                .child(div().text_lg().font_bold().child(title.to_string())),
        )
        .child(div().text_sm().text_color(muted).child(desc.to_string()))
        .child(v_flex().gap_1p5().children(items.iter().map(|item| {
            h_flex()
                .gap_1p5()
                .items_center()
                .text_xs()
                .text_color(muted)
                .child(div().w_2().h_2().rounded_full().bg(accent))
                .child(item.to_string())
        })))
        .into_any_element()
}

/// 开发日志列表。
fn logs_section(cx: &mut Context<AppSidebar>) -> AnyElement {
    let accent = cx.theme().accent;

    v_flex()
        .w_full()
        .gap_4()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xl()
                        .font_bold()
                        .text_color(cx.theme().foreground)
                        .child("开发日志"),
                )
                .child(div().text_xs().text_color(accent).child("/// LOGS")),
        )
        .child(
            v_flex()
                .w_full()
                .gap_3()
                .child(log_card(
                    cx,
                    "ARCHITECTURE",
                    "2025.11.28",
                    "工程架构",
                    "Moon LoL 的高层系统设计：Rust Core 与 Web Frontend.",
                ))
                .child(log_card(
                    cx,
                    "DATA",
                    "2025.11.28",
                    "数据流转",
                    "从 Bevy ECS 到 Web 前端的数据管线。",
                ))
                .child(log_card(
                    cx,
                    "CORE",
                    "2025.11.28",
                    "ECS 组件与系统",
                    "深入解析游戏核心逻辑：插件系统与实体组件设计。",
                )),
        )
        .into_any_element()
}

/// 单条开发日志卡片，点击跳转博客视图。
fn log_card(
    cx: &mut Context<AppSidebar>,
    tag: &str,
    date: &str,
    title: &str,
    desc: &str,
) -> AnyElement {
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;

    v_flex()
        .w_full()
        .cursor_pointer()
        .rounded_lg()
        .border_1()
        .border_color(border)
        .p_5()
        .gap_3()
        .hover(|s| s.bg(accent.opacity(0.05)).border_color(accent.opacity(0.5)))
        .on_any_mouse_down(cx.listener(move |this, _, _, cx| {
            this.navigate_to(ActiveView::Blog);
            cx.notify();
        }))
        .child(
            h_flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_full()
                        .border_1()
                        .border_color(accent.opacity(0.5))
                        .bg(accent.opacity(0.1))
                        .text_xs()
                        .font_bold()
                        .text_color(accent)
                        .child(tag.to_string()),
                )
                .child(div().text_xs().text_color(muted).child(date.to_string())),
        )
        .child(div().text_lg().font_bold().child(title.to_string()))
        .child(div().text_sm().text_color(muted).child(desc.to_string()))
        .into_any_element()
}
