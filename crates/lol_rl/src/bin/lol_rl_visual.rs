use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};

use candle_core::Tensor;
use futures_util::{SinkExt, StreamExt};
use lol_env::fiora_vs_riven::{FioraVsRivenAction, FioraVsRivenObs};
use lol_env::visual_runner::{PolicyOutputItem, VisualRunnerCmd, VisualStepOutput, run_visual_env};
use lol_env::{EnvConfig, FioraVsRivenEnv, RenderMode};
use lol_rl::device::select_device;
use lol_rl::ppo::{PPOAgent, PPOConfig};
use lol_rl_protocol::{
    ObsFeaturePayload, PolicyItem, RewardItem, VisualInFrame, VisualObsFrame, VisualOutFrame,
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

    println!(">>> 正在启动 Fiora vs Riven 可视化环境 (Port: {port})...");

    let device = select_device()?;
    let config = PPOConfig::default();
    let state_dim = FioraVsRivenObs::dim();
    let agent = if let Some(ref path) = ckpt_path {
        if path.exists() {
            println!(">>> 加载 Checkpoint: {}", path.display());
            Arc::new(PPOAgent::load(
                state_dim,
                hidden_dim,
                5,
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
                5,
                config,
                device.clone(),
            )?)
        }
    } else {
        println!(">>> 未指定 --checkpoint，使用初始随机策略启动");
        Arc::new(PPOAgent::new(
            state_dim,
            hidden_dim,
            5,
            config,
            device.clone(),
        )?)
    };

    // Construct unified Env in Window render mode
    let env = FioraVsRivenEnv::with_config(EnvConfig {
        max_steps: 0,
        render_mode: RenderMode::WindowCustomLoop,
    });

    let (cmd_tx, cmd_rx) = mpsc::channel::<VisualRunnerCmd>();
    let (step_tx, step_rx) = mpsc::channel::<VisualStepOutput>();

    let ckpt_path_clone = ckpt_path.unwrap_or_else(|| PathBuf::from("default_random"));
    let cmd_tx_clone = cmd_tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(ws_server(port, step_rx, ckpt_path_clone, cmd_tx_clone));
    });

    let agent_clone = agent.clone();
    let policy = move |obs: &FioraVsRivenObs| -> (FioraVsRivenAction, Vec<PolicyOutputItem>) {
        let obs_vec = obs.to_vector();
        let state = match Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device) {
            Ok(t) => t,
            Err(_) => {
                return (FioraVsRivenAction::AttackRiven, vec![]);
            }
        };

        let probs = agent_clone
            .actor_critic
            .policy_probs(&state, &obs_vec)
            .unwrap_or_default();

        let policy_items: Vec<PolicyOutputItem> = probs
            .into_iter()
            .enumerate()
            .map(|(idx, prob)| {
                let action = FioraVsRivenAction::from_index(idx);
                PolicyOutputItem {
                    action_id: idx,
                    action_label: action.label().to_string(),
                    prob,
                }
            })
            .collect();

        let chosen = match agent_clone
            .actor_critic
            .select_greedy_action_masked(&state, &obs_vec)
        {
            Ok(idx) => FioraVsRivenAction::from_index(idx),
            Err(_) => FioraVsRivenAction::AttackRiven,
        };

        (chosen, policy_items)
    };

    run_visual_env(env, policy, cmd_rx, step_tx);

    Ok(())
}

async fn ws_server(
    port: u16,
    step_rx: mpsc::Receiver<VisualStepOutput>,
    ckpt_path: PathBuf,
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

    // Accept first connection; receiver is single-consumer
    if let Ok((stream, _)) = listener.accept().await {
        let ckpt = ckpt_path.clone();
        if let Err(e) = handle_ws_client(stream, step_rx, ckpt, cmd_tx).await {
            tracing::warn!("WS 客户端断开: {}", e);
        }
    }
}

async fn handle_ws_client(
    stream: TcpStream,
    step_rx: mpsc::Receiver<VisualStepOutput>,
    ckpt_path: PathBuf,
    cmd_tx: mpsc::Sender<VisualRunnerCmd>,
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

    // Bridge std receiver to tokio channel
    let (telemetry_tx, mut telemetry_rx) = tokio::sync::mpsc::channel::<VisualStepOutput>(256);
    std::thread::spawn(move || {
        for output in step_rx {
            if telemetry_tx.blocking_send(output).is_err() {
                break;
            }
        }
    });

    let write_handle = tokio::spawn(async move {
        while let Some(output) = telemetry_rx.recv().await {
            let frame = step_output_to_frame(&output);
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

/// Convert a `VisualStepOutput` (from lol_env) to a `VisualOutFrame` (WS protocol).
/// This conversion lives here (in the binary) because the WS protocol is NOT the Env's concern.
fn step_output_to_frame(output: &VisualStepOutput) -> VisualOutFrame {
    let r = &output.step_result;
    let obs = &r.obs;

    let breakdown: Vec<RewardItem> = r
        .reward_breakdown
        .iter()
        .map(|b| RewardItem {
            name: b.name.clone(),
            value: b.value,
        })
        .collect();

    let vital_direction = if obs.vital_dir_x > 0.5 {
        "+X (东侧)"
    } else if obs.vital_dir_neg_x > 0.5 {
        "-X (西侧)"
    } else if obs.vital_dir_z > 0.5 {
        "+Z (北侧)"
    } else if obs.vital_dir_neg_z > 0.5 {
        "-Z (南侧)"
    } else {
        "None"
    };

    let policy: Vec<PolicyItem> = output
        .policy
        .iter()
        .map(|p| PolicyItem {
            action_id: p.action_id,
            action: p.action_label.clone(),
            prob: p.prob,
        })
        .collect();

    VisualOutFrame::Frame(VisualObsFrame {
        step: r.step,
        obs: ObsFeaturePayload {
            fiora_hp_pct: if obs.fiora_max_hp > 0.0 {
                obs.fiora_hp / obs.fiora_max_hp
            } else {
                1.0
            },
            riven_hp_pct: if obs.riven_max_hp > 0.0 {
                obs.riven_hp / obs.riven_max_hp
            } else {
                1.0
            },
            distance: obs.distance,
            q_ready: obs.q_ready,
            w_ready: obs.w_ready,
            e_ready: obs.e_ready,
            r_ready: obs.r_ready,
            has_vital: obs.has_vital,
            vital_is_active: obs.vital_is_active,
            vital_direction: vital_direction.into(),
        },
        reward: r.reward,
        reward_breakdown: breakdown,
        policy,
        terminated: r.terminated,
        truncated: r.truncated,
        fiora_alive: obs.fiora_hp > 0.0,
        riven_alive: obs.riven_hp > 0.0,
    })
}

fn parse_arg(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}
