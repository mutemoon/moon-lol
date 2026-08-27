use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, StyledExt};
use tokio::sync::mpsc;

use crate::components::sidebar::AppSidebar;
use crate::services::assets_path::resolve_assets_dir;
use crate::services::extractor_service::{run_extraction_task, ExtractionConfig, ExtractionStep};
use crate::services::tool_checker_service::{
    run_environment_health_check, validate_before_extraction, EnvironmentHealthReport,
    ToolCategory, ToolCheckItem, ToolHealthStatus,
};

#[derive(Clone, Copy)]
pub struct ExtractorTheme {
    pub bg: Hsla,
    pub border: Hsla,
    pub accent: Hsla,
    pub muted_fg: Hsla,
    pub muted: Hsla,
    pub danger: Hsla,
    pub success: Hsla,
    pub warning: Hsla,
    pub primary: Hsla,
    pub primary_fg: Hsla,
    pub sidebar: Hsla,
    pub fg: Hsla,
}

impl ExtractorTheme {
    pub fn from_cx(cx: &Context<AppSidebar>) -> Self {
        let t = cx.theme();
        Self {
            bg: t.background,
            border: t.border,
            accent: t.accent,
            muted_fg: t.muted_foreground,
            muted: t.muted,
            danger: t.danger,
            success: t.success,
            warning: t.warning,
            primary: t.primary,
            primary_fg: t.primary_foreground,
            sidebar: t.sidebar,
            fg: t.foreground,
        }
    }
}

#[derive(Clone)]
pub struct StepInfo {
    pub title: String,
    pub description: String,
}

pub struct ExtractorPageState {
    pub game_path_input: Option<Entity<InputState>>,
    pub game_path: String,
    pub extract_base_and_ui: bool,
    pub extract_shaders: bool,
    pub extract_audio: bool,
    pub skip_map_geo: bool,
    pub is_extracting: bool,
    pub current_step_index: usize,
    pub current_step_status: String,
    pub step_logs: HashMap<usize, Vec<String>>,
    pub expanded_steps: HashMap<usize, bool>,
    pub status_message: Option<String>,
    // 环境体检状态
    pub health_report: Option<EnvironmentHealthReport>,
    pub is_checking_health: bool,
    pub health_panel_expanded: bool,
}

impl Default for ExtractorPageState {
    fn default() -> Self {
        let mut expanded = HashMap::new();
        expanded.insert(0, true);
        Self {
            game_path_input: None,
            game_path: r"D:\WeGameApps\英雄联盟\Game".to_string(),
            extract_base_and_ui: true,
            extract_shaders: true,
            extract_audio: true,
            skip_map_geo: false,
            is_extracting: false,
            current_step_index: 0,
            current_step_status: "待开始".to_string(),
            step_logs: HashMap::new(),
            expanded_steps: expanded,
            status_message: None,
            health_report: None,
            is_checking_health: false,
            health_panel_expanded: true,
        }
    }
}

pub fn render_extractor(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let assets_dir = resolve_assets_dir();

    // 帧初始化输入框句柄
    if sidebar.extractor.game_path_input.is_none() {
        let default_path = sidebar.extractor.game_path.clone();
        let ed = cx.new(|cx| InputState::new(window, cx).default_value(&default_path));
        sidebar.extractor.game_path_input = Some(ed);
    }

    // 首次进入自动触发环境工具体检
    if sidebar.extractor.health_report.is_none() && !sidebar.extractor.is_checking_health {
        trigger_health_check(sidebar, cx);
    }

    let (
        game_path_input,
        extract_base,
        extract_shaders,
        extract_audio,
        skip_map_geo,
        is_extracting,
        current_step_idx,
        step_status_desc,
        step_logs,
        expanded_steps,
        status,
        health_report,
        is_checking_health,
        health_panel_expanded,
    ) = {
        let s = &sidebar.extractor;
        (
            s.game_path_input.clone(),
            s.extract_base_and_ui,
            s.extract_shaders,
            s.extract_audio,
            s.skip_map_geo,
            s.is_extracting,
            s.current_step_index,
            s.current_step_status.clone(),
            s.step_logs.clone(),
            s.expanded_steps.clone(),
            s.status_message.clone(),
            s.health_report.clone(),
            s.is_checking_health,
            s.health_panel_expanded,
        )
    };

    let theme = ExtractorTheme::from_cx(cx);

    let steps = vec![
        StepInfo {
            title: "1. Git 同步 Hash 数据".to_string(),
            description: "自动 clone / pull CommunityDragon-Data 社区哈希字典".to_string(),
        },
        StepInfo {
            title: "2. 基础资源与 UI 提取".to_string(),
            description: "提取全英雄 3D 模型、纹理贴图、地图数据与全套矢量 UI 资源".to_string(),
        },
        StepInfo {
            title: "3. 全英雄音效提取".to_string(),
            description: "解析 bnk/wpk 音频包并用 ww2ogg 转码生成 AudioBank RON 配置".to_string(),
        },
        StepInfo {
            title: "4. 着色器反编译 (ShaderCache)".to_string(),
            description: "提取 DXBC 字节码，用 dxbc-compiler 转译 SPIR-V 并生成 ShaderMap 布局索引"
                .to_string(),
        },
        StepInfo {
            title: "5. 全量完成 (Complete)".to_string(),
            description: "所有资源已同步更新至目标 assets 目录，可以投入游戏或场景使用".to_string(),
        },
    ];

    let total_steps = steps.len();

    v_flex()
        .id("extractor_page")
        .flex_1()
        .overflow_y_scroll()
        .gap_4()
        .child(
            // 顶部标题栏卡片
            v_flex()
                .w_full()
                .p_4()
                .bg(theme.bg)
                .border_1()
                .border_color(theme.border)
                .rounded_lg()
                .gap_2()
                .child(
                    h_flex().justify_between().items_center().child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .font_bold()
                                    .text_lg()
                                    .child("资源提取中心 (Resource Extractor)"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .bg(theme.accent.opacity(0.2))
                                    .rounded_sm()
                                    .text_xs()
                                    .child("Production Tool"),
                            ),
                    ),
                )
                .child(div().text_xs().text_color(theme.muted_fg).child(format!(
                    "当前输出目标 Assets 目录: {} (根据运行环境自动切换为 Dev/Release 资源路径)",
                    assets_dir.display()
                ))),
        )
        .child(
            // 环境工具体检看板卡片
            render_health_check_panel(
                health_report,
                is_checking_health,
                health_panel_expanded,
                theme,
                cx,
            ),
        )
        .child(
            // 选项配置卡片
            v_flex()
                .w_full()
                .p_4()
                .bg(theme.bg)
                .border_1()
                .border_color(theme.border)
                .rounded_lg()
                .gap_3()
                .child(div().font_bold().text_sm().child("提取参数与目标配置"))
                // 游戏路径输入
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().text_xs().font_medium().child("英雄联盟 Game 根目录"))
                        .child(h_flex().w_full().gap_2().child(
                            if let Some(ed) = &game_path_input {
                                Input::new(ed).flex_1().into_any_element()
                            } else {
                                div().into_any_element()
                            },
                        )),
                )
                // 勾选项集合
                .child(
                    h_flex()
                        .gap_6()
                        .items_center()
                        .child(
                            Checkbox::new("chk_base")
                                .label("基础模型/贴图/地图/UI 提取")
                                .checked(extract_base)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.extractor.extract_base_and_ui =
                                        !this.extractor.extract_base_and_ui;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("chk_shaders")
                                .label("ShaderCache 着色器反编译与编译")
                                .checked(extract_shaders)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.extractor.extract_shaders =
                                        !this.extractor.extract_shaders;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("chk_audio")
                                .label("全英雄音效配置 (AudioBank / ww2ogg)")
                                .checked(extract_audio)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.extractor.extract_audio = !this.extractor.extract_audio;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("chk_no_mapgeo")
                                .label("跳过地图 Mesh 优化 (Fast Mode)")
                                .checked(skip_map_geo)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.extractor.skip_map_geo = !this.extractor.skip_map_geo;
                                    cx.notify();
                                })),
                        ),
                )
                // 操作按钮
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            h_flex().gap_3().items_center().child(
                                Button::new("btn_start_extract")
                                    .primary()
                                    .label(if is_extracting {
                                        "正在提取中..."
                                    } else {
                                        "开始全量提取"
                                    })
                                    .disabled(is_extracting)
                                    .on_click(cx.listener(|this, _, _window, cx| {
                                        start_extraction_process(this, cx);
                                    })),
                            ),
                        )
                        .when_some(status, |this, msg| {
                            this.child(
                                div()
                                    .p_2()
                                    .bg(theme.danger.opacity(0.1))
                                    .border_1()
                                    .border_color(theme.danger.opacity(0.3))
                                    .rounded_md()
                                    .text_xs()
                                    .text_color(theme.danger)
                                    .child(msg),
                            )
                        }),
                ),
        )
        .child(
            // 垂直步骤流 + 内部日志展收卡片
            v_flex()
                .flex_1()
                .w_full()
                .p_4()
                .bg(theme.bg)
                .border_1()
                .border_color(theme.border)
                .rounded_lg()
                .overflow_y_scrollbar()
                .gap_3()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .font_bold()
                                .text_sm()
                                .child("提取步骤与实时过程 (Vertical Pipeline Steps)"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .opacity(0.8)
                                .child(format!("当前阶段: {}", step_status_desc)),
                        ),
                )
                .children(steps.into_iter().enumerate().map(|(idx, step_info)| {
                    let is_active = is_extracting && current_step_idx == idx;
                    let is_finished = current_step_idx > idx
                        || (!is_extracting
                            && current_step_idx == total_steps - 1
                            && step_status_desc == "提取任务已全量完成");
                    let is_expanded = expanded_steps.get(&idx).copied().unwrap_or(is_active);
                    let logs = step_logs.get(&idx).cloned().unwrap_or_default();
                    let log_count = logs.len();

                    v_flex()
                        .w_full()
                        .border_1()
                        .border_color(if is_active {
                            theme.accent
                        } else {
                            theme.border
                        })
                        .bg(if is_active {
                            theme.accent.opacity(0.05)
                        } else {
                            theme.bg
                        })
                        .rounded_md()
                        .overflow_hidden()
                        .child(
                            // 步骤标头栏
                            h_flex()
                                .w_full()
                                .p_3()
                                .justify_between()
                                .items_center()
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _window, cx| {
                                        let cur = this
                                            .extractor
                                            .expanded_steps
                                            .get(&idx)
                                            .copied()
                                            .unwrap_or(false);
                                        this.extractor.expanded_steps.insert(idx, !cur);
                                        cx.notify();
                                    }),
                                )
                                .child(
                                    h_flex()
                                        .gap_3()
                                        .items_center()
                                        .child(
                                            div()
                                                .w_6()
                                                .h_6()
                                                .rounded_full()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_xs()
                                                .font_bold()
                                                .bg(if is_finished {
                                                    theme.primary
                                                } else if is_active {
                                                    theme.accent
                                                } else {
                                                    theme.muted
                                                })
                                                .text_color(if is_finished || is_active {
                                                    theme.primary_fg
                                                } else {
                                                    theme.muted_fg
                                                })
                                                .child(if is_finished {
                                                    "✓".to_string()
                                                } else {
                                                    format!("{}", idx + 1)
                                                }),
                                        )
                                        .child(
                                            v_flex()
                                                .gap_0p5()
                                                .child(
                                                    div()
                                                        .font_bold()
                                                        .text_sm()
                                                        .text_color(if is_active {
                                                            theme.accent
                                                        } else {
                                                            theme.fg
                                                        })
                                                        .child(step_info.title.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme.muted_fg)
                                                        .child(step_info.description.clone()),
                                                ),
                                        ),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .when(is_active, |this| {
                                            this.child(
                                                div()
                                                    .px_2()
                                                    .py_0p5()
                                                    .bg(theme.accent.opacity(0.2))
                                                    .rounded_sm()
                                                    .text_xs()
                                                    .child("处理中..."),
                                            )
                                        })
                                        .child(
                                            div()
                                                .text_xs()
                                                .opacity(0.6)
                                                .child(format!("{} 条日志", log_count)),
                                        ),
                                ),
                        )
                        .when(is_expanded && !logs.is_empty(), |this| {
                            // 步骤内部内嵌控制台日志
                            this.child(
                                v_flex()
                                    .w_full()
                                    .p_3()
                                    .bg(theme.sidebar)
                                    .border_t_1()
                                    .border_color(theme.border)
                                    .font_family("Consolas, monospace")
                                    .text_xs()
                                    .gap_1()
                                    .max_h_48()
                                    .overflow_y_scrollbar()
                                    .children(logs.into_iter().map(|log| div().child(log))),
                            )
                        })
                })),
        )
        .into_any_element()
}

/// 渲染环境工具体检看板
fn render_health_check_panel(
    report: Option<EnvironmentHealthReport>,
    is_checking: bool,
    is_expanded: bool,
    theme: ExtractorTheme,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let (summary_text, status_badge_bg, status_badge_text, timestamp) = if is_checking {
        (
            "正在体检运行环境中...".to_string(),
            theme.accent.opacity(0.2),
            theme.accent,
            None,
        )
    } else if let Some(r) = &report {
        let (passed, warn, failed) = r.summary_counts();
        let total = r.items.len();
        if failed > 0 {
            (
                format!("体检发现 {} 项工具未就绪 (共 {} 项)", failed, total),
                theme.danger.opacity(0.2),
                theme.danger,
                r.check_timestamp.clone(),
            )
        } else if warn > 0 {
            (
                format!("环境就绪 (包含 {} 项建议项, 共 {} 项)", warn, total),
                theme.warning.opacity(0.2),
                theme.warning,
                r.check_timestamp.clone(),
            )
        } else {
            (
                format!("全部 {} 项工具就绪", passed),
                theme.success.opacity(0.2),
                theme.success,
                r.check_timestamp.clone(),
            )
        }
    } else {
        (
            "等待体检".to_string(),
            theme.muted.opacity(0.2),
            theme.muted_fg,
            None,
        )
    };

    v_flex()
        .w_full()
        .p_4()
        .bg(theme.bg)
        .border_1()
        .border_color(theme.border)
        .rounded_lg()
        .gap_3()
        .child(
            // 标头栏
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(
                            div()
                                .font_bold()
                                .text_sm()
                                .child("环境与二进制工具体检 (Environment Diagnostics)"),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .bg(status_badge_bg)
                                .text_color(status_badge_text)
                                .rounded_sm()
                                .text_xs()
                                .font_medium()
                                .child(summary_text),
                        )
                        .when_some(timestamp, |this, ts| {
                            this.child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_fg)
                                    .child(format!("上次检测: {}", ts)),
                            )
                        }),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Button::new("btn_recheck_health")
                                .outline()
                                .label(if is_checking {
                                    "检测中..."
                                } else {
                                    "重新体检"
                                })
                                .disabled(is_checking)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    trigger_health_check(this, cx);
                                })),
                        )
                        .child(
                            Button::new("btn_toggle_health_panel")
                                .ghost()
                                .label(if is_expanded {
                                    "收起"
                                } else {
                                    "展开详情"
                                })
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.extractor.health_panel_expanded =
                                        !this.extractor.health_panel_expanded;
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .when(is_expanded, |this| {
            if let Some(r) = &report {
                this.child(
                    v_flex().w_full().gap_2().children(
                        r.items
                            .iter()
                            .map(|item| render_tool_check_row(item, theme)),
                    ),
                )
            } else {
                this.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_fg)
                        .child("正在执行初始环境体检，请稍候..."),
                )
            }
        })
        .into_any_element()
}

/// 渲染单行工具体检项目
fn render_tool_check_row(item: &ToolCheckItem, theme: ExtractorTheme) -> AnyElement {
    let (status_icon, status_color, status_bg) = match item.status {
        ToolHealthStatus::Passed => ("✓", theme.success, theme.success.opacity(0.1)),
        ToolHealthStatus::Warning => ("⚠", theme.warning, theme.warning.opacity(0.1)),
        ToolHealthStatus::Failed => ("✕", theme.danger, theme.danger.opacity(0.1)),
        ToolHealthStatus::Checking => ("⏳", theme.muted_fg, theme.muted.opacity(0.1)),
    };

    let cat_label = match item.category {
        ToolCategory::Required => "必需",
        ToolCategory::Recommended => "推荐",
        ToolCategory::ShaderSpecific => "Shader专用",
        ToolCategory::Optional => "可选",
    };

    v_flex()
        .w_full()
        .p_2p5()
        .bg(theme.sidebar.opacity(0.5))
        .border_1()
        .border_color(theme.border)
        .rounded_md()
        .gap_1p5()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .w_5()
                                .h_5()
                                .rounded_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(status_bg)
                                .text_color(status_color)
                                .text_xs()
                                .font_bold()
                                .child(status_icon),
                        )
                        .child(
                            div()
                                .font_bold()
                                .text_xs()
                                .text_color(theme.fg)
                                .child(item.name.clone()),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .bg(theme.muted.opacity(0.2))
                                .text_color(theme.muted_fg)
                                .rounded_sm()
                                .text_xs()
                                .child(cat_label),
                        ),
                )
                .child(
                    div()
                        .font_family("Consolas, monospace")
                        .text_xs()
                        .text_color(if item.status == ToolHealthStatus::Failed {
                            theme.danger
                        } else {
                            theme.muted_fg
                        })
                        .child(
                            item.version_or_path
                                .clone()
                                .unwrap_or_else(|| "未找到".to_string()),
                        ),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_fg)
                .child(item.description.clone()),
        )
        .when_some(item.remedy_hint.clone(), |this, hint| {
            this.child(
                div()
                    .px_2()
                    .py_1()
                    .bg(if item.status == ToolHealthStatus::Failed {
                        theme.danger.opacity(0.1)
                    } else {
                        theme.warning.opacity(0.1)
                    })
                    .rounded_sm()
                    .text_xs()
                    .text_color(if item.status == ToolHealthStatus::Failed {
                        theme.danger
                    } else {
                        theme.warning
                    })
                    .child(format!("修复指引: {}", hint)),
            )
        })
        .into_any_element()
}

/// 触发异步体检
pub fn trigger_health_check(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    if sidebar.extractor.is_checking_health {
        return;
    }
    sidebar.extractor.is_checking_health = true;
    sidebar.extractor.status_message = None;

    let game_path = sidebar
        .extractor
        .game_path_input
        .as_ref()
        .map(|ed| ed.read(cx).value().to_string())
        .unwrap_or_else(|| sidebar.extractor.game_path.clone());

    cx.notify();

    let weak = cx.entity().downgrade();
    cx.spawn(|_this, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            let report = run_environment_health_check(&game_path).await;
            let _ = weak.update(&mut cx, |this, cx| {
                this.extractor.is_checking_health = false;
                this.extractor.health_report = Some(report);
                cx.notify();
            });
        }
    })
    .detach();
}

fn start_extraction_process(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let state = &mut sidebar.extractor;

    let path = state
        .game_path_input
        .as_ref()
        .map(|ed| ed.read(cx).value().to_string())
        .unwrap_or_else(|| state.game_path.clone());

    // 1. 提取前环境健康检查与拦截
    if let Some(report) = &state.health_report {
        if let Err(errors) =
            validate_before_extraction(report, state.extract_shaders, state.skip_map_geo)
        {
            state.status_message = Some(format!(
                "环境体检未通过，已拦截提取:\n• {}",
                errors.join("\n• ")
            ));
            state.health_panel_expanded = true;
            cx.notify();
            return;
        }
    }

    // 2. 正常初始化提取状态
    state.is_extracting = true;
    state.status_message = None;
    state.step_logs.clear();
    state.current_step_index = 0;
    state.current_step_status = "环境检查通过，准备启动...".to_string();

    let mut preflight_logs = vec![
        "[PREFLIGHT] 环境与二进制工具体检校验通过，准备启动后台 Worker...".to_string(),
        format!("[PREFLIGHT] 英雄联盟 Game 根目录: {}", path),
    ];
    if let Some(report) = &state.health_report {
        for item in &report.items {
            if let Some(ver) = &item.version_or_path {
                preflight_logs.push(format!("  [TOOL] {:<18}: {}", item.name, ver));
            }
        }
    }
    state.step_logs.insert(0, preflight_logs);

    let config = ExtractionConfig {
        game_path: path,
        extract_base_and_ui: state.extract_base_and_ui,
        extract_shaders: state.extract_shaders,
        extract_audio: state.extract_audio,
        skip_map_geo: state.skip_map_geo,
    };
    cx.notify();

    let (log_tx, mut log_rx) = mpsc::unbounded_channel::<(ExtractionStep, String)>();
    let (step_tx, mut step_rx) = mpsc::unbounded_channel::<(ExtractionStep, String)>();
    let weak = cx.entity().downgrade();

    cx.spawn(|_this, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            // 提取任务放全局 runtime 并行跑，本任务同时消费 log/step 进度消息
            let mut task = crate::services::runtime::tokio_runtime()
                .spawn(run_extraction_task(config, log_tx, step_tx));

            let mut result: Option<Result<(), String>> = None;
            while result.is_none() {
                tokio::select! {
                    log = log_rx.recv() => {
                        if let Some((step, log_msg)) = log {
                            let _ = weak.update(&mut cx, |this, cx| {
                                this.extractor
                                    .step_logs
                                    .entry(step as usize)
                                    .or_default()
                                    .push(log_msg);
                                cx.notify();
                            });
                        }
                    }
                    step = step_rx.recv() => {
                        if let Some((step, desc)) = step {
                            let _ = weak.update(&mut cx, |this, cx| {
                                this.extractor.current_step_index = step as usize;
                                this.extractor.current_step_status = desc;
                                this.extractor.expanded_steps.insert(step as usize, true);
                                cx.notify();
                            });
                        }
                    }
                    res = &mut task => {
                        result = Some(res.unwrap_or_else(|_| Err("提取任务被取消".to_string())));
                    }
                }
            }

            let _ = weak.update(&mut cx, |this, cx| {
                this.extractor.is_extracting = false;
                if let Err(err) = result.take().unwrap_or(Ok(())) {
                    this.extractor.status_message = Some(err);
                }
                cx.notify();
            });
        }
    })
    .detach();
}
