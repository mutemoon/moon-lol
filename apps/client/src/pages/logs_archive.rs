use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use lol_web_protocol::match_::{Match, MatchStatus};
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::services::runtime::run_on_tokio;
use crate::services::types::{LogCategory, LogEntity, LogQueryParams, LogRow, QueryLogsResult};
use crate::services::{log_service, provider};

// ── 页面本地状态 ──

struct LogsArchiveState {
    // 查询面板
    game_id: String,
    levels: Vec<String>,
    entity_id: Option<i64>,
    category: Option<String>,
    search_text: Option<String>,
    offset: i64,
    limit: i64,
    results: Option<QueryLogsResult>,
    entities: Vec<LogEntity>,
    categories: Vec<LogCategory>,
    loading: bool,
    error: Option<String>,
    // 我的对局（24h）
    matches: Vec<Match>,
    matches_loaded: bool,
    matches_loading: bool,
    matches_error: Option<String>,
    downloading: Option<String>,
    download_msg: Option<String>,
    // 加载本地 SQLite
    local_path: String,
    local_size: Option<u64>,
    local_msg: Option<String>,
}

impl Default for LogsArchiveState {
    fn default() -> Self {
        Self {
            game_id: String::new(),
            levels: Vec::new(),
            entity_id: None,
            category: None,
            search_text: None,
            offset: 0,
            limit: 50,
            results: None,
            entities: Vec::new(),
            categories: Vec::new(),
            loading: false,
            error: None,
            matches: Vec::new(),
            matches_loaded: false,
            matches_loading: false,
            matches_error: None,
            downloading: None,
            download_msg: None,
            local_path: String::new(),
            local_size: None,
            local_msg: None,
        }
    }
}

thread_local! {
    static STATE: RefCell<LogsArchiveState> = RefCell::new(LogsArchiveState::default());
    /// 手写输入框的焦点与光标（按 id 区分，跨渲染保持）。
    static EDITS: RefCell<HashMap<String, EditMeta>> = RefCell::new(HashMap::new());
}

fn with_state<R>(f: impl FnOnce(&LogsArchiveState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

fn update_state(f: impl FnOnce(&mut LogsArchiveState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

fn _level_color(level: &str, cx: &mut Context<AppSidebar>) -> Hsla {
    match level {
        "DEBUG" => cx.theme().muted_foreground,
        "INFO" => cx.theme().accent,
        "WARN" => hsla(40.0 / 360.0, 0.9, 0.5, 1.0),
        "ERROR" => cx.theme().danger,
        _ => cx.theme().muted_foreground,
    }
}

// ── 手写输入框（gpui_component Input 需要 &mut Window，这里无法持有）──

#[derive(Clone)]
struct EditMeta {
    cursor: usize,
    focus: FocusHandle,
}

fn edit_meta(id: &str, cx: &App) -> EditMeta {
    EDITS.with(|m| {
        let mut m = m.borrow_mut();
        if let Some(meta) = m.get(id) {
            return meta.clone();
        }
        let meta = EditMeta {
            cursor: 0,
            focus: cx.focus_handle(),
        };
        m.insert(id.to_string(), meta.clone());
        meta
    })
}

fn edit_cursor(id: &str) -> usize {
    EDITS.with(|m| m.borrow().get(id).map_or(0, |e| e.cursor))
}

fn set_edit_cursor(id: &str, cursor: usize) {
    EDITS.with(|m| {
        if let Some(e) = m.borrow_mut().get_mut(id) {
            e.cursor = cursor;
        }
    })
}

/// 处理单个按键，返回（新文本，新光标）。无变化返回 None。
fn apply_key(value: &str, cursor: usize, event: &KeyDownEvent) -> Option<(String, usize)> {
    let ks = &event.keystroke;
    let mods = &ks.modifiers;
    let mut chars: Vec<char> = value.chars().collect();
    let cursor = cursor.min(chars.len());

    if mods.control || mods.platform {
        return None;
    }

    if let Some(ch) = ks.key_char.as_deref() {
        let insert_chars: Vec<char> = ch.chars().collect();
        if !mods.alt && !insert_chars.is_empty() && !insert_chars.iter().any(|c| c.is_control()) {
            for (i, c) in insert_chars.iter().enumerate() {
                chars.insert(cursor + i, *c);
            }
            return Some((chars.into_iter().collect(), cursor + insert_chars.len()));
        }
    }

    match ks.key.as_str() {
        "backspace" => {
            if cursor > 0 {
                chars.remove(cursor - 1);
                Some((chars.into_iter().collect(), cursor - 1))
            } else {
                None
            }
        }
        "delete" => {
            if cursor < chars.len() {
                chars.remove(cursor);
                Some((chars.into_iter().collect(), cursor))
            } else {
                None
            }
        }
        "left" => Some((value.to_string(), cursor.saturating_sub(1))),
        "right" => Some((value.to_string(), (cursor + 1).min(chars.len()))),
        "home" => Some((value.to_string(), 0)),
        "end" => Some((value.to_string(), chars.len())),
        "space" => {
            chars.insert(cursor, ' ');
            Some((chars.into_iter().collect(), cursor + 1))
        }
        _ => None,
    }
}

/// 可聚焦、可键盘编辑的文本输入框，读写本页面 thread_local 状态。
fn render_text_input(
    cx: &mut Context<AppSidebar>,
    id: &str,
    placeholder: &str,
    get_value: impl Fn() -> String + 'static,
    set_value: impl Fn(String) + 'static,
) -> AnyElement {
    let value = get_value();
    let meta = edit_meta(id, cx);
    let focus_handle = meta.focus.clone();
    let empty = value.is_empty();
    let chars: Vec<char> = value.chars().collect();
    let cursor = meta.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let id_owned = id.to_string();

    let listener = cx.listener(move |_this, event: &KeyDownEvent, _window, cx| {
        let live = get_value();
        let cur = edit_cursor(&id_owned);
        if let Some((nv, nc)) = apply_key(&live, cur, event) {
            set_value(nv);
            set_edit_cursor(&id_owned, nc);
            cx.notify();
        }
    });

    div()
        .track_focus(&focus_handle)
        .px_2()
        .py_1()
        .w_full()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_md()
        .bg(cx.theme().background)
        .text_sm()
        .on_key_down(listener)
        .child(
            h_flex()
                .items_center()
                .when(empty, |d| {
                    d.text_color(muted).child(placeholder.to_string())
                })
                .when(!empty, |d| {
                    d.child(before)
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(after)
                }),
        )
        .into_any_element()
}

// ── 工具函数 ──

/// 云端 API 基础地址：优先 VITE_BASE_URL，缺省对齐 cloud.rs 的 127.0.0.1:8080。
fn api_base_url() -> String {
    std::env::var("VITE_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string())
        .trim_end_matches('/')
        .to_string()
}

/// 下载目录：%APPDATA%/moon-lol/matches/，无 APPDATA 时回退 .moon-lol/matches。
fn matches_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("moon-lol").join("matches")
    } else {
        PathBuf::from(".moon-lol").join("matches")
    }
}

fn status_label(s: &MatchStatus) -> &str {
    match s {
        MatchStatus::Pending => "待开始",
        MatchStatus::Running => "进行中",
        MatchStatus::Paused => "已暂停",
        MatchStatus::Finished => "已结束",
        MatchStatus::Aborted => "已中止",
    }
}

fn fmt_date(iso: &str) -> String {
    if iso.len() >= 16 {
        iso[..16].replace('T', " ")
    } else {
        iso.to_string()
    }
}

// ── 异步动作 ──

/// 拉取「我的对局」（24h）列表，按 created_at 倒序。
fn load_matches(cx: &mut Context<AppSidebar>) {
    update_state(|s| {
        s.matches_loading = true;
        s.matches_error = None;
    });
    let client = provider::cloud_client().clone();
    let _weak = cx.entity().downgrade();
    cx.spawn(
        move |_weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = _weak.clone();
            let mut cx2 = cx.clone();
            async move {
                let result = client.list_my_matches().await;
                update_state(|s| {
                    s.matches_loading = false;
                    s.matches_loaded = true;
                    match result {
                        Ok(mut list) => {
                            list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                            s.matches = list;
                        }
                        Err(e) => s.matches_error = Some(e.to_string()),
                    }
                });
                if let Some(e) = weak.upgrade() {
                    let _ = e.update(&mut cx2, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

/// 下载指定对局的 SQLite 日志 DB（GET /api/matches/{id}/log-db），
/// 存到 %APPDATA%/moon-lol/matches/。cloud.rs 无此方法，这里用 reqwest 直连并桥接 tokio。
fn download_match_db(cx: &mut Context<AppSidebar>, match_id: &str) {
    let match_id = match_id.to_string();
    let short: String = match_id.chars().take(8).collect();
    update_state(|s| {
        s.downloading = Some(match_id.clone());
        s.download_msg = None;
    });
    let url = format!("{}/api/matches/{}/log-db", api_base_url(), match_id);
    let token = provider::cloud_client().get_token();
    let dir = matches_dir();
    let dest = dir.join(format!("match-{}.sqlite", short));
    let _weak = cx.entity().downgrade();
    cx.spawn(
        move |_weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = _weak.clone();
            let mut cx2 = cx.clone();
            let url = url.clone();
            let dest = dest.clone();
            let dir = dir.clone();
            let token = token.clone();
            async move {
                let result = run_on_tokio(move || async move {
                    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                    let client = reqwest::Client::new();
                    let mut req = client.get(&url);
                    if let Some(t) = &token {
                        req = req.header("Authorization", format!("Bearer {}", t));
                    }
                    let resp = req.send().await.map_err(|e| e.to_string())?;
                    if !resp.status().is_success() {
                        return Err(format!("下载失败：HTTP {}", resp.status()));
                    }
                    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
                    fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
                    Ok(dest.display().to_string())
                })
                .await;
                update_state(|s| {
                    s.downloading = None;
                    s.download_msg = Some(match result {
                        Ok(path) => format!("已下载到 {}", path),
                        Err(e) => e,
                    });
                });
                if let Some(e) = weak.upgrade() {
                    let _ = e.update(&mut cx2, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

/// 校验本地 .sqlite 路径并展示大小（不做真正回放，回放属后续 debug 页 wave）。
fn load_local_sqlite(cx: &mut Context<AppSidebar>) {
    let path = with_state(|s| s.local_path.trim().to_string());
    if path.is_empty() {
        update_state(|s| s.local_msg = Some("请输入 .sqlite 文件路径".to_string()));
        cx.notify();
        return;
    }
    let p = Path::new(&path);
    let ext_ok = p
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("sqlite") || e.eq_ignore_ascii_case("db"));
    let size = p.metadata().ok().map(|m| m.len());
    let (size, msg) = if ext_ok {
        match size {
            Some(sz) => (Some(sz), format!("已加载 {} bytes", sz)),
            None => (None, "文件不存在".to_string()),
        }
    } else {
        (None, "不是 .sqlite 文件".to_string())
    };
    update_state(|s| {
        s.local_size = size;
        s.local_msg = Some(msg);
    });
    cx.notify();
}

// ── 查询面板动作（沿用原实现）──

fn do_query(cx: &mut Context<AppSidebar>) {
    let gid = with_state(|s| s.game_id.clone());
    if gid.is_empty() {
        return;
    }

    let params = with_state(|s| LogQueryParams {
        offset: s.offset,
        limit: s.limit,
        levels: if s.levels.is_empty() {
            None
        } else {
            Some(s.levels.clone())
        },
        entity_id: s.entity_id,
        category: s.category.clone(),
        search_text: s.search_text.clone(),
    });
    let _weak = cx.entity().downgrade();
    cx.spawn(
        move |_weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak2 = _weak.clone();
            let mut cx2 = cx.clone();
            let gid = gid.clone();
            let params = params.clone();
            async move {
                let results = log_service::query_logs(&gid, &params).await;
                update_state(|s| {
                    s.loading = false;
                    match results {
                        Ok(r) => s.results = Some(r),
                        Err(e) => s.error = Some(e),
                    }
                });
                if let Some(e) = weak2.upgrade() {
                    let _ = e.update(&mut cx2, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

fn do_load_logs(cx: &mut Context<AppSidebar>) {
    let gid = with_state(|s| s.game_id.clone());
    if gid.is_empty() {
        return;
    }
    let params = with_state(|s| LogQueryParams {
        offset: s.offset,
        limit: s.limit,
        levels: if s.levels.is_empty() {
            None
        } else {
            Some(s.levels.clone())
        },
        entity_id: s.entity_id,
        category: s.category.clone(),
        search_text: s.search_text.clone(),
    });
    let _weak = cx.entity().downgrade();
    cx.spawn(
        move |_weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak2 = _weak.clone();
            let mut cx2 = cx.clone();
            let gid = gid.clone();
            let params = params.clone();
            async move {
                let results = log_service::query_logs(&gid, &params).await;
                let entities = log_service::query_log_entities(&gid).await;
                let categories = log_service::query_log_categories(&gid).await;
                update_state(|s| {
                    s.loading = false;
                    match results {
                        Ok(r) => s.results = Some(r),
                        Err(e) => s.error = Some(e),
                    }
                    if let Ok(e) = entities {
                        s.entities = e;
                    }
                    if let Ok(c) = categories {
                        s.categories = c;
                    }
                });
                if let Some(e) = weak2.upgrade() {
                    let _ = e.update(&mut cx2, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

// ── 子区块渲染 ──

/// 「我的对局」单行：id / 模式 / 状态 / 开始时间 / 操作。
fn render_match_row(
    m: &Match,
    downloading: &Option<String>,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let mid = m.id.to_string();
    let short: String = mid.chars().take(8).collect();
    let mode = m.mode.clone();
    let status = status_label(&m.status).to_string();
    let created = fmt_date(&m.created_at);
    let is_downloading = downloading.as_ref() == Some(&mid);
    let border = cx.theme().border;
    let muted = cx.theme().muted_foreground;
    let accent = cx.theme().accent;
    h_flex()
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(border.opacity(0.3))
        .items_center()
        .child(div().w(rems(9.)).text_xs().child(short))
        .child(
            div().w(rems(7.)).child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_md()
                    .bg(accent.opacity(0.15))
                    .text_color(accent)
                    .text_xs()
                    .font_bold()
                    .child(mode),
            ),
        )
        .child(div().w(rems(6.)).text_xs().child(status))
        .child(
            div()
                .w(rems(14.))
                .text_xs()
                .text_color(muted)
                .child(created),
        )
        .child(
            h_flex()
                .flex_1()
                .gap_1p5()
                .justify_end()
                .items_center()
                .child({
                    let mid2 = mid.clone();
                    Button::new(format!("logs-use-{}", mid))
                        .ghost()
                        .label("查询")
                        .on_click(cx.listener(move |_, _, _, cx| {
                            update_state(|s| s.game_id = mid2.clone());
                            set_edit_cursor("logs-game-id", mid2.len());
                            cx.notify();
                        }))
                })
                .child({
                    let mid2 = mid.clone();
                    Button::new(format!("logs-dl-{}", mid))
                        .outline()
                        .icon(IconName::ArrowDown)
                        .label(if is_downloading {
                            "下载中…".to_string()
                        } else {
                            "下载 DB".to_string()
                        })
                        .when(is_downloading, |b| b.disabled(true))
                        .on_click(cx.listener(move |_, _, _, cx| {
                            download_match_db(cx, &mid2);
                        }))
                }),
        )
        .into_any_element()
}

// ── 公开入口 ──

/// 日志归档：
/// 1) 我的对局（24h）列表 + 下载 SQLite DB；
/// 2) 加载本地 .sqlite 校验大小；
/// 3) 按 game_id 查询日志（级别/实体/类别筛选、分页、清空）。
pub fn render_logs_archive(_sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    // 首次渲染触发拉取对局列表
    if !with_state(|s| s.matches_loaded) && !with_state(|s| s.matches_loading) {
        load_matches(cx);
    }

    // Read individual fields without cloning the whole state
    let game_id = with_state(|s| s.game_id.clone());
    let game_id_empty = game_id.is_empty();
    let levels = with_state(|s| s.levels.clone());
    let entities = with_state(|s| s.entities.len());
    let entities_empty = entities == 0;
    let categories = with_state(|s| s.categories.len());
    let categories_empty = categories == 0;
    let loading = with_state(|s| s.loading);
    let error = with_state(|s| s.error.clone());
    let has_results = with_state(|s| s.results.is_some());
    let total_count = with_state(|s| s.results.as_ref().map_or(0, |r| r.total_count));
    let total_pages = with_state(|s| {
        let total = s.results.as_ref().map_or(0, |r| r.total_count);
        if s.limit > 0 {
            ((total as f64) / (s.limit as f64)).ceil() as i64
        } else {
            0
        }
    });
    let current_page = with_state(|s| {
        if s.limit > 0 {
            s.offset / s.limit + 1
        } else {
            1
        }
    });

    let matches = with_state(|s| s.matches.clone());
    let matches_loading = with_state(|s| s.matches_loading);
    let matches_error = with_state(|s| s.matches_error.clone());
    let downloading = with_state(|s| s.downloading.clone());
    let download_msg = with_state(|s| s.download_msg.clone());
    let local_size = with_state(|s| s.local_size);
    let local_msg = with_state(|s| s.local_msg.clone());

    let entity_list: Vec<LogEntity> = STATE.with(|s| {
        let s = s.borrow();
        s.entities
            .iter()
            .map(|e| LogEntity {
                entity_id: e.entity_id,
                entity_name: e.entity_name.clone(),
            })
            .collect()
    });
    let category_list: Vec<LogCategory> = STATE.with(|s| {
        let s = s.borrow();
        s.categories
            .iter()
            .map(|c| LogCategory {
                category: c.category.clone(),
            })
            .collect()
    });
    let rows_data: Vec<LogRow> = STATE.with(|s| {
        let s = s.borrow();
        s.results.as_ref().map_or(Vec::new(), |r| {
            r.rows
                .iter()
                .map(|row| LogRow {
                    id: row.id,
                    timestamp: row.timestamp,
                    level: row.level.clone(),
                    file: row.file.clone(),
                    line: row.line,
                    entity_id: row.entity_id,
                    entity_name: row.entity_name.clone(),
                    category: row.category.clone(),
                    message: row.message.clone(),
                })
                .collect()
        })
    });

    div()
        .size_full()
        .flex_1()
        .overflow_hidden()
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .gap_4()
                .overflow_y_scrollbar()
                .p_6()
                // ── 标题行 ──
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(IconName::HardDrive)
                                .child(
                                    div()
                                        .font_bold()
                                        .text_lg()
                                        .child(t!("app.nav.title_logs_archive")),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(
                                "服务器保留最近 24 小时对局日志，可下载为 SQLite 库离线分析，或按级别/实体/类别筛选查询。",
                            ),
                        ),
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
                // ── 我的对局（24h）──
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    h_flex()
                                        .gap_1p5()
                                        .items_center()
                                        .child(IconName::Inbox)
                                        .child(div().text_sm().font_bold().child("我的对局（24h）"))
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_md()
                                                .text_xs()
                                                .font_bold()
                                                .bg(cx.theme().accent.opacity(0.15))
                                                .text_color(cx.theme().accent)
                                                .child(format!("{}", matches.len())),
                                        ),
                                )
                                .child(
                                    Button::new("logs-matches-refresh")
                                        .ghost()
                                        .icon(IconName::Redo2)
                                        .label("刷新")
                                        .when(matches_loading, |b| b.disabled(true))
                                        .on_click(cx.listener(|_, _, _, cx| load_matches(cx))),
                                ),
                        )
                        .when_some(matches_error.as_ref(), |d, err| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().danger)
                                    .child(err.clone()),
                            )
                        })
                        .when(matches_loading, |d| {
                            d.child(
                                div()
                                    .py_6()
                                    .w_full()
                                    .text_center()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("加载对局列表…"),
                            )
                        })
                        .when(!matches_loading && matches.is_empty(), |d| {
                            d.child(
                                div()
                                    .py_6()
                                    .w_full()
                                    .text_center()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("近 24 小时没有对局记录"),
                            )
                        })
                        .when(!matches.is_empty(), |d| {
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
                                                            .w(rems(9.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("对局"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(7.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("模式"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(6.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("状态"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(14.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("开始"),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .text_xs()
                                                            .font_bold()
                                                            .text_right()
                                                            .child("操作"),
                                                    ),
                                            )
                                            .children(
                                                matches.iter().map(|m| {
                                                    render_match_row(m, &downloading, cx)
                                                }),
                                            ),
                                    ),
                            )
                        })
                        .when_some(download_msg.as_ref(), |d, msg| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().accent)
                                    .child(msg.clone()),
                            )
                        }),
                )
                // ── 加载本地 SQLite ──
                .child(
                    div()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().border)
                        .p_4()
                        .child(
                            v_flex()
                                .gap_3()
                                .child(
                                    h_flex()
                                        .items_center()
                                        .justify_between()
                                        .flex_wrap()
                                        .child(
                                            v_flex()
                                                .gap_1()
                                                .child(
                                                    h_flex()
                                                        .gap_1p5()
                                                        .items_center()
                                                        .child(IconName::File)
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_bold()
                                                                .child("加载本地 SQLite"),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(
                                                            if let Some(sz) = local_size {
                                                                format!(
                                                                    "已加载 {} bytes（{:.1} MB）",
                                                                    sz,
                                                                    sz as f64
                                                                        / 1024.0
                                                                        / 1024.0
                                                                )
                                                            } else {
                                                                "输入之前下载的 .sqlite 文件路径，校验文件存在并显示大小（回放分析将在后续版本提供）".to_string()
                                                            },
                                                        ),
                                                ),
                                        )
                                        .child(
                                            h_flex()
                                                .gap_2()
                                                .items_center()
                                                .child(
                                                    div().w(rems(24.)).child(render_text_input(
                                                        cx,
                                                        "logs-local-path",
                                                        "输入 .sqlite 文件路径…",
                                                        || {
                                                            with_state(|s| {
                                                                s.local_path.clone()
                                                            })
                                                        },
                                                        |v| {
                                                            update_state(|s| {
                                                                s.local_path = v;
                                                                s.local_size = None;
                                                                s.local_msg = None;
                                                            });
                                                        },
                                                    )),
                                                )
                                                .child(
                                                    Button::new("logs-local-load")
                                                        .icon(IconName::Play)
                                                        .label("加载")
                                                        .on_click(cx.listener(|_, _, _, cx| {
                                                            load_local_sqlite(cx);
                                                        })),
                                                )
                                                .child(
                                                    Button::new("logs-local-clear")
                                                        .ghost()
                                                        .label("清除")
                                                        .on_click(cx.listener(|_, _, _, cx| {
                                                            update_state(|s| {
                                                                s.local_path.clear();
                                                                s.local_size = None;
                                                                s.local_msg = None;
                                                            });
                                                            set_edit_cursor("logs-local-path", 0);
                                                            cx.notify();
                                                        })),
                                                ),
                                        ),
                                )
                                .when_some(local_msg.as_ref(), |d, msg| {
                                    d.child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(msg.clone()),
                                    )
                                }),
                        ),
                )
                // ── 分隔线 ──
                .child(div().w_full().h_px().bg(cx.theme().border))
                // ── Game ID 输入与操作行 ──
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .flex_wrap()
                        .child(
                            h_flex()
                                .gap_1p5()
                                .items_center()
                                .child(div().text_xs().font_bold().child("对局 ID"))
                                .child(
                                    div().w(rems(24.)).child(render_text_input(
                                        cx,
                                        "logs-game-id",
                                        "输入对局 ID（或从上方列表点击「查询」）",
                                        || with_state(|s| s.game_id.clone()),
                                        |v| update_state(|s| s.game_id = v),
                                    )),
                                ),
                        )
                        .child({
                            let gid = game_id.clone();
                            Button::new("logs-load-btn")
                                .icon(IconName::Search)
                                .label("加载日志")
                                .when(gid.is_empty(), |b| b.disabled(true))
                                .on_click(cx.listener(move |_this, _, _, cx| {
                                    if gid.is_empty() {
                                        return;
                                    }
                                    update_state(|s| {
                                        s.loading = true;
                                        s.error = None;
                                        s.results = None;
                                        s.entities.clear();
                                        s.categories.clear();
                                    });
                                    do_load_logs(cx);
                                }))
                        }),
                )
                // ── 筛选控制行 ──
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .flex_wrap()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("级别:"),
                        )
                        .children(["DEBUG", "INFO", "WARN", "ERROR"].iter().map(|lv| {
                            let lvl = lv.to_string();
                            let selected = levels.contains(&lvl);
                            let btn = Button::new(format!("logs-level-{}", lvl)).label(lvl.clone());
                            if selected {
                                let lvl2 = lvl.clone();
                                btn.on_click(cx.listener(move |_this, _, _, cx| {
                                    update_state(|s| s.levels.retain(|l| l != &lvl2));
                                    cx.notify();
                                }))
                                .into_any_element()
                            } else {
                                let lvl2 = lvl.clone();
                                btn.ghost()
                                    .on_click(cx.listener(move |_this, _, _, cx| {
                                        update_state(|s| {
                                            if !s.levels.contains(&lvl2) {
                                                s.levels.push(lvl2.clone());
                                            }
                                        });
                                        cx.notify();
                                    }))
                                    .into_any_element()
                            }
                        })),
                )
                // Entity / Category 过滤器
                .when(!entities_empty || !categories_empty, |d| {
                    d.child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .flex_wrap()
                            .when(!entities_empty, |d2| {
                                d2.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("实体:"),
                                )
                                .children(entity_list.iter().map(|e| {
                                    let eid = e.entity_id;
                                    let label = e.entity_name.clone().unwrap_or_else(|| {
                                        format!("Entity {}", e.entity_id.unwrap_or(0))
                                    });
                                    let is_selected = with_state(|s| s.entity_id == eid);
                                    let btn =
                                        Button::new(format!("logs-entity-{}", eid.unwrap_or(0)))
                                            .label(label);
                                    if is_selected {
                                        btn.on_click(cx.listener(|_this, _, _, cx| {
                                            update_state(|s| s.entity_id = None);
                                            cx.notify();
                                        }))
                                        .into_any_element()
                                    } else {
                                        btn.ghost()
                                            .on_click(cx.listener(move |_this, _, _, cx| {
                                                update_state(|s| s.entity_id = eid);
                                                cx.notify();
                                            }))
                                            .into_any_element()
                                    }
                                }))
                            })
                            .when(!categories_empty, |d2| {
                                d2.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("类别:"),
                                )
                                .children(
                                    category_list.iter().map(|c| {
                                        let cat = c.category.clone().unwrap_or_default();
                                        let is_selected = with_state(|s| {
                                            s.category.as_ref().map_or(false, |sc| sc == &cat)
                                        });
                                        let btn = Button::new(format!("logs-cat-{}", cat))
                                            .label(cat.clone());
                                        if is_selected {
                                            btn.on_click(cx.listener(|_this, _, _, cx| {
                                                update_state(|s| s.category = None);
                                                cx.notify();
                                            }))
                                            .into_any_element()
                                        } else {
                                            let cat2 = cat.clone();
                                            btn.ghost()
                                                .on_click(cx.listener(move |_this, _, _, cx| {
                                                    update_state(|s| {
                                                        s.category = Some(cat2.clone())
                                                    });
                                                    cx.notify();
                                                }))
                                                .into_any_element()
                                        }
                                    }),
                                )
                            }),
                    )
                })
                // ── 操作按钮行 ──
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .justify_between()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("logs-search-btn")
                                        .icon(IconName::Search)
                                        .label("应用筛选")
                                        .when(game_id_empty, |b| b.disabled(true))
                                        .on_click(cx.listener(move |_this, _, _, cx| {
                                            update_state(|s| {
                                                s.offset = 0;
                                                s.loading = true;
                                                s.error = None;
                                            });
                                            do_query(cx);
                                        })),
                                )
                                .child(
                                    Button::new("logs-clear-btn")
                                        .ghost()
                                        .icon(IconName::Delete)
                                        .label("清空日志")
                                        .when(game_id_empty, |b| b.disabled(true))
                                        .on_click(cx.listener(move |_this, _, _, cx| {
                                            let gid2 = with_state(|s| s.game_id.clone());
                                            if gid2.is_empty() {
                                                return;
                                            }
                                            let _weak = cx.entity().downgrade();
                                            let gid3 = gid2.clone();
                                            cx.spawn(
                                                move |_weak: gpui::WeakEntity<AppSidebar>,
                                                 cx: &mut gpui::AsyncApp| {
                                                    let weak2 = _weak.clone();
                                                    let mut cx2 = cx.clone();
                                                    let gid4 = gid3.clone();
                                                    async move {
                                                        let result =
                                                            log_service::clear_logs(&gid4).await;
                                                        update_state(|s| match result {
                                                            Ok(()) => {
                                                                s.results = None;
                                                                s.entities.clear();
                                                                s.categories.clear();
                                                            }
                                                            Err(e) => s.error = Some(e),
                                                        });
                                                        if let Some(e) = weak2.upgrade() {
                                                            let _ = e.update(
                                                                &mut cx2,
                                                                |_, cx| cx.notify(),
                                                            );
                                                        }
                                                    }
                                                },
                                            )
                                            .detach();
                                        })),
                                ),
                        )
                        .when(has_results, |d| {
                            d.child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .text_xs()
                                    .font_bold()
                                    .bg(cx.theme().accent.opacity(0.15))
                                    .text_color(cx.theme().accent)
                                    .child(format!("{} 条", total_count)),
                            )
                        }),
                )
                // ── Loading ──
                .when(loading, |d| {
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
                // ── 日志表格 ──
                .when(has_results, |d| {
                    d.child(
                        div()
                            .rounded_lg()
                            .border_1()
                            .border_color(cx.theme().border)
                            .overflow_hidden()
                            .flex_1()
                            .child(
                                v_flex()
                                    .size_full()
                                    .child(
                                        h_flex()
                                            .px_4()
                                            .py_2()
                                            .border_b_1()
                                            .border_color(cx.theme().border)
                                            .bg(cx.theme().background)
                                            .child(
                                                div().w(rems(4.)).text_xs().font_bold().child("ID"),
                                            )
                                            .child(
                                                div()
                                                    .w(rems(5.))
                                                    .text_xs()
                                                    .font_bold()
                                                    .child("级别"),
                                            )
                                            .child(
                                                div()
                                                    .w(rems(10.))
                                                    .text_xs()
                                                    .font_bold()
                                                    .child("时间"),
                                            )
                                            .child(
                                                div()
                                                    .w(rems(8.))
                                                    .text_xs()
                                                    .font_bold()
                                                    .child("实体"),
                                            )
                                            .child(
                                                div()
                                                    .w(rems(6.))
                                                    .text_xs()
                                                    .font_bold()
                                                    .child("类别"),
                                            )
                                            .child(
                                                div().flex_1().text_xs().font_bold().child("消息"),
                                            ),
                                    )
                                    .child(div().flex_1().overflow_y_scrollbar().children(
                                        rows_data.iter().map(|row| render_log_row(cx, row)),
                                    )),
                            ),
                    )
                })
                // ── 分页 ──
                .when(total_pages > 1, |d| {
                    d.child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .justify_center()
                            .child(
                                Button::new("logs-prev-page")
                                    .ghost()
                                    .label("上一页")
                                    .when(current_page <= 1, |b| b.disabled(true))
                                    .on_click(cx.listener(|_this, _, _, cx| {
                                        update_state(|s| {
                                            s.offset = (s.offset - s.limit).max(0);
                                            s.loading = true;
                                        });
                                        do_query(cx);
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("第 {} / {} 页", current_page, total_pages)),
                            )
                            .child(
                                Button::new("logs-next-page")
                                    .ghost()
                                    .label("下一页")
                                    .when(current_page >= total_pages, |b| b.disabled(true))
                                    .on_click(cx.listener(|_this, _, _, cx| {
                                        update_state(|s| {
                                            s.offset += s.limit;
                                            s.loading = true;
                                        });
                                        do_query(cx);
                                    })),
                            ),
                    )
                })
                // ── 未加载占位 ──
                .when(!loading && !has_results && error.is_none(), |d| {
                    d.child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("需要先选择一个对局 ID，点击「加载日志」开始查询"),
                    )
                })
                .into_any_element(),
        )
        .into_any_element()
}

fn render_log_row(cx: &mut Context<AppSidebar>, row: &LogRow) -> AnyElement {
    let level = row.level.clone();
    let level_hsla = match level.as_str() {
        "DEBUG" => cx.theme().muted_foreground,
        "INFO" => cx.theme().accent,
        "WARN" => hsla(40.0 / 360.0, 0.9, 0.5, 1.0),
        "ERROR" => cx.theme().danger,
        _ => cx.theme().muted_foreground,
    };
    h_flex()
        .px_4()
        .py_1p5()
        .border_b_1()
        .border_color(cx.theme().border.opacity(0.3))
        .child(
            div()
                .w(rems(4.))
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{}", row.id)),
        )
        .child(
            div().w(rems(5.)).child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_md()
                    .text_xs()
                    .font_bold()
                    .bg(level_hsla.opacity(0.15))
                    .text_color(level_hsla)
                    .child(level),
            ),
        )
        .child(
            div()
                .w(rems(10.))
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{}s", row.timestamp)),
        )
        .child(
            div()
                .w(rems(8.))
                .text_xs()
                .child(row.entity_name.clone().unwrap_or_else(|| {
                    row.entity_id
                        .map(|id| format!("#{}", id))
                        .unwrap_or_else(|| "—".to_string())
                })),
        )
        .child(
            div()
                .w(rems(6.))
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(row.category.clone().unwrap_or_else(|| "—".to_string())),
        )
        .child(div().flex_1().text_xs().child(row.message.clone()))
        .into_any_element()
}
