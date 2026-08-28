use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::DialogFooter;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{
    h_flex, v_flex, ActiveTheme, IconName, Sizable as _, StyledExt, WindowExt as _,
};
use lol_rl_protocol::{
    ActionMaskRule, ActionNode, ActionSchema, EntityEncoderSpec, EnvDslSpec, ObsExpr, ObsNode,
    ObsSchema, RewardExpr, RewardFormulaSpec, RewardTermSpec,
};

use crate::components::rl::tasks_table::CreateTaskDialogView;
use crate::components::sidebar::AppSidebar;
use crate::types::ActiveView;

/// 渲染环境详情页面
pub fn render_env_detail(
    sidebar: &AppSidebar,
    env_name: String,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let spec = lol_rl_protocol::get_env_spec(&env_name);
    let dsl_spec = lol_rl_protocol::get_env_dsl_spec(&env_name);
    let dsl_source = lol_rl_protocol::get_env_dsl_source(&env_name);

    let Some(spec) = spec else {
        return v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("未找到环境 {} 的规范定义", env_name)),
            )
            .into_any_element();
    };

    let title_bar = render_env_detail_header(sidebar, spec, window, cx);
    let body_content = render_env_detail_body(sidebar, spec, dsl_spec, dsl_source, cx);

    v_flex()
        .size_full()
        .flex_1()
        .gap_4()
        .overflow_hidden()
        .child(title_bar)
        .child(
            div()
                .id("env-detail-main-scroll")
                .flex_1()
                .w_full()
                .min_h_0()
                .overflow_y_scrollbar()
                .child(body_content),
        )
        .into_any_element()
}

fn render_env_detail_header(
    sidebar: &AppSidebar,
    spec: &'static lol_rl_protocol::EnvSpec,
    _window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let env_name = spec.name.to_string();

    h_flex()
        .w_full()
        .items_center()
        .justify_between()
        .child(
            h_flex()
                .gap_3()
                .items_center()
                .child(
                    Button::new("back-to-rl-btn")
                        .ghost()
                        .icon(IconName::ChevronLeft)
                        .label("返回")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.navigate_to(ActiveView::RlTraining);
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .w(px(1.0))
                        .h(px(16.0))
                        .bg(cx.theme().border),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .font_bold()
                                .text_base()
                                .text_color(cx.theme().foreground)
                                .child(spec.label),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(cx.theme().accent.opacity(0.15))
                                .text_color(cx.theme().accent)
                                .font_bold()
                                .text_xs()
                                .child(spec.tag),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(cx.theme().muted.opacity(0.25))
                                .text_color(cx.theme().muted_foreground)
                                .text_xs()
                                .child(if spec.num_agents > 1 {
                                    "2P 自博弈"
                                } else {
                                    "单智能体"
                                }),
                        ),
                ),
        )
        .child(
            Button::new("start-train-from-detail-btn")
                .primary()
                .icon(IconName::Plus)
                .label("开始训练")
                .on_click({
                    let env_name = env_name.clone();
                    let task_count = sidebar.task_list.len() + 1;
                    cx.listener(move |_this, _, window, cx| {
                        let last_env = Some(env_name.clone());
                        let weak_sidebar = cx.entity().downgrade();
                        let dialog_view = cx.new(|cx| {
                            CreateTaskDialogView::new(
                                task_count,
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
                                                            let config = this.form.clone();
                                                            let cmd_str = config.to_cargo_run_command();
                                                            tracing::info!("🚀 [Client GUI] 启动训练任务: {}", cmd_str);
                                                            let (program, prefix_args) = lol_client::launch::resolve_executable("lol_rl", "lol_rl_cli");
                                                            let cli_args = config.to_cli_args();
                                                            let full_args: Vec<String> = prefix_args.into_iter().chain(cli_args.into_iter()).collect();

                                                            crate::services::runtime::tokio_runtime().spawn(async move {
                                                                let mut cmd = tokio::process::Command::new(&program);
                                                                cmd.args(&full_args)
                                                                    .stdout(std::process::Stdio::inherit())
                                                                    .stderr(std::process::Stdio::inherit());
                                                                if let Some(root) = lol_client::launch::install_root() {
                                                                    cmd.current_dir(root);
                                                                }
                                                                let _ = cmd.spawn();
                                                            });
                                                        });
                                                        window.close_dialog(cx);
                                                    }
                                                }),
                                        ),
                                )
                        });
                    })
                }),
        )
        .into_any_element()
}

fn render_env_detail_body(
    sidebar: &AppSidebar,
    spec: &'static lol_rl_protocol::EnvSpec,
    dsl_spec: Option<&'static EnvDslSpec>,
    dsl_source: Option<&'static str>,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let obs_schema = dsl_spec.and_then(|s| s.obs_schema.as_ref());
    let action_schema = dsl_spec.and_then(|s| s.action_schema.as_ref());
    let reward_formula = dsl_spec.and_then(|s| s.reward_formula.as_ref());

    v_flex()
        .w_full()
        .gap_4()
        .pb_8()
        // ── 顶部环境概览与参数芯片条 ──
        .child(render_env_overview_card(spec, cx))
        // ── 主体内容分栏：左栏 (仅观测空间 Obs AST)，右栏 (动作空间 + 奖励公式 + DSL 脚本) ──
        .child(
            h_flex()
                .items_start()
                .gap_4()
                .w_full()
                .child(
                    v_flex()
                        .flex_1()
                        .gap_4()
                        .child(render_obs_schema_card(sidebar, obs_schema, cx)),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .gap_4()
                        .child(render_action_schema_card(sidebar, action_schema, cx))
                        .child(render_reward_formula_card(reward_formula, cx))
                        .child(render_dsl_source_card(sidebar, dsl_source, cx)),
                ),
        )
        .into_any_element()
}

fn render_env_overview_card(
    spec: &'static lol_rl_protocol::EnvSpec,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let p = &spec.default_params;
    v_flex()
        .w_full()
        .p_3p5()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .gap_2p5()
        .child(
            h_flex().items_center().justify_between().child(
                div()
                    .text_xs()
                    .text_color(cx.theme().foreground)
                    .child(spec.description),
            ),
        )
        .child(
            h_flex()
                .items_center()
                .gap_3()
                .flex_wrap()
                .child(render_param_chip(
                    "学习率 (LR)",
                    format!("{:.0e}", p.lr),
                    cx,
                ))
                .child(render_param_chip(
                    "折扣因子 (Gamma)",
                    format!("{:.2}", p.gamma),
                    cx,
                ))
                .child(render_param_chip(
                    "GAE Lambda",
                    format!("{:.2}", p.gae_lambda),
                    cx,
                ))
                .child(render_param_chip(
                    "PPO Clip Eps",
                    format!("{:.2}", p.clip_eps),
                    cx,
                ))
                .child(render_param_chip("Epochs", p.ppo_epochs.to_string(), cx))
                .child(render_param_chip(
                    "隐藏层维度",
                    format!("{}维", p.hidden_dim),
                    cx,
                ))
                .child(render_param_chip(
                    "每轮步数",
                    format!("{}步", p.rollout_steps_per_env),
                    cx,
                ))
                .child(render_param_chip(
                    "推荐迭代",
                    format!("{}轮", p.total_iterations),
                    cx,
                )),
        )
        .into_any_element()
}

fn render_param_chip(label: &str, value: String, cx: &Context<AppSidebar>) -> AnyElement {
    h_flex()
        .items_center()
        .gap_1p5()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(cx.theme().secondary.opacity(0.4))
        .text_xs()
        .child(
            div()
                .text_color(cx.theme().muted_foreground)
                .child(label.to_string()),
        )
        .child(
            div()
                .font_bold()
                .text_color(cx.theme().foreground)
                .child(value),
        )
        .into_any_element()
}

// ── 1. 结构化观测空间 AST 渲染 ──────────────────────────────────────────────

enum FlatObsSchemaItem<'a> {
    Categorical {
        depth: usize,
        name: &'a str,
        num_classes: usize,
        embed_dim: usize,
        expr_str: String,
    },
    Scalar {
        depth: usize,
        name: &'a str,
        expr_str: String,
    },
    Vector {
        depth: usize,
        name: &'a str,
        dim: usize,
        expr_str: String,
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

fn flatten_obs_schema_nodes<'a>(
    nodes: &'a [ObsNode],
    path_prefix: &str,
    depth: usize,
    collapsed: &std::collections::HashSet<String>,
    out: &mut Vec<FlatObsSchemaItem<'a>>,
) {
    if depth > 8 {
        return;
    }
    for (i, node) in nodes.iter().enumerate() {
        let node_path = format!("{path_prefix}.{}_{}", node.name(), i);
        match node {
            ObsNode::Categorical {
                name,
                num_classes,
                embed_dim,
                expr,
            } => {
                let expr_str = expr
                    .as_ref()
                    .map(expr_to_code)
                    .unwrap_or_else(|| name.clone());
                out.push(FlatObsSchemaItem::Categorical {
                    depth,
                    name,
                    num_classes: *num_classes,
                    embed_dim: *embed_dim,
                    expr_str,
                });
            }
            ObsNode::Scalar { name, expr, .. } => {
                let expr_str = expr
                    .as_ref()
                    .map(expr_to_code)
                    .unwrap_or_else(|| name.clone());
                out.push(FlatObsSchemaItem::Scalar {
                    depth,
                    name,
                    expr_str,
                });
            }
            ObsNode::Vector { name, dim, exprs } => {
                let expr_str = format!(
                    "[{}]",
                    exprs
                        .iter()
                        .map(expr_to_code)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                out.push(FlatObsSchemaItem::Vector {
                    depth,
                    name,
                    dim: *dim,
                    expr_str,
                });
            }
            ObsNode::Struct { name, fields } => {
                let is_collapsed = collapsed.contains(&node_path);
                out.push(FlatObsSchemaItem::GroupHeader {
                    depth,
                    name,
                    path: node_path.clone(),
                    count_label: format!("{} 字段", fields.len()),
                    is_collapsed,
                    is_repeated: false,
                });
                if !is_collapsed {
                    flatten_obs_schema_nodes(fields, &node_path, depth + 1, collapsed, out);
                }
            }
            ObsNode::Repeated {
                name,
                max_count,
                item,
                encoder,
            } => {
                let is_collapsed = collapsed.contains(&node_path);
                let encoder_desc = match encoder {
                    EntityEncoderSpec::SharedMlpFlatten { hidden_dims } => {
                        format!("SharedMlpFlatten(hidden={:?})", hidden_dims)
                    }
                    EntityEncoderSpec::SharedMlpPool {
                        hidden_dims,
                        pool_type,
                    } => {
                        format!(
                            "SharedMlpPool(hidden={:?}, pool={:?})",
                            hidden_dims, pool_type
                        )
                    }
                    EntityEncoderSpec::PassThrough => "PassThrough".to_string(),
                };
                out.push(FlatObsSchemaItem::GroupHeader {
                    depth,
                    name,
                    path: node_path.clone(),
                    count_label: format!("repeated[{max_count}] -> {encoder_desc}"),
                    is_collapsed,
                    is_repeated: true,
                });
                if !is_collapsed {
                    flatten_obs_schema_nodes(
                        std::slice::from_ref(item.as_ref()),
                        &node_path,
                        depth + 1,
                        collapsed,
                        out,
                    );
                }
            }
        }
    }
}

fn render_flat_obs_schema_item(
    item: FlatObsSchemaItem<'_>,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let pl_px = (item_depth(&item) as f32) * 12.0;

    match item {
        FlatObsSchemaItem::Categorical {
            name,
            num_classes,
            embed_dim,
            expr_str,
            ..
        } => h_flex()
            .justify_between()
            .items_center()
            .text_xs()
            .pl(px(pl_px))
            .py_1()
            .border_b_1()
            .border_color(cx.theme().border.opacity(0.3))
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
                            .child("分类类别"),
                    )
                    .child(div().font_medium().child(name.to_string()))
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("= {}", expr_str)),
                    ),
            )
            .child(
                div()
                    .font_semibold()
                    .text_color(cx.theme().accent)
                    .child(format!("{}类 -> embed({}维)", num_classes, embed_dim)),
            )
            .into_any_element(),

        FlatObsSchemaItem::Scalar { name, expr_str, .. } => h_flex()
            .justify_between()
            .items_center()
            .text_xs()
            .pl(px(pl_px))
            .py_1()
            .border_b_1()
            .border_color(cx.theme().border.opacity(0.3))
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .child(
                        div()
                            .px_1()
                            .py_0p5()
                            .rounded_sm()
                            .bg(cx.theme().muted.opacity(0.3))
                            .text_color(cx.theme().muted_foreground)
                            .child("标量"),
                    )
                    .child(div().font_medium().child(name.to_string()))
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("= {}", expr_str)),
                    ),
            )
            .child(div().text_color(cx.theme().muted_foreground).child("1 维"))
            .into_any_element(),

        FlatObsSchemaItem::Vector {
            name,
            dim,
            expr_str,
            ..
        } => h_flex()
            .justify_between()
            .items_center()
            .text_xs()
            .pl(px(pl_px))
            .py_1()
            .border_b_1()
            .border_color(cx.theme().border.opacity(0.3))
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .child(
                        div()
                            .px_1()
                            .py_0p5()
                            .rounded_sm()
                            .bg(cx.theme().primary.opacity(0.15))
                            .text_color(cx.theme().primary)
                            .child("连续向量"),
                    )
                    .child(div().font_medium().child(name.to_string()))
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("= {}", expr_str)),
                    ),
            )
            .child(
                div()
                    .font_semibold()
                    .text_color(cx.theme().primary)
                    .child(format!("{} 维", dim)),
            )
            .into_any_element(),

        FlatObsSchemaItem::GroupHeader {
            name,
            path,
            count_label,
            is_collapsed,
            is_repeated,
            ..
        } => {
            let path_for_click = path.clone();
            h_flex()
                .justify_between()
                .items_center()
                .pl(px(pl_px))
                .py_1()
                .border_b_1()
                .border_color(cx.theme().border.opacity(0.3))
                .child(
                    h_flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Button::new(SharedString::from(format!("toggle-obs-hdr-{}", path)))
                                .icon(if is_collapsed {
                                    IconName::ChevronRight
                                } else {
                                    IconName::ChevronDown
                                })
                                .xsmall()
                                .ghost()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if this.env_detail_obs_collapsed.contains(&path_for_click) {
                                        this.env_detail_obs_collapsed.remove(&path_for_click);
                                    } else {
                                        this.env_detail_obs_collapsed
                                            .insert(path_for_click.clone());
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .px_1()
                                .py_0p5()
                                .rounded_sm()
                                .bg(if is_repeated {
                                    cx.theme().primary.opacity(0.15)
                                } else {
                                    cx.theme().secondary
                                })
                                .text_color(if is_repeated {
                                    cx.theme().primary
                                } else {
                                    cx.theme().foreground
                                })
                                .font_bold()
                                .text_xs()
                                .child(if is_repeated {
                                    "重复实体"
                                } else {
                                    "结构体"
                                }),
                        )
                        .child(
                            div()
                                .font_bold()
                                .text_xs()
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

fn item_depth(item: &FlatObsSchemaItem<'_>) -> usize {
    match item {
        FlatObsSchemaItem::Categorical { depth, .. } => *depth,
        FlatObsSchemaItem::Scalar { depth, .. } => *depth,
        FlatObsSchemaItem::Vector { depth, .. } => *depth,
        FlatObsSchemaItem::GroupHeader { depth, .. } => *depth,
    }
}

fn render_obs_schema_card(
    sidebar: &AppSidebar,
    schema: Option<&ObsSchema>,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let raw_dim = schema.map(|s| s.raw_dim()).unwrap_or(0);
    let nodes_count = schema.map(|s| s.nodes.len()).unwrap_or(0);

    v_flex()
        .w_full()
        .p_3p5()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .gap_2p5()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::Eye)
                        .child(div().font_bold().text_sm().child("观测空间")),
                )
                .child(
                    div()
                        .text_xs()
                        .font_bold()
                        .text_color(cx.theme().primary)
                        .child(format!("{} 维原始输入 · {} 顶层节点", raw_dim, nodes_count)),
                ),
        )
        .child(if let Some(s) = schema {
            let mut flat_items = Vec::with_capacity(32);
            flatten_obs_schema_nodes(
                &s.nodes,
                "obs_root",
                0,
                &sidebar.env_detail_obs_collapsed,
                &mut flat_items,
            );
            v_flex()
                .gap_1()
                .children(
                    flat_items
                        .into_iter()
                        .map(|item| render_flat_obs_schema_item(item, cx)),
                )
                .into_any_element()
        } else {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("暂无观测空间 AST 数据")
                .into_any_element()
        })
        .into_any_element()
}

fn expr_to_code(expr: &ObsExpr) -> String {
    match expr {
        ObsExpr::Constant(v) => format!("{v:.2}"),
        ObsExpr::Variable(var) => var.clone(),
        ObsExpr::Add(l, r) => format!("({} + {})", expr_to_code(l), expr_to_code(r)),
        ObsExpr::Sub(l, r) => format!("({} - {})", expr_to_code(l), expr_to_code(r)),
        ObsExpr::Mul(l, r) => format!("({} * {})", expr_to_code(l), expr_to_code(r)),
        ObsExpr::Div(l, r) => format!("({} / {})", expr_to_code(l), expr_to_code(r)),
        ObsExpr::Clamp { expr, min, max } => {
            format!("clamp({}, {min:.1}, {max:.1})", expr_to_code(expr))
        }
        ObsExpr::Max(l, r) => format!("max({}, {})", expr_to_code(l), expr_to_code(r)),
        ObsExpr::Min(l, r) => format!("min({}, {})", expr_to_code(l), expr_to_code(r)),
        ObsExpr::IfElse {
            cond,
            then_branch,
            else_branch,
        } => format!(
            "if({}, {}, {})",
            expr_to_code(cond),
            expr_to_code(then_branch),
            expr_to_code(else_branch)
        ),
        ObsExpr::Gt(l, r) => format!("({} > {})", expr_to_code(l), expr_to_code(r)),
        ObsExpr::Lt(l, r) => format!("({} < {})", expr_to_code(l), expr_to_code(r)),
    }
}

// ── 2. 结构化动作空间 AST 渲染 ──────────────────────────────────────────────

enum FlatActionSchemaItem<'a> {
    Continuous {
        depth: usize,
        name: &'a str,
        dim: usize,
    },
    Categorical {
        depth: usize,
        name: &'a str,
        num_classes: usize,
        labels: &'a [String],
    },
    UnitSelection {
        depth: usize,
        name: &'a str,
        max_units: usize,
        unit_embed_dim: usize,
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

fn flatten_action_schema_nodes<'a>(
    nodes: &'a [ActionNode],
    path_prefix: &str,
    depth: usize,
    collapsed: &std::collections::HashSet<String>,
    out: &mut Vec<FlatActionSchemaItem<'a>>,
) {
    if depth > 8 {
        return;
    }
    for (i, node) in nodes.iter().enumerate() {
        let node_path = format!("{path_prefix}.{}_{}", node.name(), i);
        match node {
            ActionNode::Continuous { name, dim } => {
                out.push(FlatActionSchemaItem::Continuous {
                    depth,
                    name,
                    dim: *dim,
                });
            }
            ActionNode::Categorical {
                name,
                num_classes,
                labels,
            } => {
                out.push(FlatActionSchemaItem::Categorical {
                    depth,
                    name,
                    num_classes: *num_classes,
                    labels,
                });
            }
            ActionNode::UnitSelection {
                name,
                max_units,
                unit_embed_dim,
                obs_entity_name,
            } => {
                out.push(FlatActionSchemaItem::UnitSelection {
                    depth,
                    name,
                    max_units: *max_units,
                    unit_embed_dim: *unit_embed_dim,
                    obs_entity_name,
                });
            }
            ActionNode::Struct { name, fields } => {
                let is_collapsed = collapsed.contains(&node_path);
                out.push(FlatActionSchemaItem::GroupHeader {
                    depth,
                    name,
                    path: node_path.clone(),
                    count_label: format!("{} 字段", fields.len()),
                    is_collapsed,
                });
                if !is_collapsed {
                    flatten_action_schema_nodes(fields, &node_path, depth + 1, collapsed, out);
                }
            }
        }
    }
}

fn render_flat_action_schema_item(
    item: FlatActionSchemaItem<'_>,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let pl_px = (action_item_depth(&item) as f32) * 12.0;

    match item {
        FlatActionSchemaItem::Continuous {
            depth: _,
            name,
            dim,
        } => h_flex()
            .justify_between()
            .items_center()
            .text_xs()
            .pl(px(pl_px))
            .py_1()
            .border_b_1()
            .border_color(cx.theme().border.opacity(0.3))
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .child(
                        div()
                            .px_1()
                            .py_0p5()
                            .rounded_sm()
                            .bg(cx.theme().success.opacity(0.15))
                            .text_color(cx.theme().success)
                            .font_bold()
                            .child("连续分布"),
                    )
                    .child(div().font_medium().child(name.to_string())),
            )
            .child(
                div()
                    .font_semibold()
                    .text_color(cx.theme().success)
                    .child(format!("{} 维高斯控制", dim)),
            )
            .into_any_element(),

        FlatActionSchemaItem::Categorical {
            depth: _,
            name,
            num_classes,
            labels,
        } => v_flex()
            .gap_1()
            .pl(px(pl_px))
            .py_1()
            .border_b_1()
            .border_color(cx.theme().border.opacity(0.3))
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .text_xs()
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
                                    .child("离散决策"),
                            )
                            .child(div().font_medium().child(name.to_string())),
                    )
                    .child(
                        div()
                            .font_semibold()
                            .text_color(cx.theme().accent)
                            .child(format!("{} 类别", num_classes)),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        labels
                            .iter()
                            .enumerate()
                            .map(|(i, l)| format!("{}: {}", i, l))
                            .collect::<Vec<_>>()
                            .join(" · "),
                    ),
            )
            .into_any_element(),

        FlatActionSchemaItem::UnitSelection {
            depth: _,
            name,
            max_units,
            unit_embed_dim,
            obs_entity_name,
        } => h_flex()
            .justify_between()
            .items_center()
            .text_xs()
            .pl(px(pl_px))
            .py_1()
            .border_b_1()
            .border_color(cx.theme().border.opacity(0.3))
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
                            .child("实体选择"),
                    )
                    .child(div().font_medium().child(name.to_string())),
            )
            .child(
                div()
                    .font_semibold()
                    .text_color(hsla(215.0 / 360.0, 0.85, 0.58, 1.0))
                    .child(format!(
                        "{} 实体 (源: {}, 嵌入: {}维)",
                        max_units, obs_entity_name, unit_embed_dim
                    )),
            )
            .into_any_element(),

        FlatActionSchemaItem::GroupHeader {
            depth: _,
            name,
            path,
            count_label,
            is_collapsed,
        } => {
            let path_for_click = path.clone();
            h_flex()
                .justify_between()
                .items_center()
                .pl(px(pl_px))
                .py_1()
                .border_b_1()
                .border_color(cx.theme().border.opacity(0.3))
                .child(
                    h_flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Button::new(SharedString::from(format!("toggle-act-hdr-{}", path)))
                                .icon(if is_collapsed {
                                    IconName::ChevronRight
                                } else {
                                    IconName::ChevronDown
                                })
                                .xsmall()
                                .ghost()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if this.env_detail_action_collapsed.contains(&path_for_click) {
                                        this.env_detail_action_collapsed.remove(&path_for_click);
                                    } else {
                                        this.env_detail_action_collapsed
                                            .insert(path_for_click.clone());
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
                                .child("动作组"),
                        )
                        .child(
                            div()
                                .font_bold()
                                .text_xs()
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

fn action_item_depth(item: &FlatActionSchemaItem<'_>) -> usize {
    match item {
        FlatActionSchemaItem::Continuous { depth, .. } => *depth,
        FlatActionSchemaItem::Categorical { depth, .. } => *depth,
        FlatActionSchemaItem::UnitSelection { depth, .. } => *depth,
        FlatActionSchemaItem::GroupHeader { depth, .. } => *depth,
    }
}

fn render_action_schema_card(
    sidebar: &AppSidebar,
    schema: Option<&ActionSchema>,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let enc_dim = schema.map(|s| s.encoding_dim()).unwrap_or(0);
    let num_branches = schema.map(|s| s.num_branches()).unwrap_or(0);

    v_flex()
        .w_full()
        .p_3p5()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .gap_2p5()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::Settings2)
                        .child(div().font_bold().text_sm().child("动作空间")),
                )
                .child(
                    div()
                        .text_xs()
                        .font_bold()
                        .text_color(cx.theme().primary)
                        .child(format!("{} 维编码 · {} 分支", enc_dim, num_branches)),
                ),
        )
        .child(if let Some(s) = schema {
            let mut flat_items = Vec::with_capacity(16);
            flatten_action_schema_nodes(
                &s.nodes,
                "action_root",
                0,
                &sidebar.env_detail_action_collapsed,
                &mut flat_items,
            );
            v_flex()
                .gap_2()
                .children(
                    flat_items
                        .into_iter()
                        .map(|item| render_flat_action_schema_item(item, cx)),
                )
                .when(!s.mask_rules.is_empty(), |this| {
                    this.child(
                        v_flex()
                            .gap_1p5()
                            .pt_2()
                            .border_t_1()
                            .border_color(cx.theme().border.opacity(0.5))
                            .child(
                                div()
                                    .font_semibold()
                                    .text_xs()
                                    .text_color(cx.theme().foreground)
                                    .child("动作掩码规则 (Action Mask Rules)"),
                            )
                            .children(
                                s.mask_rules
                                    .iter()
                                    .map(|rule| render_mask_rule_item(rule, cx)),
                            ),
                    )
                })
                .into_any_element()
        } else {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("暂无动作空间 AST 数据")
                .into_any_element()
        })
        .into_any_element()
}

fn render_mask_rule_item(rule: &ActionMaskRule, cx: &Context<AppSidebar>) -> AnyElement {
    h_flex()
        .justify_between()
        .items_center()
        .p_1p5()
        .rounded_md()
        .bg(cx.theme().secondary.opacity(0.3))
        .text_xs()
        .child(
            h_flex()
                .gap_1p5()
                .items_center()
                .child(
                    div()
                        .px_1()
                        .py_0p5()
                        .rounded_sm()
                        .bg(cx.theme().danger.opacity(0.15))
                        .text_color(cx.theme().danger)
                        .child("屏蔽条件"),
                )
                .child(
                    div()
                        .text_color(cx.theme().foreground)
                        .child(format!("if {}", expr_to_code(&rule.condition))),
                ),
        )
        .child(
            div()
                .font_semibold()
                .text_color(cx.theme().danger)
                .child(format!("disable {}", rule.branch_label)),
        )
        .into_any_element()
}

// ── 3. 结构化奖励公式 AST 与 LaTeX 数学公式渲染 ─────────────────────────────

fn render_reward_formula_card(
    formula: Option<&RewardFormulaSpec>,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let terms_count = formula.map(|f| f.terms.len()).unwrap_or(0);

    v_flex()
        .w_full()
        .p_3p5()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .gap_2p5()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::Heart)
                        .child(div().font_bold().text_sm().child("奖励公式")),
                )
                .child(
                    div()
                        .text_xs()
                        .font_bold()
                        .text_color(cx.theme().primary)
                        .child(format!("{} 奖励项", terms_count)),
                ),
        )
        .child(if let Some(f) = formula {
            v_flex()
                .gap_3()
                // 顶部数学公式概要栏
                .child(
                    v_flex()
                        .gap_1()
                        .p_2p5()
                        .rounded_md()
                        .bg(cx.theme().secondary.opacity(0.25))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("总收益计算公式"),
                        )
                        .child(
                            div()
                                .font_bold()
                                .text_xs()
                                .text_color(cx.theme().primary)
                                .child(format!(
                                    "R = {}",
                                    f.terms
                                        .iter()
                                        .map(|t| reward_expr_to_code(&t.expr))
                                        .collect::<Vec<_>>()
                                        .join(" + ")
                                )),
                        ),
                )
                // 逐项拆解
                .child(
                    v_flex()
                        .gap_1p5()
                        .child(
                            div()
                                .font_semibold()
                                .text_xs()
                                .text_color(cx.theme().foreground)
                                .child("单项收益权重拆解 (Reward Terms)"),
                        )
                        .children(f.terms.iter().map(|term| render_reward_term_item(term, cx))),
                )
                .into_any_element()
        } else {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("暂无奖励公式 AST 数据")
                .into_any_element()
        })
        .into_any_element()
}

fn render_reward_term_item(term: &RewardTermSpec, cx: &Context<AppSidebar>) -> AnyElement {
    h_flex()
        .justify_between()
        .items_center()
        .p_1p5()
        .rounded_md()
        .bg(cx.theme().secondary.opacity(0.3))
        .text_xs()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .font_bold()
                        .text_color(cx.theme().foreground)
                        .child(term.label.clone()),
                )
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("({})", term.id)),
                ),
        )
        .child(
            div()
                .font_semibold()
                .text_xs()
                .text_color(cx.theme().primary)
                .child(reward_expr_to_code(&term.expr)),
        )
        .into_any_element()
}

fn reward_expr_to_code(expr: &RewardExpr) -> String {
    match expr {
        RewardExpr::Constant(v) => format!("{v:+}"),
        RewardExpr::Variable(var) => var.clone(),
        RewardExpr::Add(l, r) => {
            format!("({} + {})", reward_expr_to_code(l), reward_expr_to_code(r))
        }
        RewardExpr::Sub(l, r) => {
            format!("({} - {})", reward_expr_to_code(l), reward_expr_to_code(r))
        }
        RewardExpr::Mul(l, r) => {
            format!("({} * {})", reward_expr_to_code(l), reward_expr_to_code(r))
        }
        RewardExpr::Div(l, r) => {
            format!("({} / {})", reward_expr_to_code(l), reward_expr_to_code(r))
        }
        RewardExpr::Exp(val) => format!("exp({})", reward_expr_to_code(val)),
        RewardExpr::IfElse {
            cond,
            then_branch,
            else_branch,
        } => format!(
            "if({}, {}, {})",
            reward_expr_to_code(cond),
            reward_expr_to_code(then_branch),
            reward_expr_to_code(else_branch)
        ),
        RewardExpr::Gt(l, r) => {
            format!("({} > {})", reward_expr_to_code(l), reward_expr_to_code(r))
        }
        RewardExpr::Lt(l, r) => {
            format!("({} < {})", reward_expr_to_code(l), reward_expr_to_code(r))
        }
        RewardExpr::Max(l, r) => format!(
            "max({}, {})",
            reward_expr_to_code(l),
            reward_expr_to_code(r)
        ),
        RewardExpr::Min(l, r) => format!(
            "min({}, {})",
            reward_expr_to_code(l),
            reward_expr_to_code(r)
        ),
    }
}

// ── 4. 声明式 DSL 规范源码 ──────────────────────────────────────────────────

fn render_dsl_source_card(
    sidebar: &AppSidebar,
    source: Option<&'static str>,
    cx: &Context<AppSidebar>,
) -> AnyElement {
    let copied = sidebar.env_detail_copied;

    v_flex()
        .w_full()
        .p_3p5()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .gap_2p5()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    h_flex().gap_2().items_center().child(IconName::File).child(
                        div()
                            .font_bold()
                            .text_sm()
                            .child("声明式 DSL 规范脚本 (.rl)"),
                    ),
                )
                .child(
                    Button::new("copy-dsl-code-btn")
                        .outline()
                        .xsmall()
                        .label(if copied {
                            "已复制 DSL"
                        } else {
                            "复制代码"
                        })
                        .on_click({
                            let code = source.unwrap_or("").to_string();
                            cx.listener(move |this, _, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(code.clone()));
                                this.env_detail_copied = true;
                                cx.notify();
                            })
                        }),
                ),
        )
        .child(if let Some(src) = source {
            div()
                .id("dsl-source-code-scroll")
                .p_2p5()
                .rounded_md()
                .bg(cx.theme().secondary.opacity(0.35))
                .border_1()
                .border_color(cx.theme().border.opacity(0.5))
                .max_h(px(320.0))
                .min_h_0()
                .overflow_y_scrollbar()
                .child(v_flex().gap_0p5().children(src.lines().map(|line| {
                    div()
                        .text_xs()
                        .text_color(cx.theme().foreground.opacity(0.9))
                        .child(if line.is_empty() { " " } else { line })
                })))
                .into_any_element()
        } else {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("暂无 DSL 源码文本")
                .into_any_element()
        })
        .into_any_element()
}
