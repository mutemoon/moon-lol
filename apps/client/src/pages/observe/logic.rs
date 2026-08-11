//! 观战/回放页数据加载：首次拉取 + 增量事件轮询。

use gpui::*;
use lol_web_protocol::match_::MatchStatus;
use uuid::Uuid;

use crate::components::sidebar::AppSidebar;
use crate::services::provider;
use crate::services::runtime::run_on_tokio;
use crate::types::ActiveView;

/// 每次轮询拉取的事件条数
const EVENTS_LIMIT: u32 = 200;
/// 时间线保留的最大事件数（防长时间轮询内存无限增长）
const MAX_EVENTS: usize = 1000;
/// 轮询间隔（秒）
const POLL_INTERVAL_SECS: u64 = 1;

/// 拉取对局信息 + 增量事件并写回状态。
pub(super) async fn fetch_delta(
    id: Uuid,
    weak: &gpui::WeakEntity<AppSidebar>,
    cx: &mut gpui::AsyncApp,
) {
    // 中途已切换对局则丢弃本次结果
    let cur_match_id = match weak.upgrade() {
        Some(e) => e.read_with(cx, |this, _| this.observe.match_id),
        None => return,
    };
    if cur_match_id != Some(id) {
        return;
    }
    let client = provider::cloud_client().clone();
    let id_str = id.to_string();

    // 对局信息：未加载或仍处于运行中时刷新
    let need_match = match weak.upgrade() {
        Some(e) => e.read_with(cx, |this, _| {
            this.observe
                .match_info
                .as_ref()
                .map_or(true, |m| m.status == MatchStatus::Running)
        }),
        None => return,
    };
    if need_match {
        match client.get_match(&id_str).await {
            Ok(m) => {
                weak.update(cx, |this, _| {
                    this.observe.match_info = Some(m);
                    this.observe.loading = false;
                })
                .ok();
            }
            Err(e) => {
                weak.update(cx, |this, _| {
                    this.observe.error = Some(e.to_string());
                    this.observe.loading = false;
                })
                .ok();
            }
        }
    }

    // 增量拉取事件（每次 200 条）
    let from_seq = match weak.upgrade() {
        Some(e) => e.read_with(cx, |this, _| this.observe.last_seq),
        None => return,
    };
    match client
        .get_match_events(&id_str, from_seq, EVENTS_LIMIT)
        .await
    {
        Ok(delta) => {
            if let Some(last) = delta.last() {
                let next_seq = last.seq as u32 + 1;
                weak.update(cx, |this, _| {
                    this.observe.events.extend(delta);
                    if this.observe.events.len() > MAX_EVENTS {
                        this.observe.events = this
                            .observe
                            .events
                            .split_off(this.observe.events.len() - MAX_EVENTS);
                    }
                    this.observe.last_seq = this.observe.last_seq.max(next_seq);
                    this.observe.loading = false;
                })
                .ok();
            } else {
                weak.update(cx, |this, _| this.observe.loading = false).ok();
            }
        }
        Err(e) => {
            weak.update(cx, |this, _| {
                this.observe.error = Some(e.to_string());
                this.observe.loading = false;
            })
            .ok();
        }
    }

    if let Some(entity) = weak.upgrade() {
        let _ = entity.update(cx, |_, cx| cx.notify());
    }
}

/// 首次加载。
pub(super) fn spawn_load(id: Uuid, cx: &mut Context<AppSidebar>) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                fetch_delta(id, &weak, &mut cx).await;
            }
        },
    )
    .detach();
}

/// 1 秒轮询事件增量。
/// 睡眠必须走 run_on_tokio（gpui executor 非 tokio runtime，直接 sleep 会 panic）。
pub(super) fn spawn_poll(cx: &mut Context<AppSidebar>) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                loop {
                    if run_on_tokio(|| async {
                        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS))
                            .await;
                        Ok::<(), String>(())
                    })
                    .await
                    .is_err()
                    {
                        break;
                    }
                    let (id, paused) = match weak.upgrade() {
                        Some(e) => {
                            e.read_with(&cx, |this, _| (this.observe.match_id, this.observe.paused))
                        }
                        None => break,
                    };
                    let Some(id) = id else { break };
                    if paused {
                        continue;
                    }
                    // 已离开观战页则停止轮询
                    let still_observing = weak.upgrade().map_or(false, |e| {
                        e.read_with(&cx, |s, _| s.active_view == ActiveView::Observe)
                    });
                    if !still_observing {
                        break;
                    }
                    fetch_delta(id, &weak, &mut cx).await;
                }
            }
        },
    )
    .detach();
}
