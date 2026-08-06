use std::cell::RefCell;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};

use crate::components::sidebar::AppSidebar;

// ── 文章数据（移植自 client `data/posts.ts`）──

struct Post {
    /// 文章标识（对应 client `data/posts.ts` 的 key），列表态未直接渲染
    #[allow(dead_code)]
    slug: &'static str,
    title: &'static str,
    desc: &'static str,
    date: &'static str,
    tag: &'static str,
    /// 简化后的 markdown 文本（仅支持 # / ## / ### / - / 1. / 段落 / ``` 代码块）
    content: &'static str,
}

const POSTS: &[Post] = &[
    Post {
        slug: "architecture",
        title: "工程架构",
        desc: "Moon LoL 的高层系统设计：Rust Core 与 Web Frontend.",
        date: "2025.11.28",
        tag: "ARCHITECTURE",
        content: r#"# 工程架构

```mermaid
graph TD
    subgraph "Rust Core (Bevy)"
        App[Bevy App]
        ECS[ECS World]
        Plugins[Core Plugins]
    end

    subgraph "Frontend (Vue)"
        Web[Vue App]
        Render[Renderer]
    end

    App -->|Update| ECS
    ECS -->|State/Events| Web
    Web -->|Input| ECS
```

## 核心设计理念

Moon LoL 采用高性能的分层架构。 系统的核心是基于 Rust 的 Bevy 引擎，它提供了极致的性能和类型安全的 ECS 架构。

## 核心支柱

- **Rust Core (Bevy):** 负责所有游戏逻辑、物理模拟和状态管理。利用 ECS 模式实现高并发和内存友好的数据处理。
- **Frontend (Vue 3):** 通过 WebSocket 或 HTTP 轮询获取游戏状态，利用 WebGL/Canvas 进行实时渲染，主要用于调试和观察。

## 高性能通信

前端与后端通过高效的数据协议进行通信，确保在渲染大量实体时依然保持流畅的帧率。
"#,
    },
    Post {
        slug: "data-flow",
        title: "数据流转",
        desc: "从 Bevy ECS 到 Web 前端的数据管线。",
        date: "2025.11.28",
        tag: "DATA",
        content: r#"# 数据流转

```mermaid
sequenceDiagram
    participant Bevy as Bevy System
    participant Frontend as Vue Client

    loop Game Loop
        Bevy->>Bevy: Run Systems (Movement, Attack, etc.)
        Bevy->>Bevy: Update State
    end

    loop Render Loop
        Frontend->>Bevy: Request State
        Bevy-->>Frontend: JSON Snapshot
        Frontend->>Frontend: Render Frame
    end
```

## 游戏循环

系统的数据流转完全由 Bevy 的 Schedule 驱动。

## 核心循环 (Update)

Bevy 引擎每帧执行一次 Update Schedule：

1. 处理输入事件 (Input Events)。
2. 运行所有的 Systems (移动、攻击、伤害计算)。
3. 更新组件状态 (Components)。

## 渲染循环 (Render)

前端 Vue 应用并不控制游戏逻辑，它只是一个观察者。它通过 API 定期拉取最新的游戏状态快照（通常是 JSON 格式），然后更新 DOM 或 Canvas。这种解耦设计允许后端以最大速度运行，而前端只负责可视化。
"#,
    },
    Post {
        slug: "ecs",
        title: "ECS 组件与系统",
        desc: "深入解析游戏核心逻辑：插件系统与实体组件设计。",
        date: "2025.11.28",
        tag: "CORE",
        content: r#"# ECS 组件与系统

## 一切皆实体 (Everything is an Entity)

在 Moon LoL 中，无论是英雄、小兵、防御塔，还是飞行的技能弹道，本质上都是 ECS 世界中的一个 Entity。 它们的行为差异仅仅来自于它们挂载了不同的 Component 集合。

## 核心组件 (Components)

- **Health:** 存储当前生命值和最大生命值。当生命值归零时，触发死亡逻辑。
- **Controller:** 标记该实体受外部控制（如 RL Agent 或 玩家输入）。
- **Transform:** Bevy 内置组件，定义实体在 3D 空间中的位置、旋转和缩放。
- **Skill:** 管理技能冷却、等级和施放状态。

## 系统插件 (Plugins)

我们将功能模块化为 Bevy Plugins。每个 Plugin 注册相关的 Systems 和 Resources。 以下是当前系统注册的所有核心插件：

- PluginFioraPassive
- PluginFioraE
- PluginFioraR
- PluginBarrack
- PluginChampion
- PluginCharacter
- PluginDebugSphere
- PluginFiora
- PluginHwei
- PluginMinion
- PluginTurret
- PluginAction
- PluginAnimation
- PluginAttack
- PluginAttackAuto
- PluginAggro
- PluginBase
- PluginCamera
- PluginController
- PluginDamage
- PluginGame
- PluginLife
- PluginLifetime
- PluginMap
- PluginMissile
- PluginMovement
- PluginNavigaton
- PluginParticle
- PluginResource
- PluginRotate
- PluginRun
- PluginSkill
- PluginSkin
- PluginState
- PluginUI
"#,
    },
];

// ── 页面本地状态 ──

struct BlogPageState {
    /// 当前选中的文章下标，None 表示列表态
    selected: Option<usize>,
}

impl Default for BlogPageState {
    fn default() -> Self {
        Self { selected: None }
    }
}

thread_local! {
    static STATE: RefCell<BlogPageState> = RefCell::new(BlogPageState::default());
}

fn with_state<R>(f: impl FnOnce(&BlogPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

fn update_state(f: impl FnOnce(&mut BlogPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

// ── markdown 渲染 ──

/// 去掉行内的 `**` 与反引号强调符，仅保留纯文本。
fn clean_inline(s: &str) -> String {
    s.replace("**", "").replace('`', "")
}

/// 识别 "1. xxx" 形式的数字列表项，返回（序号, 内容）。
fn split_numbered(line: &str) -> Option<(String, &str)> {
    let (num, rest) = line.split_once(". ")?;
    if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((format!("{}.", num), rest))
}

/// 渲染一个 ``` 围栏代码块（等宽小字 + 浅色底框）。
fn render_code_block(lines: &[String], muted: Hsla, border: Hsla) -> AnyElement {
    v_flex()
        .gap_1()
        .px_3()
        .py_2()
        .border_1()
        .border_color(border.opacity(0.4))
        .rounded_md()
        .bg(border.opacity(0.08))
        .children(lines.iter().map(|l| {
            div()
                .font_family("monospace")
                .text_xs()
                .text_color(muted)
                .child(l.clone())
                .into_any_element()
        }))
        .into_any_element()
}

/// 最简 markdown 渲染：标题(#/##/###)、列表(- / 1.)、段落，代码块原样等宽展示。
fn render_markdown(cx: &mut Context<AppSidebar>, content: &str) -> Vec<AnyElement> {
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let text = cx.theme().foreground;
    let border = cx.theme().border;

    let mut out: Vec<AnyElement> = Vec::new();
    let mut code_buf: Vec<String> = Vec::new();
    let mut in_code = false;

    for raw in content.lines() {
        let trimmed = raw.trim();

        if trimmed.starts_with("```") {
            if in_code {
                out.push(render_code_block(&code_buf, muted, border));
                code_buf.clear();
                in_code = false;
            } else {
                code_buf.clear();
                in_code = true;
            }
            continue;
        }

        if in_code {
            code_buf.push(trimmed.to_string());
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("### ") {
            out.push(
                div()
                    .text_lg()
                    .font_semibold()
                    .child(clean_inline(rest))
                    .into_any_element(),
            );
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            out.push(
                div()
                    .text_xl()
                    .font_bold()
                    .text_color(accent)
                    .child(clean_inline(rest))
                    .into_any_element(),
            );
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            out.push(
                div()
                    .text_2xl()
                    .font_bold()
                    .text_color(accent)
                    .child(clean_inline(rest))
                    .into_any_element(),
            );
        } else if let Some(rest) = trimmed.strip_prefix("- ") {
            out.push(
                h_flex()
                    .gap_2()
                    .items_start()
                    .child(div().text_sm().text_color(accent).child("•"))
                    .child(div().text_base().flex_1().child(clean_inline(rest)))
                    .into_any_element(),
            );
        } else if let Some((num, rest)) = split_numbered(trimmed) {
            out.push(
                h_flex()
                    .gap_2()
                    .items_start()
                    .child(div().text_sm().text_color(accent).child(num))
                    .child(div().text_base().flex_1().child(clean_inline(rest)))
                    .into_any_element(),
            );
        } else {
            out.push(
                div()
                    .text_base()
                    .text_color(text)
                    .child(clean_inline(trimmed))
                    .into_any_element(),
            );
        }
    }

    // 容错：代码块围栏未闭合时仍输出缓冲内容
    if in_code && !code_buf.is_empty() {
        out.push(render_code_block(&code_buf, muted, border));
    }

    out
}

// ── 子视图 ──

fn render_header(cx: &mut Context<AppSidebar>) -> AnyElement {
    v_flex()
        .gap_1()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(IconName::BookOpen)
                .child(div().font_bold().text_lg().child("开发日志")),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Moon LoL 开发进程与设计思路记录"),
        )
        .into_any_element()
}

fn render_post_list(cx: &mut Context<AppSidebar>) -> AnyElement {
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;

    v_flex()
        .w_full()
        .children(POSTS.iter().enumerate().map(|(i, post)| {
            let idx = i;
            h_flex()
                .w_full()
                .py_5()
                .border_t_1()
                .border_color(accent.opacity(0.3))
                .cursor_pointer()
                .id(format!("blog-item-{}", idx))
                .on_click(cx.listener(move |_this, _event, _window, cx| {
                    update_state(|s| s.selected = Some(idx));
                    cx.notify();
                }))
                .items_start()
                .justify_between()
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            h_flex()
                                .gap_3()
                                .items_center()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_bold()
                                        .text_color(accent)
                                        .child(format!("[{}]", post.tag)),
                                )
                                .child(div().text_xs().text_color(muted).child(post.date)),
                        )
                        .child(div().text_lg().font_bold().child(post.title))
                        .child(div().text_sm().text_color(muted).child(post.desc)),
                )
                .child(div().text_lg().text_color(accent).child("->"))
                .into_any_element()
        }))
        .into_any_element()
}

fn render_post_detail(post: &Post, cx: &mut Context<AppSidebar>) -> AnyElement {
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;

    v_flex()
        .gap_5()
        .child(
            Button::new("blog-back")
                .ghost()
                .icon(IconName::ArrowLeft)
                .label("返回列表")
                .on_click(cx.listener(|_this, _event, _window, cx| {
                    update_state(|s| s.selected = None);
                    cx.notify();
                })),
        )
        .child(
            v_flex()
                .gap_2()
                .child(
                    div()
                        .text_3xl()
                        .font_bold()
                        .text_color(accent)
                        .child(post.title),
                )
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .text_xs()
                        .child(
                            div()
                                .text_xs()
                                .font_bold()
                                .text_color(accent)
                                .child(format!("[{}]", post.tag)),
                        )
                        .child(div().text_xs().text_color(muted).child(post.date)),
                ),
        )
        .child(div().w_full().h_px().bg(border))
        .child(v_flex().gap_3().children(render_markdown(cx, post.content)))
        .into_any_element()
}

// ── 公开入口 ──

/// 开发日志博客：列表态展示文章标题 / 日期 / 摘要，点击进入详情态，返回按钮回到列表。
pub fn render_blog(_sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let selected = with_state(|s| s.selected);

    let divider = div().w_full().h_px().bg(cx.theme().border);
    let header = render_header(cx);
    let body = match selected {
        Some(idx) => match POSTS.get(idx) {
            Some(post) => render_post_detail(post, cx),
            None => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("文章不存在或已被移除")
                .into_any_element(),
        },
        None => render_post_list(cx),
    };

    div()
        .size_full()
        .flex_1()
        .overflow_hidden()
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .gap_5()
                .overflow_y_scrollbar()
                .p_6()
                .child(header)
                .child(divider)
                .child(body)
                .into_any_element(),
        )
        .into_any_element()
}
