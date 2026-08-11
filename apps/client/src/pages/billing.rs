use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use lol_web_protocol::essence::{BillingPlan, CheckInResult, EssenceTransaction};
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::services::provider;

// ── 页面本地状态 ──

pub struct BillingPageState {
    balance: Option<i64>,
    transactions: Vec<EssenceTransaction>,
    plans: Vec<BillingPlan>,
    current_plan: Option<BillingPlan>,
    check_in_result: Option<CheckInResult>,
    subscribing: Option<String>,
    loading: bool,
    error: Option<String>,
}

impl Default for BillingPageState {
    fn default() -> Self {
        Self {
            balance: None,
            transactions: Vec::new(),
            plans: Vec::new(),
            current_plan: None,
            check_in_result: None,
            subscribing: None,
            loading: true,
            error: None,
        }
    }
}

fn fmt_price(cents: i32) -> String {
    if cents == 0 {
        "免费".to_string()
    } else {
        format!("{} / 月", cents / 100)
    }
}

fn tx_kind_label(reason: &str) -> &str {
    match reason {
        "check_in" => "每日签到",
        "llm_token" => "模型 Token 消耗",
        "slot_purchase" => "购买 Agent 槽位",
        "recharge" => "充值",
        "subscription" => "订阅",
        _ => reason,
    }
}

fn fmt_date(iso: &str) -> String {
    if iso.len() >= 16 {
        iso[..16].replace('T', " ")
    } else {
        iso.to_string()
    }
}

// ── 公开入口 ──

/// 精粹余额、签到、交易流水、订阅套餐、订阅操作。
pub fn render_billing(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let (balance, transactions, plans, current_plan, check_in_result, subscribing, loading, error) = (
        sidebar.billing.balance,
        sidebar.billing.transactions.clone(),
        sidebar.billing.plans.clone(),
        sidebar.billing.current_plan.clone(),
        sidebar.billing.check_in_result.clone(),
        sidebar.billing.subscribing.clone(),
        sidebar.billing.loading,
        sidebar.billing.error.clone(),
    );

    // 提前构建交易行元素列表，使用 owned 数据避免 borrow 逃逸
    let tx_children: Vec<AnyElement> = transactions
        .iter()
        .map(|t| {
            let is_positive = t.amount >= 0;
            let kind = tx_kind_label(&t.reason).to_string();
            let amount_text = format!("{}{}", if is_positive { "+" } else { "" }, t.amount);
            let date_text = fmt_date(&t.created_at);
            let accent = cx.theme().accent;
            let danger = cx.theme().danger;
            let muted = cx.theme().muted_foreground;
            let border = cx.theme().border;
            h_flex()
                .px_4()
                .py_2()
                .border_b_1()
                .border_color(border.opacity(0.5))
                .child(div().flex_1().text_xs().child(kind))
                .child(
                    div()
                        .w(rems(6.))
                        .text_xs()
                        .text_right()
                        .text_color(if is_positive { accent } else { danger })
                        .child(amount_text),
                )
                .child(
                    div()
                        .w(rems(10.))
                        .text_xs()
                        .text_right()
                        .text_color(muted)
                        .child(date_text),
                )
                .into_any_element()
        })
        .collect();

    let tx_count = transactions.len();
    let tx_empty = transactions.is_empty();

    // 首次渲染时触发加载
    if loading && balance.is_none() {
        let client = provider::cloud_client().clone();
        cx.spawn(
            move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                let weak = weak.clone();
                let mut cx = cx.clone();
                let client = client.clone();
                async move {
                    let (bal, txs, plans, cur) = tokio::join!(
                        async { client.get_essence_balance().await },
                        async { client.get_essence_transactions(50, 0).await },
                        async { client.list_billing_plans().await },
                        async { client.get_current_subscription().await },
                    );
                    if let Some(e) = weak.upgrade() {
                        let _ = e.update(&mut cx, |this, cx| {
                            this.billing.balance = bal.ok();
                            this.billing.transactions = txs.unwrap_or_default();
                            this.billing.plans = plans.unwrap_or_default();
                            this.billing.current_plan = cur.ok();
                            this.billing.loading = false;
                            this.billing.error = None;
                            cx.notify();
                        });
                    }
                }
            },
        )
        .detach();
    }

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
                // ── 余额 Hero ──
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(
                            v_flex()
                                .gap_1()
                                .child(
                                    h_flex()
                                        .gap_1p5()
                                        .items_center()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(IconName::StarFill)
                                        .child(t!("app.nav.title_billing")),
                                )
                                .child(
                                    h_flex()
                                        .items_baseline()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_2xl()
                                                .font_bold()
                                                .child(balance.map_or("—".to_string(), |b| {
                                                    b.to_string()
                                                })),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("BE"),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("用于抵扣平台模型 Token 与购买 Agent 槽位"),
                                ),
                        )
                        .child(
                            v_flex()
                                .gap_2()
                                .items_end()
                                .child(
                                    Button::new("billing-check-in-btn")
                                        .icon(IconName::Calendar)
                                        .label("每日签到")
                                        .on_click(cx.listener(move |_this, _, _, cx| {
                                            let client = provider::cloud_client().clone();
                                            cx.spawn(
                                                move |weak: gpui::WeakEntity<AppSidebar>,
                                                 cx: &mut gpui::AsyncApp| {
                                                    let weak = weak.clone();
                                                    let mut cx = cx.clone();
                                                    let client = client.clone();
                                                    async move {
                                                        match client.check_in_essence().await {
                                                            Ok(res) => {
                                                                if let Some(e) = weak.upgrade() {
                                                                    let _ = e.update(
                                                                        &mut cx,
                                                                        |this, cx| {
                                                                            this.billing
                                                                                .check_in_result =
                                                                                Some(res.clone());
                                                                            this.billing.balance =
                                                                                Some(res.balance);
                                                                            cx.notify();
                                                                        },
                                                                    );
                                                                }
                                                            }
                                                            Err(e) => {
                                                                if let Some(e2) =
                                                                    weak.upgrade()
                                                                {
                                                                    let _ = e2.update(
                                                                        &mut cx,
                                                                        |this, cx| {
                                                                            this.billing.error =
                                                                                Some(e.to_string());
                                                                            cx.notify();
                                                                        },
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                },
                                            )
                                            .detach();
                                        })),
                                )
                                .when_some(check_in_result.as_ref(), |d, res| {
                                    d.child(div().text_xs().child(
                                        if res.already_checked_in {
                                            "今天已签到。".to_string()
                                        } else {
                                            format!("+{} BE 已发放", res.granted)
                                        },
                                    ))
                                }),
                        ),
                )
                // ── 分隔线 ──
                .child(div().w_full().h_px().bg(cx.theme().border))
                // ── 订阅套餐 ──
                .child(
                    v_flex()
                        .gap_4()
                        .child(
                            v_flex()
                                .gap_1()
                                .child(div().text_sm().font_bold().child("订阅套餐"))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("订阅可获得每月精粹补贴与更多 Agent 槽位"),
                                ),
                        )
                        .child(
                            h_flex()
                                .gap_4()
                                .w_full()
                                .children(plans.iter().map(|p| {
                                    let is_current =
                                        current_plan.as_ref().map_or(false, |cp| cp.id == p.id);
                                    let is_subscribing =
                                        subscribing.as_ref().map_or(false, |s| s == &p.id);
                                    let plan_id = p.id.clone();

                                    div()
                                        .flex_1()
                                        .rounded_lg()
                                        .border_1()
                                        .border_color(if is_current {
                                            cx.theme().accent
                                        } else {
                                            cx.theme().border
                                        })
                                        .p_5()
                                        .flex()
                                        .flex_col()
                                        .gap_3()
                                        .child(
                                            h_flex()
                                                .items_start()
                                                .justify_between()
                                                .child(
                                                    v_flex()
                                                        .gap_0p5()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_bold()
                                                                .child(p.name.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(
                                                                    cx.theme().muted_foreground,
                                                                )
                                                                .child(fmt_price(p.price_cents)),
                                                        ),
                                                )
                                                .when(is_current, |d| {
                                                    d.child(
                                                        div()
                                                            .px_2()
                                                            .py_0p5()
                                                            .rounded_md()
                                                            .bg(cx.theme().accent.opacity(0.15))
                                                            .text_color(cx.theme().accent)
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("当前"),
                                                    )
                                                }),
                                        )
                                        .child(
                                            v_flex().gap_1p5().children(
                                                [
                                                    format!("{} 个 Agent 槽位", p.agent_limit),
                                                    format!("每月 {} BE", p.monthly_essence),
                                                    "不限对局局数".to_string(),
                                                ]
                                                .into_iter()
                                                .map(|text| {
                                                    h_flex()
                                                        .gap_1p5()
                                                        .items_center()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(IconName::Check)
                                                        .child(text)
                                                }),
                                            ),
                                        )
                                        .when(!is_current, |d| {
                                            let pid = plan_id.clone();
                                            d.child(
                                                Button::new(format!("subscribe-{}", pid))
                                                    .outline()
                                                    .icon(IconName::Star)
                                                    .label(if is_subscribing {
                                                        "处理中…".to_string()
                                                    } else {
                                                        "选择此套餐".to_string()
                                                    })
                                                    .when(is_subscribing, |b| b.disabled(true))
                                                    .on_click(cx.listener(
                                                        move |this, _, _, cx| {
                                                            this.billing.subscribing =
                                                                Some(pid.clone());
                                                            this.billing.error = None;
                                                            let client =
                                                                provider::cloud_client().clone();
                                                            let pid2 = pid.clone();
                                                            cx.spawn(
                                                                move |weak: gpui::WeakEntity<
                                                                    AppSidebar,
                                                                >,
                                                                 cx: &mut gpui::AsyncApp| {
                                                                    let weak = weak.clone();
                                                                    let mut cx = cx.clone();
                                                                    let client = client.clone();
                                                                    let pid = pid2.clone();
                                                                    async move {
                                                                        match client
                                                                            .subscribe(&pid)
                                                                            .await
                                                                        {
                                                                            Ok(()) => {
                                                                                let (
                                                                                    bal,
                                                                                    txs,
                                                                                    plans,
                                                                                    cur,
                                                                                ) = tokio::join!(
                                                                                    async { client.get_essence_balance().await },
                                                                                    async { client.get_essence_transactions(50, 0).await },
                                                                                    async { client.list_billing_plans().await },
                                                                                    async { client.get_current_subscription().await },
                                                                                );
                                                                                if let Some(e) =
                                                                                    weak.upgrade()
                                                                                {
                                                                                    let _ = e.update(
                                                                                        &mut cx,
                                                                                        |this, cx| {
                                                                                            this.billing.balance = bal.ok();
                                                                                            this.billing.transactions = txs.unwrap_or_default();
                                                                                            this.billing.plans = plans.unwrap_or_default();
                                                                                            this.billing.current_plan = cur.ok();
                                                                                            this.billing.subscribing = None;
                                                                                            this.billing.loading = false;
                                                                                            cx.notify();
                                                                                        },
                                                                                    );
                                                                                }
                                                                            }
                                                                            Err(e) => {
                                                                                if let Some(e2) =
                                                                                    weak.upgrade()
                                                                                {
                                                                                    let _ = e2.update(
                                                                                        &mut cx,
                                                                                        |this, cx| {
                                                                                            this.billing.error =
                                                                                                Some(e.to_string());
                                                                                            this.billing.subscribing = None;
                                                                                            cx.notify();
                                                                                        },
                                                                                    );
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                },
                                                            )
                                                            .detach();
                                                        },
                                                    )),
                                            )
                                        })
                                        .into_any_element()
                                })),
                        ),
                )
                // ── 分隔线 ──
                .child(div().w_full().h_px().bg(cx.theme().border))
                // ── 精粹流水 ──
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .child(div().text_sm().font_bold().child("精粹流水"))
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .text_xs()
                                        .font_bold()
                                        .bg(cx.theme().accent.opacity(0.15))
                                        .text_color(cx.theme().accent)
                                        .child(format!("{}", tx_count)),
                                ),
                        )
                        .when(!loading && tx_empty, |d| {
                            d.child(
                                div()
                                    .py_8()
                                    .w_full()
                                    .text_center()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("暂无流水"),
                            )
                        })
                        .when(!tx_empty, |d| {
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
                                                            .child("类型"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(6.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .text_right()
                                                            .child("变动"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(10.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .text_right()
                                                            .child("时间"),
                                                    ),
                                            )
                                            .children(tx_children),
                                    ),
                            )
                        }),
                )
                // ── Loading indicator ──
                .when(loading && balance.is_none(), |d| {
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
                .into_any_element(),
        )
        .into_any_element()
}
