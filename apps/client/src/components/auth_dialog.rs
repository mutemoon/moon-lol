#![allow(dead_code)]

//! 登录 / 认证弹窗（对应 apps/client/src/components/auth/AuthDialog.vue）。
//!
//! 表单状态用 thread_local 保存（同 community.rs / heroes.rs 的 track_focus 手法），
//! 避免给 AppSidebar 增加字段。输入框为手写焦点/光标实现，因为
//! gpui_component 的 Input 需要 &mut Window，而 render 签名拿不到 window。

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};

use crate::components::sidebar::AppSidebar;
use crate::services::cloud::{CloudClient, CloudError};

// ── 模式与表单状态 ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthMode {
    CodeLogin,
    PasswordLogin,
    Register,
    ResetPassword,
}

thread_local! {
    static AUTH_MODE: Cell<AuthMode> = Cell::new(AuthMode::CodeLogin);
    static PHONE: RefCell<String> = RefCell::new(String::new());
    static PASSWORD: RefCell<String> = RefCell::new(String::new());
    static CODE: RefCell<String> = RefCell::new(String::new());
    static ERROR_MSG: RefCell<String> = RefCell::new(String::new());
    static INFO_MSG: RefCell<String> = RefCell::new(String::new());
    static SUBMITTING: Cell<bool> = Cell::new(false);
    static EDITS: RefCell<HashMap<String, EditMeta>> = RefCell::new(HashMap::new());
}

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

    // ctrl / cmd 组合键不作为字符输入
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

// ── 手写输入框（参照 community.rs） ──

fn render_input(
    cx: &mut Context<AppSidebar>,
    id: &str,
    placeholder: &str,
    mask: bool,
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

    let display = move |s: &str| -> String {
        if mask {
            "•".repeat(s.chars().count())
        } else {
            s.to_string()
        }
    };

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
                .when(empty, |d| d.text_color(muted).child(placeholder.to_string()))
                .when(!empty, |d| {
                    d.child(display(&before))
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(display(&after))
                }),
        )
        .into_any_element()
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

fn switch_mode(target: AuthMode) {
    AUTH_MODE.with(|m| m.set(target));
    ERROR_MSG.with(|e| e.borrow_mut().clear());
    INFO_MSG.with(|i| i.borrow_mut().clear());
    PASSWORD.with(|p| p.borrow_mut().clear());
    CODE.with(|c| c.borrow_mut().clear());
    SUBMITTING.with(|s| s.set(false));
}

fn set_error(msg: &str) {
    ERROR_MSG.with(|e| *e.borrow_mut() = msg.to_string());
    INFO_MSG.with(|i| i.borrow_mut().clear());
}

fn clear_form() {
    PHONE.with(|p| p.borrow_mut().clear());
    PASSWORD.with(|p| p.borrow_mut().clear());
    CODE.with(|c| c.borrow_mut().clear());
    ERROR_MSG.with(|e| e.borrow_mut().clear());
    INFO_MSG.with(|i| i.borrow_mut().clear());
    SUBMITTING.with(|s| s.set(false));
}

fn mode_tab(target: AuthMode, label: &str, cx: &mut Context<AppSidebar>) -> AnyElement {
    let active = AUTH_MODE.with(|m| m.get()) == target;
    let btn = Button::new(format!("auth-mode-{:?}", target)).label(label);
    let btn = if active { btn.primary() } else { btn.outline() };
    btn.on_click(cx.listener(move |_, _, _, cx| {
        switch_mode(target);
        cx.notify();
    }))
    .into_any_element()
}

// ── 主入口 ──

/// sidebar.show_auth_dialog 为 true 时返回居中弹窗，否则 None。
pub fn render_auth_dialog(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> Option<AnyElement> {
    if !sidebar.show_auth_dialog {
        return None;
    }
    Some(render_dialog(sidebar, cx))
}

fn render_dialog(_sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let mode = AUTH_MODE.with(|m| m.get());
    let error = ERROR_MSG.with(|e| e.borrow().clone());
    let info = INFO_MSG.with(|i| i.borrow().clone());
    let submitting = SUBMITTING.with(|s| s.get());
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
        cx,
        "auth-phone",
        "请输入 11 位手机号",
        false,
        || PHONE.with(|p| p.borrow().clone()),
        |v| PHONE.with(|p| *p.borrow_mut() = v),
    );
    let password_input = render_input(
        cx,
        "auth-password",
        if is_reset {
            "请输入新密码（至少 6 位）"
        } else {
            "请输入密码（至少 6 位）"
        },
        true,
        || PASSWORD.with(|p| p.borrow().clone()),
        |v| PASSWORD.with(|p| *p.borrow_mut() = v),
    );
    let code_input = render_input(
        cx,
        "auth-code",
        "请输入验证码",
        false,
        || CODE.with(|c| c.borrow().clone()),
        |v| CODE.with(|c| *c.borrow_mut() = v),
    );

    let tabs: Vec<AnyElement> = [
        (AuthMode::CodeLogin, "验证码登录"),
        (AuthMode::PasswordLogin, "密码登录"),
        (AuthMode::Register, "注册"),
        (AuthMode::ResetPassword, "重置密码"),
    ]
    .into_iter()
    .map(|(m, label)| mode_tab(m, label, cx))
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
                    clear_form();
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
        children.push(field(if is_reset { "新密码" } else { "密码" }, password_input));
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
    let mode = AUTH_MODE.with(|m| m.get());
    let phone = PHONE.with(|p| p.borrow().trim().to_string());
    let password = PASSWORD.with(|p| p.borrow().clone());
    let code = CODE.with(|c| c.borrow().clone());

    if phone.chars().count() != 11 {
        set_error("请输入正确的 11 位手机号");
        cx.notify();
        return;
    }
    if matches!(mode, AuthMode::PasswordLogin | AuthMode::Register)
        && password.chars().count() < 6
    {
        set_error("密码长度至少为 6 位");
        cx.notify();
        return;
    }
    if matches!(mode, AuthMode::CodeLogin | AuthMode::Register | AuthMode::ResetPassword)
        && code.is_empty()
    {
        set_error("请输入验证码");
        cx.notify();
        return;
    }

    set_error("");
    SUBMITTING.with(|s| s.set(true));
    cx.notify();

    let cloud = sidebar.cloud.clone();
    cx.spawn(move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
        let mut cx = cx.clone();
        let cloud = cloud.clone();
        async move {
            match run_auth(&cloud, mode, &phone, &password, &code).await {
                AuthResult::ResetDone => {
                    this.update(&mut cx, |_this, cx| {
                        SUBMITTING.with(|s| s.set(false));
                        INFO_MSG.with(|i| *i.borrow_mut() = "已重置，请登录".to_string());
                        AUTH_MODE.with(|m| m.set(AuthMode::PasswordLogin));
                        PASSWORD.with(|p| p.borrow_mut().clear());
                        cx.notify();
                    })
                    .ok();
                }
                AuthResult::Failed(e) => {
                    this.update(&mut cx, |_this, cx| {
                        SUBMITTING.with(|s| s.set(false));
                        set_error(&format!("{}", e));
                        cx.notify();
                    })
                    .ok();
                }
                AuthResult::Done => {
                    let me = cloud.get_current_user().await;
                    this.update(&mut cx, |this, cx| {
                        SUBMITTING.with(|s| s.set(false));
                        match me {
                            Ok(u) => {
                                this.auth_token = cloud.get_token();
                                this.current_user = Some(crate::types::UserInfo {
                                    id: u.id as i64,
                                    phone: u.phone,
                                });
                                this.show_auth_dialog = false;
                                clear_form();
                            }
                            Err(e) => set_error(&format!("获取用户信息失败：{}", e)),
                        }
                        cx.notify();
                    })
                    .ok();
                }
            }
        }
    })
    .detach();
}
