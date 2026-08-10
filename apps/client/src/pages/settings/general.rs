//! General Tab：主题与语言。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, StyledExt, Theme, ThemeMode};

use crate::components::sidebar::AppSidebar;

pub(super) fn render_general(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let locale = sidebar.locale.clone();
    let is_dark = cx.theme().is_dark();

    let theme_dark = {
        let btn = Button::new("theme-dark").label("深色");
        let btn = if is_dark {
            btn.primary()
        } else {
            btn.outline()
        };
        btn.on_click(cx.listener(|_, _, window, cx| {
            Theme::change(ThemeMode::Dark, Some(window), cx);
        }))
    };
    let theme_light = {
        let btn = Button::new("theme-light").label("浅色");
        let btn = if is_dark {
            btn.outline()
        } else {
            btn.primary()
        };
        btn.on_click(cx.listener(|_, _, window, cx| {
            Theme::change(ThemeMode::Light, Some(window), cx);
        }))
    };

    let lang_zh = {
        let btn = Button::new("lang-zh").label("简体中文");
        let btn = if locale == "zh-CN" {
            btn.primary()
        } else {
            btn.outline()
        };
        btn.on_click(cx.listener(|this, _, _, cx| {
            this.change_locale("zh-CN", cx);
        }))
    };
    let lang_en = {
        let btn = Button::new("lang-en").label("English");
        let btn = if locale == "en" {
            btn.primary()
        } else {
            btn.outline()
        };
        btn.on_click(cx.listener(|this, _, _, cx| {
            this.change_locale("en", cx);
        }))
    };

    v_flex()
        .gap_6()
        .child(
            v_flex()
                .gap_2()
                .child(div().text_xl().font_bold().child("常规设置")),
        )
        .child(
            v_flex()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .p_5()
                .gap_4()
                .child(div().text_sm().font_bold().child("外观与语言"))
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("界面主题"),
                        )
                        .child(h_flex().gap_2().child(theme_dark).child(theme_light)),
                )
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("界面语言"),
                        )
                        .child(h_flex().gap_2().child(lang_zh).child(lang_en)),
                ),
        )
        .into_any_element()
}
