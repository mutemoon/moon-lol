use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use lol_rl_protocol::{EnvSpec, AVAILABLE_ENVS};

use crate::components::sidebar::AppSidebar;
use crate::types::ActiveView;

/// 渲染环境卡片概览区
pub fn render_env_cards(
    sidebar: &AppSidebar,
    _window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    v_flex()
        .w_full()
        .gap_2()
        .child(
            h_flex().justify_between().items_center().child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::LayoutDashboard)
                    .child(
                        div()
                            .font_bold()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child("强化学习环境"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("（点击卡片查看 AST 规范与详情）"),
                    ),
            ),
        )
        .child(
            h_flex().w_full().gap_3().children(
                AVAILABLE_ENVS
                    .iter()
                    .map(|spec| render_single_env_card(spec, sidebar, cx)),
            ),
        )
        .into_any_element()
}

fn render_single_env_card(
    spec: &'static EnvSpec,
    sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let env_name = spec.name.to_string();
    let is_selected = sidebar.selected_env_name.as_deref() == Some(spec.name);

    let (obs_summary, act_summary) = match spec.name {
        lol_rl_protocol::ENV_SOLO_V0 => ("108 维 · 树状 AST", "混合控制 · 3 分支"),
        lol_rl_protocol::ENV_FIORA_V3 => ("108 维 · 树状 AST", "补刀训练 · 3 分支"),
        lol_rl_protocol::ENV_FIORA_V2 => ("48 维 · 树状 AST", "连续 + 7 离散动作 (带掩码)"),
        lol_rl_protocol::ENV_FIORA_V1 => ("7 维 · 连续移动", "连续 + 2 离散动作"),
        lol_rl_protocol::ENV_FIORA_V0 => ("7 维 · 瞬移微操", "5 离散动作 (带掩码)"),
        _ => ("结构化 AST", "控制空间"),
    };

    let mode_label = if spec.num_agents > 1 {
        "2P 自博弈"
    } else {
        "单智能体"
    };

    v_flex()
        .id(SharedString::from(format!("env-card-{}", spec.name)))
        .flex_1()
        .min_w(px(200.0))
        .p_3()
        .rounded_lg()
        .border_1()
        .border_color(if is_selected {
            cx.theme().accent
        } else {
            cx.theme().border
        })
        .bg(if is_selected {
            cx.theme().accent.opacity(0.06)
        } else {
            cx.theme().background
        })
        .cursor_pointer()
        .gap_2()
        .on_click({
            let env_name = env_name.clone();
            cx.listener(move |this, _, _, cx| {
                this.selected_env_name = Some(env_name.clone());
                this.navigate_to(ActiveView::RlEnvDetail);
                cx.notify();
            })
        })
        // 顶部行：Tag + 模式徽章
        .child(
            h_flex().justify_between().items_center().child(
                h_flex()
                    .gap_1p5()
                    .items_center()
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
                            .child(mode_label),
                    ),
            ),
        )
        // 环境标题
        .child(
            div()
                .font_bold()
                .text_sm()
                .text_color(cx.theme().foreground)
                .child(spec.label),
        )
        // 简要描述
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .h(px(32.0))
                .overflow_hidden()
                .child(spec.description),
        )
        // 空间维度属性条
        .child(
            v_flex()
                .gap_1()
                .p_2()
                .rounded_md()
                .bg(cx.theme().secondary.opacity(0.35))
                .child(
                    h_flex()
                        .justify_between()
                        .text_xs()
                        .child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .child("观测空间"),
                        )
                        .child(
                            div()
                                .font_medium()
                                .text_color(cx.theme().foreground)
                                .child(obs_summary),
                        ),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .text_xs()
                        .child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .child("动作空间"),
                        )
                        .child(
                            div()
                                .font_medium()
                                .text_color(cx.theme().foreground)
                                .child(act_summary),
                        ),
                ),
        )
        // 底部动作链接
        .child(
            h_flex().justify_between().items_center().pt_1().child(
                h_flex()
                    .items_center()
                    .gap_1()
                    .text_xs()
                    .text_color(cx.theme().accent)
                    .child("查看环境规范 (AST / DSL)")
                    .child(IconName::ChevronRight),
            ),
        )
        .into_any_element()
}
