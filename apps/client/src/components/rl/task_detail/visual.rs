use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::switch::Switch;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use lol_rl_protocol::{PolicyDisplay, PolicyItem, VisualInFrame, VisualObsFrame, ENV_FIORA_V0};
use rust_i18n::t;

use crate::components::rl::task_detail::math::render_math;
use crate::components::sidebar::AppSidebar;
use crate::types::TaskDetailTab;

/// 运行可视 Env 控制舱 (Cockpit) 状态与实时控制/观察面板
pub fn render_running_visual(sidebar: &AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let Some(model_id) = sidebar.running_visual_model.clone() else {
        return render_visual_empty(cx);
    };

    let header = render_visual_header(sidebar, &model_id, cx);
    let telemetry = render_visual_telemetry(sidebar, cx);

    v_flex()
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
        .child(telemetry)
        .into_any_element()
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
        (t!("app.rl.visual_starting"), cx.theme().primary)
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
        .child(render_visual_controls(sidebar, cx))
        .into_any_element()
}

fn render_visual_controls(sidebar: &AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let is_paused = sidebar.visual_paused;
    let auto_pause = sidebar.visual_auto_pause;
    h_flex()
        .id("vis-controls-bar")
        .gap_2()
        .items_center()
        .child(
            h_flex()
                .items_center()
                .gap_1p5()
                .child(
                    Switch::new("vis-auto-pause-switch")
                        .checked(auto_pause)
                        .on_click(cx.listener(|this, checked: &bool, _window, cx| {
                            this.set_visual_auto_pause(*checked);
                            cx.notify();
                        })),
                )
                .child(div().text_xs().text_color(cx.theme().muted_foreground).child("局末自动暂停")),
        )
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
                    this.visual_env_name = None;
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
            .text_color(cx.theme().foreground.opacity(0.85))
            .child("正在等待视觉 ENV 画面与遥测数据...")
            .into_any_element();
    };

    let is_paused = sidebar.visual_paused;
    let show_manual_actions = is_paused
        && sidebar.visual_ws_connected
        && sidebar.visual_env_name.as_deref() == Some(ENV_FIORA_V0);

    let mut left_cards = v_flex()
        .flex_1()
        .gap_3()
        .child(render_telemetry_status_card(f, cx))
        .child(render_telemetry_policy_card(f, cx))
        .child(render_telemetry_reward_card(f, cx));

    if show_manual_actions {
        left_cards = left_cards.child(render_manual_action_panel(sidebar, cx));
    }

    let right_card = div()
        .w(px(290.0))
        .flex_shrink_0()
        .child(render_telemetry_obs_card(f, cx));

    h_flex()
        .items_start()
        .gap_3()
        .w_full()
        .child(left_cards)
        .child(right_card)
        .into_any_element()
}

fn render_telemetry_obs_card(f: &VisualObsFrame, cx: &Context<AppSidebar>) -> AnyElement {
    let rows: Vec<(String, f32)> = f
        .obs_vector
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let label = f
                .obs_labels
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("dim {i}"));
            (label, *v)
        })
        .collect();

    div()
        .w_full()
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
                        .child(div().font_bold().text_sm().child("实时观测向量 (Raw Obs)"))
                        .child(
                            div()
                                .text_xs()
                                .font_bold()
                                .text_color(cx.theme().primary)
                                .child(format!("{} 维", rows.len())),
                        ),
                )
                .child(if rows.is_empty() {
                    div()
                        .text_xs()
                        .text_color(cx.theme().foreground.opacity(0.75))
                        .child("暂无观测向量数据")
                        .into_any_element()
                } else {
                    v_flex()
                        .gap_1()
                        .children(rows.iter().map(|(label, value)| {
                            h_flex()
                                .justify_between()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(cx.theme().foreground.opacity(0.85))
                                        .child(label.clone()),
                                )
                                .child(div().font_semibold().child(format!("{value:.3}")))
                        }))
                        .into_any_element()
                }),
        )
        .into_any_element()
}

fn render_telemetry_status_card(f: &VisualObsFrame, cx: &Context<AppSidebar>) -> AnyElement {
    div()
        .w_full()
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
                                    cx.theme().success
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
                        .child(
                            div()
                                .text_color(cx.theme().foreground.opacity(0.85))
                                .child("当前 Step"),
                        )
                        .child(div().font_semibold().child(format!("{}", f.step))),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .text_xs()
                        .child(
                            div()
                                .text_color(cx.theme().foreground.opacity(0.85))
                                .child("当前累积奖励"),
                        )
                        .child(
                            div()
                                .font_bold()
                                .text_color(if f.episode_reward >= 0.0 {
                                    cx.theme().success
                                } else {
                                    cx.theme().danger
                                })
                                .child(format!("{:+.3}", f.episode_reward)),
                        ),
                ),
        )
        .into_any_element()
}

fn render_telemetry_policy_card(f: &VisualObsFrame, cx: &Context<AppSidebar>) -> AnyElement {
    div()
        .w_full()
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
                .child(render_policy_display(&f.policy, cx)),
        )
        .into_any_element()
}

fn render_policy_display(policy: &PolicyDisplay, cx: &Context<AppSidebar>) -> AnyElement {
    match policy {
        PolicyDisplay::Discrete(items) if items.is_empty() => div()
            .text_xs()
            .text_color(cx.theme().foreground.opacity(0.75))
            .child("无概率数据")
            .into_any_element(),
        PolicyDisplay::Discrete(items) => render_policy_items_table(items, cx),
        PolicyDisplay::Hybrid {
            move_x,
            move_z,
            attack_prob,
            raw_attack_prob,
            is_attack_masked,
        } => {
            let item = PolicyItem {
                action_id: 1,
                action: "攻击".to_string(),
                prob: *attack_prob,
                raw_prob: *raw_attack_prob,
                is_masked: *is_attack_masked,
            };
            v_flex()
                .gap_1p5()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().foreground.opacity(0.85))
                        .child("预测下一步移动偏移 (±1 归一化)"),
                )
                .child(policy_value_row("move_x", *move_x))
                .child(policy_value_row("move_z", *move_z))
                .child(
                    v_flex()
                        .gap_1()
                        .mt_1()
                        .child(render_policy_items_table(&[item], cx)),
                )
                .into_any_element()
        }
        PolicyDisplay::HybridMulti {
            continuous_means,
            discrete_probs,
        } => {
            let offset_x = continuous_means.first().copied().unwrap_or(0.0);
            let offset_z = continuous_means.get(1).copied().unwrap_or(0.0);
            v_flex()
                .gap_1p5()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().foreground.opacity(0.85))
                        .child("连续偏移 (±1 归一化)"),
                )
                .child(policy_value_row("offset_x", offset_x))
                .child(policy_value_row("offset_z", offset_z))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().foreground.opacity(0.85))
                        .mt_1()
                        .child("动作类别概率 (Action Probs)"),
                )
                .child(render_policy_items_table(discrete_probs, cx))
                .into_any_element()
        }
    }
}

fn render_policy_table_header(cx: &Context<AppSidebar>) -> AnyElement {
    h_flex()
        .items_center()
        .gap_2()
        .w_full()
        .pb_1()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .w(px(100.0))
                .text_xs()
                .font_bold()
                .text_color(cx.theme().foreground)
                .child("动作类别"),
        )
        .child(
            h_flex()
                .flex_1()
                .gap_3()
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .font_bold()
                        .text_color(hsla(215.0 / 360.0, 0.85, 0.58, 1.0))
                        .child("未 Mask (模型倾向)"),
                )
                .child(
                    div()
                        .flex_1()
                        .text_xs()
                        .font_bold()
                        .text_color(cx.theme().success)
                        .child("Mask 后 (实际执行)"),
                ),
        )
        .into_any_element()
}

fn render_policy_items_table(items: &[PolicyItem], cx: &Context<AppSidebar>) -> AnyElement {
    v_flex()
        .gap_1p5()
        .w_full()
        .child(render_policy_table_header(cx))
        .children(items.iter().map(|item| policy_prob_row(item, cx)))
        .into_any_element()
}

fn policy_prob_row(item: &PolicyItem, cx: &Context<AppSidebar>) -> AnyElement {
    let raw_p = item.raw_prob.clamp(0.0, 1.0);
    let masked_p = item.prob.clamp(0.0, 1.0);

    h_flex()
        .items_center()
        .gap_2()
        .w_full()
        .child(
            div()
                .w(px(100.0))
                .text_xs()
                .font_semibold()
                .text_color(if item.is_masked {
                    cx.theme().foreground.opacity(0.6)
                } else {
                    cx.theme().foreground
                })
                .child(item.action.clone()),
        )
        .child(
            h_flex()
                .flex_1()
                .gap_3()
                .child(
                    // 左栏：未 Mask
                    v_flex()
                        .flex_1()
                        .gap_0p5()
                        .child(
                            h_flex()
                                .justify_between()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(cx.theme().foreground.opacity(0.8))
                                        .child("原始"),
                                )
                                .child(
                                    div()
                                        .font_bold()
                                        .text_color(hsla(215.0 / 360.0, 0.85, 0.58, 1.0))
                                        .child(format!("{:.1}%", raw_p * 100.0)),
                                ),
                        )
                        .child(
                            div()
                                .h_1p5()
                                .w_full()
                                .rounded_full()
                                .bg(cx.theme().secondary)
                                .child(
                                    div()
                                        .h_full()
                                        .rounded_full()
                                        .bg(hsla(215.0 / 360.0, 0.85, 0.58, 1.0))
                                        .w(Length::Definite(DefiniteLength::Fraction(raw_p))),
                                ),
                        ),
                )
                .child(
                    // 右栏：Mask 后
                    v_flex()
                        .flex_1()
                        .gap_0p5()
                        .child(
                            h_flex()
                                .justify_between()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(if item.is_masked {
                                            cx.theme().danger
                                        } else {
                                            cx.theme().foreground.opacity(0.8)
                                        })
                                        .child(if item.is_masked {
                                            "已屏蔽"
                                        } else {
                                            "有效"
                                        }),
                                )
                                .child(
                                    div()
                                        .font_bold()
                                        .text_color(if item.is_masked {
                                            cx.theme().danger
                                        } else {
                                            cx.theme().success
                                        })
                                        .child(format!("{:.1}%", masked_p * 100.0)),
                                ),
                        )
                        .child(
                            div()
                                .h_1p5()
                                .w_full()
                                .rounded_full()
                                .bg(cx.theme().secondary)
                                .child(
                                    div()
                                        .h_full()
                                        .rounded_full()
                                        .bg(if item.is_masked {
                                            cx.theme().danger.opacity(0.3)
                                        } else {
                                            cx.theme().success
                                        })
                                        .w(Length::Definite(DefiniteLength::Fraction(masked_p))),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

fn policy_value_row(label: &str, value: f32) -> AnyElement {
    h_flex()
        .justify_between()
        .text_xs()
        .child(div().child(label.to_string()))
        .child(div().font_bold().child(format!("{value:+.2}")))
        .into_any_element()
}

fn render_telemetry_reward_card(f: &VisualObsFrame, cx: &Context<AppSidebar>) -> AnyElement {
    let empty_vars = std::collections::HashMap::new();
    let vars = f.reward_variables.as_ref().unwrap_or(&empty_vars);

    div()
        .w_full()
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
                            v_flex()
                                .items_end()
                                .gap_0p5()
                                .child(
                                    div()
                                        .font_bold()
                                        .text_xs()
                                        .text_color(if f.reward >= 0.0 {
                                            cx.theme().success
                                        } else {
                                            cx.theme().danger
                                        })
                                        .child(format!("单步奖励: {:+.3}", f.reward)),
                                )
                                .child(
                                    div()
                                        .font_bold()
                                        .text_xs()
                                        .text_color(if f.episode_reward >= 0.0 {
                                            cx.theme().success
                                        } else {
                                            cx.theme().danger
                                        })
                                        .child(format!("累积奖励: {:+.3}", f.episode_reward)),
                                ),
                        ),
                )
                .child(if let Some(formula) = &f.reward_formula {
                    v_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().foreground.opacity(0.85))
                                .child("公式 (符号)"),
                        )
                        .child(render_math(&formula.to_latex(), cx))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().foreground.opacity(0.85))
                                .child("代入当前步"),
                        )
                        .child(render_math(&formula.to_latex_substituted(vars), cx))
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
                                            cx.theme().success
                                        } else {
                                            cx.theme().danger
                                        })
                                        .child(format!("{:+}", r.value)),
                                )
                        }))
                        .into_any_element()
                } else {
                    div()
                        .text_xs()
                        .text_color(cx.theme().foreground.opacity(0.75))
                        .child("无细拆项")
                        .into_any_element()
                }),
        )
        .into_any_element()
}

fn render_manual_action_panel(sidebar: &AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let actions: Vec<(usize, String)> = sidebar
        .latest_visual_frame
        .as_ref()
        .and_then(|frame| match &frame.policy {
            PolicyDisplay::Discrete(items) if !items.is_empty() => Some(
                items
                    .iter()
                    .map(|p| (p.action_id, p.action.clone()))
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(legacy_manual_actions);

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
                .text_color(cx.theme().foreground.opacity(0.85))
                .child("点击下方按钮执行对应 action 并步进一步"),
        )
        .child(
            h_flex()
                .id("manual-action-buttons-flex")
                .flex_wrap()
                .gap_2()
                .children(actions.into_iter().map(|(action_id, label)| {
                    let btn_id = SharedString::from(format!("manual-action-btn-{}", action_id));
                    let wrapper_id =
                        SharedString::from(format!("manual-action-wrap-{}", action_id));
                    div()
                        .id(wrapper_id)
                        .child(
                            Button::new(btn_id)
                                .outline()
                                .label(label)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.send_visual_cmd(VisualInFrame::StepWithAction {
                                        action_id,
                                    });
                                    cx.notify();
                                })),
                        )
                })),
        )
        .into_any_element()
}

fn legacy_manual_actions() -> Vec<(usize, String)> {
    vec![
        (0, "MoveEast50 (东侧50u)".to_string()),
        (1, "MoveWest50 (西侧50u)".to_string()),
        (2, "MoveNorth50 (北侧50u)".to_string()),
        (3, "MoveSouth50 (南侧50u)".to_string()),
        (4, "AttackRiven (攻击瑞雯)".to_string()),
    ]
}
