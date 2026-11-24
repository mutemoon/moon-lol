//! `lol`：本地游戏会话的命令行。

mod commands;
mod error;
mod paths;
mod room;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use error::{Error, Result};
use lol_client::GameClient;

#[derive(Parser)]
#[command(name = "lol")]
#[command(about = "本地游戏会话：创建房间、启动游戏、截图与全量快照", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 房间：创建 / 配置 / 启动 / 管理本地游戏会话
    Room {
        /// 房间 id（省略时由子命令给出，如 `lol room create` 自动生成）
        #[arg(long, global = true)]
        id: Option<String>,

        #[command(subcommand)]
        action: RoomAction,
    },

    /// 游戏：对一个正在运行的房间执行观察操作
    Game {
        /// 房间 id；省略时使用唯一在跑的房间
        #[arg(long, global = true)]
        id: Option<String>,

        #[command(subcommand)]
        action: GameAction,
    },

    /// 列出所有房间
    Ls,

    /// 把房间的游戏日志打印到标准输出
    Log {
        #[arg(long)]
        id: Option<String>,
        /// 最多打印的行数（从末尾算起）
        #[arg(long, default_value_t = 80)]
        lines: usize,
        /// 跟随日志输出（Ctrl-C 退出）
        #[arg(short, long)]
        follow: bool,
    },
}

#[derive(Subcommand)]
enum RoomAction {
    /// 新建房间（自动生成 id，除非 `--id`）
    Create {
        /// 自机英雄（小写，如 riven / darius）
        #[arg(long)]
        champion: Option<String>,
        /// 场景名；省略用默认 classic
        #[arg(long)]
        scene: Option<String>,
    },
    /// 修改房间配置（暂停计划等）
    Config {
        /// 游戏内时间自动暂停，逗号分隔（如 00:10,00:13,00:20 或 10,13.5）
        #[arg(long)]
        pause_at: Option<String>,
        #[arg(long)]
        champion: Option<String>,
        #[arg(long)]
        scene: Option<String>,
    },
    /// 启动游戏进程
    Start,
    /// 终止房间的游戏进程
    Kill,
    /// 等待游戏控制面就绪并打印状态
    Wait {
        /// 最长等待秒数
        #[arg(long, default_value_t = 180)]
        timeout: u64,
    },
}

#[derive(Subcommand)]
enum GameAction {
    /// 截图主窗口到 <房间>/shots/
    Screenshot {
        /// 输出路径；省略时写 <房间>/shots/<时间戳>.png
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// 导出所有实体的所有组件为 JSON，写到 <房间>/snaps/<时间戳>.json
    Snapshot {
        /// 只保留名字含该子串的组件
        #[arg(long)]
        component: Option<String>,
        /// 只保留组件值含该子串的实体
        #[arg(long)]
        value_contains: Option<String>,
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// 恢复游戏时间
    Unpause,
    /// 暂停游戏时间
    Pause,
    /// 查询暂停状态与英雄
    State,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("错误: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Room { id, action } => room_command(id, action).await,
        Commands::Game { id, action } => game_command(id, action).await,
        Commands::Ls => commands::list(),
        Commands::Log { id, lines, follow } => log_command(id, lines, follow).await,
    }
}

async fn room_command(id: Option<String>, action: RoomAction) -> Result<()> {
    match action {
        RoomAction::Create { champion, scene } => {
            let id = id.unwrap_or_else(new_room_id);
            ensure_valid_id(&id)?;
            commands::create(&id, champion, scene)
        }
        RoomAction::Config {
            pause_at,
            champion,
            scene,
        } => {
            let id = require_id(id)?;
            ensure_valid_id(&id)?;
            let pause_at = pause_at.map(|spec| parse_times(&spec)).transpose()?;
            commands::configure(&id, pause_at, champion, scene)
        }
        RoomAction::Start => {
            let id = require_id(id)?;
            ensure_valid_id(&id)?;
            commands::start(&id).await
        }
        RoomAction::Kill => {
            let id = require_id(id)?;
            ensure_valid_id(&id)?;
            commands::kill(&id)
        }
        RoomAction::Wait { timeout } => {
            let id = require_id(id)?;
            wait_ready(&id, timeout).await
        }
    }
}

async fn game_command(id: Option<String>, action: GameAction) -> Result<()> {
    let id = resolve_room(id)?;
    let state = commands::live_state(&id)?;
    let session = lol_client::start_ws_client(state.port, None)
        .await
        .map_err(|source| Error::NoGameControl {
            port: state.port,
            source,
        })?;
    let client = GameClient::new(session);

    match action {
        GameAction::Screenshot { out } => {
            let path = commands::screenshot(&client, &state, out).await?;
            println!("已保存图片到 {}", path.display());
        }
        GameAction::Snapshot {
            component,
            value_contains,
            out,
        } => {
            let (path, count) =
                commands::snapshot(&client, &state, component, value_contains, out).await?;
            println!("已保存快照到 {}（{count} 个实体）", path.display());
        }
        GameAction::Unpause => {
            let switched = client
                .unpause()
                .await
                .map_err(|e| crate::error::control_error(state.port, &e))?;
            println!(
                "{}",
                if switched {
                    "已恢复游戏时间"
                } else {
                    "游戏时间本就在运行"
                }
            );
        }
        GameAction::Pause => {
            let switched = client
                .pause()
                .await
                .map_err(|e| crate::error::control_error(state.port, &e))?;
            println!(
                "{}",
                if switched {
                    "已暂停游戏时间"
                } else {
                    "游戏时间本就已暂停"
                }
            );
        }
        GameAction::State => {
            let resp = client
                .state()
                .await
                .map_err(|e| crate::error::control_error(state.port, &e))?;
            print_json(resp.data)?;
        }
    }
    Ok(())
}

async fn log_command(id: Option<String>, lines: usize, follow: bool) -> Result<()> {
    let id = resolve_room(id)?;
    let state = room::RoomState::load(&id)?;
    if follow {
        // 简版 tail -f：反复读取新内容直到 Ctrl-C。
        let mut file = std::fs::File::open(&state.log)?;
        use std::io::{BufRead, BufReader, Seek, SeekFrom};
        file.seek(SeekFrom::End(0))?;
        let mut reader = BufReader::new(file);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line)? {
                0 => tokio::time::sleep(std::time::Duration::from_millis(200)).await,
                _ => print!("{line}"),
            }
        }
    }
    let text = std::fs::read_to_string(&state.log)?;
    let tail: Vec<&str> = text.lines().rev().take(lines).collect();
    for line in tail.into_iter().rev() {
        println!("{line}");
    }
    Ok(())
}

/// `lol room --id X wait`：轮询控制面直到可用（用于脚本里替代固定 sleep）。
async fn wait_ready(id: &str, timeout_secs: u64) -> Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        let state = commands::live_state(id)?;
        if let Ok(session) = lol_client::start_ws_client(state.port, None).await {
            let client = GameClient::new(session);
            if client.state().await.is_ok() {
                println!("房间 {id} 已就绪（端口 {}）", state.port);
                return Ok(());
            }
        }
        if std::time::Instant::now() > deadline {
            return Err(Error::User(format!(
                "等待房间 {id} 就绪超时（{timeout_secs}s）"
            )));
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}

/// 省略 `--id` 时：唯一在跑的房间。
fn resolve_room(id: Option<String>) -> Result<String> {
    if let Some(id) = id {
        ensure_valid_id(&id)?;
        return Ok(id);
    }
    let live: Vec<String> = room::list_rooms()
        .into_iter()
        .filter(|id| {
            room::RoomState::load(id)
                .map(|s| s.is_alive())
                .unwrap_or(false)
        })
        .collect();
    match live.len() {
        1 => Ok(live.into_iter().next().unwrap()),
        0 => Err(Error::User(
            "当前没有运行中的房间，先 `lol room create && lol room start`".into(),
        )),
        _ => Err(Error::User(format!(
            "有多个运行中的房间（{}），请用 --id 指定",
            live.join(", ")
        ))),
    }
}

fn require_id(id: Option<String>) -> Result<String> {
    id.ok_or_else(|| Error::User("缺少 --id（先 `lol room create` 得到 id）".into()))
}

fn ensure_valid_id(id: &str) -> Result<()> {
    if paths::is_valid_room_id(id) {
        Ok(())
    } else {
        Err(Error::User(format!(
            "非法房间 id {id:?}（只允许小写字母、数字与 '-'）"
        )))
    }
}

fn new_room_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("local-{:x}", nanos)
}

/// 解析暂停时刻：`00:10,00:13.5` 或 `10,13.5`。
fn parse_times(spec: &str) -> Result<Vec<f64>> {
    spec.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_time)
        .collect()
}

fn parse_time(text: &str) -> Result<f64> {
    let invalid = || Error::User(format!("无法解析暂停时刻 {text:?}（格式 mm:ss 或秒）"));
    let secs = match text.split_once(':') {
        Some((mm, ss)) => {
            let minutes: f64 = mm.parse().map_err(|_| invalid())?;
            let seconds: f64 = ss.parse().map_err(|_| invalid())?;
            minutes * 60.0 + seconds
        }
        None => text.parse().map_err(|_| invalid())?,
    };
    if !secs.is_finite() || secs < 0.0 {
        return Err(invalid());
    }
    Ok(secs)
}

fn print_json(value: Option<serde_json::Value>) -> Result<()> {
    match value {
        Some(value) => println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(|e| Error::Serialize(e.to_string()))?
        ),
        None => println!("（无数据）"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mm_ss_and_seconds() {
        assert_eq!(parse_times("00:10,00:13.5").unwrap(), vec![10.0, 13.5]);
        assert_eq!(parse_times("10, 20").unwrap(), vec![10.0, 20.0]);
        assert_eq!(parse_times("  ").unwrap(), Vec::<f64>::new());
    }

    #[test]
    fn rejects_garbage_times() {
        assert!(parse_times("abc").is_err());
        assert!(parse_times("-1").is_err());
        assert!(parse_times("1:xx").is_err());
    }
}
