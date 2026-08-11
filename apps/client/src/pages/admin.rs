use std::time::Duration;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use lol_web_protocol::admin::AdminMetrics;
use lol_web_protocol::match_::Match;
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::services::cloud::CloudClient;
use crate::services::provider;

// ── 页面本地状态 ──

pub struct AdminPageState {
    metrics: Option<AdminMetrics>,
    running: Vec<Match>,
    abort_target: Option<Match>,
    aborting: bool,
    loading: bool,
    /// 3 秒自动轮询是否已启动（防止重复 spawn 轮询 loop）
    polling: bool,
    error: Option<String>,
}

impl Default for AdminPageState {
    fn default() -> Self {
        Self {
            metrics: None,
            running: Vec::new(),
            abort_target: None,
            aborting: false,
            loading: true,
            polling: false,
            error: None,
        }
    }
}

fn short_id(id: &str) -> String {
    if id.len() > 8 {
        id[..8].to_string()
    } else {
        id.to_string()
    }
}

fn ago_iso(iso: &str) -> String {
    if iso.len() >= 16 {
        iso[..16].replace('T', " ")
    } else {
        iso.to_string()
    }
}

// ── 数据拉取 ──

/// 并行拉取 Admin 指标 + 运行中对局并写入 state。
/// 云请求在 cloud.rs 内部已桥接 tokio，在 gpui AsyncApp 里 await 安全。
async fn refresh_admin_data(
    client: &CloudClient,
    weak: &gpui::WeakEntity<AppSidebar>,
    cx: &mut gpui::AsyncApp,
) {
    let (m, r) = tokio::join!(async { client.get_admin_metrics().await }, async {
        client.list_running_matches().await
    },);
    if let Some(e) = weak.upgrade() {
        let _ = e.update(cx, |this, cx| {
            this.admin.metrics = m.ok();
            this.admin.running = r.unwrap_or_default();
            this.admin.loading = false;
            cx.notify();
        });
    }
}

/// 启动 3 秒自动轮询。页面运行在 gpui AsyncApp（非 tokio runtime），
/// 延时必须经 run_on_tokio 桥接到全局 tokio runtime，直接 sleep 会 panic。
fn start_polling(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    sidebar.admin.polling = true;
    let client = provider::cloud_client().clone();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let client = client.clone();
            async move {
                loop {
                    refresh_admin_data(&client, &weak, &mut cx).await;
                    // sidebar 实体被销毁则停止轮询，避免后台空转请求
                    if weak.upgrade().is_none() {
                        break;
                    }
                    let _ = crate::services::runtime::run_on_tokio(|| async {
                        tokio::time::sleep(Duration::from_secs(3)).await;
                        Ok::<(), String>(())
                    })
                    .await;
                }
            }
        },
    )
    .detach();
}

// ── 公开入口 ──

/// Admin 指标、运行中对局、强制中止。首次渲染即加载，并每 3 秒自动轮询刷新。
pub fn render_admin(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    // 首次渲染即启动 3 秒轮询（polling 标志位防止重复 spawn）
    if !sidebar.admin.polling {
        start_polling(sidebar, cx);
    }

    let (metrics, running, abort_target, aborting, loading, error) = (
        sidebar.admin.metrics.clone(),
        sidebar.admin.running.clone(),
        sidebar.admin.abort_target.clone(),
        sidebar.admin.aborting,
        sidebar.admin.loading,
        sidebar.admin.error.clone(),
    );

    // 刷新按钮
    let refresh_btn = {
        let client = provider::cloud_client().clone();
        Button::new("admin-refresh-btn")
            .outline()
            .icon(IconName::Redo)
            .label(t!("app.rl.refresh_list"))
            .on_click(cx.listener(move |_this, _, _, cx| {
                let client = client.clone();
                cx.spawn(
                    move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                        let weak = weak.clone();
                        let mut cx = cx.clone();
                        let client = client.clone();
                        async move {
                            refresh_admin_data(&client, &weak, &mut cx).await;
                        }
                    },
                )
                .detach();
            }))
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
                .gap_6()
                .overflow_y_scrollbar()
                .p_6()
                // ── 标题行 ──
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(
                            v_flex()
                                .gap_1()
                                .child(h_flex().gap_2().items_center().child(IconName::Cpu).child(
                                    div().font_bold().text_lg().child(t!("app.nav.title_admin")),
                                ))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("服务器并发对局算力与内存调度"),
                                ),
                        )
                        .child(refresh_btn),
                )
                // ── 错误提示 ──
                .when_some(error.as_ref(), |d, err| {
                    d.child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(cx.theme().danger.opacity(0.1))
                            .text_color(cx.theme().danger)
                            .text_xs()
                            .child(err.clone()),
                    )
                })
                // ── Stat 行 ──
                .child(
                    h_flex()
                        .gap_4()
                        .w_full()
                        .child(stat_card(
                            cx,
                            IconName::Play,
                            "运行中对局",
                            metrics
                                .as_ref()
                                .map_or("—".to_string(), |m| m.running_matches.to_string()),
                        ))
                        .child(stat_card(
                            cx,
                            IconName::MemoryStick,
                            "排队中",
                            metrics
                                .as_ref()
                                .map_or("—".to_string(), |m| m.pending_matches.to_string()),
                        ))
                        .child(stat_card(
                            cx,
                            IconName::Bot,
                            "排队 Agent",
                            metrics
                                .as_ref()
                                .map_or("—".to_string(), |m| m.queued_agents.to_string()),
                        ))
                        .child(stat_card(
                            cx,
                            IconName::SquareTerminal,
                            "托管进程",
                            metrics
                                .as_ref()
                                .map_or("—".to_string(), |m| m.managed_processes.to_string()),
                        )),
                )
                // ── 分隔线 ──
                .child(div().w_full().h_px().bg(cx.theme().border))
                // ── 运行中对局表格 ──
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .child(div().text_sm().font_bold().child("进行中对局"))
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .text_xs()
                                        .font_bold()
                                        .bg(cx.theme().accent.opacity(0.15))
                                        .text_color(cx.theme().accent)
                                        .child(format!("{}", running.len())),
                                ),
                        )
                        .when(loading && running.is_empty(), |d| {
                            d.child(
                                div()
                                    .py_12()
                                    .w_full()
                                    .text_center()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("加载中…"),
                            )
                        })
                        .when(!loading && running.is_empty(), |d| {
                            d.child(
                                div()
                                    .py_12()
                                    .w_full()
                                    .text_center()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("当前没有运行中的对局"),
                            )
                        })
                        .when(!running.is_empty(), |d| {
                            d.child(
                                div()
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .overflow_hidden()
                                    .child(
                                        v_flex()
                                            .child(
                                                h_flex()
                                                    .px_4()
                                                    .py_2()
                                                    .border_b_1()
                                                    .border_color(cx.theme().border)
                                                    .bg(cx.theme().background)
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("对局 ID"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(5.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("模式"),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("所属"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(4.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("端口"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(10.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("创建时间"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(8.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .text_right()
                                                            .child("操作"),
                                                    ),
                                            )
                                            .children(running.iter().map(|m| {
                                                let match_id = m.id.to_string();
                                                let mid = match_id.clone();
                                                h_flex()
                                                    .px_4()
                                                    .py_2()
                                                    .border_b_1()
                                                    .border_color(cx.theme().border.opacity(0.5))
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .text_xs()
                                                            .child(short_id(&match_id)),
                                                    )
                                                    .child(
                                                        div().w(rems(5.)).child(
                                                            div()
                                                                .px_2()
                                                                .py_0p5()
                                                                .rounded_md()
                                                                .bg(cx.theme().accent.opacity(0.1))
                                                                .text_xs()
                                                                .child(m.mode.clone()),
                                                        ),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(
                                                                if let Some(ref rid) = m.room_id {
                                                                    format!(
                                                                        "房间 {}",
                                                                        short_id(&rid.to_string())
                                                                    )
                                                                } else {
                                                                    format!(
                                                                        "用户 #{}",
                                                                        m.owner_user_id.map_or(
                                                                            "—".to_string(),
                                                                            |id| id.to_string()
                                                                        )
                                                                    )
                                                                },
                                                            ),
                                                    )
                                                    .child(div().w(rems(4.)).text_xs().child(
                                                        m.ws_port.map_or("—".to_string(), |p| {
                                                            p.to_string()
                                                        }),
                                                    ))
                                                    .child(
                                                        div()
                                                            .w(rems(10.))
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(ago_iso(&m.created_at)),
                                                    )
                                                    .child(div().w(rems(8.)).text_right().child({
                                                        let m_clone = m.clone();
                                                        Button::new(format!("abort-{}", mid))
                                                            .ghost()
                                                            .icon(IconName::CircleX)
                                                            .label("强制中止")
                                                            .on_click(cx.listener(
                                                                move |this, _, _, cx| {
                                                                    this.admin.abort_target =
                                                                        Some(m_clone.clone());
                                                                    cx.notify();
                                                                },
                                                            ))
                                                    }))
                                            })),
                                    ),
                            )
                        }),
                )
                .into_any_element(),
        )
        // ── 中止确认对话框 ──
        .when_some(abort_target.as_ref(), |d, target| {
            let t = target.clone();
            let tid = t.id.to_string();
            d.child(
                div()
                    .absolute()
                    .inset_0()
                    .bg(black().opacity(0.5))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .rounded_lg()
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().background)
                            .p_6()
                            .w(rems(24.))
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(div().font_bold().text_sm().child("强制中止对局"))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(
                                                "该对局将被立即终止并释放算力。此操作不可恢复。",
                                            ),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .justify_end()
                                    .child(
                                        Button::new("cancel-abort-btn")
                                            .ghost()
                                            .label("取消")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.admin.abort_target = None;
                                                cx.notify();
                                            })),
                                    )
                                    .child({
                                        let aid = tid.clone();
                                        let client = provider::cloud_client().clone();
                                        Button::new("confirm-abort-btn")
                                            .label("确认中止")
                                            .when(aborting, |b| b.disabled(true))
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.admin.aborting = true;
                                                let aid2 = aid.clone();
                                                let client2 = client.clone();
                                                cx.spawn(
                                                    move |weak: gpui::WeakEntity<AppSidebar>,
                                                     cx: &mut gpui::AsyncApp| {
                                                        let weak = weak.clone();
                                                        let mut cx = cx.clone();
                                                        let aid3 = aid2.clone();
                                                        let client3 = client2.clone();
                                                        async move {
                                                            let _ = client3
                                                                .force_abort_match(&aid3)
                                                                .await;
                                                            refresh_admin_data(
                                                                &client3,
                                                                &weak,
                                                                &mut cx,
                                                            )
                                                            .await;
                                                            if let Some(e) = weak.upgrade() {
                                                                let _ = e.update(
                                                                    &mut cx,
                                                                    |this, cx| {
                                                                        this.admin.aborting = false;
                                                                        this.admin.abort_target =
                                                                            None;
                                                                        cx.notify();
                                                                    },
                                                                );
                                                            }
                                                        }
                                                    },
                                                )
                                                .detach();
                                            }))
                                    }),
                            ),
                    ),
            )
        })
        .into_any_element()
}

fn stat_card(
    cx: &mut Context<AppSidebar>,
    icon: IconName,
    label: impl Into<SharedString>,
    value: String,
) -> AnyElement {
    div()
        .flex_1()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            h_flex()
                .gap_1p5()
                .items_center()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(icon)
                .child(label.into()),
        )
        .child(div().text_2xl().font_bold().child(value))
        .into_any_element()
}
