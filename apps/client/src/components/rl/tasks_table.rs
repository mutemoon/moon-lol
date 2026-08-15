use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::table::{Column, DataTable, TableDelegate, TableState};
use gpui_component::{
    h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable as _, StyledExt, WindowExt as _,
};
use lol_rl_protocol::{InFrame, TaskConfigPayload, TaskOverviewItem};
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::types::ActiveView;

/// 可选算法模型（后端当前仅实现 PPO）。
const AGENT_OPTIONS: &[&str] = &["PPO"];

/// 任务概览表 delegate：驱动 `DataTable` 的列宽/单元格渲染，并反向通信到 `AppSidebar`。
pub struct TaskTableDelegate {
    tasks: Vec<TaskOverviewItem>,
    columns: Vec<Column>,
    sidebar: gpui::WeakEntity<AppSidebar>,
}

impl TaskTableDelegate {
    pub fn new(sidebar: gpui::WeakEntity<AppSidebar>) -> Self {
        Self {
            tasks: Vec::new(),
            columns: vec![
                Column::new("task", t!("app.rl.col_task"))
                    .width(px(180.))
                    .min_width(px(120.)),
                Column::new("agent", t!("app.rl.col_algorithm")).width(px(90.)),
                Column::new("env", t!("app.rl.col_env")).width(px(160.)),
                Column::new("status", t!("app.rl.col_status"))
                    .width(px(90.))
                    .text_center(),
                Column::new("steps_per_iter", t!("app.rl.col_steps_per_iter"))
                    .width(px(80.))
                    .text_right(),
                Column::new("total_iters", t!("app.rl.col_total_iters"))
                    .width(px(80.))
                    .text_right(),
                Column::new("hidden_dim", t!("app.rl.col_hidden_dim"))
                    .width(px(90.))
                    .text_right(),
                Column::new("parallel", t!("app.rl.col_parallel"))
                    .width(px(80.))
                    .text_right(),
                Column::new("lr", t!("app.rl.col_lr"))
                    .width(px(80.))
                    .text_right(),
                Column::new("return", t!("app.rl.col_return"))
                    .width(px(90.))
                    .text_right(),
                Column::new("ckpt", t!("app.rl.col_checkpoints"))
                    .width(px(70.))
                    .text_right(),
                Column::new("actions", t!("app.rl.col_actions"))
                    .width(px(130.))
                    .selectable(false),
            ],
            sidebar,
        }
    }

    /// 更新数据。不触发 `TableState::refresh()`，以保留用户手动调整过的列宽。
    pub fn set_tasks(&mut self, tasks: Vec<TaskOverviewItem>) {
        self.tasks = tasks;
    }
}

impl TableDelegate for TaskTableDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.tasks.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        self.columns.get(col_ix).cloned().unwrap_or_default()
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let col = self.column(col_ix, cx);
        div()
            .size_full()
            .h_flex()
            .items_center()
            .when(col.align == TextAlign::Right, |this| this.justify_end())
            .when(col.align == TextAlign::Center, |this| this.justify_center())
            .child(col.name.clone())
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let Some(task) = self.tasks.get(row_ix) else {
            return div().into_any_element();
        };
        let cell_content: AnyElement = match col_ix {
            0 => div()
                .font_bold()
                .child(task.name.clone())
                .into_any_element(),
            1 => div()
                .child(agent_label(&task.agent_type))
                .into_any_element(),
            2 => div().child(task.env_name.clone()).into_any_element(),
            3 => {
                let is_running = task.status == "running";
                div()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .text_xs()
                    .font_bold()
                    .bg(if is_running {
                        cx.theme().accent
                    } else {
                        cx.theme().secondary
                    })
                    .child(status_label(&task.status))
                    .into_any_element()
            }
            4 => div()
                .w_full()
                .text_right()
                .child(task.rollout_steps_per_env.to_string())
                .into_any_element(),
            5 => div()
                .w_full()
                .text_right()
                .child(task.total_iterations.to_string())
                .into_any_element(),
            6 => div()
                .w_full()
                .text_right()
                .child(task.hidden_dim.to_string())
                .into_any_element(),
            7 => div()
                .w_full()
                .text_right()
                .child(if task.parallel_envs == 0 {
                    "自适应".to_string()
                } else {
                    task.parallel_envs.to_string()
                })
                .into_any_element(),
            8 => div()
                .w_full()
                .text_right()
                .child(format_lr(task.lr))
                .into_any_element(),
            9 => div()
                .w_full()
                .text_right()
                .child(format!("{:.2}", task.ep_return))
                .into_any_element(),
            10 => div()
                .w_full()
                .text_right()
                .child(task.checkpoints_count.to_string())
                .into_any_element(),
            11 => render_actions(
                task.id.clone(),
                task.status == "running",
                self.sidebar.clone(),
                cx,
            ),
            _ => div().into_any_element(),
        };

        div()
            .size_full()
            .h_flex()
            .items_center()
            .child(cell_content)
            .into_any_element()
    }
}

fn agent_label(raw: &str) -> String {
    raw.split_whitespace().next().unwrap_or(raw).to_string()
}

fn status_label(status: &str) -> String {
    match status {
        "running" => "训练中",
        "queued" => "排队中",
        "finished" => "已完成",
        "stopped" => "已停止",
        "interrupted" => "已中断",
        other => other,
    }
    .to_string()
}

fn format_lr(lr: f32) -> String {
    format!("{lr:.0e}")
}

fn render_actions(
    task_id: String,
    is_running: bool,
    weak: gpui::WeakEntity<AppSidebar>,
    cx: &mut Context<TableState<TaskTableDelegate>>,
) -> AnyElement {
    h_flex()
        .gap_1()
        .justify_end()
        .child(
            Button::new(format!("view-{task_id}"))
                .compact()
                .ghost()
                .icon(IconName::Eye)
                .on_click({
                    let weak = weak.clone();
                    let tid = task_id.clone();
                    cx.listener(move |_, _, _, cx| {
                        let _ = weak.update(cx, |s, s_cx| {
                            s.selected_task_id = Some(tid.clone());
                            s.navigate_to(ActiveView::RlTaskDetail);
                            s_cx.notify();
                        });
                    })
                }),
        )
        .child(
            Button::new(format!("stop-{task_id}"))
                .compact()
                .ghost()
                .icon(IconName::CircleX)
                .disabled(!is_running)
                .on_click({
                    let weak = weak.clone();
                    let tid = task_id.clone();
                    cx.listener(move |_, _, _, cx| {
                        let _ = weak.update(cx, |s, s_cx| {
                            if let Some(tx) = &s.tx {
                                let _ = tx.send(InFrame::Control {
                                    task_id: tid.clone(),
                                    command: "stop".into(),
                                    config_json: None,
                                });
                            }
                            s_cx.notify();
                        });
                    })
                }),
        )
        .child(
            Button::new(format!("delete-{task_id}"))
                .compact()
                .ghost()
                .icon(IconName::Delete)
                .on_click({
                    let weak = weak.clone();
                    let tid = task_id.clone();
                    cx.listener(move |_, _, window, cx| {
                        // on_ok 拿不到 AppSidebar，提前取出发帧通道
                        let tx = weak.update(cx, |s, _| s.tx.clone()).unwrap_or(None);
                        let tid_dialog = tid.clone();
                        window.open_alert_dialog(cx, move |alert, _, _| {
                            let tx_dialog = tx.clone();
                            let tid_ok = tid_dialog.clone();
                            alert
                                .confirm()
                                .title(t!("app.rl.delete_task_title"))
                                .description(t!("app.rl.delete_task_desc"))
                                .on_ok(move |_, _, _| {
                                    if let Some(tx) = &tx_dialog {
                                        let _ = tx.send(InFrame::DeleteTask {
                                            task_id: tid_ok.clone(),
                                        });
                                    }
                                    true
                                })
                        });
                    })
                }),
        )
        .into_any_element()
}

// 渲染多强化学习任务/学习实例概览表格
pub fn render_tasks_table(
    sidebar: &mut AppSidebar,
    _window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let main_content = v_flex()
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
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let task_count = this.task_list.len() + 1;
                                    let tx = this.tx.clone();
                                    let dialog_view = cx.new(|cx| {
                                        CreateTaskDialogView::new(task_count, tx, window, cx)
                                    });
                                    window.open_dialog(cx, move |dialog, _window, _cx| {
                                        dialog
                                            .w(px(640.))
                                            .max_h(px(680.))
                                            .overlay_closable(false)
                                            .child(dialog_view.clone())
                                    });
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
        .child(match &sidebar.table_state {
            Some(table_state) => div()
                .id("tasks-table-scroll")
                .flex_1()
                .w_full()
                .child(
                    DataTable::new(table_state)
                        .bordered(true)
                        .stripe(false)
                        .with_size(px(52.)),
                )
                .into_any_element(),
            None => div().flex_1().into_any_element(),
        });

    main_content.into_any_element()
}

pub struct CreateTaskDialogView {
    pub form: TaskConfigPayload,
    pub default_name: String,
    pub tx: Option<tokio::sync::mpsc::UnboundedSender<InFrame>>,
    pub name_input: Entity<InputState>,
}

impl CreateTaskDialogView {
    pub fn new(
        task_count: usize,
        tx: Option<tokio::sync::mpsc::UnboundedSender<InFrame>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let default_name = format!("RL 对战训练任务 #{}", task_count);
        let mut form = TaskConfigPayload::default();
        form.name = default_name.clone();
        form.parallel_envs = 0; // 自适应吞吐探测
        form.max_steps = 0;

        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("输入任务名称")
                .default_value(&form.name)
        });

        let sub_entity = name_input.clone();
        cx.subscribe(&sub_entity, |this, state, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.form.name = state.read(cx).value().to_string();
            }
        })
        .detach();

        Self {
            form,
            default_name,
            tx,
            name_input,
        }
    }

    fn render_num_setting<F>(
        &self,
        label: &str,
        display_val: String,
        presets: Vec<(&'static str, f32)>,
        current_val: f32,
        cx: &mut Context<Self>,
        setter: F,
    ) -> AnyElement
    where
        F: Fn(&mut Self, f32) + Send + Sync + 'static + Copy,
    {
        v_flex()
            .gap_1()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .font_bold()
                            .text_color(cx.theme().muted_foreground)
                            .child(label.to_string()),
                    )
                    .child(div().text_xs().font_bold().child(display_val)),
            )
            .child(
                h_flex()
                    .gap_1()
                    .children(presets.into_iter().map(|(name, val)| {
                        let is_selected = (val - current_val).abs() < 1e-6;
                        let btn = if is_selected {
                            Button::new(format!("preset-{label}-{name}"))
                                .primary()
                                .compact()
                                .label(name)
                        } else {
                            Button::new(format!("preset-{label}-{name}"))
                                .outline()
                                .compact()
                                .label(name)
                        };
                        btn.on_click(cx.listener(move |this, _, _, cx| {
                            setter(this, val);
                            cx.notify();
                        }))
                    })),
            )
            .into_any_element()
    }

    fn render_int_setting<F>(
        &self,
        label: &str,
        current_val: usize,
        presets: Vec<usize>,
        cx: &mut Context<Self>,
        setter: F,
    ) -> AnyElement
    where
        F: Fn(&mut Self, usize) + Send + Sync + 'static + Copy,
    {
        v_flex()
            .gap_1()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .font_bold()
                            .text_color(cx.theme().muted_foreground)
                            .child(label.to_string()),
                    )
                    .child(div().text_xs().font_bold().child(current_val.to_string())),
            )
            .child(h_flex().gap_1().children(presets.into_iter().map(|val| {
                let is_selected = val == current_val;
                let btn = if is_selected {
                    Button::new(format!("preset-int-{label}-{val}"))
                        .primary()
                        .compact()
                        .label(val.to_string())
                } else {
                    Button::new(format!("preset-int-{label}-{val}"))
                        .outline()
                        .compact()
                        .label(val.to_string())
                };
                btn.on_click(cx.listener(move |this, _, _, cx| {
                    setter(this, val);
                    cx.notify();
                }))
            })))
            .into_any_element()
    }
}

impl Render for CreateTaskDialogView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cfg = &self.form;
        let current_agent = cfg.agent_type.clone();

        v_flex()
            .gap_4()
            .child(
                // 弹窗 Header
                h_flex()
                    .items_center()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(IconName::Plus)
                            .child(div().font_bold().text_lg().child("开始训练")),
                    ),
            )
            .child(
                // 滚动表单区域
                v_flex()
                    .id("modal-form-scroll")
                    .flex_1()
                    .gap_4()
                    .overflow_y_scrollbar()
                    .p_1()
                    // 1. 任务名称
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_bold()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("任务名称"),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .child(Input::new(&self.name_input)),
                            ),
                    )
                    // 2. 算法与环境标识
                    .child(
                        h_flex()
                            .gap_4()
                            .child(
                                v_flex()
                                    .flex_1()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_bold()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("算法模型 (Agent)"),
                                    )
                                    .child(
                                        Button::new("agent-dropdown")
                                            .label(agent_label(&current_agent))
                                            .dropdown_caret(true)
                                            .outline()
                                            .w_full()
                                            .dropdown_menu({
                                                let current_agent = current_agent.clone();
                                                let weak = cx.entity().downgrade();
                                                move |menu, _window, _cx| {
                                                    let mut menu = menu;
                                                    for &alg in AGENT_OPTIONS {
                                                        let alg = alg.to_string();
                                                        let checked = alg == current_agent;
                                                        let weak = weak.clone();
                                                        let alg_val = alg.clone();
                                                        menu = menu.item(
                                                            PopupMenuItem::new(alg.clone())
                                                                .checked(checked)
                                                                .on_click(move |_, _, cx| {
                                                                    if let Some(view) = weak.upgrade() {
                                                                        let _ = view.update(cx, |this, cx| {
                                                                            this.form.agent_type = alg_val.clone();
                                                                            cx.notify();
                                                                        });
                                                                    }
                                                                }),
                                                        );
                                                    }
                                                    menu
                                                }
                                            }),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .flex_1()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_bold()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("训练环境 (Env)"),
                                    )
                                    .child(
                                        h_flex()
                                            .gap_1()
                                            .child(
                                                Button::new("env-real-btn")
                                                    .when(
                                                        cfg.env_name == lol_rl_protocol::ENV_FIORA_VS_RIVEN_REAL,
                                                        |b| b.primary(),
                                                    )
                                                    .when(
                                                        cfg.env_name != lol_rl_protocol::ENV_FIORA_VS_RIVEN_REAL,
                                                        |b| b.outline(),
                                                    )
                                                    .compact()
                                                    .label("真实移动 (10f)")
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.form.env_name = lol_rl_protocol::ENV_FIORA_VS_RIVEN_REAL.to_string();
                                                        cx.notify();
                                                    })),
                                            )
                                            .child(
                                                Button::new("env-legacy-btn")
                                                    .when(
                                                        cfg.env_name == lol_rl_protocol::ENV_FIORA_VS_RIVEN_LEGACY,
                                                        |b| b.primary(),
                                                    )
                                                    .when(
                                                        cfg.env_name != lol_rl_protocol::ENV_FIORA_VS_RIVEN_LEGACY,
                                                        |b| b.outline(),
                                                    )
                                                    .compact()
                                                    .label("瞬移站位 (Legacy)")
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.form.env_name = lol_rl_protocol::ENV_FIORA_VS_RIVEN_LEGACY.to_string();
                                                        cx.notify();
                                                    })),
                                            ),
                                    ),
                            ),
                    )
                    // 3. 学习率
                    .child(self.render_num_setting(
                        "学习率 (Learning Rate / lr)",
                        format!("{:.5}", cfg.lr),
                        vec![
                            ("1e-4", 0.0001),
                            ("3e-4", 0.0003),
                            ("5e-4", 0.0005),
                            ("1e-3", 0.0010),
                        ],
                        cfg.lr,
                        cx,
                        |this, val| this.form.lr = val,
                    ))
                    // 4. 折扣因子 Gamma
                    .child(self.render_num_setting(
                        "折扣因子 (Gamma / γ)",
                        format!("{:.3}", cfg.gamma),
                        vec![
                            ("0.90", 0.90),
                            ("0.95", 0.95),
                            ("0.98", 0.98),
                            ("0.99", 0.99),
                            ("0.999", 0.999),
                        ],
                        cfg.gamma,
                        cx,
                        |this, val| this.form.gamma = val,
                    ))
                    // 5. GAE Lambda
                    .child(self.render_num_setting(
                        "GAE 因子 (Lambda / λ)",
                        format!("{:.2}", cfg.gae_lambda),
                        vec![
                            ("0.80", 0.80),
                            ("0.90", 0.90),
                            ("0.95", 0.95),
                            ("0.98", 0.98),
                        ],
                        cfg.gae_lambda,
                        cx,
                        |this, val| this.form.gae_lambda = val,
                    ))
                    // 6. PPO Clip Epsilon
                    .child(self.render_num_setting(
                        "PPO Clip 范围 (Clip Epsilon / ε)",
                        format!("{:.2}", cfg.clip_eps),
                        vec![
                            ("0.10", 0.10),
                            ("0.15", 0.15),
                            ("0.20", 0.20),
                            ("0.25", 0.25),
                            ("0.30", 0.30),
                        ],
                        cfg.clip_eps,
                        cx,
                        |this, val| this.form.clip_eps = val,
                    ))
                    // 7. PPO Epochs & 隐藏层维度
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().flex_1().child(self.render_int_setting(
                                "每轮训练 Epochs",
                                cfg.ppo_epochs,
                                vec![1, 2, 4, 8],
                                cx,
                                |this, val| this.form.ppo_epochs = val,
                            )))
                            .child(div().flex_1().child(self.render_int_setting(
                                "隐藏层神经元 (Hidden Dim)",
                                cfg.hidden_dim,
                                vec![32, 64, 128, 256],
                                cx,
                                |this, val| this.form.hidden_dim = val,
                            ))),
                    )
                    // 8. 总训练迭代轮次与自适应吞吐
                    .child(
                        v_flex()
                            .gap_2()
                            .child(self.render_int_setting(
                                "总训练迭代轮次 (Total Iterations)",
                                cfg.total_iterations,
                                vec![20, 50, 80, 150, 300],
                                cx,
                                |this, val| this.form.total_iterations = val,
                            ))
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .p_2()
                                    .rounded_md()
                                    .bg(cx.theme().muted.opacity(0.3))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("⚡ 并行对局数与 GPU 推理/训练批大小将由 AutoTuner 在任务启动时自动探测硬件算力并求解最优吞吐。"),
                                    ),
                            ),
                    ),
            )
            .child(
                // 底部 Action
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        Button::new("reset-default-btn")
                            .outline()
                            .label("恢复默认配置")
                            .on_click(cx.listener(|this, _, window, cx| {
                                let name = this.default_name.clone();
                                let mut def = TaskConfigPayload::default();
                                def.name = name.clone();
                                def.parallel_envs = 0;
                                def.max_steps = 0;
                                this.form = def;
                                this.name_input.update(cx, |input, cx| {
                                    input.set_value(name, window, cx);
                                });
                                cx.notify();
                            })),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("cancel-create-btn")
                                    .ghost()
                                    .label("取消")
                                    .on_click(cx.listener(|_, _, window, cx| {
                                        window.close_dialog(cx);
                                    })),
                            )
                            .child(
                                    Button::new("confirm-create-btn")
                                        .primary()
                                        .icon(IconName::Plus)
                                        .label("确认创建任务")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            let mut config = this.form.clone();
                                            config.parallel_envs = 0;
                                            config.max_steps = 0;
                                            if let Some(tx) = &this.tx {
                                                let _ = tx.send(InFrame::CreateTask { config });
                                            }
                                            window.close_dialog(cx);
                                        })),
                            ),
                    ),
            )
    }
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
