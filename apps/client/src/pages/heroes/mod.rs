//! 英雄/选手管理页 — 对应 apps/client/src/pages/heroes.vue
//!
//! 编辑态字段全部可编辑：名称/英雄/类型/提示词/模型（provider 下拉 + 手动模型）、
//! RL 配置（model_path / inference_endpoint / reward_shaper）、Script 脚本、
//! JSON 导入导出、删除确认弹窗、上游 Fork diff 预览与「应用上游」。

mod browse;
mod edit;
mod input;
mod publish;
mod types;
mod utils;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, StyledExt};
use lol_web_protocol::agent::{Agent, CreateAgentDto, UpdateAgentDto};
use lol_web_protocol::spawn_preset::Visibility;
pub use types::HeroesState;

use self::browse::render_browse;
use self::edit::render_edit;
use self::types::{default_rewards, HeroesMode, HeroesTab, PLATFORM_PROVIDER_ID};
use self::utils::{apply_import_json, cfg_str, draft_config, export_json, pretty_config};
use crate::components::sidebar::AppSidebar;

// ── 主渲染函数 ──

pub fn render_heroes(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    // 首次进入自动加载英雄与快照
    if sidebar.heroes.agents.is_empty() && !sidebar.heroes.loading {
        sidebar.heroes.loading = true;
        let cloud = sidebar.cloud.clone();
        cx.spawn(
            |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                let this = this.clone();
                let mut cx = cx.clone();
                async move {
                    let agents = cloud.list_agents().await.unwrap_or_default();
                    use std::collections::HashMap;
                    let mut snapshots = HashMap::new();
                    for a in &agents {
                        if let Ok(snaps) = cloud.list_snapshots(&a.id.to_string()).await {
                            snapshots.insert(a.id, snaps);
                        }
                    }
                    this.update(&mut cx, |this, ctx| {
                        this.heroes.agents = agents;
                        this.heroes.snapshots = snapshots;
                        this.heroes.loading = false;
                        ctx.notify();
                    })
                    .ok();
                }
            },
        )
        .detach();
    }
    if sidebar.heroes.loading && sidebar.heroes.agents.is_empty() {
        return v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .child("加载中…")
            .into_any_element();
    }

    if matches!(sidebar.heroes.mode, HeroesMode::Browse) {
        render_browse(sidebar, cx)
    } else {
        render_edit(sidebar, window, cx)
    }
}

// ── 删除确认弹窗 ──

pub(super) fn render_delete_modal(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let name = sidebar.heroes.draft_name.clone();
    let deleting = sidebar.heroes.deleting;

    div()
        .absolute()
        .top_0()
        .bottom_0()
        .left_0()
        .right_0()
        .bg(rgba(0x00000073))
        .flex()
        .items_center()
        .justify_center()
        .child(
            v_flex()
                .w(px(380.))
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .p_6()
                .gap_4()
                .child(
                    div()
                        .text_lg()
                        .font_bold()
                        .child(format!("删除选手「{}」？", name)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("该操作不可撤销。引用此选手的场景槽位需手动重新选择。"),
                )
                .child(
                    h_flex()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new("delete-cancel")
                                .outline()
                                .label("取消")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.heroes.show_delete_confirm = false;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("delete-confirm")
                                .danger()
                                .label(if deleting { "删除中…" } else { "删除" })
                                .disabled(deleting)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    handle_delete(this, cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}

// ── 数据加载 ──

/// 首次进入编辑态时懒加载 model providers 与平台模型清单。
pub(super) fn ensure_providers_loaded(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    if sidebar.heroes.providers_loaded {
        return;
    }
    sidebar.heroes.providers_loaded = true;
    let cloud = sidebar.cloud.clone();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let providers = cloud.list_model_providers().await.unwrap_or_default();
                let platform = cloud.list_platform_models().await.unwrap_or_default();
                this.update(&mut cx, |this, cx| {
                    this.heroes.model_providers = providers;
                    this.heroes.platform_models = platform;
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

/// 加载当前编辑 agent 的上游 Agent（用于 diff 对照）。
fn spawn_load_upstream(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit {
            editing_id: Some(id),
        } => *id,
        _ => return,
    };
    let upstream_id = sidebar
        .heroes
        .agents
        .iter()
        .find(|a| a.id == editing_id)
        .and_then(|a| a.upstream_agent_id.or(a.forked_from));
    let Some(uid) = upstream_id else {
        return;
    };
    let cloud = sidebar.cloud.clone();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let up = cloud.get_agent(&uid.to_string()).await.ok();
                this.update(&mut cx, |this, cx| {
                    this.heroes.upstream_agent = up;
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

// ── 操作逻辑 ──

pub(super) fn start_new(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    sidebar.heroes = HeroesState {
        mode: HeroesMode::Edit { editing_id: None },
        ..Default::default()
    };
    ensure_providers_loaded(sidebar, cx);
    cx.notify();
}

/// 进入编辑态：用云端 Agent 填充草稿字段，并加载上游与 providers。
pub(super) fn enter_edit(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>, agent: &Agent) {
    let cfg = agent.config_json.clone();
    let td = cfg
        .as_ref()
        .and_then(|v| v.get("thinking_depth"))
        .and_then(|v| v.as_u64())
        .map_or(2, |n| n as u32);
    let provider_id = cfg
        .as_ref()
        .and_then(|v| v.get("provider_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| PLATFORM_PROVIDER_ID.to_string());

    let mut rewards = default_rewards();
    if let Some(rs) = cfg
        .as_ref()
        .and_then(|v| v.get("reward_shaper"))
        .and_then(|v| v.as_object())
    {
        for (k, v) in rs {
            if let Some(n) = v.as_f64() {
                rewards.insert(k.clone(), n);
            }
        }
    }

    sidebar.heroes.mode = HeroesMode::Edit {
        editing_id: Some(agent.id),
    };
    sidebar.heroes.draft_name = agent.name.clone();
    sidebar.heroes.draft_champion = agent.champion.clone();
    sidebar.heroes.draft_agent_type = agent.agent_type;
    sidebar.heroes.draft_prompt = agent.prompt.clone();
    sidebar.heroes.draft_model = agent.model.clone().unwrap_or_default();
    sidebar.heroes.draft_visibility = agent.visibility;
    sidebar.heroes.draft_thinking_depth = td;
    sidebar.heroes.draft_provider_id = provider_id;
    sidebar.heroes.draft_manual_model = false;
    sidebar.heroes.draft_rl_model_path = cfg_str(&cfg, "model_path");
    sidebar.heroes.draft_rl_endpoint = cfg_str(&cfg, "inference_endpoint");
    sidebar.heroes.draft_rl_rewards = rewards;
    sidebar.heroes.draft_script = cfg_str(&cfg, "script");
    sidebar.heroes.draft_config_json_str = pretty_config(&agent.config_json);
    sidebar.heroes.upstream_agent = None;
    sidebar.heroes.show_delete_confirm = false;
    sidebar.heroes.deleting = false;
    sidebar.heroes.error_msg.clear();
    sidebar.heroes.success_msg.clear();
    sidebar.heroes.selected_tab = HeroesTab::Config;
    ensure_providers_loaded(sidebar, cx);
    spawn_load_upstream(sidebar, cx);
    cx.notify();
}

pub(super) fn handle_save(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let name = sidebar.heroes.draft_name.trim().to_string();
    if name.is_empty() {
        sidebar.heroes.error_msg = "请填写选手名称".to_string();
        cx.notify();
        return;
    }

    let champion = sidebar.heroes.draft_champion.clone();
    let prompt = sidebar.heroes.draft_prompt.clone();
    let model = sidebar.heroes.draft_model.clone();
    let agent_type = sidebar.heroes.draft_agent_type;
    let visibility = sidebar.heroes.draft_visibility;
    let config_json = Some(draft_config(&sidebar.heroes));
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit { editing_id } => *editing_id,
        _ => None,
    };

    let cloud = sidebar.cloud.clone();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = if let Some(id) = editing_id {
                    let dto = UpdateAgentDto {
                        name: Some(name),
                        champion: Some(champion),
                        agent_type: Some(agent_type),
                        prompt: Some(prompt),
                        model: if model.is_empty() { None } else { Some(model) },
                        config_json,
                        visibility: Some(visibility),
                    };
                    cloud.update_agent(&id.to_string(), &dto).await
                } else {
                    let dto = CreateAgentDto {
                        name,
                        champion,
                        agent_type,
                        prompt,
                        model: if model.is_empty() { None } else { Some(model) },
                        config_json,
                        visibility: Some(visibility),
                    };
                    cloud.create_agent(&dto).await
                };

                this.update(&mut cx, |this, cx| {
                    match result {
                        Ok(agent) => {
                            // 新建成功后进入该 agent 的编辑态（enter_edit 会清空提示）
                            if matches!(this.heroes.mode, HeroesMode::Edit { editing_id: None }) {
                                enter_edit(this, cx, &agent);
                            }
                            this.heroes.success_msg = "已保存".to_string();
                        }
                        Err(e) => {
                            this.heroes.error_msg = format!("保存失败：{}", e);
                        }
                    }
                    cx.notify();
                })
                .ok();

                // 重新加载列表
                let cloud2 = match this.update(&mut cx, |this, _| this.cloud.clone()) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                let agents = cloud2.list_agents().await.unwrap_or_default();
                let mut snaps = std::collections::HashMap::new();
                for a in &agents {
                    if let Ok(s) = cloud2.list_snapshots(&a.id.to_string()).await {
                        snaps.insert(a.id, s);
                    }
                }
                this.update(&mut cx, |this, cx| {
                    this.heroes.agents = agents;
                    this.heroes.snapshots = snaps;
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

pub(super) fn handle_publish(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit {
            editing_id: Some(id),
        } => *id,
        _ => return,
    };
    sidebar.heroes.publishing = true;
    let cloud = sidebar.cloud.clone();
    let id_str = editing_id.to_string();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = cloud.publish_snapshot(&id_str).await;
                this.update(&mut cx, |this, cx| {
                    this.heroes.publishing = false;
                    match result {
                        Ok(snap) => {
                            this.heroes
                                .snapshots
                                .entry(editing_id)
                                .or_default()
                                .insert(0, snap.clone());
                            this.heroes.success_msg = format!("已发布 v{}", snap.version);
                        }
                        Err(e) => {
                            this.heroes.error_msg = format!("发布失败：{}", e);
                        }
                    }
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

pub(super) fn handle_visibility_change(
    sidebar: &mut AppSidebar,
    v: Visibility,
    cx: &mut Context<AppSidebar>,
) {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit {
            editing_id: Some(id),
        } => *id,
        _ => return,
    };
    let cloud = sidebar.cloud.clone();
    let id_str = editing_id.to_string();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = cloud.update_agent_visibility(&id_str, v).await;
                this.update(&mut cx, |this, cx| {
                    match result {
                        Ok(()) => {
                            this.heroes.draft_visibility = v;
                            if let Some(a) =
                                this.heroes.agents.iter_mut().find(|a| a.id == editing_id)
                            {
                                a.visibility = v;
                            }
                            this.heroes.success_msg = "可见性已更新".to_string();
                        }
                        Err(e) => {
                            this.heroes.error_msg = format!("更新可见性失败：{}", e);
                        }
                    }
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

pub(super) fn handle_pull_upstream(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit {
            editing_id: Some(id),
        } => *id,
        _ => return,
    };
    let cloud = sidebar.cloud.clone();
    let id_str = editing_id.to_string();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = cloud.pull_upstream(&id_str).await;

                // 重新加载列表
                let cloud2 = match this.update(&mut cx, |this, _| this.cloud.clone()) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                let agents = cloud2.list_agents().await.unwrap_or_default();
                let mut snaps = std::collections::HashMap::new();
                for a in &agents {
                    if let Ok(s) = cloud2.list_snapshots(&a.id.to_string()).await {
                        snaps.insert(a.id, s);
                    }
                }
                let agent = agents.iter().find(|a| a.id == editing_id).cloned();
                this.update(&mut cx, |this, cx| {
                    this.heroes.agents = agents;
                    this.heroes.snapshots = snaps;
                    // 用拉取后的最新数据刷新编辑态（enter_edit 会清空提示）
                    if let Some(a) = agent {
                        enter_edit(this, cx, &a);
                    }
                    match result {
                        Ok(_) => {
                            this.heroes.success_msg =
                                "已拉取上游策略，请重新发布快照使其在 Rank 生效".to_string();
                        }
                        Err(e) => {
                            this.heroes.error_msg = format!("拉取上游失败：{}", e);
                        }
                    }
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

fn handle_delete(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit {
            editing_id: Some(id),
        } => *id,
        _ => return,
    };
    sidebar.heroes.deleting = true;
    let cloud = sidebar.cloud.clone();
    let id_str = editing_id.to_string();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = cloud.delete_agent(&id_str).await;
                this.update(&mut cx, |this, cx| {
                    this.heroes.deleting = false;
                    this.heroes.show_delete_confirm = false;
                    match result {
                        Ok(()) => {
                            this.heroes.mode = HeroesMode::Browse;
                            this.heroes.success_msg = "已删除".to_string();
                        }
                        Err(e) => {
                            this.heroes.error_msg = format!("删除失败：{}", e);
                        }
                    }
                    cx.notify();
                })
                .ok();

                // 重新加载列表
                let cloud2 = match this.update(&mut cx, |this, _| this.cloud.clone()) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                let agents = cloud2.list_agents().await.unwrap_or_default();
                let mut snaps = std::collections::HashMap::new();
                for a in &agents {
                    if let Ok(s) = cloud2.list_snapshots(&a.id.to_string()).await {
                        snaps.insert(a.id, s);
                    }
                }
                this.update(&mut cx, |this, cx| {
                    this.heroes.agents = agents;
                    this.heroes.snapshots = snaps;
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

pub(super) fn handle_export_json(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let json = export_json(&sidebar.heroes);
    sidebar.heroes.draft_config_json_str = json.clone();
    cx.write_to_clipboard(ClipboardItem::new_string(json));
    sidebar.heroes.success_msg = "已导出当前配置到下方 JSON 框并复制到剪贴板".to_string();
    cx.notify();
}

pub(super) fn handle_import_json(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let s = sidebar.heroes.draft_config_json_str.clone();
    match apply_import_json(&mut sidebar.heroes, &s) {
        Ok(()) => {
            sidebar.heroes.success_msg = "已应用导入的 JSON 配置".to_string();
        }
        Err(e) => {
            sidebar.heroes.error_msg = format!("导入失败：{}", e);
        }
    }
    cx.notify();
}
