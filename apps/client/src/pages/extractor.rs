use std::cell::RefCell;
use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use tokio::sync::mpsc;

use crate::components::sidebar::AppSidebar;
use crate::services::assets_path::resolve_assets_dir;
use crate::services::extractor_service::{run_extraction_task, ExtractionConfig, ExtractionStep};

#[derive(Clone)]
pub struct StepInfo {
    pub title: String,
    pub description: String,
    pub icon: IconName,
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
        }
    }
}

thread_local! {
    static EXTRACTOR_STATE: RefCell<ExtractorPageState> = RefCell::new(ExtractorPageState::default());
}

pub fn render_extractor(
    _sidebar: &AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let assets_dir = resolve_assets_dir();

    // 帧初始化输入框句柄
    EXTRACTOR_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.game_path_input.is_none() {
            let default_path = state.game_path.clone();
            let ed = cx.new(|cx| InputState::new(window, cx).default_value(&default_path));
            state.game_path_input = Some(ed);
        }
    });

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
    ) = EXTRACTOR_STATE.with(|cell| {
        let state = cell.borrow();
        (
            state.game_path_input.clone(),
            state.extract_base_and_ui,
            state.extract_shaders,
            state.extract_audio,
            state.skip_map_geo,
            state.is_extracting,
            state.current_step_index,
            state.current_step_status.clone(),
            state.step_logs.clone(),
            state.expanded_steps.clone(),
            state.status_message.clone(),
        )
    });

    let theme = cx.theme();

    let steps = vec![
        StepInfo {
            title: "1. Git 同步 Hash 数据".to_string(),
            description: "自动 clone / pull CommunityDragon-Data 社区哈希字典".to_string(),
            icon: IconName::Folder,
        },
        StepInfo {
            title: "2. 基础资源与 UI 提取".to_string(),
            description: "提取全英雄 3D 模型、纹理贴图、地图数据与全套矢量 UI 资源".to_string(),
            icon: IconName::File,
        },
        StepInfo {
            title: "3. 全英雄音效提取".to_string(),
            description: "解析 bnk/wpk 音频包并用 ww2ogg 转码生成 AudioBank RON 配置".to_string(),
            icon: IconName::Palette,
        },
        StepInfo {
            title: "4. 着色器反编译 (ShaderCache)".to_string(),
            description: "提取 DXBC 字节码，用 HLSLDecompiler 转 HLSL 再由 DXC 编译 SPIR-V"
                .to_string(),
            icon: IconName::Settings,
        },
        StepInfo {
            title: "5. 全量完成 (Complete)".to_string(),
            description: "所有资源已同步更新至目标 assets 目录，可以投入游戏或场景使用".to_string(),
            icon: IconName::Check,
        },
    ];

    let total_steps = steps.len();

    v_flex()
        .w_full()
        .h_full()
        .p_4()
        .gap_4()
        .child(
            // 顶部标题栏卡片
            v_flex()
                .w_full()
                .p_4()
                .bg(theme.background)
                .border_1()
                .border_color(theme.border)
                .rounded_lg()
                .gap_2()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(div().font_bold().text_lg().child("资源提取中心 (Resource Extractor)"))
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
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(format!(
                            "当前输出目标 Assets 目录: {} (根据运行环境自动切换为 Dev/Release 资源路径)",
                            assets_dir.display()
                        )),
                ),
        )
        .child(
            // 选项配置卡片
            v_flex()
                .w_full()
                .p_4()
                .bg(theme.background)
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
                        .child(
                            h_flex()
                                .w_full()
                                .gap_2()
                                .child(if let Some(ed) = &game_path_input {
                                    Input::new(ed).flex_1().into_any_element()
                                } else {
                                    div().into_any_element()
                                }),
                        ),
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
                                .on_click(cx.listener(|_, _, _window, cx| {
                                    EXTRACTOR_STATE.with(|cell| {
                                        let mut state = cell.borrow_mut();
                                        state.extract_base_and_ui = !state.extract_base_and_ui;
                                    });
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("chk_shaders")
                                .label("ShaderCache 着色器反编译与编译")
                                .checked(extract_shaders)
                                .on_click(cx.listener(|_, _, _window, cx| {
                                    EXTRACTOR_STATE.with(|cell| {
                                        let mut state = cell.borrow_mut();
                                        state.extract_shaders = !state.extract_shaders;
                                    });
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("chk_audio")
                                .label("全英雄音效配置 (AudioBank / ww2ogg)")
                                .checked(extract_audio)
                                .on_click(cx.listener(|_, _, _window, cx| {
                                    EXTRACTOR_STATE.with(|cell| {
                                        let mut state = cell.borrow_mut();
                                        state.extract_audio = !state.extract_audio;
                                    });
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("chk_no_mapgeo")
                                .label("跳过地图 Mesh 优化 (Fast Mode)")
                                .checked(skip_map_geo)
                                .on_click(cx.listener(|_, _, _window, cx| {
                                    EXTRACTOR_STATE.with(|cell| {
                                        let mut state = cell.borrow_mut();
                                        state.skip_map_geo = !state.skip_map_geo;
                                    });
                                    cx.notify();
                                })),
                        ),
                )
                // 操作按钮
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(
                            Button::new("btn_start_extract")
                                .primary()
                                .label(if is_extracting { "正在提取中..." } else { "开始全量提取" })
                                .disabled(is_extracting)
                                .on_click(cx.listener(|_, _, _window, cx| {
                                    start_extraction_process(cx);
                                })),
                        )
                        .when_some(status, |this, msg| {
                            this.child(div().text_xs().text_color(theme.danger).child(msg))
                        }),
                ),
        )
        .child(
            // 垂直步骤流 + 内部日志展收卡片
            v_flex()
                .flex_1()
                .w_full()
                .p_4()
                .bg(theme.background)
                .border_1()
                .border_color(theme.border)
                .rounded_lg()
                .overflow_y_scrollbar()
                .gap_3()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(div().font_bold().text_sm().child("提取步骤与实时过程 (Vertical Pipeline Steps)"))
                        .child(
                            div()
                                .text_xs()
                                .opacity(0.8)
                                .child(format!("当前阶段: {}", step_status_desc)),
                        ),
                )
                .children(steps.into_iter().enumerate().map(|(idx, step_info)| {
                    let is_active = is_extracting && current_step_idx == idx;
                    let is_finished = current_step_idx > idx || (!is_extracting && current_step_idx == total_steps - 1 && step_status_desc == "提取任务已全量完成");
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
                            theme.background
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
                                    cx.listener(move |_, _, _window, cx| {
                                        EXTRACTOR_STATE.with(|cell| {
                                            let mut state = cell.borrow_mut();
                                            let cur = state.expanded_steps.get(&idx).copied().unwrap_or(false);
                                            state.expanded_steps.insert(idx, !cur);
                                        });
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
                                                    theme.primary_foreground
                                                } else {
                                                    theme.muted_foreground
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
                                                            theme.foreground
                                                        })
                                                        .child(step_info.title.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme.muted_foreground)
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
                                    .children(logs.into_iter().map(|log| {
                                        div().child(log)
                                    })),
                            )
                        })
                })),
        )
        .into_any_element()
}

fn start_extraction_process(cx: &mut Context<AppSidebar>) {
    let (config, _game_path_input) = EXTRACTOR_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.is_extracting = true;
        state.status_message = None;
        state.step_logs.clear();
        state.current_step_index = 0;
        state.current_step_status = "准备启动...".to_string();

        let path = state
            .game_path_input
            .as_ref()
            .map(|ed| ed.read(cx).value().to_string())
            .unwrap_or_else(|| state.game_path.clone());

        (
            ExtractionConfig {
                game_path: path,
                extract_base_and_ui: state.extract_base_and_ui,
                extract_shaders: state.extract_shaders,
                extract_audio: state.extract_audio,
                skip_map_geo: state.skip_map_geo,
            },
            state.game_path_input.clone(),
        )
    });
    cx.notify();

    let (log_tx, mut log_rx) = mpsc::unbounded_channel::<(ExtractionStep, String)>();
    let (step_tx, mut step_rx) = mpsc::unbounded_channel::<(ExtractionStep, String)>();
    let weak_entity_log = cx.entity().downgrade();
    let weak_entity_step = cx.entity().downgrade();
    let weak_entity_task = cx.entity().downgrade();

    // 监听日志消息推送到对应的 Step UI 内部
    cx.spawn(|_this, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            while let Some((step, log_msg)) = log_rx.recv().await {
                let step_idx = step as usize;
                let _ = weak_entity_log.update(&mut cx, |_, cx| {
                    EXTRACTOR_STATE.with(|cell| {
                        let mut state = cell.borrow_mut();
                        state.step_logs.entry(step_idx).or_default().push(log_msg);
                    });
                    cx.notify();
                });
            }
        }
    })
    .detach();

    // 监听步骤状态变化并更新进度
    cx.spawn(|_this, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            while let Some((step, desc)) = step_rx.recv().await {
                let step_idx = step as usize;
                let _ = weak_entity_step.update(&mut cx, |_, cx| {
                    EXTRACTOR_STATE.with(|cell| {
                        let mut state = cell.borrow_mut();
                        state.current_step_index = step_idx;
                        state.current_step_status = desc;
                        state.expanded_steps.insert(step_idx, true);
                    });
                    cx.notify();
                });
            }
        }
    })
    .detach();

    // 异步执行主任务
    cx.spawn(|_this, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            let res = run_extraction_task(config, log_tx, step_tx).await;
            let _ = weak_entity_task.update(&mut cx, |_, cx| {
                EXTRACTOR_STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    state.is_extracting = false;
                    if let Err(err) = res {
                        state.status_message = Some(err);
                    }
                });
                cx.notify();
            });
        }
    })
    .detach();
}
