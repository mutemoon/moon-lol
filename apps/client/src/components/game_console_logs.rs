use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, ActiveTheme, Sizable, StyledExt};

use crate::components::sidebar::AppSidebar;

// ── 数据模型 ──

/// 一条控制台日志行，供 debug / observe 页面复用。
#[derive(Debug, Clone)]
pub struct ConsoleLogRow {
    /// 日志级别："INFO" / "WARN" / "ERROR" / "DEBUG"。
    pub level: String,
    /// 来源模块（可为空字符串）。
    pub category: String,
    /// 关联实体（可为空字符串）。
    pub entity: String,
    /// 日志正文。
    pub message: String,
    /// 时间戳（可选，缺失时显示占位符）。
    pub timestamp: Option<String>,
}

// ── 本地过滤状态 ──

/// 顶部过滤按钮的选择状态（存于 AppSidebar.game_console_logs）。多个页面复用一个
/// 组件时共享此过滤选择，简化起见不做按页面隔离。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LevelFilter {
    All,
    Info,
    Warn,
    Error,
}

impl LevelFilter {
    fn matches(self, level: &str) -> bool {
        match self {
            LevelFilter::All => true,
            LevelFilter::Info => level == "INFO",
            LevelFilter::Warn => level == "WARN",
            LevelFilter::Error => level == "ERROR",
        }
    }
}

/// 控制台日志过滤状态（存于 AppSidebar.game_console_logs）。
pub struct GameConsoleLogsState {
    filter: LevelFilter,
}

impl Default for GameConsoleLogsState {
    fn default() -> Self {
        Self {
            filter: LevelFilter::All,
        }
    }
}

// ── 渲染辅助 ──

/// 按日志级别返回配色。
fn level_color(level: &str, cx: &Context<AppSidebar>) -> Hsla {
    match level {
        "INFO" => cx.theme().accent,
        "WARN" => cx.theme().warning,
        "ERROR" => cx.theme().danger,
        "DEBUG" => cx.theme().muted_foreground,
        _ => cx.theme().muted_foreground,
    }
}

/// 小徽标（category / entity）。
fn meta_badge(text: &str, color: Hsla) -> Div {
    div()
        .px_1p5()
        .py_0p5()
        .rounded_md()
        .text_xs()
        .font_bold()
        .flex_none()
        .bg(color.opacity(0.12))
        .text_color(color)
        .child(text.to_string())
}

/// 顶部过滤按钮：选中时用默认实心样式，未选中用 ghost 样式。
fn filter_btn(
    id: &'static str,
    label: &'static str,
    target: LevelFilter,
    current: LevelFilter,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let selected = current == target;
    let click = cx.listener(move |this, _event, _window, cx| {
        // 点击当前级别时回到「全部」，否则切换到该级别。
        this.game_console_logs.filter = if this.game_console_logs.filter == target {
            LevelFilter::All
        } else {
            target
        };
        cx.notify();
    });
    let button = Button::new(id).small().label(label);
    if selected {
        button.on_click(click).into_any_element()
    } else {
        button.ghost().on_click(click).into_any_element()
    }
}

/// 渲染一行日志：级别徽标 + 时间戳 + category/entity 徽标 + 消息文本。
fn render_log_row(cx: &Context<AppSidebar>, row: &ConsoleLogRow) -> AnyElement {
    let color = level_color(&row.level, cx);
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let foreground = cx.theme().foreground;
    let border = cx.theme().border;
    let timestamp = row
        .timestamp
        .clone()
        .unwrap_or_else(|| "--:--:--".to_string());

    let mut meta_children: Vec<AnyElement> = Vec::new();
    if !row.category.is_empty() {
        meta_children.push(meta_badge(&row.category, muted).into_any_element());
    }
    if !row.entity.is_empty() {
        meta_children.push(meta_badge(&row.entity, accent).into_any_element());
    }

    h_flex()
        .px_4()
        .py_1p5()
        .gap_2()
        .items_start()
        .border_b_1()
        .border_color(border.opacity(0.3))
        .child(
            div()
                .px_1p5()
                .py_0p5()
                .rounded_md()
                .text_xs()
                .font_bold()
                .flex_none()
                .bg(color.opacity(0.15))
                .text_color(color)
                .child(row.level.clone()),
        )
        .child(
            div()
                .w(rems(8.))
                .flex_none()
                .text_xs()
                .text_color(muted)
                .child(timestamp),
        )
        .child(
            h_flex()
                .flex_none()
                .gap_1()
                .items_center()
                .children(meta_children),
        )
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(foreground.opacity(0.85))
                .child(row.message.clone()),
        )
        .into_any_element()
}

// ── 公开入口 ──

/// 实时游戏控制台日志查看器（debug / observe 页面复用）。
///
/// 顶部一行过滤按钮（全部 / INFO / WARN / ERROR 本地切换）+ 可滚动日志列表，
/// 每行展示级别徽标、时间戳、category/entity 与消息文本；无数据时显示「暂无日志」。
/// 简化实现：不做分组、分页、复制与分析面板。
pub fn render_game_console_logs(
    rows: &[ConsoleLogRow],
    sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let filter = sidebar.game_console_logs.filter;
    let filtered: Vec<&ConsoleLogRow> = rows
        .iter()
        .filter(|row| filter.matches(row.level.as_str()))
        .collect();
    let row_children: Vec<AnyElement> =
        filtered.iter().map(|row| render_log_row(cx, row)).collect();
    let muted = cx.theme().muted_foreground;
    let accent = cx.theme().accent;
    let border = cx.theme().border;
    let background = cx.theme().background;

    div()
        .size_full()
        .flex_1()
        .flex_col()
        .overflow_hidden()
        .rounded_lg()
        .border_1()
        .border_color(border)
        .child(
            h_flex()
                .px_4()
                .py_2()
                .gap_2()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(border)
                .bg(background)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(filter_btn(
                            "gcl-filter-all",
                            "全部",
                            LevelFilter::All,
                            filter,
                            cx,
                        ))
                        .child(filter_btn(
                            "gcl-filter-info",
                            "INFO",
                            LevelFilter::Info,
                            filter,
                            cx,
                        ))
                        .child(filter_btn(
                            "gcl-filter-warn",
                            "WARN",
                            LevelFilter::Warn,
                            filter,
                            cx,
                        ))
                        .child(filter_btn(
                            "gcl-filter-error",
                            "ERROR",
                            LevelFilter::Error,
                            filter,
                            cx,
                        )),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .text_xs()
                        .font_bold()
                        .bg(accent.opacity(0.15))
                        .text_color(accent)
                        .child(format!("{} 条", filtered.len())),
                ),
        )
        .when(filtered.is_empty(), |d| {
            d.child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .text_color(muted)
                    .child("暂无日志"),
            )
        })
        .when(!filtered.is_empty(), |d| {
            d.child(div().flex_1().overflow_y_scrollbar().children(row_children))
        })
        .into_any_element()
}
