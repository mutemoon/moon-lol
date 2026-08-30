use std::process::exit;
use std::time::Duration;

use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use lol_rl_protocol::{
    CurriculumConfig, DEFAULT_RL_SERVER_ADDR, InFrame, OutFrame, PolicyBackbone, RlAlgorithm,
    TaskConfigPayload, get_env_training_params,
};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Parser, Debug)]
#[command(name = "lol_rl_cli")]
#[command(
    about = "Moon LOL 强化学习训练任务控制 CLI 工具：与 RL 服务通信触发训练，供 Agent 自主调试与自动化运行",
    long_about = None
)]
pub struct Cli {
    /// RL 训练服务 WebSocket 地址 (如: 127.0.0.1:8765)
    #[arg(long, default_value = DEFAULT_RL_SERVER_ADDR)]
    pub server_addr: String,

    /// 任务名称
    #[arg(long, default_value = "RL 对战训练任务")]
    pub name: String,

    /// 训练环境名称 (如: solo_v0, fiora_v0, fiora_v1, fiora_v2)
    #[arg(long, default_value = "fiora_v2")]
    pub env: String,

    /// 强化学习训练算法 (ppo 或 grpo)
    #[arg(long, default_value = "ppo")]
    pub algo: String,

    /// 特征主干网络架构 (mlp 或 mamba)
    #[arg(long, default_value = "mlp")]
    pub backbone: String,

    /// 训练引擎模式 (async 或 sync)
    #[arg(long, default_value = "sync")]
    pub engine: String,

    /// 学习率 (Learning Rate / lr)
    #[arg(long)]
    pub lr: Option<f32>,

    /// 折扣因子 (Gamma / γ)
    #[arg(long)]
    pub gamma: Option<f32>,

    /// GAE 折扣因子 (Lambda / λ)
    #[arg(long)]
    pub gae_lambda: Option<f32>,

    /// PPO/GRPO Clip 截断系数 (Clip Eps / ε)
    #[arg(long)]
    pub clip_eps: Option<f32>,

    /// 每轮训练 Epochs
    #[arg(long)]
    pub ppo_epochs: Option<usize>,

    /// 策略网络隐藏层维度 / d_model
    #[arg(long)]
    pub hidden_dim: Option<usize>,

    /// 并行对局环境数 (0 为自动自适应探测)
    #[arg(long, default_value_t = 0)]
    pub parallel_envs: usize,

    /// 每环境每轮采样步数 (rollout steps)
    #[arg(long)]
    pub rollout_steps_per_env: Option<usize>,

    /// 总训练迭代轮次 (total iterations)
    #[arg(long, short = 'n')]
    pub total_iterations: Option<usize>,

    /// 课程学习配置 JSON 字符串 (可选)
    #[arg(long)]
    pub curriculum_json: Option<String>,

    /// GRPO 算法每组环境/轨迹大小（默认 4）
    #[arg(long)]
    pub grpo_group_size: Option<usize>,

    /// 直接传入完整的 TaskConfigPayload JSON 字符串 (可选，提供时覆盖其他所有参数)
    #[arg(long)]
    pub config_json: Option<String>,

    /// 提交任务后立即退出（不实时跟随日志与指标）
    #[arg(long)]
    pub detach: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = if let Some(json_str) = &cli.config_json {
        match serde_json::from_str::<TaskConfigPayload>(json_str) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("❌ 解析 --config-json 失败: {e}");
                exit(1);
            }
        }
    } else {
        let env_params = get_env_training_params(&cli.env);
        let algorithm = cli.algo.parse::<RlAlgorithm>().unwrap_or_else(|_| {
            eprintln!("⚠️ 未知算法 '{}'，默认使用 PPO", cli.algo);
            RlAlgorithm::Ppo
        });
        let backbone = cli.backbone.parse::<PolicyBackbone>().unwrap_or_else(|_| {
            eprintln!("⚠️ 未知主干架构 '{}'，默认使用 MLP", cli.backbone);
            PolicyBackbone::Mlp
        });
        let engine_mode = cli
            .engine
            .parse::<lol_rl_protocol::EngineMode>()
            .unwrap_or_else(|_| {
                eprintln!("⚠️ 未知引擎模式 '{}'，默认使用 Async", cli.engine);
                lol_rl_protocol::EngineMode::Async
            });

        let curriculum = if let Some(curriculum_str) = &cli.curriculum_json {
            match serde_json::from_str::<CurriculumConfig>(curriculum_str) {
                Ok(c) => Some(c),
                Err(e) => {
                    eprintln!("❌ 解析 --curriculum-json 失败: {e}");
                    exit(1);
                }
            }
        } else {
            None
        };

        TaskConfigPayload {
            name: cli.name,
            env_name: cli.env,
            algorithm,
            backbone,
            engine_mode,
            lr: cli.lr.unwrap_or(env_params.lr),
            gamma: cli.gamma.unwrap_or(env_params.gamma),
            gae_lambda: cli.gae_lambda.unwrap_or(env_params.gae_lambda),
            clip_eps: cli.clip_eps.unwrap_or(env_params.clip_eps),
            ppo_epochs: cli.ppo_epochs.unwrap_or(env_params.ppo_epochs),
            hidden_dim: cli.hidden_dim.unwrap_or(env_params.hidden_dim),
            parallel_envs: cli.parallel_envs,
            rollout_steps_per_env: cli
                .rollout_steps_per_env
                .unwrap_or(env_params.rollout_steps_per_env),
            total_iterations: cli.total_iterations.unwrap_or(env_params.total_iterations),
            curriculum,
            grpo_group_size: cli.grpo_group_size,
        }
    };

    let ws_url = if cli.server_addr.starts_with("ws://") {
        cli.server_addr.clone()
    } else {
        format!("ws://{}", cli.server_addr)
    };

    println!("🚀 [lol_rl_cli] 正在连接 RL 训练服务: {ws_url} ...");
    let (ws_stream, _) = match connect_async(&ws_url).await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("❌ 连接 RL 训练服务失败 ({ws_url}): {e}");
            eprintln!("💡 请确保后台 lol_server / lol_rl 服务正在运行。");
            exit(1);
        }
    };
    println!("✅ 成功连接到 RL 训练服务！");

    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    let create_frame = InFrame::CreateTask {
        config: config.clone(),
    };
    let json_msg = serde_json::to_string(&create_frame).expect("序列化 CreateTask 帧失败");
    if let Err(e) = ws_writer.send(Message::Text(json_msg.into())).await {
        eprintln!("❌ 发送创建任务请求失败: {e}");
        exit(1);
    }
    println!("📤 已提交训练任务配置...");

    let mut created_task_id = None;
    let listen_result = tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(msg_res) = ws_reader.next().await {
            match msg_res {
                Ok(Message::Text(text)) => {
                    if let Ok(out_frame) = serde_json::from_str::<OutFrame>(&text) {
                        match out_frame {
                            OutFrame::Status { task_id, status } => {
                                println!("📢 [服务端状态更新] 任务: {task_id}, 状态: {status}");
                                created_task_id = Some(task_id);
                                if status == "running" {
                                    break;
                                }
                            }
                            OutFrame::TaskList { tasks } => {
                                if let Some(latest) = tasks.first() {
                                    created_task_id = Some(latest.id.clone());
                                }
                            }
                            OutFrame::Log { message, .. } => {
                                println!("📝 {message}");
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    eprintln!("⚠️ 服务端关闭了连接");
                    break;
                }
                Err(e) => {
                    eprintln!("⚠️ 读取 WebSocket 帧异常: {e}");
                    break;
                }
                _ => {}
            }
        }
    })
    .await;

    if listen_result.is_err() {
        eprintln!("⚠️ 等待服务端任务确认超时 (10s)，但请求已发送至服务端。");
    }

    println!("\n================================================================================");
    println!("✅ [lol_rl] RL 训练任务已成功启动！");
    println!("--------------------------------------------------------------------------------");
    if let Some(tid) = &created_task_id {
        println!("📋 任务 ID:       {}", tid);
    }
    println!("🏷️ 任务名称:     {}", config.name);
    println!("🎮 训练环境:     {}", config.env_name);
    println!(
        "🧠 训练算法:     {} (Backbone: {})",
        config.algorithm.display_name(),
        config.backbone.display_name()
    );
    println!("⚙️ 隐藏层维度:   {}", config.hidden_dim);
    println!(
        "⚡ 并行环境数:   {}",
        if config.parallel_envs == 0 {
            "0 (自适应算力探测)".to_string()
        } else {
            config.parallel_envs.to_string()
        }
    );
    println!(
        "🔄 迭代总轮次:   {} 轮 (每轮采样: {} 步)",
        config.total_iterations, config.rollout_steps_per_env
    );
    println!(
        "📈 学习率超参:   lr={}, gamma={}, gae_lambda={}, clip_eps={}",
        config.lr, config.gamma, config.gae_lambda, config.clip_eps
    );
    println!("--------------------------------------------------------------------------------");
    println!("💡 正在实时同步训练遥测与性能分析日志 (按 Ctrl+C 可断开监听，后台任务不受影响)...");
    println!("================================================================================\n");

    if cli.detach {
        let _ = ws_writer.close().await;
        return;
    }

    // 持续监听来自服务端的日志与指标
    let active_task_id = created_task_id.unwrap_or_default();
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("\n🛑 [lol_rl_cli] 已断开控制台监听（训练任务正在服务端后台持续运行）。");
                break;
            }
            msg_opt = ws_reader.next() => {
                let Some(msg_res) = msg_opt else {
                    println!("⚠️ [lol_rl_cli] 与服务端的数据流已结束。");
                    break;
                };

                match msg_res {
                    Ok(Message::Text(text)) => {
                        if let Ok(out_frame) = serde_json::from_str::<OutFrame>(&text) {
                            match out_frame {
                                OutFrame::Log { task_id, message, .. } => {
                                    if active_task_id.is_empty() || task_id == active_task_id {
                                        println!("📝 {message}");
                                    }
                                }
                                OutFrame::Metrics { task_id, step, ep_return, fps, total_loss, policy_loss, value_loss, .. } => {
                                    if active_task_id.is_empty() || task_id == active_task_id {
                                        println!(
                                            "📊 [Step {:>7}] FPS: {:>5} | Avg Return: {:>6.2} | Total Loss: {:>7.4} (P: {:>6.4}, V: {:>6.4})",
                                            step, fps, ep_return, total_loss, policy_loss, value_loss
                                        );
                                    }
                                }
                                OutFrame::Status { task_id, status } => {
                                    if active_task_id.is_empty() || task_id == active_task_id {
                                        println!("📢 [状态变更] 任务 {task_id}: {status}");
                                        if status == "finished" {
                                            println!("🎉 [lol_rl] 训练任务已成功收敛完成！");
                                            break;
                                        } else if status == "stopped" || status == "interrupted" {
                                            println!("🛑 [lol_rl] 训练任务已停止 ({status})。");
                                            break;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("⚠️ [lol_rl_cli] 服务端关闭了连接");
                        break;
                    }
                    Err(e) => {
                        eprintln!("⚠️ [lol_rl_cli] 读取异常: {e}");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    let _ = ws_writer.close().await;
}
