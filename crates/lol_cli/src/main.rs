use clap::{Parser, Subcommand};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Parser)]
#[command(name = "lol-cli")]
#[command(about = "命令行工具：自动连接当前游戏，供外部 Agent 对其进行观测与控制", long_about = None)]
struct Cli {
    /// 游戏 WebSocket 服务的端口号
    #[arg(long, default_value = "9001")]
    port: u16,

    /// 指定操作或观测的英雄实体 ID
    #[arg(long)]
    entity_id: Option<u64>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// 获取当前英雄的局势观测数据 (Observe JSON)
    #[command(alias = "obs")]
    Observe,

    /// 下达英雄动作指令
    #[command(subcommand)]
    #[command(alias = "act")]
    Action(ActionSubcommand),

    /// 暂停游戏时间流速
    Pause,

    /// 继续/恢复游戏时间流速
    Unpause,

    /// 获取调试服务当前的基本状态
    State,
}

#[derive(Subcommand, Clone, Debug)]
enum ActionSubcommand {
    /// 移动到指定坐标
    Move {
        /// 目标 X 坐标
        x: f32,
        /// 目标 Y 坐标
        y: f32,
    },

    /// 攻击指定目标实体
    Attack {
        /// 目标实体 ID
        entity: u64,
    },

    /// 停止所有动作
    Stop,

    /// 释放指定索引的技能到指定坐标
    Skill {
        /// 技能索引 (0-3)
        index: usize,
        /// 目标 X 坐标
        x: f32,
        /// 目标 Y 坐标
        y: f32,
    },

    /// 升级指定索引的技能
    #[command(alias = "upgrade")]
    SkillLevelUp {
        /// 技能索引 (0-3)
        index: usize,
    },
}

#[derive(Serialize)]
struct WsRequest {
    id: u64,
    cmd: String,
    params: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct WsResponse {
    pub id: u64,
    ok: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let url = format!("ws://127.0.0.1:{}", cli.port);

    // 建立 WebSocket 连接
    let (ws_stream, _) = match connect_async(&url).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "错误: 无法连接到游戏 WebSocket 服务端 {}。请确保游戏主程序已启动并在该端口监听。",
                url
            );
            eprintln!("底层错误详情: {}", e);
            std::process::exit(1);
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // 针对 Pause/Unpause 进行幂等性预检测
    match &cli.command {
        Commands::Pause | Commands::Unpause => {
            // 首先查询状态
            let req = WsRequest {
                id: 1,
                cmd: "get_state".to_string(),
                params: serde_json::Value::Null,
            };
            write
                .send(Message::Text(serde_json::to_string(&req)?.into()))
                .await?;

            if let Some(msg_res) = read.next().await {
                let msg = msg_res?;
                if let Message::Text(text) = msg {
                    let resp: WsResponse = serde_json::from_str(&text)?;
                    if resp.ok {
                        if let Some(data) = resp.data {
                            let current_paused = data
                                .get("paused")
                                .and_then(|p| p.as_bool())
                                .unwrap_or(false);

                            match &cli.command {
                                Commands::Pause => {
                                    if current_paused {
                                        println!("游戏时间流速已经是暂停状态，无需操作。");
                                        return Ok(());
                                    }
                                }
                                Commands::Unpause => {
                                    if !current_paused {
                                        println!("游戏时间流速已经是运行状态，无需操作。");
                                        return Ok(());
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    // 组装并发送请求包
    let (cmd_str, params) = match &cli.command {
        Commands::Observe => {
            let p = if let Some(eid) = cli.entity_id {
                serde_json::json!({ "entity_id": eid })
            } else {
                serde_json::Value::Null
            };
            ("get_observe".to_string(), p)
        }
        Commands::Action(subcmd) => {
            let val = match subcmd {
                ActionSubcommand::Move { x, y } => serde_json::json!({ "Move": [x, y] }),
                ActionSubcommand::Attack { entity } => serde_json::json!({ "Attack": entity }),
                ActionSubcommand::Stop => serde_json::json!("Stop"),
                ActionSubcommand::Skill { index, x, y } => serde_json::json!({
                    "Skill": { "index": index, "point": [x, y] }
                }),
                ActionSubcommand::SkillLevelUp { index } => serde_json::json!({ "SkillLevelUp": index }),
            };
            let p = if let Some(eid) = cli.entity_id {
                serde_json::json!({ "entity_id": eid, "action": val })
            } else {
                val
            };
            ("action".to_string(), p)
        }
        Commands::Pause | Commands::Unpause => {
            ("toggle_pause".to_string(), serde_json::Value::Null)
        }
        Commands::State => ("get_state".to_string(), serde_json::Value::Null),
    };

    let req = WsRequest {
        id: 2,
        cmd: cmd_str,
        params,
    };

    let req_text = serde_json::to_string(&req)?;
    write.send(Message::Text(req_text.into())).await?;

    // 读取并漂亮打印结果
    if let Some(msg_res) = read.next().await {
        let msg = msg_res?;
        if let Message::Text(text) = msg {
            let resp: WsResponse = serde_json::from_str(&text)?;
            if resp.ok {
                if let Some(data) = resp.data {
                    // 格式化输出 JSON 数据供 Agent 完美读取
                    println!("{}", serde_json::to_string_pretty(&data)?);
                } else {
                    println!("指令执行成功。");
                }
            } else {
                let err_msg = resp.error.unwrap_or_else(|| "未知错误".to_string());
                eprintln!("错误: 指令执行失败: {}", err_msg);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
