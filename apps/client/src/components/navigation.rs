use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem,
    SidebarToggleButton,
};
use gpui_component::{h_flex, ActiveTheme, Collapsible, IconName, StyledExt, Theme, ThemeMode};
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::types::ActiveView;

pub fn render_topbar(sidebar: &AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let active = sidebar.active_view;
    let weak = cx.entity().downgrade();
    let current_zh = sidebar.locale == "zh-CN";

    h_flex()
        .w_full()
        .items_center()
        .justify_between()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    SidebarToggleButton::new()
                        .collapsed(sidebar.sidebar_collapsed)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.sidebar_collapsed = !this.sidebar_collapsed;
                            cx.notify();
                        })),
                )
                .child(div().text_xl().font_bold().child(match active {
                    ActiveView::Home => t!("app.nav.title_home"),
                    ActiveView::Launcher => t!("app.nav.title_launcher"),
                    ActiveView::Heroes => t!("app.nav.title_heroes"),
                    ActiveView::Rooms => t!("app.nav.title_rooms"),
                    ActiveView::Rank => t!("app.nav.title_rank"),
                    ActiveView::Leaderboard => t!("app.nav.title_leaderboard"),
                    ActiveView::Community => t!("app.nav.title_community"),
                    ActiveView::Billing => t!("app.nav.title_billing"),
                    ActiveView::RlTraining => t!("app.nav.title_rl_training"),
                    ActiveView::Particles => t!("app.nav.title_particles"),
                    ActiveView::LogsArchive => t!("app.nav.title_logs_archive"),
                    ActiveView::Admin => t!("app.nav.title_admin"),
                    ActiveView::Settings => t!("app.nav.title_settings"),
                    ActiveView::Games => t!("app.nav.title_games"),
                    ActiveView::History => t!("app.nav.title_history"),
                    ActiveView::Blog => t!("app.nav.title_blog"),
                    ActiveView::Debug => t!("app.nav.title_debug"),
                    ActiveView::Mock => t!("app.nav.title_mock"),
                    ActiveView::Observe => t!("app.nav.title_observe"),
                    ActiveView::RoomDetail => t!("app.nav.title_room_detail"),
                    ActiveView::Hero => t!("app.nav.title_hero"),
                })),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(if let Some(user) = &sidebar.current_user {
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(user.phone.clone()),
                        )
                        .child(
                            Button::new("top-logout")
                                .outline()
                                .label("退出")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.cloud.logout();
                                    this.current_user = None;
                                    this.auth_token = None;
                                    cx.notify();
                                })),
                        )
                        .into_any_element()
                } else {
                    Button::new("top-login")
                        .primary()
                        .icon(IconName::CircleUser)
                        .label("登录")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.show_auth_dialog = true;
                            cx.notify();
                        }))
                        .into_any_element()
                })
                .child(
                    Button::new("lang-switcher")
                        .label(if current_zh {
                            "简体中文"
                        } else {
                            "English"
                        })
                        .icon(IconName::Globe)
                        .outline()
                        .dropdown_menu(move |menu, _window, _cx| {
                            let zh_weak = weak.clone();
                            let en_weak = weak.clone();
                            menu.item(PopupMenuItem::new("简体中文").checked(current_zh).on_click(
                                move |_, _, cx| {
                                    let _ = zh_weak
                                        .update(cx, |this, cx| this.change_locale("zh-CN", cx));
                                },
                            ))
                            .item(
                                PopupMenuItem::new("English").checked(!current_zh).on_click(
                                    move |_, _, cx| {
                                        let _ = en_weak
                                            .update(cx, |this, cx| this.change_locale("en", cx));
                                    },
                                ),
                            )
                        }),
                )
                .child(
                    Button::new("top-theme-toggle")
                        .icon(if cx.theme().is_dark() {
                            IconName::Sun
                        } else {
                            IconName::Moon
                        })
                        .label(if cx.theme().is_dark() {
                            t!("app.nav.theme_light")
                        } else {
                            t!("app.nav.theme_dark")
                        })
                        .outline()
                        .on_click(cx.listener(|_, _, window, cx| {
                            let new_mode = if cx.theme().is_dark() {
                                ThemeMode::Light
                            } else {
                                ThemeMode::Dark
                            };
                            Theme::change(new_mode, Some(window), cx);
                        })),
                ),
        )
        .into_any_element()
}

pub fn render_sidebar_menu(sidebar: &AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let active = sidebar.active_view;
    let collapsed = sidebar.sidebar_collapsed;

    Sidebar::new("app-sidebar")
        .collapsed(collapsed)
        .header(
            SidebarHeader::new()
                .collapsed(collapsed)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::Bot)
                        .when(!collapsed, |this| {
                            this.child(div().font_bold().child(t!("app.nav.brand")))
                        }),
                )
                .when(!collapsed, |this| {
                    this.child(SidebarToggleButton::new().collapsed(collapsed).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.sidebar_collapsed = !this.sidebar_collapsed;
                            cx.notify();
                        }),
                    ))
                }),
        )
        // ── 核心组：首页、新建对局、英雄预设 ──
        .child(
            SidebarGroup::new(t!("app.nav.group_core")).child(
                SidebarMenu::new()
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_home"))
                            .icon(IconName::LayoutDashboard)
                            .active(active == ActiveView::Home)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Home;
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_launcher"))
                            .icon(IconName::Plus)
                            .active(active == ActiveView::Launcher)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Launcher;
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_heroes"))
                            .icon(IconName::Frame)
                            .active(active == ActiveView::Heroes)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Heroes;
                                cx.notify();
                            })),
                    ),
            ),
        )
        // ── 对局组：房间、Rank、排行榜、社区 ──
        .child(
            SidebarGroup::new(t!("app.nav.group_match")).child(
                SidebarMenu::new()
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_rooms"))
                            .icon(IconName::Inbox)
                            .active(active == ActiveView::Rooms)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Rooms;
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_games"))
                            .icon(IconName::Play)
                            .active(active == ActiveView::Games)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Games;
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_rank"))
                            .icon(IconName::ChartPie)
                            .active(active == ActiveView::Rank)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Rank;
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_leaderboard"))
                            .icon(IconName::SortDescending)
                            .active(active == ActiveView::Leaderboard)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Leaderboard;
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_community"))
                            .icon(IconName::User)
                            .active(active == ActiveView::Community)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Community;
                                cx.notify();
                            })),
                    ),
            ),
        )
        // ── 工具组：RL 训练、粒子播放、日志归档 ──
        .child(
            SidebarGroup::new(t!("app.nav.group_tools")).child(
                SidebarMenu::new()
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_rl_training"))
                            .icon(IconName::Settings2)
                            .active(active == ActiveView::RlTraining)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::RlTraining;
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_particles"))
                            .icon(IconName::Palette)
                            .active(active == ActiveView::Particles)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Particles;
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_logs_archive"))
                            .icon(IconName::File)
                            .active(active == ActiveView::LogsArchive)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::LogsArchive;
                                cx.notify();
                            })),
                    ),
            ),
        )
        // ── 系统组：精粹订阅、对局池监控、设置 ──
        .child(
            SidebarGroup::new(t!("app.nav.group_system")).child(
                SidebarMenu::new()
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_billing"))
                            .icon(IconName::Star)
                            .active(active == ActiveView::Billing)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Billing;
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_admin"))
                            .icon(IconName::Inspector)
                            .active(active == ActiveView::Admin)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Admin;
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_settings"))
                            .icon(IconName::Settings)
                            .active(active == ActiveView::Settings)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.active_view = ActiveView::Settings;
                                cx.notify();
                            })),
                    ),
            ),
        )
        .footer(
            SidebarFooter::new().child(
                h_flex().w_full().items_center().justify_between().child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::Settings)
                        .when(!collapsed, |this| this.child(t!("app.nav.footer_settings"))),
                ),
            ),
        )
        .into_any_element()
}
