use gpui::*;
use gpui_component::chart::LineChart;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, StyledExt};
use lol_rl_protocol::MetricsRow;
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::types::LocalTaskDetail;

/// Tab 1: 图表与训练指标面板
pub fn render_tab_metrics(detail: &LocalTaskDetail, cx: &mut Context<AppSidebar>) -> AnyElement {
    let mut container = v_flex()
        .size_full()
        .flex_1()
        .gap_4()
        .overflow_hidden()
        .child(
            div()
                .font_bold()
                .text_base()
                .child(t!("app.rl.metrics_title")),
        );

    if let Some(obs) = &detail.latest_obs {
        container = container.child(
            h_flex()
                .justify_between()
                .items_center()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().accent.opacity(0.4))
                .bg(cx.theme().accent.opacity(0.05))
                .child(
                    h_flex()
                        .gap_4()
                        .items_center()
                        .child(div().font_bold().text_xs().child("最新步进 Observe 采样"))
                        .child(div().text_xs().child(format!(
                            "血量: {:.0}% vs {:.0}%",
                            obs.fiora_hp_pct * 100.0,
                            obs.riven_hp_pct * 100.0
                        )))
                        .child(div().text_xs().child(format!("距离: {:.1}u", obs.distance)))
                        .child(div().text_xs().child(format!(
                            "破绽: {} ({})",
                            if obs.has_vital { "在场" } else { "无" },
                            obs.vital_direction
                        ))),
                )
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(cx.theme().accent)
                        .child("Observe 采样已同步"),
                ),
        );
    }

    container
        .child(if detail.metrics_history.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(t!("app.rl.metrics_empty"))
                .into_any_element()
        } else {
            div()
                .id("metrics-charts-scroll")
                .flex_1()
                .overflow_y_scrollbar()
                .child(v_flex().gap_4().p_1().children([
                    metric_chart(
                        t!("app.rl.m_episode_return"),
                        "chart-ep-return",
                        &detail.metrics_history,
                        |m| m.ep_return as f64,
                        cx.theme().chart_1,
                        cx,
                    ),
                    metric_chart(
                        t!("app.rl.m_loss"),
                        "chart-loss",
                        &detail.metrics_history,
                        |m| m.loss as f64,
                        cx.theme().chart_2,
                        cx,
                    ),
                    metric_chart(
                        t!("app.rl.m_kl"),
                        "chart-kl",
                        &detail.metrics_history,
                        |m| m.kl as f64,
                        cx.theme().chart_3,
                        cx,
                    ),
                    metric_chart(
                        t!("app.rl.m_entropy"),
                        "chart-entropy",
                        &detail.metrics_history,
                        |m| m.entropy as f64,
                        cx.theme().chart_4,
                        cx,
                    ),
                    metric_chart(
                        t!("app.rl.m_value"),
                        "chart-value",
                        &detail.metrics_history,
                        |m| m.value as f64,
                        cx.theme().chart_5,
                        cx,
                    ),
                    metric_chart(
                        t!("app.rl.m_fps"),
                        "chart-fps",
                        &detail.metrics_history,
                        |m| m.fps as f64,
                        cx.theme().chart_1,
                        cx,
                    ),
                ]))
                .into_any_element()
        })
        .into_any_element()
}

/// 单个训练指标折线图卡片（x=step，y=指标值）。
fn metric_chart(
    title: impl IntoElement,
    chart_id: &'static str,
    rows: &[MetricsRow],
    y_fn: impl Fn(&MetricsRow) -> f64 + 'static,
    stroke: Hsla,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    v_flex()
        .gap_2()
        .child(div().font_semibold().text_sm().child(title))
        .child(
            div()
                .h(px(170.))
                .w_full()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .p_2()
                .child(
                    LineChart::new(rows.to_vec())
                        .x(|d| d.step.to_string())
                        .y(y_fn)
                        .stroke(stroke)
                        .dot()
                        .tick_margin((rows.len() / 8).max(1))
                        .id(chart_id),
                ),
        )
        .into_any_element()
}
