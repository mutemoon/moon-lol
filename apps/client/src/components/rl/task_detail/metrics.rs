use std::rc::Rc;

use gpui::*;
use gpui_component::plot::scale::{Scale, ScaleLinear, ScalePoint};
use gpui_component::plot::shape::Line;
use gpui_component::plot::tooltip::{CrossLine, Dot, Tooltip, TooltipState};
use gpui_component::plot::{AxisText, Grid, IntoPlot, Plot, PlotAxis, StrokeStyle, AXIS_GAP};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, StyledExt};
use lol_rl_protocol::MetricsRow;
use rust_i18n::t;

use crate::components::rl::task_detail::math::render_math;
use crate::components::sidebar::AppSidebar;
use crate::types::LocalTaskDetail;

/// 渲染前单图表最大点数，超出按步长均匀抽稀，避免上千点后重绘卡顿
const MAX_POINTS: usize = 1200;

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

    if let Some(formula) = &detail.reward_formula {
        container = container.child(
            v_flex()
                .gap_2()
                .p_3()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .child(div().font_semibold().text_sm().child("奖励公式"))
                .child(render_math(&formula.to_latex(), cx)),
        );
    }

    let rows = sample_rows(&detail.metrics_history);
    let clip_eps = detail.latest_clip_eps;

    container
        .child(if rows.is_empty() {
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
                    // 训练目标
                    metric_chart(
                        t!("app.rl.m_episode_return"),
                        "chart-ep-return",
                        &rows,
                        |m| m.ep_return as f64,
                        cx.theme().chart_1,
                        None,
                        cx,
                    ),
                    metric_chart(
                        t!("app.rl.m_value"),
                        "chart-value",
                        &rows,
                        |m| m.value as f64,
                        cx.theme().chart_5,
                        None,
                        cx,
                    ),
                    ep_steps_chart(&rows, cx),
                    reward_breakdown_chart(&rows, cx),
                    // PPO 优化指标
                    loss_breakdown_chart(&rows, cx),
                    metric_chart(
                        t!("app.rl.m_kl"),
                        "chart-kl",
                        &rows,
                        |m| m.kl as f64,
                        cx.theme().chart_3,
                        Some(clip_eps as f64),
                        cx,
                    ),
                    metric_chart(
                        t!("app.rl.m_entropy"),
                        "chart-entropy",
                        &rows,
                        |m| m.entropy as f64,
                        cx.theme().chart_4,
                        None,
                        cx,
                    ),
                    metric_chart(
                        "clip 比例",
                        "chart-clip-frac",
                        &rows,
                        |m| m.clip_frac as f64,
                        cx.theme().chart_2,
                        None,
                        cx,
                    ),
                    // 性能
                    perf_section(&rows, cx),
                ]))
                .into_any_element()
        })
        .into_any_element()
}

/// 均匀抽稀：超过 MAX_POINTS 时按步长采样，并保证首尾点保留。
fn sample_rows(rows: &[MetricsRow]) -> Vec<MetricsRow> {
    let n = rows.len();
    if n <= MAX_POINTS {
        return rows.to_vec();
    }
    let stride = n.div_ceil(MAX_POINTS);
    let mut out: Vec<MetricsRow> = rows.iter().step_by(stride).cloned().collect();
    if out.last().map(|r| r.step) != rows.last().map(|r| r.step) {
        out.push(rows[n - 1].clone());
    }
    out
}

/// 带 EMA 平滑值的单点（alpha 越大越贴原始线）。
struct MetricPoint {
    row: MetricsRow,
    ema: f64,
}

fn to_metric_points(
    rows: &[MetricsRow],
    y: impl Fn(&MetricsRow) -> f64,
    alpha: f64,
) -> Vec<MetricPoint> {
    let mut ema = 0.0f64;
    let mut initialized = false;
    rows.iter()
        .map(|r| {
            let v = y(r);
            if initialized {
                ema = alpha * v + (1.0 - alpha) * ema;
            } else {
                ema = v;
                initialized = true;
            }
            MetricPoint {
                row: r.clone(),
                ema,
            }
        })
        .collect()
}

/// 单指标折线图卡片（x=step，y=指标值），原始线叠加 EMA 平滑线；hline 为 y 参考横线。
fn metric_chart(
    title: impl IntoElement,
    chart_id: &'static str,
    rows: &[MetricsRow],
    y_fn: impl Fn(&MetricsRow) -> f64 + 'static,
    stroke: Hsla,
    hline: Option<f64>,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let pts = to_metric_points(rows, &y_fn, 0.2);
    let mut chart = MultiLineChart::new(pts)
        .id(chart_id)
        .x(|p| p.row.step)
        .tick_margin((rows.len() / 8).max(1))
        .series("原始", stroke, move |p| y_fn(&p.row))
        .series("EMA", stroke.opacity(0.4), move |p| p.ema);
    // clip_eps 未同步（如从 DB 载入）时为 0，不画参考线
    if let Some(v) = hline.filter(|v| *v > 0.0) {
        chart = chart.hline(v, cx.theme().danger);
    }
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
                .child(chart),
        )
        .into_any_element()
}

/// PPO 损失拆解：policy / value / total 三线，替代原先 policy+value 混合的 loss 图。
fn loss_breakdown_chart(rows: &[MetricsRow], cx: &Context<AppSidebar>) -> AnyElement {
    v_flex()
        .gap_2()
        .child(div().font_semibold().text_sm().child("PPO 损失拆解"))
        .child(h_flex().gap_4().children([
            legend_item("policy".into(), cx.theme().chart_1, cx),
            legend_item("value".into(), cx.theme().chart_2, cx),
            legend_item("total".into(), cx.theme().chart_3, cx),
        ]))
        .child(
            div()
                .h(px(170.))
                .w_full()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .p_2()
                .child(
                    MultiLineChart::new(rows.to_vec())
                        .id("chart-loss-breakdown")
                        .x(|r| r.step)
                        .tick_margin((rows.len() / 8).max(1))
                        .series("policy", cx.theme().chart_1, |r| r.policy_loss as f64)
                        .series("value", cx.theme().chart_2, |r| r.value_loss as f64)
                        .series("total", cx.theme().chart_3, |r| r.total_loss as f64),
                ),
        )
        .into_any_element()
}

/// 奖励构成：每个奖励项一条线（每迭代每步平均贡献），能看出策略在靠什么拿分。
fn reward_breakdown_chart(rows: &[MetricsRow], cx: &Context<AppSidebar>) -> AnyElement {
    let mut terms: Vec<String> = Vec::new();
    for r in rows {
        for it in &r.reward_breakdown {
            if !terms.contains(&it.name) {
                terms.push(it.name.clone());
            }
        }
    }
    let colors = [
        cx.theme().chart_1,
        cx.theme().chart_2,
        cx.theme().chart_3,
        cx.theme().chart_4,
        cx.theme().chart_5,
        cx.theme().accent,
    ];
    let mut chart = MultiLineChart::new(rows.to_vec())
        .id("chart-reward-breakdown")
        .x(|r| r.step)
        .tick_margin((rows.len() / 8).max(1));
    for (i, term) in terms.iter().enumerate() {
        let t = term.clone();
        let label = short_label(&t).to_string();
        chart = chart.series(label, colors[i % colors.len()], move |r| term_value(r, &t));
    }
    v_flex()
        .gap_2()
        .child(div().font_semibold().text_sm().child("奖励构成 (每步平均)"))
        .child(
            h_flex().gap_4().flex_wrap().children(
                terms
                    .iter()
                    .enumerate()
                    .map(|(i, t)| legend_item(short_label(t).into(), colors[i % colors.len()], cx)),
            ),
        )
        .child(
            div()
                .h(px(170.))
                .w_full()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .p_2()
                .child(chart),
        )
        .into_any_element()
}

/// 截掉 " (English)" 后缀得到短标签。
fn short_label(name: &str) -> &str {
    name.split(" (").next().unwrap_or(name)
}

/// 取某行某个奖励项的每步平均贡献，缺省视为 0。
fn term_value(r: &MetricsRow, term: &str) -> f64 {
    r.reward_breakdown
        .iter()
        .find(|it| it.name == term)
        .map(|it| it.value as f64)
        .unwrap_or(0.0)
}

/// 单局步数（最大/最小/平均）三线图卡片。
fn ep_steps_chart(rows: &[MetricsRow], cx: &Context<AppSidebar>) -> AnyElement {
    v_flex()
        .gap_2()
        .child(
            div()
                .font_semibold()
                .text_sm()
                .child("单局步数 (每迭代每环境各一局)"),
        )
        .child(h_flex().gap_4().children([
            legend_item("最大步数".into(), cx.theme().chart_1, cx),
            legend_item("最小步数".into(), cx.theme().chart_2, cx),
            legend_item("平均步数".into(), cx.theme().chart_3, cx),
        ]))
        .child(
            div()
                .h(px(170.))
                .w_full()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .p_2()
                .child(
                    MultiLineChart::new(rows.to_vec())
                        .id("chart-ep-steps")
                        .x(|r| r.step)
                        .tick_margin((rows.len() / 8).max(1))
                        .series("最大步数", cx.theme().chart_1, |r| {
                            r.ep_steps_max as f64
                        })
                        .series("最小步数", cx.theme().chart_2, |r| {
                            r.ep_steps_min as f64
                        })
                        .series("平均步数", cx.theme().chart_3, |r| {
                            r.ep_steps_avg as f64
                        }),
                ),
        )
        .into_any_element()
}

/// 性能分区：FPS 与训练指标分开，附累计步数上下文。
fn perf_section(rows: &[MetricsRow], cx: &Context<AppSidebar>) -> AnyElement {
    let summary = rows
        .last()
        .map(|last| format!("累计步数 {} | 最新 FPS {}", last.step, last.fps))
        .unwrap_or_default();
    v_flex()
        .gap_2()
        .child(div().font_bold().text_sm().child("性能指标"))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(summary),
        )
        .child(
            div()
                .h(px(170.))
                .w_full()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .p_2()
                .child(
                    MultiLineChart::new(rows.to_vec())
                        .id("chart-fps")
                        .x(|r| r.step)
                        .tick_margin((rows.len() / 8).max(1))
                        .series("FPS", cx.theme().chart_1, |r| r.fps as f64),
                ),
        )
        .into_any_element()
}

/// 图例小圆点 + 标签。
fn legend_item(label: SharedString, color: Hsla, cx: &Context<AppSidebar>) -> AnyElement {
    h_flex()
        .gap_1p5()
        .items_center()
        .text_xs()
        .child(div().size_2().rounded_full().bg(color))
        .child(div().text_color(cx.theme().muted_foreground).child(label))
        .into_any_element()
}

/// 单条折线的系列描述（图例名 + 颜色 + 取值器）。
struct Series<T> {
    label: SharedString,
    color: Hsla,
    value: Rc<dyn Fn(&T) -> f64>,
}

/// y 参考横线（如 KL 图的 clip_eps 界）。
struct HLine {
    value: f64,
    color: Hsla,
}

/// 多线折线图：单图叠加多条 y 线，共享数值 x 轴（step）与 y 量纲（从 0 起）。
#[derive(IntoPlot)]
struct MultiLineChart<T>
where
    T: 'static,
{
    data: Vec<T>,
    x: Rc<dyn Fn(&T) -> usize>,
    series: Vec<Series<T>>,
    hlines: Vec<HLine>,
    tick_margin: usize,
    id: Option<ElementId>,
}

impl<T: 'static> MultiLineChart<T> {
    fn new(data: Vec<T>) -> Self {
        Self {
            data,
            x: Rc::new(|_| 0),
            series: Vec::new(),
            hlines: Vec::new(),
            tick_margin: 1,
            id: None,
        }
    }

    fn x(mut self, x: impl Fn(&T) -> usize + 'static) -> Self {
        self.x = Rc::new(x);
        self
    }

    fn series(
        mut self,
        label: impl Into<SharedString>,
        color: impl Into<Hsla>,
        value: impl Fn(&T) -> f64 + 'static,
    ) -> Self {
        self.series.push(Series {
            label: label.into(),
            color: color.into(),
            value: Rc::new(value),
        });
        self
    }

    fn hline(mut self, value: f64, color: impl Into<Hsla>) -> Self {
        self.hlines.push(HLine {
            value,
            color: color.into(),
        });
        self
    }

    fn tick_margin(mut self, margin: usize) -> Self {
        self.tick_margin = margin;
        self
    }

    fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 所有系列共享的 x 点比例尺。
    fn x_scale(&self, width: f32) -> ScalePoint<usize> {
        ScalePoint::new(
            self.data.iter().map(|d| (self.x)(d)).collect(),
            vec![0., width],
        )
    }

    /// 覆盖所有系列取值与参考横线（含 0 基线）的 y 线性比例尺。
    fn y_scale(&self, height: f32) -> ScaleLinear<f64> {
        let mut domain: Vec<f64> = self
            .data
            .iter()
            .flat_map(|d| self.series.iter().map(move |s| (s.value)(d)))
            .collect();
        domain.push(0.0);
        for h in &self.hlines {
            domain.push(h.value);
        }
        ScaleLinear::new(domain, vec![height, 10.])
    }
}

impl<T: 'static> Plot for MultiLineChart<T> {
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32() - AXIS_GAP;
        let x = self.x_scale(width);
        let y = self.y_scale(height);

        // x 轴 + 刻度标签
        let labels: Vec<AxisText> = self
            .data
            .iter()
            .enumerate()
            .filter(|(i, _)| (i + 1) % self.tick_margin == 0)
            .filter_map(|(i, d)| {
                x.tick(&(self.x)(d)).map(|xt| {
                    let align = match i {
                        0 => TextAlign::Left,
                        i if i == self.data.len() - 1 => TextAlign::Right,
                        _ => TextAlign::Center,
                    };
                    AxisText::new((self.x)(d).to_string(), px(xt), cx.theme().muted_foreground)
                        .align(align)
                })
            })
            .collect();
        PlotAxis::new()
            .stroke(cx.theme().border)
            .x(height)
            .x_label(labels)
            .paint(&bounds, window, cx);

        // 网格
        Grid::new()
            .y((0..=3).map(|i| height * i as f32 / 4.0).collect())
            .stroke(cx.theme().border)
            .dash_array(&[px(4.), px(2.)])
            .paint(&bounds, window);

        // 参考横线
        for h in &self.hlines {
            let ys = y.clone();
            let v = h.value;
            let color = h.color;
            Line::new()
                .data(vec![(0.0f32, v), (1.0f32, v)])
                .x(move |d| Some(d.0 * width))
                .y(move |d| ys.tick(&d.1))
                .stroke(color)
                .stroke_style(StrokeStyle::Linear)
                .stroke_width(px(1.))
                .paint(&bounds, window);
        }

        // 逐系列画线（x/y 比例尺与取值器被闭包 move 捕获，需每线 clone）
        for s in &self.series {
            let xf = self.x.clone();
            let xs = x.clone();
            let ys = y.clone();
            let val = s.value.clone();
            let color = s.color;
            Line::new()
                .data(&self.data)
                .x(move |d| xs.tick(&xf(*d)))
                .y(move |d| ys.tick(&val(*d)))
                .stroke(color)
                .stroke_style(StrokeStyle::Linear)
                .stroke_width(px(2.))
                .paint(&bounds, window);
        }
    }

    fn id(&self) -> Option<ElementId> {
        self.id.clone()
    }

    fn tooltip_state(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
        _cx: &App,
    ) -> Option<TooltipState> {
        // 忽略 x 轴标签槽，避免悬停刻度标签弹 tooltip
        if position.y.as_f32() > bounds.size.height.as_f32() - AXIS_GAP {
            return None;
        }
        let height = bounds.size.height.as_f32() - AXIS_GAP;
        let x = self.x_scale(bounds.size.width.as_f32());
        let y = self.y_scale(height);

        let index = x.least_index(position.x.as_f32());
        let d = self.data.get(index)?;
        let x_tick = x.tick(&(self.x)(d))?;
        let dots: Vec<Point<Pixels>> = self
            .series
            .iter()
            .filter_map(|s| y.tick(&(s.value)(d)).map(|yt| point(px(x_tick), px(yt))))
            .collect();
        Some(TooltipState::new(
            index,
            point(px(x_tick), position.y),
            dots,
        ))
    }

    fn tooltip(
        &self,
        state: &TooltipState,
        cursor: Point<Pixels>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement> {
        let d = self.data.get(state.index)?;
        let mut tip = Tooltip::new(cursor, bounds.size)
            .gap(px(8.))
            .cross_line(
                CrossLine::new(state.cross_line).height(bounds.size.height.as_f32() - AXIS_GAP),
            )
            .title((self.x)(d).to_string());
        for s in &self.series {
            tip = tip.row(s.color, s.label.clone(), fmt_val((s.value)(d)));
        }
        let dots = state
            .dots
            .iter()
            .zip(self.series.iter())
            .map(|(p, s)| Dot::new(*p).stroke(cx.theme().background).fill(s.color))
            .collect::<Vec<_>>();
        Some(tip.dots(dots).into_any_element())
    }
}

/// 整数值显示为整数，其余保留两位小数。
fn fmt_val(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v.round() as i64)
    } else {
        format!("{:.2}", v)
    }
}
