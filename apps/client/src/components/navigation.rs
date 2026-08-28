use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::sidebar::{
    Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem, SidebarToggleButton,
};
use gpui_component::{h_flex, Collapsible, Disableable, IconName, StyledExt};
use rust_i18n::t;

use crate::components::auth_dialog::open_auth_dialog;
use crate::components::sidebar::AppSidebar;
use crate::types::ActiveView;

struct TopbarDragState {
    should_move: bool,
}

impl Render for TopbarDragState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub fn render_topbar(
    sidebar: &AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let active = sidebar.active_view;
    let is_windows = cfg!(target_os = "windows");

    let state = window.use_state(cx, |_, _| TopbarDragState { should_move: false });

    h_flex()
        .w_full()
        .items_center()
        .justify_between()
        .on_mouse_down_out(window.listener_for(&state, |state, _, _, _| {
            state.should_move = false;
        }))
        .on_mouse_down(
            MouseButton::Left,
            window.listener_for(&state, |state, _, _, _| {
                state.should_move = true;
            }),
        )
        .on_mouse_up(
            MouseButton::Left,
            window.listener_for(&state, |state, _, _, _| {
                state.should_move = false;
            }),
        )
        .on_mouse_move(window.listener_for(&state, |state, _, window, _| {
            if state.should_move {
                state.should_move = false;
                window.start_window_move();
            }
        }))
        .child(
            h_flex()
                .flex_1()
                .items_center()
                .gap_2()
                .py_4()
                .when(is_windows, |this| {
                    this.window_control_area(WindowControlArea::Drag)
                })
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .on_mouse_down(MouseButton::Left, |_, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        })
                        .child(
                            SidebarToggleButton::new()
                                .collapsed(sidebar.sidebar_collapsed)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.sidebar_collapsed = !this.sidebar_collapsed;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("nav-back")
                                .icon(IconName::ChevronLeft)
                                .ghost()
                                .disabled(!sidebar.can_go_back())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if this.go_back() {
                                        cx.notify();
                                    }
                                })),
                        )
                        .child(
                            Button::new("nav-forward")
                                .icon(IconName::ChevronRight)
                                .ghost()
                                .disabled(!sidebar.can_go_forward())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if this.go_forward() {
                                        cx.notify();
                                    }
                                })),
                        ),
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
                    ActiveView::LogsBrowser => t!("app.nav.title_logs_browser"),
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
                    ActiveView::RlTaskDetail => {
                        if let Some(tid) = &sidebar.selected_task_id {
                            if let Some(detail) = sidebar.task_details.get(tid) {
                                format!("任务详情 - {}", detail.name).into()
                            } else {
                                "任务详情".into()
                            }
                        } else {
                            "任务详情".into()
                        }
                    }
                    ActiveView::RlEnvDetail => {
                        if let Some(env_name) = &sidebar.selected_env_name {
                            if let Some(spec) = lol_rl_protocol::get_env_spec(env_name) {
                                format!("环境详情 - {}", spec.label).into()
                            } else {
                                "环境详情".into()
                            }
                        } else {
                            "环境详情".into()
                        }
                    }
                    ActiveView::VisualEnv => "可视环境监控".into(),
                    ActiveView::WadBrowser => "WAD 文件浏览器".into(),
                    ActiveView::Extractor => "资源提取中心".into(),
                })),
        )
        .child(
            h_flex()
                .gap_1()
                .items_center()
                .child(
                    div()
                        .when(is_windows, |this| {
                            this.window_control_area(WindowControlArea::Min)
                        })
                        .child(
                            Button::new("win-minimize")
                                .ghost()
                                .icon(IconName::WindowMinimize)
                                .on_click(cx.listener(|_, _, _window, _cx| {
                                    // window.minimize_window();
                                })),
                        ),
                )
                .child(
                    div()
                        .when(is_windows, |this| {
                            this.window_control_area(WindowControlArea::Max)
                        })
                        .child(Button::new("win-maximize").ghost().icon(
                            if window.is_maximized() {
                                IconName::WindowRestore
                            } else {
                                IconName::WindowMaximize
                            },
                        )),
                )
                .child(
                    div()
                        .when(is_windows, |this| {
                            this.window_control_area(WindowControlArea::Close)
                        })
                        .child(
                            Button::new("win-close")
                                .ghost()
                                .icon(IconName::WindowClose)
                                .on_click(cx.listener(|_, _, window, _cx| {
                                    window.remove_window();
                                })),
                        ),
                ),
        )
        .into_any_element()
}

pub fn render_sidebar_menu(sidebar: &AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let active = sidebar.active_view;
    let collapsed = sidebar.sidebar_collapsed;

    // footer 账号菜单：状态快照 + 弱句柄
    let account_label = sidebar
        .current_user
        .as_ref()
        .map(|u| u.phone.clone())
        .unwrap_or_else(|| "点击登录".to_string());
    let logged_in = sidebar.current_user.is_some();
    let weak = cx.entity().downgrade();

    Sidebar::new("app-sidebar")
        .collapsed(collapsed)
        .header(
            SidebarHeader::new().collapsed(collapsed).child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Bot)
                    .when(!collapsed, |this| {
                        this.child(div().font_bold().child(t!("app.nav.brand")))
                    }),
            ),
        )
        // ── 在线组（无标题）：对局 / 账号等在线功能 ──
        .child(
            SidebarGroup::new("").child(
                SidebarMenu::new()
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_home"))
                            .icon(IconName::LayoutDashboard)
                            .active(active == ActiveView::Home)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Home);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_launcher"))
                            .icon(IconName::Plus)
                            .active(active == ActiveView::Launcher)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Launcher);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_heroes"))
                            .icon(IconName::Frame)
                            .active(active == ActiveView::Heroes)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Heroes);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_hero"))
                            .icon(IconName::Frame)
                            .active(active == ActiveView::Hero)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Hero);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_rooms"))
                            .icon(IconName::Inbox)
                            .active(active == ActiveView::Rooms)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Rooms);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_games"))
                            .icon(IconName::Play)
                            .active(active == ActiveView::Games)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Games);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_rank"))
                            .icon(IconName::ChartPie)
                            .active(active == ActiveView::Rank)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Rank);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_leaderboard"))
                            .icon(IconName::SortDescending)
                            .active(active == ActiveView::Leaderboard)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Leaderboard);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_community"))
                            .icon(IconName::User)
                            .active(active == ActiveView::Community)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Community);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_observe"))
                            .icon(IconName::Play)
                            .active(active == ActiveView::Observe)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Observe);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_history"))
                            .icon(IconName::File)
                            .active(active == ActiveView::History)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::History);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_room_detail"))
                            .icon(IconName::Inbox)
                            .active(active == ActiveView::RoomDetail)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::RoomDetail);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_billing"))
                            .icon(IconName::Star)
                            .active(active == ActiveView::Billing)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Billing);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_admin"))
                            .icon(IconName::Inspector)
                            .active(active == ActiveView::Admin)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Admin);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_logs_archive"))
                            .icon(IconName::HardDrive)
                            .active(active == ActiveView::LogsArchive)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::LogsArchive);
                                cx.notify();
                            })),
                    ),
            ),
        )
        // ── 工具组：离线工具 ──
        .child(
            SidebarGroup::new(t!("app.nav.group_tools")).child(
                SidebarMenu::new()
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_rl_training"))
                            .icon(IconName::Settings2)
                            .active(matches!(
                                active,
                                ActiveView::RlTraining
                                    | ActiveView::RlEnvDetail
                                    | ActiveView::RlTaskDetail
                            ))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::RlTraining);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_particles"))
                            .icon(IconName::Palette)
                            .active(active == ActiveView::Particles)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Particles);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_logs_browser"))
                            .icon(IconName::File)
                            .active(active == ActiveView::LogsBrowser)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::LogsBrowser);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new("WAD 浏览器")
                            .icon(IconName::File)
                            .active(active == ActiveView::WadBrowser)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::WadBrowser);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new("资源提取中心")
                            .icon(IconName::Folder)
                            .active(active == ActiveView::Extractor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Extractor);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_visual_env"))
                            .icon(IconName::Settings2)
                            .active(active == ActiveView::VisualEnv)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::VisualEnv);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_mock"))
                            .icon(IconName::Settings2)
                            .active(active == ActiveView::Mock)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Mock);
                                cx.notify();
                            })),
                    )
                    .child(
                        SidebarMenuItem::new(t!("app.nav.menu_blog"))
                            .icon(IconName::File)
                            .active(active == ActiveView::Blog)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Blog);
                                cx.notify();
                            })),
                    ),
            ),
        )
        // ── 设置（置底：账号卡上方）──
        .child(
            SidebarGroup::new("").child(
                SidebarMenu::new().child(
                    SidebarMenuItem::new(t!("app.nav.menu_settings"))
                        .icon(IconName::Settings)
                        .active(active == ActiveView::Settings)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.navigate_to(ActiveView::Settings);
                            cx.notify();
                        })),
                ),
            ),
        )
        .footer(
            h_flex()
                .w_full()
                .px_2()
                .py_1p5()
                .items_center()
                // 账号菜单：点卡片弹动作菜单（设置 / 退出 / 登录）
                .child(
                    Button::new("sidebar-account")
                        .ghost()
                        .icon(IconName::CircleUser)
                        .label(if collapsed {
                            ""
                        } else {
                            account_label.as_str()
                        })
                        .dropdown_menu(move |menu, _window, _cx| {
                            if logged_in {
                                menu.item(PopupMenuItem::new("退出登录").on_click({
                                    let w = weak.clone();
                                    move |_, _, cx| {
                                        w.update(cx, |this, cx| {
                                            this.cloud.logout();
                                            this.current_user = None;
                                            this.auth_token = None;
                                            cx.notify();
                                        })
                                        .ok();
                                    }
                                }))
                            } else {
                                menu.item(
                                    PopupMenuItem::new("登录")
                                        .icon(IconName::CircleUser)
                                        .on_click({
                                            let w = weak.clone();
                                            move |_, window, cx| {
                                                let _ = w.update(cx, |_, cx| {
                                                    open_auth_dialog(window, cx)
                                                });
                                            }
                                        }),
                                )
                            }
                        }),
                ),
        )
        .into_any_element()
}
