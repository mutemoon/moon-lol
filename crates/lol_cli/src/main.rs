use std::process::exit;

use clap::{Parser, Subcommand};
use lol_client::{Action, GameClient, WsResponse, start_ws_client};

#[derive(Parser)]
#[command(name = "lol-cli")]
#[command(about = "命令行工具：自动连接当前游戏，供外部 Agent 对其进行观测与控制", long_about = None)]
struct Cli {
    /// 游戏 WebSocket 服务的端口号
    #[arg(long, default_value = "9001")]
    port: u16,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// 获取当前英雄的局势观测数据 (Observe JSON)
    Observe {
        /// 指定操作或观测的英雄实体 ID
        #[arg(short, long)]
        entity_id: u64,
    },

    /// 下达英雄动作指令
    Action {
        /// 指定操作或观测的英雄实体 ID
        #[arg(short, long)]
        entity_id: u64,

        #[command(subcommand)]
        subcommand: ActionSubcommand,
    },

    /// 暂停游戏时间流速（幂等：已暂停则不操作）
    Pause,

    /// 继续/恢复游戏时间流速（幂等：已运行则不操作）
    Unpause,

    /// 获取调试服务当前的基本状态
    State,

    /// 列出当前所有英雄实体及其 agent_id
    Agents,

    /// 切换当前英雄
    SwitchChampion {
        /// 英雄名称 (如 Riven / Fiora)
        name: String,
    },

    /// 开关上帝模式（免疫伤害）
    GodMode {
        /// true 启用，false 关闭
        enabled: bool,
    },

    /// 开关技能冷却
    ToggleCooldown {
        /// true 关闭冷却（Manual），false 恢复冷却
        enabled: bool,
    },

    /// 重置当前英雄位置到原点
    ResetPosition,

    /// 为指定实体附加/热重载 Script Agent 脚本
    SetScript {
        /// 目标英雄实体 ID
        #[arg(short, long)]
        entity_id: u64,
        /// 脚本源码
        source: String,
    },

    /// RL 环境 reset：初始化并返回初始观测
    RlReset {
        /// 目标英雄实体 ID
        #[arg(short, long)]
        entity_id: u64,
        /// 可选的 reward 配置 JSON
        #[arg(long)]
        config_json: Option<String>,
    },

    /// RL 环境 step：推进指定帧数并返回 reward 与新观测
    RlStep {
        /// 目标英雄实体 ID
        #[arg(short, long)]
        entity_id: u64,
        /// 可选的推进帧数 (默认由服务端定)
        #[arg(short, long)]
        frames: Option<u32>,
    },
}

#[derive(Subcommand, Clone, Debug)]
enum ActionSubcommand {
    /// 移动到指定坐标
    Move { x: f32, y: f32 },
    /// 攻击指定目标实体
    Attack { entity: u64 },
    /// 停止所有动作
    Stop,
    /// 释放指定索引的技能到指定坐标
    Skill {
        /// 技能索引 (0-3)
        index: usize,
        x: f32,
        y: f32,
    },
    /// 升级指定索引的技能
    #[command(alias = "upgrade")]
    SkillLevelUp {
        /// 技能索引 (0-3)
        index: usize,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let url = format!("ws://127.0.0.1:{}", cli.port);

    let session = match start_ws_client(cli.port, None).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "错误: 无法连接到游戏 WebSocket 服务端 {}。请确保游戏主程序已启动并在该端口监听。",
                url
            );
            eprintln!("底层错误详情: {}", e);
            exit(1);
        }
    };

    let client = GameClient::new(session);

    match run(&client, cli.command).await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("错误: {}", e);
            exit(1);
        }
    }
}

async fn run(client: &GameClient, command: Commands) -> Result<(), String> {
    match command {
        Commands::Observe { entity_id } => print_data(client.observe(entity_id, false).await?),
        Commands::Action {
            entity_id,
            subcommand,
        } => {
            let action = match subcommand {
                ActionSubcommand::Move { x, y } => Action::Move([x, y]),
                ActionSubcommand::Attack { entity } => Action::Attack(entity),
                ActionSubcommand::Stop => Action::Stop,
                ActionSubcommand::Skill { index, x, y } => Action::Skill {
                    index,
                    point: [x, y],
                },
                ActionSubcommand::SkillLevelUp { index } => Action::SkillLevelUp(index),
            };
            print_data(client.action(entity_id, action).await?)
        }
        Commands::Pause => {
            if client.pause().await? {
                println!("已暂停游戏时间流速。");
            } else {
                println!("游戏时间流速已经是暂停状态，无需操作。");
            }
            Ok(())
        }
        Commands::Unpause => {
            if client.unpause().await? {
                println!("已恢复游戏时间流速。");
            } else {
                println!("游戏时间流速已经是运行状态，无需操作。");
            }
            Ok(())
        }
        Commands::State => print_data(client.state().await?),
        Commands::Agents => print_data(client.agents().await?),
        Commands::SwitchChampion { name } => print_data(client.switch_champion(&name).await?),
        Commands::GodMode { enabled } => print_data(client.god_mode(enabled).await?),
        Commands::ToggleCooldown { enabled } => print_data(client.toggle_cooldown(enabled).await?),
        Commands::ResetPosition => print_data(client.reset_position().await?),
        Commands::SetScript { entity_id, source } => {
            print_data(client.set_script(entity_id, &source).await?)
        }
        Commands::RlReset {
            entity_id,
            config_json,
        } => {
            let cfg = match config_json {
                Some(s) => Some(serde_json::from_str(&s).map_err(|e| e.to_string())?),
                None => None,
            };
            print_data(client.rl_reset(entity_id, cfg).await?)
        }
        Commands::RlStep { entity_id, frames } => print_data(client.rl_step(entity_id, frames).await?),
    }
}

/// 漂亮打印响应：成功时输出 data（无 data 则提示成功），失败时返回错误。
fn print_data(resp: WsResponse) -> Result<(), String> {
    if resp.ok {
        match resp.data {
            Some(serde_json::Value::String(s)) => println!("{}", s),
            Some(data) => println!("{}", serde_json::to_string_pretty(&data).unwrap()),
            None => println!("指令执行成功。"),
        }
        Ok(())
    } else {
        Err(resp.error.unwrap_or_else(|| "未知错误".to_string()))
    }
}
