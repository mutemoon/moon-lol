//! CLI 错误类型：所有面向用户的失败都折叠成带可读消息的枚举。

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// 房间尚未 start（没有 room.ron）。
    NotStarted(String),
    /// 房间进程已退出。
    Dead(String),
    /// 控制面连不上（游戏还在启动 / 已崩）。
    NoGameControl {
        port: u16,
        source: String,
    },
    /// WS 端口耗尽。
    PortPoolExhausted,
    BadState {
        path: PathBuf,
        reason: String,
    },
    Serialize(String),
    Io(std::io::Error),
    User(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotStarted(id) => {
                write!(f, "房间 {id} 尚未启动，先执行 `lol room --id {id} start`")
            }
            Error::Dead(id) => write!(f, "房间 {id} 的游戏进程已退出，请重新 `start`"),
            Error::NoGameControl { port, source } => write!(
                f,
                "连接游戏控制面 127.0.0.1:{port} 失败：{source}（可能仍在启动，或已崩溃）"
            ),
            Error::PortPoolExhausted => write!(f, "本地端口 9100-9199 已耗尽"),
            Error::BadState { path, reason } => write!(f, "无法解析 {}: {reason}", path.display()),
            Error::Serialize(reason) => write!(f, "序列化失败: {reason}"),
            Error::Io(e) => write!(f, "IO 错误: {e}"),
            Error::User(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// 把 [`lol_client`] 侧的字符串错误折叠成「控制面连不上」。
pub fn control_error(port: u16, source: &str) -> Error {
    Error::NoGameControl {
        port,
        source: source.to_string(),
    }
}
