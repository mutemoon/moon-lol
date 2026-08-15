use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};

use candle_core::Tensor;
use futures_util::{SinkExt, StreamExt};
use lol_env::visual_runner::{VisualRunnerCmd, VisualStepOutput, run_visual_env};
use lol_env::{EnvConfig, RenderMode, VisualEnvironment};
use lol_rl::device::select_device;
use lol_rl::ppo::{PPOAgent, PPOConfig};
use lol_rl_protocol::{
    ObsFeaturePayload, PolicyDisplay, RewardItem, VisualInFrame, VisualObsFrame, VisualOutFrame,
};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let port = parse_arg("--port")
        .unwrap_or_else(|| "9320".to_string())
        .parse::<u16>()?;
    let ckpt_path = parse_arg("--checkpoint").map(PathBuf::from);
    let hidden_dim = parse_arg("--hidden-dim")
        .or_else(|| parse_arg("--hidden_dim"))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(64);
    let env_name = parse_arg("--env")
        .or_else(|| parse_arg("--env_name"))
        .unwrap_or_else(|| lol_rl_protocol::ENV_FIORA_VS_RIVEN_REAL.to_string());

    if env_name == lol_rl_protocol::ENV_FIORA_VS_RIVEN_LEGACY {
        start_visual_runner_for_env::<lol_env::FioraVsRivenEnv>(port, ckpt_path, hidden_dim)
    } else {
        start_visual_runner_for_env::<lol_env::FioraVsRivenRealEnv>(port, ckpt_path, hidden_dim)
    }
}

fn start_visual_runner_for_env<E: VisualEnvironment>(
    port: u16,
    ckpt_path: Option<PathBuf>,
    hidden_dim: usize,
) -> anyhow::Result<()> {
    let env_name = E::env_name().to_string();
    let display_name = E::display_name();
    println!(">>> 正在启动 {display_name} 可视化环境 (Port: {port})...");

    let device = select_device()?;
    let config = PPOConfig::default();
    let state_dim = E::state_dim();
    let action_space = E::action_space();
    let action_labels: Vec<String> = E::action_labels().iter().map(|s| s.to_string()).collect();

    let agent = if let Some(ref path) = ckpt_path {
        if path.exists() {
            println!(">>> 加载 Checkpoint: {}", path.display());
            Arc::new(PPOAgent::load(
                state_dim,
                hidden_dim,
                action_space,
                config,
                device.clone(),
                path,
            )?)
        } else {
            println!(
                ">>> 指定的 Checkpoint 不存在 ({})，使用初始策略启动",
                path.display()
            );
            Arc::new(PPOAgent::new(
                state_dim,
                hidden_dim,
                action_space,
                config,
                device.clone(),
            )?)
        }
    } else {
        println!(">>> 未指定 --checkpoint，使用初始随机策略启动");
        Arc::new(PPOAgent::new(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device.clone(),
        )?)
    };

    let env = E::with_config(EnvConfig {
        max_steps: 0,
        render_mode: RenderMode::WindowCustomLoop,
    });

    let (cmd_tx, cmd_rx) = mpsc::channel::<VisualRunnerCmd>();
    let (step_tx, step_rx) = mpsc::channel::<VisualStepOutput>();

    let ckpt_path_clone = ckpt_path.unwrap_or_else(|| PathBuf::from("default_random"));
    let cmd_tx_clone = cmd_tx.clone();
    let env_name_clone = env_name.clone();
    let action_labels_clone = action_labels.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(ws_server(
            port,
            step_rx,
            ckpt_path_clone,
            env_name_clone,
            action_labels_clone,
            cmd_tx_clone,
        ));
    });

    let agent_clone = agent.clone();
    let enc_dim = action_space.encoding_dim();
    let policy = move |obs: &E::Obs| -> (E::Action, PolicyDisplay) {
        let obs_vec = E::obs_to_vector(obs);
        let state = match Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device) {
            Ok(t) => t,
            Err(_) => {
                return (
                    E::action_from_encoding(&vec![0.0; enc_dim]),
                    PolicyDisplay::Discrete(vec![]),
                );
            }
        };

        let labels = E::action_labels();
        let display = agent_clone
            .actor_critic
            .policy_display_real(&state, &obs_vec, labels)
            .unwrap_or(PolicyDisplay::Discrete(vec![]));

        let chosen = match agent_clone
            .actor_critic
            .select_greedy_action(&state, &obs_vec)
        {
            Ok(encoded) => E::action_from_encoding(&encoded),
            Err(_) => E::action_from_encoding(&vec![0.0; enc_dim]),
        };

        (chosen, display)
    };

    run_visual_env(env, policy, cmd_rx, step_tx);

    Ok(())
}

async fn ws_server(
    port: u16,
    step_rx: mpsc::Receiver<VisualStepOutput>,
    ckpt_path: PathBuf,
    env_name: String,
    action_labels: Vec<String>,
    cmd_tx: mpsc::Sender<VisualRunnerCmd>,
) {
    let listener = match TcpListener::bind(format!("127.0.0.1:{port}")) {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("WS 端口 {} 绑定失败: {}", port, e);
            return;
        }
    };
    listener.set_nonblocking(true).ok();
    let listener = tokio::net::TcpListener::from_std(listener).expect("TcpListener");

    tracing::info!("视觉 WS 服务启动在 ws://127.0.0.1:{}", port);

    let (broadcast_tx, _) = tokio::sync::broadcast::channel::<VisualObsFrame>(512);
    let latest_frame = Arc::new(std::sync::Mutex::new(None::<VisualObsFrame>));

    let latest_frame_clone = latest_frame.clone();
    let broadcast_tx_clone = broadcast_tx.clone();
    std::thread::spawn(move || {
        for output in step_rx {
            let frame = step_output_to_frame_data(&output);
            if let Ok(mut lock) = latest_frame_clone.lock() {
                *lock = Some(frame.clone());
            }
            let _ = broadcast_tx_clone.send(frame);
        }
    });

    while let Ok((stream, _)) = listener.accept().await {
        let ckpt = ckpt_path.clone();
        let cmd = cmd_tx.clone();
        let frame_rx = broadcast_tx.subscribe();
        let latest_frame_sub = latest_frame.clone();
        let env_name_sub = env_name.clone();
        let action_labels_sub = action_labels.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_ws_client(
                stream,
                frame_rx,
                latest_frame_sub,
                ckpt,
                env_name_sub,
                action_labels_sub,
                cmd,
            )
            .await
            {
                tracing::warn!("WS 客户端连接关闭: {}", e);
            }
        });
    }
}

async fn handle_ws_client(
    stream: TcpStream,
    mut frame_rx: tokio::sync::broadcast::Receiver<VisualObsFrame>,
    latest_frame: Arc<std::sync::Mutex<Option<VisualObsFrame>>>,
    ckpt_path: PathBuf,
    env_name: String,
    action_labels: Vec<String>,
    cmd_tx: mpsc::Sender<VisualRunnerCmd>,
) -> anyhow::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    // 1. 发送包含环境与动作元数据的 Ready 帧
    let ready = VisualOutFrame::Ready {
        checkpoint_path: ckpt_path.to_string_lossy().to_string(),
        env_name,
        env_max_steps: 100,
        action_labels,
    };
    let bytes = bincode::serialize(&ready)?;
    ws_writer.send(Message::Binary(bytes.into())).await?;

    // 2. 如果存在最新的遥测帧，立即向新接入客户端推送，避免等待下一步才更新
    let initial_frame = {
        if let Ok(lock) = latest_frame.lock() {
            lock.clone()
        } else {
            None
        }
    };
    if let Some(frame) = initial_frame {
        let frame_msg = VisualOutFrame::Frame(frame);
        if let Ok(data) = bincode::serialize(&frame_msg) {
            let _ = ws_writer.send(Message::Binary(data.into())).await;
        }
    }

    // 3. 遥测广播写入任务
    let write_handle = tokio::spawn(async move {
        while let Ok(frame) = frame_rx.recv().await {
            let frame_msg = VisualOutFrame::Frame(frame);
            if let Ok(data) = bincode::serialize(&frame_msg) {
                if ws_writer.send(Message::Binary(data.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // 4. 从客户端读取控制命令
    while let Some(Ok(msg)) = ws_reader.next().await {
        if let Message::Binary(data) = msg {
            if let Ok(in_frame) = bincode::deserialize::<VisualInFrame>(&data) {
                let vcmd = match in_frame {
                    VisualInFrame::Reset => VisualRunnerCmd::Reset,
                    VisualInFrame::Pause => VisualRunnerCmd::Pause,
                    VisualInFrame::Resume => VisualRunnerCmd::Resume,
                    VisualInFrame::StepOnce => VisualRunnerCmd::StepOnce,
                    VisualInFrame::StepWithAction { action_id } => {
                        VisualRunnerCmd::StepWithAction(action_id)
                    }
                };
                let _ = cmd_tx.send(vcmd);
            }
        }
    }

    write_handle.abort();
    Ok(())
}

/// Convert a generic `VisualStepOutput` (from lol_env) to a `VisualObsFrame`.
fn step_output_to_frame_data(output: &VisualStepOutput) -> VisualObsFrame {
    let breakdown: Vec<RewardItem> = output
        .reward_breakdown
        .iter()
        .map(|b| RewardItem {
            name: b.name.clone(),
            value: b.value,
        })
        .collect();

    let policy = output.policy.clone();

    let obs_payload = output.obs_payload.clone().unwrap_or(ObsFeaturePayload {
        fiora_hp_pct: 1.0,
        riven_hp_pct: 1.0,
        distance: 0.0,
        q_ready: true,
        w_ready: true,
        e_ready: true,
        r_ready: true,
        has_vital: false,
        vital_is_active: false,
        vital_direction: "None".into(),
    });

    VisualObsFrame {
        step: output.step,
        obs: obs_payload.clone(),
        reward: output.reward,
        reward_breakdown: breakdown,
        policy,
        terminated: output.terminated,
        truncated: output.truncated,
        fiora_alive: obs_payload.fiora_hp_pct > 0.0,
        riven_alive: obs_payload.riven_hp_pct > 0.0,
        reward_formula: output.reward_formula.clone(),
        reward_variables: Some(output.reward_variables.clone()),
    }
}

fn parse_arg(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}
