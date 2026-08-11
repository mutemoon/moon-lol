//! 日志归档页数据加载与处理：工具函数、异步动作、查询面板动作。

use std::fs;
use std::path::{Path, PathBuf};

use gpui::*;
use gpui_component::ActiveTheme;
use lol_web_protocol::match_::MatchStatus;

use crate::components::sidebar::AppSidebar;
use crate::services::runtime::run_on_tokio;
use crate::services::types::LogQueryParams;
use crate::services::{log_service, provider};

// ── 工具函数 ──

/// 云端 API 基础地址：优先 VITE_BASE_URL，缺省对齐 cloud.rs 的 127.0.0.1:8080。
pub(super) fn api_base_url() -> String {
    std::env::var("VITE_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string())
        .trim_end_matches('/')
        .to_string()
}

/// 下载目录：%APPDATA%/moon-lol/matches/，无 APPDATA 时回退 .moon-lol/matches。
pub(super) fn matches_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("moon-lol").join("matches")
    } else {
        PathBuf::from(".moon-lol").join("matches")
    }
}

pub(super) fn status_label(s: &MatchStatus) -> &str {
    match s {
        MatchStatus::Pending => "待开始",
        MatchStatus::Running => "进行中",
        MatchStatus::Paused => "已暂停",
        MatchStatus::Finished => "已结束",
        MatchStatus::Aborted => "已中止",
    }
}

pub(super) fn fmt_date(iso: &str) -> String {
    if iso.len() >= 16 {
        iso[..16].replace('T', " ")
    } else {
        iso.to_string()
    }
}

/// 级别颜色（当前未使用，保留）。
fn _level_color(level: &str, cx: &mut Context<AppSidebar>) -> Hsla {
    match level {
        "DEBUG" => cx.theme().muted_foreground,
        "INFO" => cx.theme().accent,
        "WARN" => hsla(40.0 / 360.0, 0.9, 0.5, 1.0),
        "ERROR" => cx.theme().danger,
        _ => cx.theme().muted_foreground,
    }
}

// ── 异步动作 ──

/// 拉取「我的对局」（24h）列表，按 created_at 倒序。
pub(super) fn load_matches(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    sidebar.logs_archive.matches_loading = true;
    sidebar.logs_archive.matches_error = None;
    let client = provider::cloud_client().clone();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = client.list_my_matches().await;
                this.update(&mut cx, |this, cx| {
                    this.logs_archive.matches_loading = false;
                    this.logs_archive.matches_loaded = true;
                    match result {
                        Ok(mut list) => {
                            list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                            this.logs_archive.matches = list;
                        }
                        Err(e) => this.logs_archive.matches_error = Some(e.to_string()),
                    }
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

/// 下载指定对局的 SQLite 日志 DB（GET /api/matches/{id}/log-db），
/// 存到 %APPDATA%/moon-lol/matches/。cloud.rs 无此方法，这里用 reqwest 直连并桥接 tokio。
pub(super) fn download_match_db(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    match_id: &str,
) {
    let match_id = match_id.to_string();
    let short: String = match_id.chars().take(8).collect();
    sidebar.logs_archive.downloading = Some(match_id.clone());
    sidebar.logs_archive.download_msg = None;
    let url = format!("{}/api/matches/{}/log-db", api_base_url(), match_id);
    let token = provider::cloud_client().get_token();
    let dir = matches_dir();
    let dest = dir.join(format!("match-{}.sqlite", short));
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
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
                this.update(&mut cx, |this, cx| {
                    this.logs_archive.downloading = None;
                    this.logs_archive.download_msg = Some(match result {
                        Ok(path) => format!("已下载到 {}", path),
                        Err(e) => e,
                    });
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

/// 校验本地 .sqlite 路径并展示大小（不做真正回放，回放属后续 debug 页 wave）。
pub(super) fn load_local_sqlite(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let path = sidebar.logs_archive.local_path.trim().to_string();
    if path.is_empty() {
        sidebar.logs_archive.local_msg = Some("请输入 .sqlite 文件路径".to_string());
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
    sidebar.logs_archive.local_size = size;
    sidebar.logs_archive.local_msg = Some(msg);
    cx.notify();
}

// ── 查询面板动作 ──

pub(super) fn do_query(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let gid = sidebar.logs_archive.game_id.clone();
    if gid.is_empty() {
        return;
    }

    let params = LogQueryParams {
        offset: sidebar.logs_archive.offset,
        limit: sidebar.logs_archive.limit,
        levels: if sidebar.logs_archive.levels.is_empty() {
            None
        } else {
            Some(sidebar.logs_archive.levels.clone())
        },
        entity_id: sidebar.logs_archive.entity_id,
        category: sidebar.logs_archive.category.clone(),
        search_text: sidebar.logs_archive.search_text.clone(),
    };
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            let gid = gid.clone();
            let params = params.clone();
            async move {
                let results = log_service::query_logs(&gid, &params).await;
                this.update(&mut cx, |this, cx| {
                    this.logs_archive.loading = false;
                    match results {
                        Ok(r) => this.logs_archive.results = Some(r),
                        Err(e) => this.logs_archive.error = Some(e),
                    }
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

pub(super) fn do_load_logs(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let gid = sidebar.logs_archive.game_id.clone();
    if gid.is_empty() {
        return;
    }
    let params = LogQueryParams {
        offset: sidebar.logs_archive.offset,
        limit: sidebar.logs_archive.limit,
        levels: if sidebar.logs_archive.levels.is_empty() {
            None
        } else {
            Some(sidebar.logs_archive.levels.clone())
        },
        entity_id: sidebar.logs_archive.entity_id,
        category: sidebar.logs_archive.category.clone(),
        search_text: sidebar.logs_archive.search_text.clone(),
    };
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            let gid = gid.clone();
            let params = params.clone();
            async move {
                let results = log_service::query_logs(&gid, &params).await;
                let entities = log_service::query_log_entities(&gid).await;
                let categories = log_service::query_log_categories(&gid).await;
                this.update(&mut cx, |this, cx| {
                    this.logs_archive.loading = false;
                    match results {
                        Ok(r) => this.logs_archive.results = Some(r),
                        Err(e) => this.logs_archive.error = Some(e),
                    }
                    if let Ok(e) = entities {
                        this.logs_archive.entities = e;
                    }
                    if let Ok(c) = categories {
                        this.logs_archive.categories = c;
                    }
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}
