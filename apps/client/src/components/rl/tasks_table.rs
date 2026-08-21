use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::DialogFooter;
use gpui_component::input::{Input, InputEvent, InputState, NumberInput};
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

/// 可选算法模型（PPO + Mamba 状态空间模型 或 PPO + MLP 无状态感知机）。
const AGENT_OPTIONS: &[&str] = &[
    lol_rl_protocol::AGENT_PPO_MAMBA,
    lol_rl_protocol::AGENT_PPO_MLP,
];

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
                Column::new("agent", t!("app.rl.col_algorithm")).width(px(110.)),
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
    pub fn set_tasks(&mut self, mut tasks: Vec<TaskOverviewItem>) {
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
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
            2 => div().child(env_label(&task.env_name)).into_any_element(),
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
    raw.to_string()
}

fn env_label(raw: &str) -> String {
    if let Some(spec) = lol_rl_protocol::get_env_spec(raw) {
        format!("{} ({})", spec.label, spec.tag)
    } else {
        raw.to_string()
    }
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
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let task_count = this.task_list.len() + 1;
                                    let tx = this.tx.clone();
                                    let last_env = this.last_chosen_env.clone();
                                    let weak_sidebar = cx.entity().downgrade();
                                    let dialog_view = cx.new(|cx| {
                                        CreateTaskDialogView::new(
                                            task_count,
                                            tx,
                                            last_env,
                                            Some(weak_sidebar),
                                            window,
                                            cx,
                                        )
                                    });
                                    window.open_dialog(cx, move |dialog, _window, _cx| {
                                        let dialog_view = dialog_view.clone();
                                        dialog
                                            .title("开始训练")
                                            .w(px(680.))
                                            .max_h(px(720.))
                                            .overlay_closable(false)
                                            .child(dialog_view.clone())
                                            .footer(
                                                DialogFooter::new()
                                                    .justify_between()
                                                    .child(
                                                        Button::new("reset-default-btn")
                                                            .outline()
                                                            .label("恢复默认配置")
                                                            .on_click({
                                                                let dialog_view = dialog_view.clone();
                                                                move |_, window, cx| {
                                                                    dialog_view.update(cx, |this, cx| {
                                                                        let name = this.default_name.clone();
                                                                        let current_env = this.form.env_name.clone();
                                                                        this.apply_env_params(&current_env, window, cx);
                                                                        this.form.name = name.clone();
                                                                        this.name_input.update(cx, |input, cx| {
                                                                            input.set_value(name, window, cx);
                                                                        });
                                                                        cx.notify();
                                                                    });
                                                                }
                                                            }),
                                                    )
                                                    .child(
                                                        h_flex()
                                                            .gap_2()
                                                            .child(
                                                                Button::new("cancel-create-btn")
                                                                    .ghost()
                                                                    .label("取消")
                                                                    .on_click(|_, window, cx| {
                                                                        window.close_dialog(cx);
                                                                    }),
                                                            )
                                                            .child(
                                                                Button::new("confirm-create-btn")
                                                                    .primary()
                                                                    .icon(IconName::Plus)
                                                                    .label("确认创建任务")
                                                                    .on_click({
                                                                        let dialog_view = dialog_view.clone();
                                                                        move |_, window, cx| {
                                                                            dialog_view.update(cx, |this, _cx| {
                                                                                let mut config = this.form.clone();
                                                                                config.parallel_envs = 0;
                                                                                if let Some(tx) = &this.tx {
                                                                                    let _ = tx.send(InFrame::CreateTask { config });
                                                                                }
                                                                            });
                                                                            window.close_dialog(cx);
                                                                        }
                                                                    }),
                                                            ),
                                                    ),
                                            )
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
    pub sidebar: Option<gpui::WeakEntity<AppSidebar>>,
    pub name_input: Entity<InputState>,
    pub lr_input: Entity<InputState>,
    pub gamma_input: Entity<InputState>,
    pub gae_lambda_input: Entity<InputState>,
    pub clip_eps_input: Entity<InputState>,
    pub ppo_epochs_input: Entity<InputState>,
    pub hidden_dim_input: Entity<InputState>,
    pub total_iterations_input: Entity<InputState>,
}

macro_rules! bind_num_input {
    ($cx:expr, $window:expr, $form:expr, $field:ident, $type:ty) => {{
        let input =
            $cx.new(|cx| InputState::new($window, cx).default_value(&$form.$field.to_string()));
        let sub = input.clone();
        $cx.subscribe(&sub, |this: &mut Self, state, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                if let Ok(val) = state.read(cx).value().parse::<$type>() {
                    this.form.$field = val;
                }
            }
        })
        .detach();
        input
    }};
}

impl CreateTaskDialogView {
    pub fn new(
        task_count: usize,
        tx: Option<tokio::sync::mpsc::UnboundedSender<InFrame>>,
        last_env: Option<String>,
        sidebar: Option<gpui::WeakEntity<AppSidebar>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let env_to_use = last_env.unwrap_or_else(|| lol_rl_protocol::ENV_FIORA_V2.to_string());
        let default_name = format!("RL 对战训练任务 #{}", task_count);
        let mut form = TaskConfigPayload::default_for_env(&env_to_use);
        form.name = default_name.clone();
        form.parallel_envs = 0; // 自适应吞吐探测

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

        let lr_input = bind_num_input!(cx, window, form, lr, f32);
        let gamma_input = bind_num_input!(cx, window, form, gamma, f32);
        let gae_lambda_input = bind_num_input!(cx, window, form, gae_lambda, f32);
        let clip_eps_input = bind_num_input!(cx, window, form, clip_eps, f32);
        let ppo_epochs_input = bind_num_input!(cx, window, form, ppo_epochs, usize);
        let hidden_dim_input = bind_num_input!(cx, window, form, hidden_dim, usize);
        let total_iterations_input = bind_num_input!(cx, window, form, total_iterations, usize);

        Self {
            form,
            default_name,
            tx,
            sidebar,
            name_input,
            lr_input,
            gamma_input,
            gae_lambda_input,
            clip_eps_input,
            ppo_epochs_input,
            hidden_dim_input,
            total_iterations_input,
        }
    }

    /// 应用指定环境的自带推荐参数（唯一真实来源来自 lol_rl_protocol）。
    pub fn apply_env_params(
        &mut self,
        env_name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(weak_sidebar) = &self.sidebar {
            let env_str = env_name.to_string();
            let _ = weak_sidebar.update(cx, |s, _| {
                s.last_chosen_env = Some(env_str);
            });
        }
        let params = lol_rl_protocol::get_env_training_params(env_name);
        self.form.env_name = env_name.to_string();
        self.form.lr = params.lr;
        self.form.gamma = params.gamma;
        self.form.gae_lambda = params.gae_lambda;
        self.form.clip_eps = params.clip_eps;
        self.form.ppo_epochs = params.ppo_epochs;
        self.form.hidden_dim = params.hidden_dim;
        self.form.rollout_steps_per_env = params.rollout_steps_per_env;
        self.form.total_iterations = params.total_iterations;

        // 同步刷新所有输入框的文本
        self.lr_input
            .update(cx, |i, cx| i.set_value(params.lr.to_string(), window, cx));
        self.gamma_input.update(cx, |i, cx| {
            i.set_value(params.gamma.to_string(), window, cx)
        });
        self.gae_lambda_input.update(cx, |i, cx| {
            i.set_value(params.gae_lambda.to_string(), window, cx)
        });
        self.clip_eps_input.update(cx, |i, cx| {
            i.set_value(params.clip_eps.to_string(), window, cx)
        });
        self.ppo_epochs_input.update(cx, |i, cx| {
            i.set_value(params.ppo_epochs.to_string(), window, cx)
        });
        self.hidden_dim_input.update(cx, |i, cx| {
            i.set_value(params.hidden_dim.to_string(), window, cx)
        });
        self.total_iterations_input.update(cx, |i, cx| {
            i.set_value(params.total_iterations.to_string(), window, cx)
        });
        cx.notify();
    }

    fn render_num_setting<F>(
        &self,
        label: &str,
        input_entity: &Entity<InputState>,
        presets: Vec<(&'static str, f32)>,
        current_val: f32,
        cx: &mut Context<Self>,
        setter: F,
    ) -> AnyElement
    where
        F: Fn(&mut Self, f32) + Send + Sync + 'static + Copy,
    {
        let weak_input = input_entity.downgrade();
        h_flex()
            .w_full()
            .items_start()
            .gap_3()
            .child(
                div()
                    .w(px(140.))
                    .flex_shrink_0()
                    .pt_1p5()
                    .text_xs()
                    .font_medium()
                    .text_color(cx.theme().muted_foreground)
                    .child(label.to_string()),
            )
            .child(
                v_flex()
                    .flex_1()
                    .gap_1()
                    .child(div().w_full().child(NumberInput::new(input_entity)))
                    .child(h_flex().w_full().gap_1().children(presets.into_iter().map(
                        |(name, val)| {
                            let is_selected = (val - current_val).abs() < 1e-6;
                            let btn = if is_selected {
                                Button::new(format!("preset-{label}-{name}"))
                                    .primary()
                                    .xsmall()
                                    .compact()
                                    .flex_1()
                                    .label(name)
                            } else {
                                Button::new(format!("preset-{label}-{name}"))
                                    .outline()
                                    .xsmall()
                                    .compact()
                                    .flex_1()
                                    .label(name)
                            };
                            let weak_input = weak_input.clone();
                            btn.on_click(cx.listener(move |this, _, window, cx| {
                                setter(this, val);
                                if let Some(input) = weak_input.upgrade() {
                                    input.update(cx, |i, cx| {
                                        i.set_value(val.to_string(), window, cx)
                                    });
                                }
                                cx.notify();
                            }))
                        },
                    ))),
            )
            .into_any_element()
    }

    fn render_int_setting<F>(
        &self,
        label: &str,
        input_entity: &Entity<InputState>,
        current_val: usize,
        presets: Vec<usize>,
        cx: &mut Context<Self>,
        setter: F,
    ) -> AnyElement
    where
        F: Fn(&mut Self, usize) + Send + Sync + 'static + Copy,
    {
        let weak_input = input_entity.downgrade();
        h_flex()
            .w_full()
            .items_start()
            .gap_3()
            .child(
                div()
                    .w(px(140.))
                    .flex_shrink_0()
                    .pt_1p5()
                    .text_xs()
                    .font_medium()
                    .text_color(cx.theme().muted_foreground)
                    .child(label.to_string()),
            )
            .child(
                v_flex()
                    .flex_1()
                    .gap_1()
                    .child(div().w_full().child(NumberInput::new(input_entity)))
                    .child(
                        h_flex()
                            .w_full()
                            .gap_1()
                            .children(presets.into_iter().map(|val| {
                                let is_selected = val == current_val;
                                let btn = if is_selected {
                                    Button::new(format!("preset-int-{label}-{val}"))
                                        .primary()
                                        .xsmall()
                                        .compact()
                                        .flex_1()
                                        .label(val.to_string())
                                } else {
                                    Button::new(format!("preset-int-{label}-{val}"))
                                        .outline()
                                        .xsmall()
                                        .compact()
                                        .flex_1()
                                        .label(val.to_string())
                                };
                                let weak_input = weak_input.clone();
                                btn.on_click(cx.listener(move |this, _, window, cx| {
                                    setter(this, val);
                                    if let Some(input) = weak_input.upgrade() {
                                        input.update(cx, |i, cx| {
                                            i.set_value(val.to_string(), window, cx)
                                        });
                                    }
                                    cx.notify();
                                }))
                            })),
                    ),
            )
            .into_any_element()
    }
}

impl Render for CreateTaskDialogView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cfg = &self.form;
        let current_agent = cfg.agent_type.clone();
        let current_env_spec = lol_rl_protocol::get_env_spec(&cfg.env_name);
        let current_env_tag = current_env_spec.map(|s| s.tag).unwrap_or("Env");

        v_flex()
            .id("modal-form-scroll")
            .size_full()
            .gap_4()
            .overflow_y_scrollbar()
            .p_1()
                    // ══════════════════════════════════════════════════
                    // 第一层级：核心基础配置 (任务名称、训练环境、算法模型)
                    // ══════════════════════════════════════════════════
                    .child(
                        v_flex()
                            .gap_3()
                            .p_3()
                            .rounded_lg()
                            .bg(cx.theme().muted.opacity(0.15))
                            .border_1()
                            .border_color(cx.theme().border.opacity(0.6))
                            // 1. 任务名称
                            .child(
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .w(px(140.))
                                            .flex_shrink_0()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .child("任务名称"),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .child(Input::new(&self.name_input)),
                                    ),
                            )
                            // 2. 选择环境 (下拉框)
                            .child(
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .w(px(140.))
                                            .flex_shrink_0()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .child("训练环境 (Env)"),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .child(
                                                Button::new("env-dropdown")
                                                    .label(
                                                        current_env_spec
                                                            .map(|s| s.label.to_string())
                                                            .unwrap_or_else(|| cfg.env_name.clone()),
                                                    )
                                                    .dropdown_caret(true)
                                                    .outline()
                                                    .w_full()
                                                    .dropdown_menu({
                                                        let current_env_name = cfg.env_name.clone();
                                                        let weak = cx.entity().downgrade();
                                                        move |menu, _window, _cx| {
                                                            let mut menu = menu;
                                                            for spec in lol_rl_protocol::AVAILABLE_ENVS {
                                                                let checked = spec.name == current_env_name;
                                                                let env_name = spec.name;
                                                                let label = spec.label.to_string();
                                                                let weak = weak.clone();
                                                                menu = menu.item(
                                                                    PopupMenuItem::new(label)
                                                                        .checked(checked)
                                                                        .on_click(move |_, window, cx| {
                                                                            if let Some(view) = weak.upgrade() {
                                                                                let _ = view.update(cx, |this, cx| {
                                                                                    this.apply_env_params(env_name, window, cx);
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
                                    ),
                            )
                            // 环境特性与自带参数摘要说明
                            .when_some(current_env_spec, |this, spec| {
                                this.child(
                                    h_flex()
                                        .w_full()
                                        .items_center()
                                        .gap_3()
                                        .child(div().w(px(140.)).flex_shrink_0())
                                        .child(
                                            div()
                                                .flex_1()
                                                .px_2()
                                                .py_1p5()
                                                .rounded_md()
                                                .bg(cx.theme().muted.opacity(0.35))
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!(
                                                    "💡 特性: {} (单局上限 {} 步, 推荐总轮次: {} 轮, 隐藏层: {})",
                                                    spec.description,
                                                    spec.default_params.rollout_steps_per_env,
                                                    spec.default_params.total_iterations,
                                                    spec.default_params.hidden_dim
                                                )),
                                        ),
                                )
                            })
                            // 3. 算法模型
                            .child(
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .w(px(140.))
                                            .flex_shrink_0()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(cx.theme().foreground)
                                            .child("算法模型 (Agent)"),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
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
                                                                        .on_click(move |_, window, cx| {
                                                                            if let Some(view) = weak.upgrade() {
                                                                                let _ = view.update(cx, |this, cx| {
                                                                                    this.form.agent_type = alg_val.clone();
                                                                                    let is_mlp = alg_val.contains("MLP");
                                                                                    this.form.backbone = Some(if is_mlp {
                                                                                        lol_rl_protocol::PolicyBackbone::Mlp
                                                                                    } else {
                                                                                        lol_rl_protocol::PolicyBackbone::Mamba
                                                                                    });
                                                                                    if is_mlp && this.form.hidden_dim == 64 {
                                                                                        this.form.hidden_dim = 256;
                                                                                        this.hidden_dim_input.update(cx, |i, cx| {
                                                                                            i.set_value("256".to_string(), window, cx);
                                                                                        });
                                                                                    } else if !is_mlp && this.form.hidden_dim == 256 {
                                                                                        this.form.hidden_dim = 64;
                                                                                        this.hidden_dim_input.update(cx, |i, cx| {
                                                                                            i.set_value("64".to_string(), window, cx);
                                                                                        });
                                                                                    }
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
                                    ),
                            ),
                    )
                    // ══════════════════════════════════════════════════
                    // 第二层级：高级超参数配置 (PPO Hyperparameters)
                    // ══════════════════════════════════════════════════
                    .child(
                        v_flex()
                            .gap_3()
                            .p_3()
                            .rounded_lg()
                            .border_1()
                            .border_color(cx.theme().border.opacity(0.8))
                            .child(
                                // 第二层级 Header
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .pb_2()
                                    .border_b_1()
                                    .border_color(cx.theme().border.opacity(0.4))
                                    .child(
                                        h_flex()
                                            .gap_1p5()
                                            .items_center()
                                            .child(
                                                div()
                                                    .font_semibold()
                                                    .text_xs()
                                                    .text_color(cx.theme().foreground)
                                                    .child("高级超参数配置 (Hyperparameters)"),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().accent)
                                            .child(format!("已联动 {} 推荐参数", current_env_tag)),
                                    ),
                            )
                            // 学习率
                            .child(self.render_num_setting(
                                "学习率 (Learning Rate / lr)",
                                &self.lr_input,
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
                            // 折扣因子 Gamma
                            .child(self.render_num_setting(
                                "折扣因子 (Gamma / γ)",
                                &self.gamma_input,
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
                            // GAE Lambda
                            .child(self.render_num_setting(
                                "GAE 因子 (Lambda / λ)",
                                &self.gae_lambda_input,
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
                            // PPO Clip Epsilon
                            .child(self.render_num_setting(
                                "PPO Clip (Clip Eps / ε)",
                                &self.clip_eps_input,
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
                            // 每轮训练 Epochs
                            .child(self.render_int_setting(
                                "每轮训练 Epochs",
                                &self.ppo_epochs_input,
                                cfg.ppo_epochs,
                                vec![1, 2, 4, 8],
                                cx,
                                |this, val| this.form.ppo_epochs = val,
                            ))
                            // 策略模型工作维度（动态判断 MLP 或 Mamba）
                            .child({
                                let is_mlp = cfg.agent_type.contains("MLP") || cfg.backbone == Some(lol_rl_protocol::PolicyBackbone::Mlp);
                                let dim_label = if is_mlp {
                                    "MLP 隐藏层 (Hidden Dim)"
                                } else {
                                    "Mamba 维度 (d_model)"
                                };
                                let dim_presets = if is_mlp {
                                    vec![32, 64, 128, 256, 512]
                                } else {
                                    vec![32, 64, 96, 128, 256]
                                };
                                self.render_int_setting(
                                    dim_label,
                                    &self.hidden_dim_input,
                                    cfg.hidden_dim,
                                    dim_presets,
                                    cx,
                                    |this, val| this.form.hidden_dim = val,
                                )
                            })
                            // 总训练迭代轮次
                            .child(self.render_int_setting(
                                "总迭代轮次 (Iterations)",
                                &self.total_iterations_input,
                                cfg.total_iterations,
                                vec![20, 50, 80, 100, 150, 300],
                                cx,
                                |this, val| this.form.total_iterations = val,
                            ))
                            // 自适应吞吐提示
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
