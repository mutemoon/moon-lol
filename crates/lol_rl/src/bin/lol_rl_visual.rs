use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};

use candle_core::Tensor;
use futures_util::{SinkExt, StreamExt};
use lol_env::visual_runner::{VisualRunnerCmd, VisualStepOutput, run_visual_env};
use lol_env::{EnvConfig, RenderMode, VisualEnvironment};
use lol_rl::device::select_device;
use lol_rl::algo::ppo::{PPOAgent, PPOConfig};
use lol_rl_protocol::{
    ActionSpace, ObsFeaturePayload, PolicyDisplay, RewardItem, VisualInFrame, VisualObsFrame,
    VisualOutFrame,
};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(
                    "warn,lol_rl=info,lol_env=info,lol_rl_protocol=info,lol_rl_visual=info",
                )
            }),
        )
        .init();

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
        .unwrap_or_else(|| lol_rl_protocol::ENV_FIORA_V2.to_string());

    macro_rules! dispatch_visual_env {
        ($(($env_ty:ty, $name:expr)),*) => {
            match env_name.as_str() {
                $(
                    s if s == $name => start_visual_runner_for_env::<$env_ty>(port, ckpt_path, hidden_dim),
                )*
                unknown => {
                    tracing::warn!("未知环境名称 {unknown}，使用默认环境");
                    start_visual_runner_for_env::<lol_env::FioraV2Env>(port, ckpt_path, hidden_dim)
                }
            }
        };
    }
    lol_env::for_all_rl_environments!(dispatch_visual_env)
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
    let device_desc = match &device {
        candle_core::Device::Cpu => "CPU (单步微秒级低延迟模式)".to_string(),
        candle_core::Device::Cuda(_) => "CUDA (GPU 加速模式)".to_string(),
        candle_core::Device::Metal(_) => "Metal".to_string(),
    };
    println!(">>> 推理设备: {device_desc}");
    let config = PPOConfig::default();
    let state_dim = E::state_dim();
    let action_space = E::action_space();
    let action_labels: Vec<String> = E::action_labels().iter().map(|s| s.to_string()).collect();

    let agent = if let Some(ref path) = ckpt_path {
        if path.exists() {
            println!(">>> 加载 Checkpoint: {}", path.display());
            Arc::new(PPOAgent::load_for_env::<E>(
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
            Arc::new(PPOAgent::create_for_env::<E>(
                state_dim,
                hidden_dim,
                action_space,
                config,
                device.clone(),
            )?)
        }
    } else {
        println!(">>> 未指定 --checkpoint，使用初始随机策略启动");
        Arc::new(PPOAgent::create_for_env::<E>(
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
    let env_max_steps = env.max_steps();
    let obs_schema = E::obs_schema();
    let action_schema = E::action_schema();

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
            env_max_steps,
            action_labels_clone,
            obs_schema,
            action_schema,
            cmd_tx_clone,
        ));
    });

    let agent_clone = agent.clone();
    let enc_dim = action_space.encoding_dim();
    // 按「离散动作类别」构造真实标注（policy_display_real 按下标取标签；
    // 注意与 `action_labels()` 的 UI 预设按钮标签区分——两者索引空间不同）。
    let class_labels: Vec<&str> = match action_space {
        ActionSpace::Discrete(n) => (0..n)
            .map(|i| E::action_name(E::action_from_encoding(&[i as f32])))
            .collect(),
        ActionSpace::Hybrid {
            discrete_classes, ..
        } => (0..discrete_classes)
            .map(|i| E::action_name(E::action_from_encoding(&[0.0, 0.0, i as f32])))
            .collect(),
        ActionSpace::Continuous(_) => Vec::new(),
    };
    let policy = move |obs: &E::Obs| -> (E::Action, PolicyDisplay) {
        let obs_vec = E::obs_to_vector(obs);
        let mask = E::action_mask(obs);
        let state = match Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device) {
            Ok(t) => t,
            Err(_) => {
                return (
                    E::action_from_encoding(&vec![0.0; enc_dim]),
                    PolicyDisplay::Discrete(vec![]),
                );
            }
        };

        let display = agent_clone
            .actor_critic
            .policy
            .policy_display_real(&state, mask.as_deref(), &class_labels)
            .unwrap_or(PolicyDisplay::Discrete(vec![]));

        let chosen = match agent_clone
            .actor_critic
            .policy
            .select_greedy_action(&state, mask.as_deref())
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
    env_max_steps: usize,
    action_labels: Vec<String>,
    obs_schema: Option<lol_rl_protocol::ObsSchema>,
    action_schema: Option<lol_rl_protocol::ActionSchema>,
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
        let schema_sub = obs_schema.clone();
        let action_schema_sub = action_schema.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_ws_client(
                stream,
                frame_rx,
                latest_frame_sub,
                ckpt,
                env_name_sub,
                env_max_steps,
                action_labels_sub,
                schema_sub,
                action_schema_sub,
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
    env_max_steps: usize,
    action_labels: Vec<String>,
    obs_schema: Option<lol_rl_protocol::ObsSchema>,
    action_schema: Option<lol_rl_protocol::ActionSchema>,
    cmd_tx: mpsc::Sender<VisualRunnerCmd>,
) -> anyhow::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    // 1. 发送包含环境、动作及 AST 结构元数据的 Ready 帧
    let ready = VisualOutFrame::Ready {
        checkpoint_path: ckpt_path.to_string_lossy().to_string(),
        env_name,
        env_max_steps,
        action_labels,
        obs_schema,
        action_schema,
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
        match bincode::serialize(&frame_msg) {
            Ok(data) => {
                let _ = ws_writer.send(Message::Binary(data.into())).await;
            }
            Err(e) => {
                tracing::error!("序列化 initial_frame 失败: {e}");
            }
        }
    }

    // 3. 遥测广播写入任务
    let write_handle = tokio::spawn(async move {
        while let Ok(frame) = frame_rx.recv().await {
            let frame_msg = VisualOutFrame::Frame(frame);
            match bincode::serialize(&frame_msg) {
                Ok(data) => {
                    if ws_writer.send(Message::Binary(data.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("序列化广播帧失败: {e}");
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
                    VisualInFrame::SetAutoPause(auto) => VisualRunnerCmd::SetAutoPause(auto),
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
        ..Default::default()
    });

    VisualObsFrame {
        step: output.step,
        obs: obs_payload.clone(),
        reward: output.reward,
        episode_reward: output.episode_reward,
        reward_breakdown: breakdown,
        policy,
        terminated: output.terminated,
        truncated: output.truncated,
        self_alive: obs_payload.fiora_hp_pct > 0.0,
        target_alive: obs_payload.riven_hp_pct > 0.0,
        fiora_alive: obs_payload.fiora_hp_pct > 0.0,
        riven_alive: obs_payload.riven_hp_pct > 0.0,
        reward_formula: output.reward_formula.clone(),
        reward_variables: Some(output.reward_variables.clone()),
        obs_vector: output.obs_vector.clone(),
        obs_labels: output.obs_labels.clone(),
        obs_tree: output.obs_tree.clone(),
        is_paused: output.is_paused,
    }
}

fn parse_arg(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}
