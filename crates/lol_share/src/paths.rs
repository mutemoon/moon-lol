//! 统一管理 `moon-lol` 的数据与配置目录路径。
//!
//! 优先顺序：
//! 1. 环境变量 `MOON_LOL_HOME`（便于测试、多实例隔离与 CI）
//! 2. 用户主目录：
//!    - Windows: `USERPROFILE` -> `HOMEDRIVE`+`HOMEPATH` -> `HOME`
//!    - Non-Windows: `HOME`
//! 3. 兜底回退到当前工作目录下的 `.moon-lol`

use std::path::{Path, PathBuf};

/// 获取 `moon-lol` 全局主数据目录（`~/.moon-lol` 或 `$MOON_LOL_HOME`）。
pub fn moon_home_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MOON_LOL_HOME") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir.trim());
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            if !profile.trim().is_empty() {
                return PathBuf::from(profile.trim()).join(".moon-lol");
            }
        }
        if let (Ok(drive), Ok(path)) = (std::env::var("HOMEDRIVE"), std::env::var("HOMEPATH")) {
            if !drive.trim().is_empty() && !path.trim().is_empty() {
                return PathBuf::from(format!("{}{}", drive.trim(), path.trim())).join(".moon-lol");
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                return PathBuf::from(home.trim()).join(".moon-lol");
            }
        }
        PathBuf::from(".moon-lol")
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                return PathBuf::from(home.trim()).join(".moon-lol");
            }
        }
        PathBuf::from(".moon-lol")
    }
}

/// 确保目录存在（自动递归创建父级目录），并返回该路径。
pub fn ensure_dir<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
    let p = path.as_ref();
    std::fs::create_dir_all(p)?;
    Ok(p.to_path_buf())
}

/// Bevy 动态场景 RON 文件目录：`~/.moon-lol/games/`
pub fn games_dir() -> PathBuf {
    moon_home_dir().join("games")
}

/// 日志目录：`~/.moon-lol/logs/`
pub fn logs_dir() -> PathBuf {
    moon_home_dir().join("logs")
}

/// 默认 debug 日志 DB 路径：`~/.moon-lol/logs/debug.db`
pub fn default_log_db_path() -> PathBuf {
    logs_dir().join("debug.db")
}

/// 单局日志 DB 路径：`~/.moon-lol/logs/{game_id}.db`
pub fn log_db_path(game_id: &str) -> PathBuf {
    logs_dir().join(format!("{game_id}.db"))
}

/// 强化学习 checkpoints 根目录：`~/.moon-lol/checkpoints/`
pub fn checkpoints_dir() -> PathBuf {
    moon_home_dir().join("checkpoints")
}

/// 某个训练任务的 checkpoints 目录：`~/.moon-lol/checkpoints/{task_id}/`
pub fn checkpoint_task_dir(task_id: &str) -> PathBuf {
    checkpoints_dir().join(task_id)
}

/// 某个训练任务的具体 checkpoint 权重文件路径：`~/.moon-lol/checkpoints/{task_id}/{ckpt_id}.safetensors`
pub fn checkpoint_path(task_id: &str, ckpt_id: &str) -> PathBuf {
    checkpoint_task_dir(task_id).join(format!("{ckpt_id}.safetensors"))
}

/// 桌面端认证 Token 路径：`~/.moon-lol/auth_token`
pub fn auth_token_path() -> PathBuf {
    moon_home_dir().join("auth_token")
}

/// 对局归档下载目录：`~/.moon-lol/matches/`
pub fn matches_dir() -> PathBuf {
    moon_home_dir().join("matches")
}

/// 多语言配置路径：`~/.moon-lol/locale`
pub fn locale_file() -> PathBuf {
    moon_home_dir().join("locale")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_derivation() {
        let base = moon_home_dir();
        assert_eq!(games_dir(), base.join("games"));
        assert_eq!(logs_dir(), base.join("logs"));
        assert_eq!(default_log_db_path(), base.join("logs").join("debug.db"));
        assert_eq!(log_db_path("game_123"), base.join("logs").join("game_123.db"));
        assert_eq!(checkpoints_dir(), base.join("checkpoints"));
        assert_eq!(
            checkpoint_task_dir("fiora_v1"),
            base.join("checkpoints").join("fiora_v1")
        );
        assert_eq!(
            checkpoint_path("fiora_v1", "step_100"),
            base.join("checkpoints")
                .join("fiora_v1")
                .join("step_100.safetensors")
        );
        assert_eq!(auth_token_path(), base.join("auth_token"));
        assert_eq!(matches_dir(), base.join("matches"));
        assert_eq!(locale_file(), base.join("locale"));
    }
}
