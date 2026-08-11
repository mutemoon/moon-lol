//! 登录 / 认证弹窗（对应 apps/client/src/components/auth/AuthDialog.vue）。
//!
//! 表单状态存于 AppSidebar.auth（AuthDialogState），输入框复用共享组件
//! （gpui_component Input 封装）。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};

use crate::components::sidebar::AppSidebar;
use crate::services::cloud::{CloudClient, CloudError};

// ── 模式与表单状态 ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    CodeLogin,
    PasswordLogin,
    Register,
    ResetPassword,
}

/// 登录表单状态（存于 AppSidebar.auth）。
pub struct AuthDialogState {
    pub mode: AuthMode,
    pub phone: String,
    pub password: String,
    pub code: String,
    pub error: String,
    pub info: String,
    pub submitting: bool,
}

impl Default for AuthDialogState {
    fn default() -> Self {
        Self {
            mode: AuthMode::CodeLogin,
            phone: String::new(),
            password: String::new(),
            code: String::new(),
            error: String::new(),
            info: String::new(),
            submitting: false,
        }
    }
}

// ── 输入框（复用共享组件） ──

fn render_input(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    sidebar: &AppSidebar,
    id: &str,
    placeholder: &str,
    mask: bool,
    get_value: impl Fn(&AppSidebar) -> String + 'static,
    set_value: impl Fn(&mut AppSidebar, String) + 'static,
) -> AnyElement {
    crate::components::text_input::render_edit_input(
        window,
        cx,
        sidebar,
        id,
        placeholder,
        crate::components::text_input::EditOptions {
            masked: mask,
            ..Default::default()
        },
        get_value,
        set_value,
    )
}

fn field(label: &str, input: AnyElement) -> AnyElement {
    v_flex()
        .gap_1()
        .child(div().text_sm().font_bold().child(label.to_string()))
        .child(input)
        .into_any_element()
}

fn error_banner(text: &str, cx: &Context<AppSidebar>) -> AnyElement {
    div()
        .w_full()
        .px_3()
        .py_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().danger)
        .text_sm()
        .text_color(cx.theme().danger)
        .child(text.to_string())
        .into_any_element()
}

fn info_banner(text: &str, cx: &Context<AppSidebar>) -> AnyElement {
    div()
        .w_full()
        .px_3()
        .py_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().accent)
        .text_sm()
        .text_color(cx.theme().accent)
        .child(text.to_string())
        .into_any_element()
}

// ── 模式切换与表单操作 ──

fn switch_mode(sidebar: &mut AppSidebar, target: AuthMode) {
    sidebar.auth.mode = target;
    sidebar.auth.error.clear();
    sidebar.auth.info.clear();
    sidebar.auth.password.clear();
    sidebar.auth.code.clear();
    sidebar.auth.submitting = false;
}

fn set_error(sidebar: &mut AppSidebar, msg: &str) {
    sidebar.auth.error = msg.to_string();
    sidebar.auth.info.clear();
}

fn clear_form(sidebar: &mut AppSidebar) {
    sidebar.auth.phone.clear();
    sidebar.auth.password.clear();
    sidebar.auth.code.clear();
    sidebar.auth.error.clear();
    sidebar.auth.info.clear();
    sidebar.auth.submitting = false;
}

fn mode_tab(
    target: AuthMode,
    label: &str,
    sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let active = sidebar.auth.mode == target;
    let btn = Button::new(format!("auth-mode-{:?}", target)).label(label);
    let btn = if active { btn.primary() } else { btn.outline() };
    btn.on_click(cx.listener(move |this, _, _, cx| {
        switch_mode(this, target);
        cx.notify();
    }))
    .into_any_element()
}

// ── 主入口 ──

/// sidebar.show_auth_dialog 为 true 时返回居中弹窗，否则 None。
pub fn render_auth_dialog(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> Option<AnyElement> {
    if !sidebar.show_auth_dialog {
        return None;
    }
    Some(render_dialog(sidebar, window, cx))
}

fn render_dialog(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let mode = sidebar.auth.mode;
    let error = sidebar.auth.error.clone();
    let info = sidebar.auth.info.clone();
    let submitting = sidebar.auth.submitting;
    let is_reset = mode == AuthMode::ResetPassword;

    let title = match mode {
        AuthMode::CodeLogin => "验证码登录 / 注册",
        AuthMode::PasswordLogin => "密码登录",
        AuthMode::Register => "注册账号",
        AuthMode::ResetPassword => "重置密码",
    };

    let show_password = matches!(
        mode,
        AuthMode::PasswordLogin | AuthMode::Register | AuthMode::ResetPassword
    );
    let show_code = matches!(
        mode,
        AuthMode::CodeLogin | AuthMode::Register | AuthMode::ResetPassword
    );

    let phone_input = render_input(
        window,
        cx,
        &*sidebar,
        "auth-phone",
        "请输入 11 位手机号",
        false,
        |s: &AppSidebar| s.auth.phone.clone(),
        |s: &mut AppSidebar, v: String| s.auth.phone = v,
    );
    let password_input = render_input(
        window,
        cx,
        &*sidebar,
        "auth-password",
        if is_reset {
            "请输入新密码（至少 6 位）"
        } else {
            "请输入密码（至少 6 位）"
        },
        true,
        |s: &AppSidebar| s.auth.password.clone(),
        |s: &mut AppSidebar, v: String| s.auth.password = v,
    );
    let code_input = render_input(
        window,
        cx,
        &*sidebar,
        "auth-code",
        "请输入验证码",
        false,
        |s: &AppSidebar| s.auth.code.clone(),
        |s: &mut AppSidebar, v: String| s.auth.code = v,
    );

    let tabs: Vec<AnyElement> = [
        (AuthMode::CodeLogin, "验证码登录"),
        (AuthMode::PasswordLogin, "密码登录"),
        (AuthMode::Register, "注册"),
        (AuthMode::ResetPassword, "重置密码"),
    ]
    .into_iter()
    .map(|(m, label)| mode_tab(m, label, &*sidebar, cx))
    .collect();

    let header = h_flex()
        .items_center()
        .justify_between()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .p_2()
                        .rounded_md()
                        .bg(cx.theme().muted)
                        .child(IconName::CircleUser),
                )
                .child(div().text_lg().font_bold().child(title.to_string())),
        )
        .child(
            Button::new("auth-close")
                .ghost()
                .icon(IconName::Close)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.show_auth_dialog = false;
                    clear_form(this);
                    cx.notify();
                })),
        );

    let submit_label = if submitting {
        "请稍候…".to_string()
    } else {
        match mode {
            AuthMode::CodeLogin => "确认登录 / 注册".to_string(),
            AuthMode::PasswordLogin => "登录".to_string(),
            AuthMode::Register => "注册".to_string(),
            AuthMode::ResetPassword => "重置密码".to_string(),
        }
    };

    let submit_btn = Button::new("auth-submit")
        .primary()
        .w_full()
        .label(submit_label)
        .disabled(submitting)
        .on_click(cx.listener(|this, _, _, cx| submit(this, cx)));

    let mut children: Vec<AnyElement> = vec![
        header.into_any_element(),
        h_flex().gap_2().children(tabs).into_any_element(),
        field("手机号", phone_input),
    ];
    if show_password {
        children.push(field(
            if is_reset { "新密码" } else { "密码" },
            password_input,
        ));
    }
    if show_code {
        children.push(field("验证码", code_input));
    }
    if !error.is_empty() {
        children.push(error_banner(&error, cx));
    }
    if !info.is_empty() {
        children.push(info_banner(&info, cx));
    }
    children.push(submit_btn.into_any_element());

    let card = v_flex()
        .w(px(400.))
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .p_6()
        .gap_4()
        .children(children);

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
        .child(card)
        .into_any_element()
}

// ── 动作：登录 / 验证码登录 / 注册 / 重置密码 ──

enum AuthResult {
    Done,
    ResetDone,
    Failed(CloudError),
}

async fn run_auth(
    cloud: &CloudClient,
    mode: AuthMode,
    phone: &str,
    password: &str,
    code: &str,
) -> AuthResult {
    let res = match mode {
        AuthMode::PasswordLogin => cloud.login(phone, password).await.map(|_| ()),
        AuthMode::CodeLogin => cloud.code_login(phone, code).await.map(|_| ()),
        AuthMode::Register => cloud.register(phone, password, code).await.map(|_| ()),
        AuthMode::ResetPassword => cloud.reset_password(phone, code, password).await,
    };
    match res {
        Ok(()) if mode == AuthMode::ResetPassword => AuthResult::ResetDone,
        Ok(()) => AuthResult::Done,
        Err(e) => AuthResult::Failed(e),
    }
}

fn submit(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let mode = sidebar.auth.mode;
    let phone = sidebar.auth.phone.trim().to_string();
    let password = sidebar.auth.password.clone();
    let code = sidebar.auth.code.clone();

    if phone.chars().count() != 11 {
        set_error(sidebar, "请输入正确的 11 位手机号");
        cx.notify();
        return;
    }
    if matches!(mode, AuthMode::PasswordLogin | AuthMode::Register) && password.chars().count() < 6
    {
        set_error(sidebar, "密码长度至少为 6 位");
        cx.notify();
        return;
    }
    if matches!(
        mode,
        AuthMode::CodeLogin | AuthMode::Register | AuthMode::ResetPassword
    ) && code.is_empty()
    {
        set_error(sidebar, "请输入验证码");
        cx.notify();
        return;
    }

    set_error(sidebar, "");
    sidebar.auth.submitting = true;
    cx.notify();

    let cloud = sidebar.cloud.clone();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            let cloud = cloud.clone();
            async move {
                match run_auth(&cloud, mode, &phone, &password, &code).await {
                    AuthResult::ResetDone => {
                        this.update(&mut cx, |this, cx| {
                            this.auth.submitting = false;
                            this.auth.info = "已重置，请登录".to_string();
                            this.auth.mode = AuthMode::PasswordLogin;
                            this.auth.password.clear();
                            cx.notify();
                        })
                        .ok();
                    }
                    AuthResult::Failed(e) => {
                        this.update(&mut cx, |this, cx| {
                            this.auth.submitting = false;
                            this.auth.error = format!("{}", e);
                            this.auth.info.clear();
                            cx.notify();
                        })
                        .ok();
                    }
                    AuthResult::Done => {
                        let me = cloud.get_current_user().await;
                        this.update(&mut cx, |this, cx| {
                            this.auth.submitting = false;
                            match me {
                                Ok(u) => {
                                    this.auth_token = cloud.get_token();
                                    this.current_user = Some(crate::types::UserInfo {
                                        id: u.id as i64,
                                        phone: u.phone,
                                    });
                                    this.show_auth_dialog = false;
                                    clear_form(this);
                                }
                                Err(e) => {
                                    this.auth.error = format!("获取用户信息失败：{}", e);
                                    this.auth.info.clear();
                                }
                            }
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
