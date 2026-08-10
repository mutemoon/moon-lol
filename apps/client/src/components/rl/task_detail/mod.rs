pub mod metrics;
pub mod models;
pub mod visual;

use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, IconName};
use lol_rl_protocol::InFrame;
use rust_i18n::t;
pub use visual::render_running_visual;

use crate::components::sidebar::AppSidebar;
use crate::types::{LocalTaskDetail, TaskDetailTab};

// 渲染特定训练任务的详细指标与图表 Dashboard
pub fn render_task_detail(
    sidebar: &mut AppSidebar,
    task_id: String,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    // 若当前任务详情在本地缺乏历史 metrics、logs 或 checkpoints，向服务端请求恢复完整详情
    let is_empty = sidebar
        .task_details
        .get(&task_id)
        .map(|d| d.metrics_history.is_empty() && d.logs.is_empty() && d.checkpoints.is_empty())
        .unwrap_or(true);
    if is_empty {
        sidebar.send_in_frame(InFrame::GetTaskDetail {
            task_id: task_id.clone(),
        });
    }

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
            reward_formula: None,
            latest_reward_variables: None,
            logs: vec![],
        });

    let current_tab = sidebar.task_detail_tab;
    let is_vis_running = sidebar.running_visual_model.is_some();

    v_flex()
        .size_full()
        .flex_1()
        .gap_4()
        .overflow_hidden()
        // ── 顶部 Header：Tabs 选项卡 + 全局控制按钮 ──
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Button::new("tab-metrics")
                                .icon(IconName::ChartPie)
                                .label("图表与指标")
                                .when(current_tab == TaskDetailTab::Metrics, |b| b.primary())
                                .when(current_tab != TaskDetailTab::Metrics, |b| b.ghost())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.task_detail_tab = TaskDetailTab::Metrics;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("tab-models")
                                .icon(IconName::HardDrive)
                                .label(format!("模型 ({})", detail.checkpoints.len()))
                                .when(current_tab == TaskDetailTab::Models, |b| b.primary())
                                .when(current_tab != TaskDetailTab::Models, |b| b.ghost())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.task_detail_tab = TaskDetailTab::Models;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("tab-visual")
                                .icon(IconName::Play)
                                .label(if is_vis_running {
                                    "运行中的 ENV (推演中)"
                                } else {
                                    "运行中的 ENV"
                                })
                                .when(current_tab == TaskDetailTab::VisualEnv, |b| b.primary())
                                .when(current_tab != TaskDetailTab::VisualEnv, |b| b.ghost())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.task_detail_tab = TaskDetailTab::VisualEnv;
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("save-detail-ckpt")
                                .outline()
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
        // ── 根据当前选中的 Tab 渲染主体内容 ──
        .child(match current_tab {
            TaskDetailTab::Metrics => metrics::render_tab_metrics(&detail, cx),
            TaskDetailTab::Models => models::render_tab_models(sidebar, &detail, &task_id, cx),
            TaskDetailTab::VisualEnv => visual::render_running_visual(sidebar, cx),
        })
        .into_any_element()
}
