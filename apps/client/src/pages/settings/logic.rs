//! 设置页操作逻辑：供应商选中 / 增删改测 / 远程模型刷新。

use std::collections::HashSet;

use gpui::*;
use lol_web_protocol::model_provider::{
    ModelConfig, ModelProvider, ModelProviderInput, TestModelProviderResponse,
};
use uuid::Uuid;

use super::presets::{ProviderPreset, PROVIDER_PRESETS};
use super::types::{NEW_KEY, PLATFORM_KEY, PRESET_PREFIX};
use crate::components::sidebar::AppSidebar;

pub(super) fn is_new_key(key: &str) -> bool {
    key == NEW_KEY || key.starts_with(PRESET_PREFIX)
}

pub(super) fn is_provider_key(key: &str) -> bool {
    key != PLATFORM_KEY && !is_new_key(key)
}

fn find_preset_by_key(key: &str) -> Option<&'static ProviderPreset> {
    let preset_key = key.strip_prefix(PRESET_PREFIX)?;
    PROVIDER_PRESETS
        .iter()
        .find(|p| p.preset_type == preset_key)
}

pub(super) fn api_key_placeholder(sidebar: &AppSidebar) -> String {
    if sidebar.settings.form_has_api_key {
        "已设置，留空保持不变".to_string()
    } else if let Some(preset) = find_preset_by_key(&sidebar.settings.selected_key) {
        if preset.api_key_url.is_empty() {
            "输入 API Key".to_string()
        } else {
            format!("输入 API Key（可在 {} 申请）", preset.api_key_url)
        }
    } else {
        "输入 API Key".to_string()
    }
}

fn reset_form(sidebar: &mut AppSidebar) {
    sidebar.settings.form_name.clear();
    sidebar.settings.form_base_url.clear();
    sidebar.settings.form_api_key.clear();
    sidebar.settings.form_api_format = "anthropic".to_string();
    sidebar.settings.form_models.clear();
    sidebar.settings.form_has_api_key = false;
    sidebar.settings.form_category = "custom".to_string();
    sidebar.settings.form_preset_type.clear();
    sidebar.settings.form_website_url.clear();
    sidebar.settings.form_api_key_url.clear();
    sidebar.settings.form_icon.clear();
    sidebar.settings.form_icon_color.clear();
    sidebar.settings.form_sort_order = 0;
}

fn apply_provider_to_form(sidebar: &mut AppSidebar, p: &ModelProvider) {
    sidebar.settings.form_name = p.name.clone();
    sidebar.settings.form_base_url = p.base_url.clone();
    sidebar.settings.form_api_key.clear();
    sidebar.settings.form_api_format = p.api_format.clone();
    sidebar.settings.form_models = p.models.clone();
    sidebar.settings.form_has_api_key = p.has_api_key;
    sidebar.settings.form_category = p.category.clone();
    sidebar.settings.form_preset_type = p.preset_type.clone();
    sidebar.settings.form_website_url = p.website_url.clone().unwrap_or_default();
    sidebar.settings.form_api_key_url = p.api_key_url.clone().unwrap_or_default();
    sidebar.settings.form_icon = p.icon.clone().unwrap_or_default();
    sidebar.settings.form_icon_color = p.icon_color.clone().unwrap_or_default();
    sidebar.settings.form_sort_order = p.sort_order;
}

pub(super) fn select_provider(sidebar: &mut AppSidebar, key: &str, cx: &mut Context<AppSidebar>) {
    sidebar.settings.selected_key = key.to_string();
    sidebar.settings.error_msg.clear();
    sidebar.settings.success_msg.clear();

    if key == PLATFORM_KEY {
        cx.notify();
        return;
    }

    // 预设项：以「新增」模式预填表单
    if let Some(preset) = find_preset_by_key(key) {
        reset_form(sidebar);
        sidebar.settings.form_name = preset.name.to_string();
        sidebar.settings.form_base_url = preset.base_url.to_string();
        sidebar.settings.form_api_format = preset.api_format.to_string();
        sidebar.settings.form_models = preset
            .default_models
            .iter()
            .map(|n| ModelConfig {
                name: n.to_string(),
                max_tokens: 200000,
            })
            .collect();
        sidebar.settings.form_category = "preset".to_string();
        sidebar.settings.form_preset_type = preset.preset_type.to_string();
        sidebar.settings.form_website_url = preset.website_url.to_string();
        sidebar.settings.form_api_key_url = preset.api_key_url.to_string();
        sidebar.settings.form_icon = preset.icon.to_string();
        sidebar.settings.form_icon_color = preset.icon_color.to_string();
        cx.notify();
        return;
    }

    if key == NEW_KEY {
        reset_form(sidebar);
        cx.notify();
        return;
    }

    if let Some(p) = sidebar
        .settings
        .providers
        .iter()
        .find(|p| p.id.to_string() == key)
        .cloned()
    {
        apply_provider_to_form(sidebar, &p);
    }
    cx.notify();
}

pub(super) fn handle_save_provider(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let name = sidebar.settings.form_name.trim().to_string();
    if name.is_empty() {
        sidebar.settings.error_msg = "名称不能为空".to_string();
        cx.notify();
        return;
    }

    sidebar.settings.saving = true;
    let input = ModelProviderInput {
        name,
        category: sidebar.settings.form_category.clone(),
        preset_type: sidebar.settings.form_preset_type.clone(),
        base_url: sidebar.settings.form_base_url.trim().to_string(),
        api_key: sidebar.settings.form_api_key.clone(),
        api_format: sidebar.settings.form_api_format.clone(),
        models: sidebar.settings.form_models.clone(),
        enabled: true,
        website_url: sidebar.settings.form_website_url.clone(),
        api_key_url: sidebar.settings.form_api_key_url.clone(),
        icon: sidebar.settings.form_icon.clone(),
        icon_color: sidebar.settings.form_icon_color.clone(),
        sort_order: sidebar.settings.form_sort_order,
    };

    let cloud = sidebar.cloud.clone();
    let editing_id = if is_provider_key(&sidebar.settings.selected_key) {
        Uuid::parse_str(&sidebar.settings.selected_key).ok()
    } else {
        None
    };

    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let created = if let Some(id) = editing_id {
                    cloud
                        .update_model_provider(&id.to_string(), &input)
                        .await
                        .map(|_| None)
                } else {
                    cloud.create_model_provider(&input).await.map(Some)
                };

                // reload providers
                let c2 = match this.update(&mut cx, |t, _| t.cloud.clone()) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                if let Ok(pros) = c2.list_model_providers().await {
                    this.update(&mut cx, |t, cx| {
                        t.settings.providers = pros;
                        cx.notify();
                    })
                    .ok();
                }

                this.update(&mut cx, |this, cx| {
                    this.settings.saving = false;
                    match created {
                        Ok(Some(p)) => {
                            this.settings.selected_key = p.id.to_string();
                            apply_provider_to_form(this, &p);
                            this.settings.success_msg = "已保存".to_string();
                        }
                        Ok(None) => {
                            this.settings.success_msg = "已保存".to_string();
                        }
                        Err(e) => {
                            this.settings.error_msg = format!("{}", e);
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

pub(super) fn handle_delete_provider(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let id = match Uuid::parse_str(&sidebar.settings.selected_key) {
        Ok(id) => id,
        Err(_) => return,
    };
    let cloud = sidebar.cloud.clone();
    let id_str = id.to_string();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let _ = cloud.delete_model_provider(&id_str).await;

                // reload providers
                let c2 = match this.update(&mut cx, |t, _| t.cloud.clone()) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                if let Ok(pros) = c2.list_model_providers().await {
                    this.update(&mut cx, |t, cx| {
                        t.settings.providers = pros;
                        cx.notify();
                    })
                    .ok();
                }

                this.update(&mut cx, |this, cx| {
                    this.settings.selected_key = PLATFORM_KEY.to_string();
                    reset_form(this);
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

pub(super) fn handle_test_model(
    sidebar: &mut AppSidebar,
    idx: usize,
    cx: &mut Context<AppSidebar>,
) {
    let base_url = sidebar.settings.form_base_url.trim().to_string();
    if base_url.is_empty() {
        sidebar.settings.error_msg = "Base URL 不能为空".to_string();
        cx.notify();
        return;
    }

    let model = match sidebar.settings.form_models.get(idx) {
        Some(m) => m.clone(),
        None => return,
    };

    sidebar.settings.testing_model_idx = Some(idx);
    sidebar.settings.test_result = None;

    let cloud = sidebar.cloud.clone();
    let api_key = sidebar.settings.form_api_key.clone();
    let api_format = sidebar.settings.form_api_format.clone();

    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let input = lol_web_protocol::model_provider::TestModelProviderInput {
                    provider_id: None,
                    base_url,
                    api_key: if api_key.is_empty() {
                        None
                    } else {
                        Some(api_key)
                    },
                    api_format,
                    model: model.name,
                    max_tokens: Some(model.max_tokens),
                };
                let result = cloud.test_model_provider(&input).await;

                this.update(&mut cx, |this, cx| {
                    this.settings.testing_model_idx = None;
                    match result {
                        Ok(resp) => {
                            this.settings.test_result = Some(resp);
                            this.settings.show_test_result = true;
                        }
                        Err(e) => {
                            this.settings.test_result = Some(TestModelProviderResponse {
                                success: false,
                                message: e.to_string(),
                            });
                            this.settings.show_test_result = true;
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

// ── 远程模型刷新（可选能力）：GET {baseUrl}/v1/models，按名称去重合并进表单模型列表 ──

pub(super) fn handle_refresh_models(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let base_url = sidebar.settings.form_base_url.trim().to_string();
    if base_url.is_empty() {
        sidebar.settings.error_msg = "请先填写 Base URL".to_string();
        cx.notify();
        return;
    }

    let api_key = sidebar.settings.form_api_key.clone();
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
    let existing: HashSet<String> = sidebar
        .settings
        .form_models
        .iter()
        .map(|m| m.name.clone())
        .collect();

    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = crate::services::runtime::run_on_tokio(move || async move {
                    let client = reqwest::Client::new();
                    let mut req = client.get(&url);
                    if !api_key.is_empty() {
                        req = req.header("Authorization", format!("Bearer {}", api_key));
                    }
                    let resp = req.send().await.map_err(|e| e.to_string())?;
                    let text = resp.text().await.map_err(|e| e.to_string())?;
                    let data: serde_json::Value =
                        serde_json::from_str(&text).map_err(|e| e.to_string())?;
                    Ok::<serde_json::Value, String>(data)
                })
                .await;

                match result {
                    Ok(data) => {
                        let mut new_models: Vec<String> = Vec::new();
                        let arr = data.get("data").or_else(|| data.get("models"));
                        if let Some(serde_json::Value::Array(items)) = arr {
                            for item in items {
                                let name = match item {
                                    serde_json::Value::String(s) => s.clone(),
                                    serde_json::Value::Object(o) => o
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_default(),
                                    _ => String::new(),
                                };
                                if !name.is_empty() && !existing.contains(&name) {
                                    new_models.push(name);
                                }
                            }
                        }
                        let count = new_models.len();
                        this.update(&mut cx, |this, cx| {
                            for n in new_models {
                                this.settings.form_models.push(ModelConfig {
                                    name: n,
                                    max_tokens: 200000,
                                });
                            }
                            this.settings.success_msg = format!("已合并 {} 个远程模型", count);
                            cx.notify();
                        })
                        .ok();
                    }
                    Err(e) => {
                        this.update(&mut cx, |this, cx| {
                            this.settings.error_msg = format!("刷新失败：{}", e);
                            cx.notify();
                        })
                        .ok();
                    }
                }
            }
        },
    )
    .detach();
}
