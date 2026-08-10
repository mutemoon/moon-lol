use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use lol_rl_protocol::{VisualInFrame, VisualObsFrame};
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::types::TaskDetailTab;

/// 运行可视 Env 控制舱 (Cockpit) 状态与实时控制/观察面板
pub fn render_running_visual(sidebar: &AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let Some(model_id) = sidebar.running_visual_model.clone() else {
        return render_visual_empty(cx);
    };

    let is_paused = sidebar.visual_paused;
    let header = render_visual_header(sidebar, &model_id, is_paused, cx);
    let telemetry = render_visual_telemetry(sidebar, cx);

    let mut container = v_flex()
        .size_full()
        .flex_1()
        .overflow_y_scrollbar()
        .gap_3()
        .p_4()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().accent)
        .bg(cx.theme().accent.opacity(0.08))
        .child(header)
        .child(telemetry);

    if is_paused && sidebar.visual_ws_connected {
        container = container.child(render_manual_action_panel(cx));
    }

    container.into_any_element()
}

fn render_visual_empty(cx: &mut Context<AppSidebar>) -> AnyElement {
    v_flex()
        .size_full()
        .flex_1()
        .items_center()
        .justify_center()
        .gap_3()
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("当前未运行任何可视环境，请在任务详情中选择 Checkpoint 启动观察"),
        )
        .child(
            Button::new("vis-empty-back")
                .primary()
                .label("去选择模型")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.task_detail_tab = TaskDetailTab::Models;
                    cx.notify();
                })),
        )
        .into_any_element()
}

fn render_visual_header(
    sidebar: &AppSidebar,
    model_id: &str,
    is_paused: bool,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let is_terminated = sidebar
        .latest_visual_frame
        .as_ref()
        .map_or(false, |f| f.terminated);

    let (status_text, status_color) = if let Some(err) = &sidebar.visual_error {
        (
            t!("app.rl.visual_error", err = err.clone()),
            cx.theme().danger,
        )
    } else if sidebar.visual_ws_connected {
        if sidebar.visual_paused {
            if is_terminated {
                ("对局结束已自动暂停".into(), cx.theme().warning)
            } else {
                ("已暂停".into(), cx.theme().warning)
            }
        } else {
            (t!("app.rl.visual_connected"), cx.theme().success)
        }
    } else {
        (t!("app.rl.visual_starting"), cx.theme().muted_foreground)
    };

    h_flex()
        .items_center()
        .justify_between()
        .child(
            h_flex().gap_2().items_center().child(IconName::Play).child(
                v_flex()
                    .child(
                        div()
                            .font_bold()
                            .text_base()
                            .child(format!("运行 ENV 控制舱 (Checkpoint: {})", model_id)),
                    )
                    .child(div().text_xs().text_color(status_color).child(status_text)),
            ),
        )
        .child(render_visual_controls(is_paused, cx))
        .into_any_element()
}

fn render_visual_controls(is_paused: bool, cx: &mut Context<AppSidebar>) -> AnyElement {
    h_flex()
        .id("vis-controls-bar")
        .gap_2()
        .child(div().id("vis-pause-resume-toggle").child(if is_paused {
            Button::new("vis-resume")
                .primary()
                .icon(IconName::Play)
                .label("继续")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.send_visual_cmd(VisualInFrame::Resume);
                    cx.notify();
                }))
        } else {
            Button::new("vis-pause")
                .outline()
                .icon(IconName::Pause)
                .label("暂停")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.send_visual_cmd(VisualInFrame::Pause);
                    cx.notify();
                }))
        }))
        .child(
            Button::new("vis-step")
                .outline()
                .icon(IconName::ChevronRight)
                .label("步进")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.send_visual_cmd(VisualInFrame::StepOnce);
                    cx.notify();
                })),
        )
        .child(
            Button::new("vis-reset")
                .outline()
                .icon(IconName::Loader)
                .label("重置对局")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.send_visual_cmd(VisualInFrame::Reset);
                    cx.notify();
                })),
        )
        .child(
            Button::new("close-visual")
                .ghost()
                .icon(IconName::Close)
                .label("停止")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.visual_session = None;
                    this.visual_in_tx = None;
                    this.latest_visual_frame = None;
                    this.visual_error = None;
                    this.visual_ws_connected = false;
                    this.visual_paused = false;
                    this.visual_task_id = None;
                    this.running_visual_model = None;
                    this.task_detail_tab = TaskDetailTab::Models;
                    cx.notify();
                })),
        )
        .into_any_element()
}

fn render_visual_telemetry(sidebar: &AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let Some(f) = &sidebar.latest_visual_frame else {
        return div()
            .p_4()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .child("正在等待视觉 ENV 画面与遥测数据...")
            .into_any_element();
    };

    v_flex()
        .gap_3()
        .w_full()
        .child(
            h_flex()
                .gap_3()
                .w_full()
                .child(render_telemetry_status_card(f, cx))
                .child(render_telemetry_obs_card(f, cx)),
        )
        .child(
            h_flex()
                .gap_3()
                .w_full()
                .child(render_telemetry_policy_card(f, cx))
                .child(render_telemetry_reward_card(f, cx)),
        )
        .into_any_element()
}

fn render_telemetry_obs_card(f: &VisualObsFrame, cx: &Context<AppSidebar>) -> AnyElement {
    let obs = &f.obs;

    let vec_preview = format!(
        "[{:.1}, {:.1}, {:.1}, {:.1}]",
        if obs.vital_direction.contains("+X") {
            1.0
        } else {
            0.0
        },
        if obs.vital_direction.contains("-X") {
            1.0
        } else {
            0.0
        },
        if obs.vital_direction.contains("+Z") {
            1.0
        } else {
            0.0
        },
        if obs.vital_direction.contains("-Z") {
            1.0
        } else {
            0.0
        },
    );

    div()
        .flex_1()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(
            v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .justify_between()
                        .child(
                            div()
                                .font_bold()
                                .text_sm()
                                .child("实时步进观测 (Step Observe)"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_bold()
                                .text_color(cx.theme().accent)
                                .child(format!("Step #{}", f.step)),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .justify_between()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("菲奥娜/瑞雯血量"),
                                )
                                .child(div().font_semibold().child(format!(
                                    "{:.1}% / {:.1}%",
                                    obs.fiora_hp_pct * 100.0,
                                    obs.riven_hp_pct * 100.0
                                ))),
                        )
                        .child(
                            h_flex()
                                .justify_between()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("英雄间极坐标距离"),
                                )
                                .child(
                                    div().font_semibold().child(format!("{:.1}u", obs.distance)),
                                ),
                        )
                        .child(
                            h_flex()
                                .justify_between()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("弱点破绽 (Vital)"),
                                )
                                .child(div().font_semibold().child(if obs.has_vital {
                                    format!(
                                        "{} ({})",
                                        if obs.vital_is_active {
                                            "激活"
                                        } else {
                                            "未激活"
                                        },
                                        obs.vital_direction
                                    )
                                } else {
                                    "无破绽".into()
                                })),
                        )
                        .child(
                            h_flex()
                                .justify_between()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("存活状态 (Alive)"),
                                )
                                .child(div().font_semibold().child(format!(
                                    "菲奥娜: {} | 瑞雯: {}",
                                    if f.fiora_alive { "存活" } else { "阵亡" },
                                    if f.riven_alive { "存活" } else { "阵亡" }
                                ))),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .mt_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_bold()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("4维破绽方向观察向量 (Obs Vector Preview):"),
                                )
                                .child(
                                    div()
                                        .p_1p5()
                                        .rounded_md()
                                        .bg(cx.theme().secondary)
                                        .text_xs()
                                        .overflow_x_hidden()
                                        .child(vec_preview),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

fn render_telemetry_status_card(f: &VisualObsFrame, cx: &Context<AppSidebar>) -> AnyElement {
    div()
        .flex_1()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(
            v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .justify_between()
                        .child(div().font_bold().text_sm().child("ENV 对战状态"))
                        .child(
                            div()
                                .text_xs()
                                .font_bold()
                                .text_color(if f.terminated {
                                    cx.theme().danger
                                } else {
                                    cx.theme().accent
                                })
                                .child(if f.terminated {
                                    "战斗结束"
                                } else {
                                    "对战进行中"
                                }),
                        ),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .text_xs()
                        .child(div().child(format!("Step: {}", f.step)))
                        .child(div().child(format!("Step Reward: {:.2}", f.reward))),
                )
                .child(hp_bar("菲奥娜", f.obs.fiora_hp_pct, cx))
                .child(hp_bar("瑞雯", f.obs.riven_hp_pct, cx))
                .child(
                    h_flex()
                        .justify_between()
                        .text_xs()
                        .child(div().child(format!("距离: {:.1}u", f.obs.distance)))
                        .child(div().child(format!(
                            "破绽: {} ({})",
                            if f.obs.has_vital { "在场" } else { "无" },
                            f.obs.vital_direction
                        ))),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .text_xs()
                        .child(skill_badge("Q", f.obs.q_ready, cx))
                        .child(skill_badge("W", f.obs.w_ready, cx))
                        .child(skill_badge("E", f.obs.e_ready, cx))
                        .child(skill_badge("R", f.obs.r_ready, cx)),
                ),
        )
        .into_any_element()
}

fn render_telemetry_policy_card(f: &VisualObsFrame, cx: &Context<AppSidebar>) -> AnyElement {
    div()
        .flex_1()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(
            v_flex()
                .gap_2()
                .child(
                    div()
                        .font_bold()
                        .text_sm()
                        .child("实时动作概率 (Policy Probs)"),
                )
                .child(if f.policy.is_empty() {
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("无概率数据")
                        .into_any_element()
                } else {
                    v_flex()
                        .gap_1()
                        .children(f.policy.iter().map(|p| {
                            v_flex()
                                .gap_0p5()
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .text_xs()
                                        .child(div().child(p.action.clone()))
                                        .child(
                                            div()
                                                .font_bold()
                                                .child(format!("{:.1}%", p.prob * 100.0)),
                                        ),
                                )
                                .child(
                                    div()
                                        .h_1p5()
                                        .w_full()
                                        .rounded_full()
                                        .bg(cx.theme().secondary)
                                        .child(
                                            div().h_full().rounded_full().bg(cx.theme().accent).w(
                                                Length::Definite(DefiniteLength::Fraction(p.prob)),
                                            ),
                                        ),
                                )
                        }))
                        .into_any_element()
                }),
        )
        .into_any_element()
}

fn render_telemetry_reward_card(f: &VisualObsFrame, cx: &Context<AppSidebar>) -> AnyElement {
    let empty_vars = std::collections::HashMap::new();
    let vars = f.reward_variables.as_ref().unwrap_or(&empty_vars);

    div()
        .flex_1()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(
            v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .font_bold()
                                .text_sm()
                                .child("结构化单步奖励推导 (Reward Formula)"),
                        )
                        .child(
                            div()
                                .font_bold()
                                .text_xs()
                                .text_color(if f.reward >= 0.0 {
                                    cx.theme().accent
                                } else {
                                    cx.theme().muted_foreground
                                })
                                .child(format!("单步总奖励: {:+.2}", f.reward)),
                        ),
                )
                .child(if let Some(formula) = &f.reward_formula {
                    v_flex()
                        .gap_1p5()
                        .children(formula.terms.iter().map(|term| {
                            let term_val = term.eval(vars);
                            let formula_str = term.expr.to_display_string();

                            v_flex()
                                .p_1p5()
                                .rounded_md()
                                .bg(cx.theme().secondary)
                                .gap_0p5()
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .items_center()
                                        .child(
                                            div().font_bold().text_xs().child(term.label.clone()),
                                        )
                                        .child(
                                            div()
                                                .font_bold()
                                                .text_xs()
                                                .text_color(if term_val >= 0.0 {
                                                    cx.theme().accent
                                                } else {
                                                    cx.theme().muted_foreground
                                                })
                                                .child(format!("{:+}", term_val)),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("公式: {}", formula_str)),
                                )
                        }))
                        .into_any_element()
                } else if !f.reward_breakdown.is_empty() {
                    v_flex()
                        .gap_1()
                        .children(f.reward_breakdown.iter().map(|r| {
                            h_flex()
                                .justify_between()
                                .items_center()
                                .p_1p5()
                                .rounded_md()
                                .bg(cx.theme().secondary)
                                .child(div().text_xs().child(r.name.clone()))
                                .child(
                                    div()
                                        .font_bold()
                                        .text_xs()
                                        .text_color(if r.value >= 0.0 {
                                            cx.theme().accent
                                        } else {
                                            cx.theme().muted_foreground
                                        })
                                        .child(format!("{:+}", r.value)),
                                )
                        }))
                        .into_any_element()
                } else {
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("无细拆项")
                        .into_any_element()
                }),
        )
        .into_any_element()
}

fn render_manual_action_panel(cx: &mut Context<AppSidebar>) -> AnyElement {
    const ACTIONS: &[(usize, &str)] = &[
        (0, "移动东50u"),
        (1, "移动西50u"),
        (2, "移动北50u"),
        (3, "移动南50u"),
        (4, "攻击瑞雯"),
    ];

    v_flex()
        .id("manual-action-panel-container")
        .gap_2()
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(div().font_bold().text_sm().child("手动 Action 步进 (调试)"))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("点击下方按钮执行对应 action 并步进一步"),
        )
        .child(
            h_flex()
                .id("manual-action-buttons-flex")
                .flex_wrap()
                .gap_2()
                .children(ACTIONS.iter().map(|(id, label)| {
                    let action_id = *id;
                    let btn_id = SharedString::from(format!("manual-action-btn-{}", action_id));
                    let wrapper_id =
                        SharedString::from(format!("manual-action-wrap-{}", action_id));
                    div().id(wrapper_id).child(
                        Button::new(btn_id)
                            .outline()
                            .label(*label)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.send_visual_cmd(VisualInFrame::StepWithAction { action_id });
                                cx.notify();
                            })),
                    )
                })),
        )
        .into_any_element()
}

fn skill_badge(name: &'static str, ready: bool, cx: &Context<AppSidebar>) -> AnyElement {
    div()
        .px_1p5()
        .py_0p5()
        .rounded_md()
        .text_xs()
        .font_bold()
        .bg(if ready {
            cx.theme().accent.opacity(0.2)
        } else {
            cx.theme().secondary
        })
        .text_color(if ready {
            cx.theme().accent
        } else {
            cx.theme().muted_foreground
        })
        .child(format!("{}: {}", name, if ready { "READY" } else { "CD" }))
        .into_any_element()
}

/// 单个英雄血量进度条（pct ∈ [0, 1]）。
fn hp_bar(label: &'static str, pct: f32, cx: &Context<AppSidebar>) -> AnyElement {
    let pct = pct.clamp(0.0, 1.0);
    h_flex()
        .gap_2()
        .items_center()
        .child(div().w_16().text_xs().child(label))
        .child(
            div()
                .flex_1()
                .h_2()
                .rounded_full()
                .bg(cx.theme().secondary)
                .child(
                    div()
                        .h_full()
                        .rounded_full()
                        .bg(cx.theme().accent)
                        .w(Length::Definite(DefiniteLength::Fraction(pct))),
                ),
        )
        .child(div().w_12().text_xs().child(format!("{:.0}%", pct * 100.0)))
        .into_any_element()
}
