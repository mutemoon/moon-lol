use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::switch::Switch;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable, StyledExt};
use lol_rl_protocol::{
    ActionBranchDisplay, ActionNode, ObsValueNode, PolicyDisplay, PolicyItem, VisualInFrame,
    VisualObsFrame,
};
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
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("局末自动暂停"),
                ),
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

    let left_cards = v_flex()
        .flex_1()
        .gap_3()
        .child(render_telemetry_status_card(f, cx))
        .child(render_telemetry_policy_card(f, cx))
        .child(render_telemetry_reward_card(f, cx));

    let right_card = v_flex()
        .w(px(330.0))
        .flex_shrink_0()
        .gap_3()
        .child(render_telemetry_obs_card(sidebar, f, cx))
        .child(render_telemetry_action_schema_card(sidebar, cx));

    h_flex()
        .items_start()
        .gap_3()
        .w_full()
        .child(left_cards)
        .child(right_card)
        .into_any_element()
}

enum FlatObsItem<'a> {
    Scalar {
        depth: usize,
        name: &'a str,
        value: f32,
    },
    Vector {
        depth: usize,
        name: &'a str,
        values: &'a [f32],
    },
    Categorical {
        depth: usize,
        name: &'a str,
        class_id: usize,
    },
    GroupHeader {
        depth: usize,
        name: &'a str,
        path: String,
        count_label: String,
        is_collapsed: bool,
        is_repeated: bool,
    },
}

fn flatten_obs_nodes<'a>(
    nodes: &'a [ObsValueNode],
    path_prefix: &str,
    depth: usize,
    collapsed: &std::collections::HashSet<String>,
    out: &mut Vec<FlatObsItem<'a>>,
) {
    if depth > 10 {
        return;
    }
    for (i, node) in nodes.iter().enumerate() {
        let node_path = format!("{path_prefix}.{}_{}", node.name(), i);
        match node {
            ObsValueNode::Scalar { name, value } => {
                out.push(FlatObsItem::Scalar {
                    depth,
                    name,
                    value: *value,
                });
            }
            ObsValueNode::Vector { name, values } => {
                out.push(FlatObsItem::Vector {
                    depth,
                    name,
                    values,
                });
            }
            ObsValueNode::Categorical { name, class_id, .. } => {
                out.push(FlatObsItem::Categorical {
                    depth,
                    name,
                    class_id: *class_id,
                });
            }
            ObsValueNode::Struct { name, fields } => {
                let is_collapsed = collapsed.contains(&node_path);
                out.push(FlatObsItem::GroupHeader {
                    depth,
                    name,
                    path: node_path.clone(),
                    count_label: format!("{} 项", fields.len()),
                    is_collapsed,
                    is_repeated: false,
                });
                if !is_collapsed {
                    flatten_obs_nodes(fields, &node_path, depth + 1, collapsed, out);
                }
            }
            ObsValueNode::Repeated { name, items } => {
                let is_collapsed = collapsed.contains(&node_path);
                out.push(FlatObsItem::GroupHeader {
                    depth,
                    name,
                    path: node_path.clone(),
                    count_label: format!("{} 实体", items.len()),
                    is_collapsed,
                    is_repeated: true,
                });
                if !is_collapsed {
                    flatten_obs_nodes(items, &node_path, depth + 1, collapsed, out);
                }
            }
        }
    }
}

fn render_flat_obs_item(item: FlatObsItem<'_>, cx: &Context<AppSidebar>) -> AnyElement {
    match item {
        FlatObsItem::Scalar { depth, name, value } => {
            let val_str = if value.fract() == 0.0 {
                format!("{:.0}", value)
            } else {
                format!("{:.3}", value)
            };
            h_flex()
                .justify_between()
                .items_center()
                .text_xs()
                .pl(px(depth as f32 * 10.0))
                .py_0p5()
                .child(
                    div()
                        .text_color(cx.theme().foreground.opacity(0.85))
                        .child(name.to_string()),
                )
                .child(
                    div()
                        .font_bold()
                        .text_color(cx.theme().foreground)
                        .child(val_str),
                )
                .into_any_element()
        }
        FlatObsItem::Vector {
            depth,
            name,
            values,
        } => {
            let vals_str = values
                .iter()
                .map(|v| format!("{v:.2}"))
                .collect::<Vec<_>>()
                .join(", ");
            h_flex()
                .justify_between()
                .items_center()
                .text_xs()
                .pl(px(depth as f32 * 10.0))
                .py_0p5()
                .child(
                    div()
                        .text_color(cx.theme().foreground.opacity(0.85))
                        .child(name.to_string()),
                )
                .child(
                    div()
                        .font_bold()
                        .text_color(cx.theme().primary)
                        .child(format!("[{vals_str}]")),
                )
                .into_any_element()
        }
        FlatObsItem::Categorical {
            depth,
            name,
            class_id,
        } => h_flex()
            .justify_between()
            .items_center()
            .text_xs()
            .pl(px(depth as f32 * 10.0))
            .py_0p5()
            .child(
                div()
                    .text_color(cx.theme().foreground.opacity(0.85))
                    .child(name.to_string()),
            )
            .child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_sm()
                    .bg(cx.theme().accent.opacity(0.2))
                    .text_color(cx.theme().accent)
                    .font_bold()
                    .child(format!("ID: {class_id}")),
            )
            .into_any_element(),
        FlatObsItem::GroupHeader {
            depth,
            name,
            path,
            count_label,
            is_collapsed,
            is_repeated,
        } => {
            let path_for_click = path.clone();
            h_flex()
                .items_center()
                .justify_between()
                .w_full()
                .pl(px(depth as f32 * 10.0))
                .py_0p5()
                .child(
                    h_flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Button::new(format!("toggle-group-{}", path))
                                .icon(if is_collapsed {
                                    IconName::ChevronRight
                                } else {
                                    IconName::ChevronDown
                                })
                                .xsmall()
                                .ghost()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if this.visual_obs_collapsed.contains(&path_for_click) {
                                        this.visual_obs_collapsed.remove(&path_for_click);
                                    } else {
                                        this.visual_obs_collapsed.insert(path_for_click.clone());
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(if is_repeated {
                                    cx.theme().primary
                                } else {
                                    cx.theme().foreground
                                })
                                .child(name.to_string()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(count_label),
                )
                .into_any_element()
        }
    }
}

fn render_telemetry_obs_card(
    sidebar: &AppSidebar,
    f: &VisualObsFrame,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let tree = f.obs_tree.clone().or_else(|| {
        sidebar
            .visual_obs_schema
            .as_ref()
            .map(|s| s.decode_tree(&f.obs_vector))
    });

    div()
        .w_full()
        .max_h(px(520.0))
        .p_3()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(
            v_flex()
                .overflow_y_scrollbar()
                .gap_2()
                .child(
                    h_flex()
                        .justify_between()
                        .child(
                            div()
                                .font_bold()
                                .text_sm()
                                .child("结构化观测 AST (Obs Tree)"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_bold()
                                .text_color(cx.theme().primary)
                                .child(format!("{} 维", f.obs_vector.len())),
                        ),
                )
                .child(if let Some(nodes) = tree {
                    if nodes.is_empty() {
                        div()
                            .text_xs()
                            .text_color(cx.theme().foreground.opacity(0.75))
                            .child("暂无观测数据")
                            .into_any_element()
                    } else {
                        let mut flat_items = Vec::with_capacity(32);
                        flatten_obs_nodes(
                            &nodes,
                            "root",
                            0,
                            &sidebar.visual_obs_collapsed,
                            &mut flat_items,
                        );

                        v_flex()
                            .gap_1()
                            .overflow_y_scrollbar()
                            .children(
                                flat_items
                                    .into_iter()
                                    .map(|item| render_flat_obs_item(item, cx)),
                            )
                            .into_any_element()
                    }
                } else if f.obs_vector.is_empty() {
                    div()
                        .text_xs()
                        .text_color(cx.theme().foreground.opacity(0.75))
                        .child("暂无观测向量数据")
                        .into_any_element()
                } else {
                    v_flex()
                        .gap_1()
                        .overflow_y_scrollbar()
                        .children(f.obs_vector.iter().enumerate().map(|(i, value)| {
                            let label = f
                                .obs_labels
                                .get(i)
                                .cloned()
                                .unwrap_or_else(|| format!("dim {i}"));
                            h_flex()
                                .justify_between()
                                .text_xs()
                                .child(
                                    div()
                                        .text_color(cx.theme().foreground.opacity(0.85))
                                        .child(label),
                                )
                                .child(div().font_semibold().child(format!("{value:.3}")))
                        }))
                        .into_any_element()
                }),
        )
        .into_any_element()
}

enum FlatActionItem<'a> {
    Categorical {
        depth: usize,
        name: &'a str,
        num_classes: usize,
        labels: &'a [String],
    },
    Continuous {
        depth: usize,
        name: &'a str,
        dim: usize,
    },
    UnitSelection {
        depth: usize,
        name: &'a str,
        max_units: usize,
        obs_entity_name: &'a str,
    },
    GroupHeader {
        depth: usize,
        name: &'a str,
        path: String,
        count_label: String,
        is_collapsed: bool,
    },
}

fn flatten_action_nodes<'a>(
    nodes: &'a [ActionNode],
    path_prefix: &str,
    depth: usize,
    collapsed: &std::collections::HashSet<String>,
    out: &mut Vec<FlatActionItem<'a>>,
) {
    if depth > 10 {
        return;
    }
    for (i, node) in nodes.iter().enumerate() {
        let node_path = format!("{path_prefix}.{}_{}", node.name(), i);
        match node {
            ActionNode::Categorical {
                name,
                num_classes,
                labels,
            } => {
                out.push(FlatActionItem::Categorical {
                    depth,
                    name,
                    num_classes: *num_classes,
                    labels,
                });
            }
            ActionNode::Continuous { name, dim } => {
                out.push(FlatActionItem::Continuous {
                    depth,
                    name,
                    dim: *dim,
                });
            }
            ActionNode::UnitSelection {
                name,
                max_units,
                obs_entity_name,
                ..
            } => {
                out.push(FlatActionItem::UnitSelection {
                    depth,
                    name,
                    max_units: *max_units,
                    obs_entity_name,
                });
            }
            ActionNode::Struct { name, fields } => {
                let is_collapsed = collapsed.contains(&node_path);
                out.push(FlatActionItem::GroupHeader {
                    depth,
                    name,
                    path: node_path.clone(),
                    count_label: format!("{} 字段", fields.len()),
                    is_collapsed,
                });
                if !is_collapsed {
                    flatten_action_nodes(fields, &node_path, depth + 1, collapsed, out);
                }
            }
        }
    }
}

fn render_flat_action_item(item: FlatActionItem<'_>, cx: &Context<AppSidebar>) -> AnyElement {
    match item {
        FlatActionItem::Categorical {
            depth,
            name,
            num_classes,
            labels,
        } => {
            let label_preview = if labels.len() <= 3 {
                labels.join(", ")
            } else {
                format!(
                    "{}, ...共{}类",
                    labels[..2.min(labels.len())].join(", "),
                    num_classes
                )
            };
            h_flex()
                .justify_between()
                .items_center()
                .text_xs()
                .pl(px(depth as f32 * 10.0))
                .py_0p5()
                .child(
                    h_flex()
                        .gap_1p5()
                        .items_center()
                        .child(
                            div()
                                .px_1()
                                .py_0p5()
                                .rounded_sm()
                                .bg(cx.theme().accent.opacity(0.15))
                                .text_color(cx.theme().accent)
                                .font_bold()
                                .text_xs()
                                .child("离散"),
                        )
                        .child(
                            div()
                                .text_color(cx.theme().foreground.opacity(0.85))
                                .child(name.to_string()),
                        ),
                )
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{num_classes}类 ({label_preview})")),
                )
                .into_any_element()
        }
        FlatActionItem::Continuous { depth, name, dim } => h_flex()
            .justify_between()
            .items_center()
            .text_xs()
            .pl(px(depth as f32 * 10.0))
            .py_0p5()
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .child(
                        div()
                            .px_1()
                            .py_0p5()
                            .rounded_sm()
                            .bg(hsla(140.0 / 360.0, 0.75, 0.45, 0.15))
                            .text_color(cx.theme().success)
                            .font_bold()
                            .text_xs()
                            .child("连续"),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().foreground.opacity(0.85))
                            .child(name.to_string()),
                    ),
            )
            .child(
                div()
                    .font_bold()
                    .text_color(cx.theme().success)
                    .child(format!("{dim} 维高斯")),
            )
            .into_any_element(),
        FlatActionItem::UnitSelection {
            depth,
            name,
            max_units,
            obs_entity_name,
        } => h_flex()
            .justify_between()
            .items_center()
            .text_xs()
            .pl(px(depth as f32 * 10.0))
            .py_0p5()
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .child(
                        div()
                            .px_1()
                            .py_0p5()
                            .rounded_sm()
                            .bg(hsla(215.0 / 360.0, 0.85, 0.58, 0.15))
                            .text_color(hsla(215.0 / 360.0, 0.85, 0.58, 1.0))
                            .font_bold()
                            .text_xs()
                            .child("目标选择"),
                    )
                    .child(
                        div()
                            .text_color(cx.theme().foreground.opacity(0.85))
                            .child(name.to_string()),
                    ),
            )
            .child(
                div()
                    .text_color(cx.theme().primary)
                    .child(format!("{max_units} 槽位 ➔ {obs_entity_name}")),
            )
            .into_any_element(),
        FlatActionItem::GroupHeader {
            depth,
            name,
            path,
            count_label,
            is_collapsed,
        } => {
            let path_for_click = path.clone();
            h_flex()
                .items_center()
                .justify_between()
                .w_full()
                .pl(px(depth as f32 * 10.0))
                .py_0p5()
                .child(
                    h_flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Button::new(format!("toggle-act-group-{}", path))
                                .icon(if is_collapsed {
                                    IconName::ChevronRight
                                } else {
                                    IconName::ChevronDown
                                })
                                .xsmall()
                                .ghost()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if this.visual_action_collapsed.contains(&path_for_click) {
                                        this.visual_action_collapsed.remove(&path_for_click);
                                    } else {
                                        this.visual_action_collapsed.insert(path_for_click.clone());
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .px_1()
                                .py_0p5()
                                .rounded_sm()
                                .bg(cx.theme().secondary)
                                .text_color(cx.theme().foreground)
                                .font_bold()
                                .text_xs()
                                .child("结构体"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(cx.theme().foreground)
                                .child(name.to_string()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(count_label),
                )
                .into_any_element()
        }
    }
}

fn render_telemetry_action_schema_card(
    sidebar: &AppSidebar,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let schema = sidebar.visual_action_schema.as_ref();
    let enc_dim = schema.map(|s| s.encoding_dim()).unwrap_or(0);
    let num_branches = schema.map(|s| s.num_branches()).unwrap_or(0);

    div()
        .w_full()
        .max_h(px(400.0))
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
                                .child("结构化动作空间 AST (Action Schema)"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_bold()
                                .text_color(cx.theme().primary)
                                .child(format!("{enc_dim} 维编码 · {num_branches} 分支")),
                        ),
                )
                .child(if let Some(s) = schema {
                    let mut flat_items = Vec::with_capacity(16);
                    flatten_action_nodes(
                        &s.nodes,
                        "action_root",
                        0,
                        &sidebar.visual_action_collapsed,
                        &mut flat_items,
                    );
                    v_flex()
                        .gap_1()
                        .overflow_y_scrollbar()
                        .children(
                            flat_items
                                .into_iter()
                                .map(|item| render_flat_action_item(item, cx)),
                        )
                        .into_any_element()
                } else {
                    div()
                        .text_xs()
                        .text_color(cx.theme().foreground.opacity(0.75))
                        .child("暂无动作空间 Schema 元数据")
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
        PolicyDisplay::Structured(branches) => {
            if branches.is_empty() {
                div()
                    .text_xs()
                    .text_color(cx.theme().foreground.opacity(0.75))
                    .child("无结构化动作分布数据")
                    .into_any_element()
            } else {
                v_flex()
                    .gap_3()
                    .children(branches.iter().map(|b| render_action_branch_display(b, cx)))
                    .into_any_element()
            }
        }
    }
}

fn render_action_branch_display(
    branch: &ActionBranchDisplay,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    match branch {
        ActionBranchDisplay::Continuous {
            name,
            means,
            labels,
        } => v_flex()
            .gap_1p5()
            .child(
                div()
                    .text_xs()
                    .font_semibold()
                    .text_color(cx.theme().success)
                    .child(format!("连续控制分布 [{name}] (高斯均值)")),
            )
            .children(
                labels
                    .iter()
                    .zip(means.iter())
                    .map(|(lbl, &val)| policy_value_row(lbl, val)),
            )
            .into_any_element(),
        ActionBranchDisplay::Categorical { name, items } => v_flex()
            .gap_1p5()
            .child(
                div()
                    .text_xs()
                    .font_semibold()
                    .text_color(cx.theme().accent)
                    .child(format!("离散决策分布 [{name}]")),
            )
            .child(render_policy_items_table(items, cx))
            .into_any_element(),
        ActionBranchDisplay::UnitSelection {
            name,
            obs_entity_name,
            items,
        } => v_flex()
            .gap_1p5()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .font_semibold()
                            .text_color(hsla(215.0 / 360.0, 0.85, 0.58, 1.0))
                            .child(format!("目标注意力选择 [{name}]")),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("源: {obs_entity_name}")),
                    ),
            )
            .child(render_policy_items_table(items, cx))
            .into_any_element(),
        ActionBranchDisplay::Struct { name, fields } => v_flex()
            .gap_2()
            .p_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary.opacity(0.2))
            .child(
                div()
                    .text_xs()
                    .font_bold()
                    .text_color(cx.theme().foreground)
                    .child(format!("复合动作域: {name}")),
            )
            .children(fields.iter().map(|f| render_action_branch_display(f, cx)))
            .into_any_element(),
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

