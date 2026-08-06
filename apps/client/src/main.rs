mod components;
mod i18n;
mod pages;
mod services;
mod types;

use components::AppSidebar;
use gpui::*;
use gpui_component::{Root, TitleBar};

rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(move |cx| {
            // 注册顺序关键：extend! 必须先于 gpui_component::init
            rust_i18n::extend!(gpui_component);
            // 启动即应用持久化语言（set_locale 只需 &str，全局生效）
            let locale = i18n::read_persisted_locale();
            gpui_component::set_locale(&locale);
            gpui_component::init(cx);

            cx.spawn(async move |cx| {
                let options = WindowOptions {
                    titlebar: Some(TitleBar::title_bar_options()),
                    ..Default::default()
                };

                cx.open_window(options, |window, cx| {
                    let view = cx.new(|cx| AppSidebar::new(cx));
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("Failed to open window");
            })
            .detach();
        });
}
