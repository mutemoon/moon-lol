//! 本地房间（room）：一次可复用的游戏进程会话。
//!
//! 房间的全部状态都落在 `~/.moon-lol/rooms/<id>/`：
//!
//! ```text
//! ~/.moon-lol/rooms/<id>/
//!   config.ron   房间配置（英雄/场景/模式/暂停计划），游戏以 --profile 读回
//!   room.ron     运行期状态（pid、ws 端口、状态），lol game 靠它找进程
//!   game.log     子进程 stdout/stderr
//!   shots/       截图
//!   snaps/       快照 JSON
//! ```

use std::path::{Path, PathBuf};
use std::process::Child;

use lol_client::launch::{BevyGameConfig, BevySpawnRequest, build_command};
use serde::{Deserialize, Serialize};

use crate::paths::{room_config_file, room_dir, room_state_file, rooms_dir};

/// 房间配置（`config.ron`）：CLI 是唯一写入方，游戏按 `--profile` 读同一份文件。
///
/// 「配置即配置档」：`pause_at` 是游戏需要的字段，其余是 CLI 启动游戏时的参数，
/// 游戏用自己的 `--profile` 反序列化时忽略多余字段。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomConfig {
    pub champion: String,
    #[serde(default)]
    pub scene: Option<String>,
    pub mode: String,
    /// 游戏内时间（秒）到点自动暂停。
    #[serde(default)]
    pub pause_at: Vec<f64>,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            champion: "riven".into(),
            scene: None,
            mode: "sandbox".into(),
            pause_at: vec![],
        }
    }
}

impl RoomConfig {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| ron::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> crate::Result<()> {
        write_ron(path, self)
    }

    /// 启动游戏进程的参数：进程只负责把 `--profile` 指回房间配置。
    pub fn game_config(&self, room_id: &str) -> BevyGameConfig {
        BevyGameConfig {
            mode: Some(self.mode.clone()),
            champion: Some(self.champion.clone()),
            scene: self.scene.clone(),
            headless: false,
            profile: Some(room_config_file(room_id)),
        }
    }
}

/// 房间运行期状态（`room.ron`）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomState {
    pub id: String,
    pub pid: u32,
    /// 游戏 WS 控制端口。
    pub port: u16,
    pub status: RoomStatus,
    /// 子进程日志路径。
    pub log: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomStatus {
    Starting,
    Running,
    Stopped,
}

impl RoomStatus {
    pub fn label(self) -> &'static str {
        match self {
            RoomStatus::Starting => "starting",
            RoomStatus::Running => "running",
            RoomStatus::Stopped => "stopped",
        }
    }
}

impl RoomState {
    pub fn load(room_id: &str) -> crate::Result<Self> {
        let path = room_state_file(room_id);
        let text =
            std::fs::read_to_string(&path).map_err(|_| crate::Error::NotStarted(room_id.into()))?;
        ron::from_str(&text).map_err(|e| crate::Error::BadState {
            path,
            reason: e.to_string(),
        })
    }

    pub fn save(&self) -> crate::Result<()> {
        write_ron(&room_state_file(&self.id), self)
    }

    pub fn is_alive(&self) -> bool {
        pid_alive(self.pid)
    }

    pub fn shots_dir(&self) -> PathBuf {
        room_dir(&self.id).join("shots")
    }

    pub fn snaps_dir(&self) -> PathBuf {
        room_dir(&self.id).join("snaps")
    }
}

/// 已启动的房间：持有子进程句柄。
///
/// CLI 进程退出后游戏继续运行（句柄只是不再被等待），续接请用 `lol game --id`。
pub struct RunningRoom {
    pub state: RoomState,
    child: Child,
}

impl RunningRoom {
    pub fn spawn(
        room_id: &str,
        config: &RoomConfig,
        program: &str,
        prefix: &[String],
    ) -> crate::Result<Self> {
        let dir = room_dir(room_id);
        std::fs::create_dir_all(dir.join("shots")).map_err(crate::Error::Io)?;
        std::fs::create_dir_all(dir.join("snaps")).map_err(crate::Error::Io)?;
        // 配置即配置档：先落盘，游戏启动时立即读它。
        config.save(&room_config_file(room_id))?;

        let port = free_port().ok_or(crate::Error::PortPoolExhausted)?;
        let log_path = dir.join("game.log");
        let log = std::fs::File::create(&log_path).map_err(crate::Error::Io)?;
        let log_err = log.try_clone().map_err(crate::Error::Io)?;

        let request = BevySpawnRequest {
            program: program.to_string(),
            prefix_args: prefix.to_vec(),
            port,
            game_config: config.game_config(room_id),
            cwd: lol_client::launch::workspace_root(),
            rust_log: Some(lol_client::launch::default_rust_log().to_string()),
            log_db: None,
        };

        let mut command = build_command(&request);
        command
            .stdout(std::process::Stdio::from(log))
            .stderr(std::process::Stdio::from(log_err));
        let child = command.spawn().map_err(crate::Error::Io)?;

        let state = RoomState {
            id: room_id.to_string(),
            pid: child.id(),
            port,
            status: RoomStatus::Starting,
            log: log_path,
        };
        state.save()?;
        Ok(Self { state, child })
    }

    /// 子进程是否已退出；退出时返回退出码。
    pub fn exited(&mut self) -> crate::Result<Option<Option<i32>>> {
        Ok(self
            .child
            .try_wait()
            .map_err(crate::Error::Io)?
            .map(|s| s.code()))
    }

    pub fn set_status(&mut self, status: RoomStatus) -> crate::Result<()> {
        self.state.status = status;
        self.state.save()
    }
}

/// 进程是否还活着（Windows 用 `tasklist`，Unix 用 `kill -0`）。
pub fn pid_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

/// 杀掉整棵进程树（Windows 下 `cargo run` 会套一层，必须 `/T`）。
pub fn kill_tree(pid: u32) -> crate::Result<()> {
    #[cfg(windows)]
    {
        let status = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()
            .map_err(crate::Error::Io)?;
        if !status.success() && pid_alive(pid) {
            return Err(crate::Error::User(format!(
                "终止进程 {pid} 失败（taskkill 退出码 {:?}）",
                status.code()
            )));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status()
            .map_err(crate::Error::Io)?;
        Ok(())
    }
}

/// 列出 `~/.moon-lol/rooms/` 下所有房间 id（按目录名排序）。
pub fn list_rooms() -> Vec<String> {
    let mut ids = std::fs::read_dir(rooms_dir())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

/// 本地时间戳 `YYYYMMDD-HHMMSS`，用于截图/快照文件名。
pub fn timestamp() -> String {
    // 不引入 chrono：用自 1970 起的 UTC 秒数拼文件名，只保证单调、可排序。
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let secs = now % 86_400;
    let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    format!("{}-{h:02}{m:02}{s:02}", now / 86_400)
}

/// 从 `9100..=9199` 里挑一个当前既没被占用、也没监听的端口。
fn free_port() -> Option<u16> {
    let taken: Vec<u16> = list_rooms()
        .into_iter()
        .filter_map(|id| RoomState::load(&id).ok())
        .filter(|s| s.is_alive())
        .map(|s| s.port)
        .collect();
    (9100..=9199).find(|p| !taken.contains(p) && !port_listening(*p))
}

fn port_listening(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        std::time::Duration::from_millis(50),
    )
    .is_ok()
}

fn write_ron<T: Serialize>(path: &Path, value: &T) -> crate::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(crate::Error::Io)?;
    }
    let text = ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())
        .map_err(|e| crate::Error::Serialize(e.to_string()))?;
    std::fs::write(path, text).map_err(crate::Error::Io)
}

#[cfg(test)]
mod tests {
    use lol_client::launch::RoomProfile;

    use super::*;

    #[test]
    fn config_roundtrip_through_disk() {
        let dir = std::env::temp_dir().join(format!("lol-room-cfg-{}", std::process::id()));
        let path = dir.join("config.ron");
        let config = RoomConfig {
            champion: "darius".into(),
            scene: Some("user_games://classic.ron".into()),
            pause_at: vec![10.0, 13.5],
            ..Default::default()
        };
        config.save(&path).unwrap();
        assert_eq!(RoomConfig::load(&path).pause_at, vec![10.0, 13.5]);
        assert_eq!(RoomConfig::load(&path).champion, "darius");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_config_falls_back_to_default() {
        let config = RoomConfig::load(Path::new("nope/none.ron"));
        assert_eq!(config.champion, RoomConfig::default().champion);
        assert!(config.pause_at.is_empty());
    }

    #[test]
    fn room_config_is_a_valid_game_profile() {
        // 游戏用 RoomProfile 读同一份文件，必须能忽略多余字段。
        let config = RoomConfig {
            pause_at: vec![1.0, 2.0],
            ..Default::default()
        };
        let text = ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default()).unwrap();
        let profile: RoomProfile = ron::from_str(&text).unwrap();
        assert_eq!(profile.pause_at, vec![1.0, 2.0]);
    }
}
