//! 设置页 — 对应 apps/client/src/pages/settings.vue
//!
//! 包含「常规设置」（主题 / 语言）与「模型设置」（供应商侧栏 / 表单 / 模型增删改测）。
//! 供应商预设数据移植自 apps/client/src/config/providerPresets.ts。

mod general;
mod input;
mod logic;
mod models;
mod presets;
mod types;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::separator::Separator;
use gpui_component::{h_flex, v_flex};
pub use types::SettingsState;

use self::general::render_general;
use self::models::render_model_settings;
use self::types::SettingsTab;
use crate::components::sidebar::AppSidebar;

/// 主渲染函数：顶部 Tab（常规 / 模型设置）+ 内容区。
pub fn render_settings(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    // 首次进入自动加载模型供应商
    if sidebar.settings.providers.is_empty() && !sidebar.settings.loading {
        sidebar.settings.loading = true;
        let cloud = sidebar.cloud.clone();
        cx.spawn(
            |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                let this = this.clone();
                let mut cx = cx.clone();
                async move {
                    let providers = cloud.list_model_providers().await.unwrap_or_default();
                    this.update(&mut cx, |this, ctx| {
                        this.settings.providers = providers;
                        this.settings.loading = false;
                        ctx.notify();
                    })
                    .ok();
                }
            },
        )
        .detach();
    }
    let tab = sidebar.settings.active_tab;

    let tab_general = make_tab(
        "tab-general",
        "常规设置",
        tab == SettingsTab::General,
        cx,
        |this, cx| {
            this.settings.active_tab = SettingsTab::General;
            cx.notify();
        },
    );
    let tab_models = make_tab(
        "tab-models",
        "模型设置",
        tab == SettingsTab::ModelSettings,
        cx,
        |this, cx| {
            this.settings.active_tab = SettingsTab::ModelSettings;
            cx.notify();
        },
    );

    let content = match tab {
        SettingsTab::General => render_general(sidebar, cx),
        SettingsTab::ModelSettings => render_model_settings(sidebar, window, cx),
    };

    v_flex()
        .size_full()
        .overflow_hidden()
        .child(
            h_flex()
                .gap_1()
                .px_4()
                .py_2()
                .child(tab_general)
                .child(tab_models),
        )
        .child(Separator::horizontal())
        .child(
            div()
                .flex_1()
                .w_full()
                .id("settings-scroll")
                .px_6()
                .py_6()
                .child(content),
        )
        .into_any_element()
}

fn make_tab(
    id: &str,
    label: impl Into<SharedString>,
    active: bool,
    cx: &mut Context<AppSidebar>,
    on_click: impl Fn(&mut AppSidebar, &mut Context<AppSidebar>) + 'static,
) -> AnyElement {
    let el_id: SharedString = id.to_string().into();
    let btn = Button::new(el_id).label(label);
    let btn = if active { btn.primary() } else { btn.ghost() };
    btn.on_click(cx.listener(move |this, _, _, cx| on_click(this, cx)))
        .into_any_element()
}
