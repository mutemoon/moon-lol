use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use lol_rl_protocol::InFrame;
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::types::{LocalTaskDetail, TaskDetailTab};

/// Tab 2: 模型与 Checkpoints 列表
pub fn render_tab_models(
    _sidebar: &AppSidebar,
    detail: &LocalTaskDetail,
    task_id: &str,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    v_flex()
        .size_full()
        .flex_1()
        .overflow_hidden()
        .p_4()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .gap_3()
        .child(
            div()
                .font_bold()
                .child(t!("app.rl.checkpoints_count", n = detail.checkpoints.len())),
        )
        .child(if detail.checkpoints.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("暂无已保存的模型 Checkpoint")
                .into_any_element()
        } else {
            div()
                .id("ckpt-list-scroll")
                .flex_1()
                .overflow_y_scrollbar()
                .child(
                    v_flex()
                        .gap_3()
                        .p_1()
                        .children(detail.checkpoints.iter().map(|ckpt| {
                            let ckpt_id = ckpt.id.clone();
                            let tid = task_id.to_string();

                            h_flex()
                                .justify_between()
                                .items_center()
                                .p_4()
                                .min_h(px(60.))
                                .min_w_64()
                                .rounded_md()
                                .border_1()
                                .border_color(cx.theme().border)
                                .child(
                                    v_flex()
                                        .child(div().font_bold().text_xs().child(ckpt.id.clone()))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!("Iteration: {}", ckpt.step)),
                                        ),
                                )
                                .child(
                                    Button::new(format!("run-model-{}", ckpt_id))
                                        .primary()
                                        .icon(IconName::Play)
                                        .label(t!("app.rl.run_visual_env"))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.visual_session = None;
                                            this.visual_in_tx = None;
                                            this.latest_visual_frame = None;
                                            this.visual_error = None;
                                            this.visual_ws_connected = false;
                                            this.visual_paused = false;
                                            this.visual_task_id = Some(tid.clone());
                                            this.running_visual_model = Some(ckpt_id.clone());
                                            this.send_in_frame(InFrame::ApplyCheckpoint {
                                                task_id: tid.clone(),
                                                id: ckpt_id.clone(),
                                            });
                                            this.task_detail_tab = TaskDetailTab::VisualEnv;
                                            cx.notify();
                                        })),
                                )
                        })),
                )
                .into_any_element()
        })
        .into_any_element()
}
