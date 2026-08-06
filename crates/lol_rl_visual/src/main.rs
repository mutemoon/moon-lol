use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};

use candle_core::Tensor;
use futures_util::{SinkExt, StreamExt};
use lol_env::fiora_vs_riven::FioraVsRivenObs;
use lol_env::visual_loop::{VisualCmd, VisualTelemetry, run_visual_loop};
use lol_rl::device::select_device;
use lol_rl::ppo::{PPOAgent, PPOConfig};
use lol_rl_protocol::{
    ObsFeaturePayload, RewardItem, VisualInFrame, VisualObsFrame, VisualOutFrame,
};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let port = parse_arg("--port")
        .unwrap_or_else(|| "9320".to_string())
        .parse::<u16>()?;
    let ckpt_path = PathBuf::from(
        parse_arg("--checkpoint").ok_or_else(|| anyhow::anyhow!("缺少 --checkpoint <路径>"))?,
    );

    let device = select_device()?;
    let config = PPOConfig::default();
    let state_dim = FioraVsRivenObs::dim();
    let agent = Arc::new(PPOAgent::load(
        state_dim,
        64,
        9,
        config,
        device.clone(),
        &ckpt_path,
    )?);

    tracing::info!("加载 checkpoint: {}", ckpt_path.display());

    let (_cmd_tx, cmd_rx) = mpsc::channel::<VisualCmd>();
    let (frame_tx, frame_rx) = mpsc::channel::<VisualTelemetry>();

    let ckpt_path_clone = ckpt_path.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(ws_server(port, frame_rx, ckpt_path_clone));
    });

    let agent_clone = agent.clone();
    let policy = move |obs: &FioraVsRivenObs| -> lol_env::fiora_vs_riven::FioraVsRivenAction {
        let obs_vec = obs.to_vector();
        let state = match Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device) {
            Ok(t) => t,
            Err(_) => return lol_env::fiora_vs_riven::FioraVsRivenAction::AttackRiven,
        };
        match agent_clone
            .actor_critic
            .select_greedy_action_masked(&state, &obs_vec)
        {
            Ok(idx) => lol_env::fiora_vs_riven::FioraVsRivenAction::from_index(idx),
            Err(_) => lol_env::fiora_vs_riven::FioraVsRivenAction::AttackRiven,
        }
    };

    run_visual_loop(100, policy, cmd_rx, frame_tx);

    Ok(())
}

async fn ws_server(port: u16, frame_rx: mpsc::Receiver<VisualTelemetry>, ckpt_path: PathBuf) {
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

    // Accept first connection; receiver is single-consumer
    if let Ok((stream, _)) = listener.accept().await {
        let ckpt = ckpt_path.clone();
        if let Err(e) = handle_ws_client(stream, frame_rx, ckpt).await {
            tracing::warn!("WS 客户端断开: {}", e);
        }
    }
}

async fn handle_ws_client(
    stream: TcpStream,
    frame_rx: mpsc::Receiver<VisualTelemetry>,
    ckpt_path: PathBuf,
) -> anyhow::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut ws_writer, mut ws_reader) = ws_stream.split();

    // Send Ready frame
    let ready = VisualOutFrame::Ready {
        checkpoint_path: ckpt_path.to_string_lossy().to_string(),
        env_max_steps: 100,
    };
    let bytes = bincode::serialize(&ready)?;
    ws_writer.send(Message::Binary(bytes.into())).await?;

    // Bridge crossbeam receiver to tokio channel
    let (telemetry_tx, mut telemetry_rx) = tokio::sync::mpsc::channel::<VisualTelemetry>(256);
    std::thread::spawn(move || {
        for telemetry in frame_rx {
            if telemetry_tx.blocking_send(telemetry).is_err() {
                break;
            }
        }
    });

    let write_handle = tokio::spawn(async move {
        while let Some(telemetry) = telemetry_rx.recv().await {
            let frame = telemetry_to_frame(&telemetry);
            if let Ok(data) = bincode::serialize(&frame) {
                if ws_writer.send(Message::Binary(data.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Read commands from client
    while let Some(Ok(msg)) = ws_reader.next().await {
        if let Message::Binary(data) = msg {
            let _ = bincode::deserialize::<VisualInFrame>(&data);
        }
    }

    write_handle.abort();
    Ok(())
}

fn telemetry_to_frame(t: &VisualTelemetry) -> VisualOutFrame {
    let breakdown: Vec<RewardItem> = t
        .reward_breakdown
        .iter()
        .map(|b| RewardItem {
            name: b.name.clone(),
            value: b.value,
        })
        .collect();

    let vital_direction = if t.obs.vital_dir_x > 0.5 {
        "+X (东侧)"
    } else if t.obs.vital_dir_neg_x > 0.5 {
        "-X (西侧)"
    } else if t.obs.vital_dir_z > 0.5 {
        "+Z (北侧)"
    } else if t.obs.vital_dir_neg_z > 0.5 {
        "-Z (南侧)"
    } else {
        "None"
    };

    VisualOutFrame::Frame(VisualObsFrame {
        step: t.step,
        obs: ObsFeaturePayload {
            fiora_hp_pct: if t.fiora_max_hp > 0.0 {
                t.fiora_hp / t.fiora_max_hp
            } else {
                1.0
            },
            riven_hp_pct: if t.riven_max_hp > 0.0 {
                t.riven_hp / t.riven_max_hp
            } else {
                1.0
            },
            distance: t.obs.distance,
            q_ready: t.obs.q_ready,
            w_ready: t.obs.w_ready,
            e_ready: t.obs.e_ready,
            r_ready: t.obs.r_ready,
            has_vital: t.obs.has_vital,
            vital_is_active: t.obs.vital_is_active,
            vital_direction: vital_direction.into(),
        },
        reward: t.reward,
        reward_breakdown: breakdown,
        terminated: t.terminated,
        truncated: t.truncated,
        fiora_alive: t.fiora_hp > 0.0,
        riven_alive: t.riven_hp > 0.0,
    })
}

fn parse_arg(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}
