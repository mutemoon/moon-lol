//! 本地数据目录：所有 `lol` 会话数据都落在 `~/.moon-lol` 下。
//!
//! 与 `lol_share::paths` 同一套规则（`MOON_LOL_HOME` → 用户主目录 → 当前目录兜底），
//! 这里单独持有是为了让 CLI 不依赖 bevy 栈（CLI 是独立进程，不应拖入整个游戏依赖图）。

use std::path::PathBuf;

/// `~/.moon-lol`（或 `$MOON_LOL_HOME`）。
pub fn moon_home() -> PathBuf {
    if let Some(dir) = std::env::var_os("MOON_LOL_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from);
    home.map(|h| h.join(".moon-lol"))
        .unwrap_or_else(|| PathBuf::from(".moon-lol"))
}

/// 所有房间的根目录：`~/.moon-lol/rooms`
pub fn rooms_dir() -> PathBuf {
    moon_home().join("rooms")
}

/// 单个房间目录：`~/.moon-lol/rooms/<id>`（`id` 必须是本 CLI 生成的合法 id）。
pub fn room_dir(id: &str) -> PathBuf {
    rooms_dir().join(id)
}

/// 校验房间 id：只允许小写字母、数字与 `-`（防止 `..` 之类的目录穿越）。
pub fn is_valid_room_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// 房间启动配置：`~/.moon-lol/rooms/<id>/config.ron`
pub fn room_config_file(id: &str) -> PathBuf {
    room_dir(id).join("config.ron")
}

/// 房间运行期状态：`~/.moon-lol/rooms/<id>/room.ron`
pub fn room_state_file(id: &str) -> PathBuf {
    room_dir(id).join("room.ron")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_under_home() {
        let base = moon_home();
        assert_eq!(rooms_dir(), base.join("rooms"));
        assert_eq!(room_dir("r1"), base.join("rooms").join("r1"));
        assert_eq!(
            room_state_file("r1"),
            base.join("rooms").join("r1").join("room.ron")
        );
    }

    #[test]
    fn rejects_unsafe_room_ids() {
        assert!(is_valid_room_id("local-ab12cd"));
        assert!(!is_valid_room_id("../escape"));
        assert!(!is_valid_room_id(""));
        assert!(!is_valid_room_id("UPPER"));
    }
}
