use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use lol_rl_protocol::{InFrame, TaskConfigPayload};
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::types::LocalTaskDetail;

// 渲染多强化学习任务/学习实例概览表格
pub fn render_tasks_table(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let tasks = sidebar.task_list.clone();

    v_flex()
        .size_full()
        .flex_1()
        .gap_4()
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::LayoutDashboard)
                        .child(div().font_bold().text_lg().child(t!("app.rl.page_title"))),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(ws_status_badge(sidebar.ws_connected, cx))
                        .child(
                            Button::new("new-task-btn")
                                .primary()
                                .icon(IconName::Plus)
                                .label(t!("app.rl.new_task"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let task_count = this.task_list.len() + 1;
                                    let config = TaskConfigPayload {
                                        name: format!("RL 对战训练任务 #{}", task_count),
                                        agent_type: "PPO (Candle)".into(),
                                        env_name: "FioraVsRivenEnv-v0".into(),
                                        lr: 3e-4,
                                        parallel_envs: 4,
                                        max_steps: 10000,
                                    };
                                    this.send_in_frame(InFrame::CreateTask { config });
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("refresh-btn")
                                .outline()
                                .icon(IconName::Loader)
                                .label(t!("app.rl.refresh_list"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.send_in_frame(InFrame::GetTaskList);
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .child(
            div()
                .id("tasks-table-scroll")
                .flex_1()
                .w_full()
                .overflow_y_scroll()
                .child(
                    Table::new()
                        .child(
                            TableHeader::new().child(
                                TableRow::new()
                                    .child(TableHead::new().child(t!("app.rl.col_task")))
                                    .child(TableHead::new().child(t!("app.rl.col_algorithm")))
                                    .child(TableHead::new().child(t!("app.rl.col_status")))
                                    .child(TableHead::new().child(t!("app.rl.col_steps")))
                                    .child(TableHead::new().child(t!("app.rl.col_return")))
                                    .child(TableHead::new().child(t!("app.rl.col_checkpoints")))
                                    .child(TableHead::new().child(t!("app.rl.col_actions"))),
                            ),
                        )
                        .child(TableBody::new().children(tasks.into_iter().map(|t| {
                            let task_id = t.id.clone();
                            let is_running = t.status == "running";

                            TableRow::new()
                                .child(
                                    TableCell::new().child(
                                        v_flex().child(div().font_bold().child(t.name)).child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(t.id.clone()),
                                        ),
                                    ),
                                )
                                .child(
                                    TableCell::new().child(
                                        v_flex().child(div().child(t.agent_type)).child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(t.env_name),
                                        ),
                                    ),
                                )
                                .child(
                                    TableCell::new().child(
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_md()
                                            .text_xs()
                                            .font_bold()
                                            .bg(if is_running {
                                                cx.theme().accent
                                            } else {
                                                cx.theme().secondary
                                            })
                                            .child(t.status.to_uppercase()),
                                    ),
                                )
                                .child(TableCell::new().child(t.current_step.to_string()))
                                .child(TableCell::new().child(format!("{:.2}", t.ep_return)))
                                .child(TableCell::new().child(t.checkpoints_count.to_string()))
                                .child(
                                    TableCell::new().child(
                                        h_flex()
                                            .gap_1()
                                            .child(
                                                Button::new(format!("view-{}", task_id))
                                                    .ghost()
                                                    .icon(IconName::Eye)
                                                    .label(t!("app.rl.enter_detail"))
                                                    .on_click({
                                                        let tid = task_id.clone();
                                                        cx.listener(move |this, _, _, cx| {
                                                            this.selected_task_id =
                                                                Some(tid.clone());
                                                            cx.notify();
                                                        })
                                                    }),
                                            )
                                            .child(
                                                Button::new(format!("stop-{}", task_id))
                                                    .ghost()
                                                    .icon(IconName::CircleX)
                                                    .label(t!("app.rl.stop"))
                                                    .disabled(!is_running)
                                                    .on_click({
                                                        let tid = task_id.clone();
                                                        cx.listener(move |this, _, _, cx| {
                                                            this.send_in_frame(InFrame::Control {
                                                                task_id: tid.clone(),
                                                                command: "stop".into(),
                                                                config_json: None,
                                                            });
                                                            cx.notify();
                                                        })
                                                    }),
                                            )
                                            .child(
                                                Button::new(format!("save-{}", task_id))
                                                    .ghost()
                                                    .icon(IconName::HardDrive)
                                                    .label(t!("app.rl.save_model"))
                                                    .on_click({
                                                        let tid = task_id;
                                                        cx.listener(move |this, _, _, cx| {
                                                            this.send_in_frame(
                                                                InFrame::SaveCheckpoint {
                                                                    task_id: tid.clone(),
                                                                },
                                                            );
                                                            cx.notify();
                                                        })
                                                    }),
                                            ),
                                    ),
                                )
                        }))),
                ),
        )
        .into_any_element()
}

// WebSocket 连接状态：小圆点 + 简洁文案，位于标题行右侧
fn ws_status_badge(connected: bool, cx: &mut Context<AppSidebar>) -> AnyElement {
    let (color, label) = if connected {
        (cx.theme().accent, t!("app.rl.ws_connected"))
    } else {
        (cx.theme().muted_foreground, t!("app.rl.ws_disconnected"))
    };
    h_flex()
        .gap_1()
        .items_center()
        .text_xs()
        .child(div().w_2().h_2().rounded_full().bg(color))
        .child(label)
        .into_any_element()
}

// 渲染特定训练任务的详细指标与图表 Dashboard
pub fn render_task_detail(
    sidebar: &mut AppSidebar,
    task_id: String,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let detail = sidebar
        .task_details
        .get(&task_id)
        .cloned()
        .unwrap_or_else(|| LocalTaskDetail {
            id: task_id.clone(),
            name: format!("任务 {}", task_id),
            agent_type: "PPO".into(),
            env_name: "FioraVsRivenEnv".into(),
            status: "running".into(),
            current_step: 0,
            ep_return: 0.0,
            checkpoints: vec![],
            metrics_history: vec![],
            latest_policy: vec![],
            latest_reward_breakdown: vec![],
            latest_obs: None,
            logs: vec![],
        });

    let running_visual = sidebar.running_visual_model.clone();

    v_flex()
        .size_full()
        .flex_1()
        .gap_4()
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Button::new("back-table")
                                .outline()
                                .icon(IconName::ChevronLeft)
                                .label(t!("app.rl.back_to_list"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.selected_task_id = None;
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .font_bold()
                                .text_lg()
                                .child(t!("app.rl.task_detail_title", id = detail.name)),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("save-detail-ckpt")
                                .primary()
                                .icon(IconName::HardDrive)
                                .label(t!("app.rl.save_current_model"))
                                .on_click({
                                    let tid = task_id.clone();
                                    cx.listener(move |this, _, _, cx| {
                                        this.send_in_frame(InFrame::SaveCheckpoint {
                                            task_id: tid.clone(),
                                        });
                                        cx.notify();
                                    })
                                }),
                        )
                        .child(
                            Button::new("stop-detail-task")
                                .ghost()
                                .icon(IconName::CircleX)
                                .label(t!("app.rl.stop_running"))
                                .on_click({
                                    let tid = task_id.clone();
                                    cx.listener(move |this, _, _, cx| {
                                        this.send_in_frame(InFrame::Control {
                                            task_id: tid.clone(),
                                            command: "stop".into(),
                                            config_json: None,
                                        });
                                        cx.notify();
                                    })
                                }),
                        ),
                ),
        )
        .child(if let Some(model_id) = running_visual {
            div()
                .p_4()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().accent)
                .bg(cx.theme().accent.opacity(0.1))
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(
                            h_flex().gap_2().items_center().child(IconName::Play).child(
                                div()
                                    .font_bold()
                                    .child(t!("app.rl.running_visual_banner", model = model_id)),
                            ),
                        )
                        .child(
                            Button::new("close-visual")
                                .ghost()
                                .icon(IconName::Close)
                                .label(t!("app.rl.close_visual"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.running_visual_model = None;
                                    cx.notify();
                                })),
                        ),
                )
                .into_any_element()
        } else {
            div().into_any_element()
        })
        // 第一排图表 & 观测卡片
        .child(
            h_flex()
                .gap_4()
                .w_full()
                .child(
                    // 动作概率分布图表 (Action Probabilities)
                    div()
                        .flex_1()
                        .p_4()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().border)
                        .child(
                            v_flex()
                                .gap_2()
                                .child(div().font_bold().child(t!("app.rl.section_policy")))
                                .child(v_flex().gap_1().children(detail.latest_policy.iter().map(
                                    |p| {
                                        v_flex()
                                            .gap_0p5()
                                            .child(
                                                h_flex()
                                                    .justify_between()
                                                    .text_xs()
                                                    .child(div().child(p.action.clone()))
                                                    .child(
                                                        div().font_bold().child(format!(
                                                            "{:.1}%",
                                                            p.prob * 100.0
                                                        )),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .h_2()
                                                    .w_full()
                                                    .rounded_full()
                                                    .bg(cx.theme().secondary)
                                                    .child(
                                                        div()
                                                            .h_full()
                                                            .rounded_full()
                                                            .bg(cx.theme().accent)
                                                            .w(Length::Definite(
                                                                DefiniteLength::Fraction(p.prob),
                                                            )),
                                                    ),
                                            )
                                    },
                                ))),
                        ),
                )
                .child(
                    // 细化奖励拆解 (Reward Breakdown)
                    div()
                        .flex_1()
                        .p_4()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().border)
                        .child(
                            v_flex()
                                .gap_2()
                                .child(div().font_bold().child(t!("app.rl.section_reward")))
                                .child(v_flex().gap_2().children(
                                    detail.latest_reward_breakdown.iter().map(|r| {
                                        h_flex()
                                            .justify_between()
                                            .items_center()
                                            .p_2()
                                            .rounded_md()
                                            .bg(cx.theme().secondary)
                                            .child(div().text_sm().child(r.name.clone()))
                                            .child(
                                                div()
                                                    .font_bold()
                                                    .text_sm()
                                                    .text_color(if r.value >= 0.0 {
                                                        cx.theme().accent
                                                    } else {
                                                        cx.theme().muted_foreground
                                                    })
                                                    .child(format!("{:+}", r.value)),
                                            )
                                    }),
                                )),
                        ),
                )
                .child(
                    // Checkpoints 权重库
                    div()
                        .w_72()
                        .p_4()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().border)
                        .child(
                            v_flex()
                                .gap_2()
                                .child(div().font_bold().child(t!(
                                    "app.rl.checkpoints_count",
                                    n = detail.checkpoints.len()
                                )))
                                .child(v_flex().gap_2().children(detail.checkpoints.iter().map(
                                    |ckpt| {
                                        let ckpt_id = ckpt.id.clone();
                                        let tid = task_id.clone();

                                        h_flex()
                                            .justify_between()
                                            .items_center()
                                            .p_2()
                                            .rounded_md()
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .child(
                                                v_flex()
                                                    .child(
                                                        div()
                                                            .font_bold()
                                                            .text_xs()
                                                            .child(ckpt.id.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(format!("Step: {}", ckpt.step)),
                                                    ),
                                            )
                                            .child(
                                                Button::new(format!("run-model-{}", ckpt_id))
                                                    .primary()
                                                    .icon(IconName::Play)
                                                    .label(t!("app.rl.run_visual_env"))
                                                    .on_click(cx.listener(
                                                        move |this, _, _, cx| {
                                                            this.running_visual_model =
                                                                Some(ckpt_id.clone());
                                                            this.send_in_frame(
                                                                InFrame::ApplyCheckpoint {
                                                                    task_id: tid.clone(),
                                                                    id: ckpt_id.clone(),
                                                                },
                                                            );
                                                            cx.notify();
                                                        },
                                                    )),
                                            )
                                    },
                                ))),
                        ),
                ),
        )
        // 第二排：训练指标 Data Table 统计
        .child(
            v_flex()
                .flex_1()
                .gap_2()
                .overflow_hidden()
                .child(
                    div()
                        .font_bold()
                        .text_base()
                        .child(t!("app.rl.metrics_title")),
                )
                .child(
                    div()
                        .id("metrics-table-scroll")
                        .flex_1()
                        .overflow_y_scroll()
                        .child(
                            Table::new()
                                .child(
                                    TableHeader::new().child(
                                        TableRow::new()
                                            .child(TableHead::new().child(t!("app.rl.m_step")))
                                            .child(
                                                TableHead::new()
                                                    .child(t!("app.rl.m_episode_return")),
                                            )
                                            .child(TableHead::new().child(t!("app.rl.m_loss")))
                                            .child(TableHead::new().child(t!("app.rl.m_kl")))
                                            .child(TableHead::new().child(t!("app.rl.m_entropy")))
                                            .child(TableHead::new().child(t!("app.rl.m_value")))
                                            .child(TableHead::new().child(t!("app.rl.m_fps"))),
                                    ),
                                )
                                .child(TableBody::new().children(
                                    detail.metrics_history.iter().map(|m| {
                                        TableRow::new()
                                            .child(TableCell::new().child(m.step.to_string()))
                                            .child(
                                                TableCell::new()
                                                    .child(format!("{:.2}", m.ep_return)),
                                            )
                                            .child(TableCell::new().child(format!("{:.4}", m.loss)))
                                            .child(TableCell::new().child(format!("{:.4}", m.kl)))
                                            .child(
                                                TableCell::new().child(format!("{:.2}", m.entropy)),
                                            )
                                            .child(
                                                TableCell::new().child(format!("{:.2}", m.value)),
                                            )
                                            .child(TableCell::new().child(m.fps.to_string()))
                                    }),
                                )),
                        ),
                ),
        )
        .into_any_element()
}
