use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use lol_rl_protocol::{InFrame, OutFrame, DEFAULT_RL_SERVER_ADDR};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::components::sidebar::AppSidebar;
use crate::types::LocalTaskDetail;

enum WsEvent {
    Connected(bool),
    Frame(OutFrame),
}

pub fn spawn_ws_service(
    entity_weak: gpui::WeakEntity<AppSidebar>,
    cx: &mut gpui::Context<AppSidebar>,
    mut rx: mpsc::UnboundedReceiver<InFrame>,
) {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<WsEvent>();

    let entity_weak_ui = entity_weak.clone();
    cx.spawn(move |_, cx: &mut gpui::AsyncApp| {
        let mut cx = cx.clone();
        async move {
            while let Some(event) = event_rx.recv().await {
                let _ = entity_weak_ui.update(&mut cx, |sidebar, cx| {
                    match event {
                        WsEvent::Connected(connected) => {
                            sidebar.ws_connected = connected;
                        }
                        WsEvent::Frame(out_frame) => match out_frame {
                            OutFrame::TaskList { tasks } => {
                                sidebar.task_list = tasks;
                            }
                            OutFrame::Status { task_id, status } => {
                                if let Some(item) =
                                    sidebar.task_list.iter_mut().find(|t| t.id == task_id)
                                {
                                    item.status = status.clone();
                                }
                                if let Some(detail) = sidebar.task_details.get_mut(&task_id) {
                                    detail.status = status;
                                }
                            }
                            OutFrame::Metrics {
                                task_id,
                                step,
                                ep_return,
                                loss,
                                kl,
                                entropy,
                                value,
                                fps,
                                policy,
                                reward_breakdown,
                                obs_feature,
                            } => {
                                if let Some(item) =
                                    sidebar.task_list.iter_mut().find(|t| t.id == task_id)
                                {
                                    item.current_step = step;
                                    item.ep_return = ep_return;
                                }
                                let metric_row = lol_rl_protocol::MetricsRow {
                                    step,
                                    ep_return,
                                    loss,
                                    kl,
                                    entropy,
                                    value,
                                    fps,
                                };
                                let detail = sidebar
                                    .task_details
                                    .entry(task_id.clone())
                                    .or_insert_with(|| LocalTaskDetail {
                                        id: task_id.clone(),
                                        name: task_id.clone(),
                                        agent_type: "PPO".to_string(),
                                        env_name: "FioraVsRiven".to_string(),
                                        status: "Running".to_string(),
                                        current_step: step,
                                        ep_return,
                                        checkpoints: Vec::new(),
                                        metrics_history: Vec::new(),
                                        latest_policy: Vec::new(),
                                        latest_reward_breakdown: Vec::new(),
                                        latest_obs: None,
                                        logs: Vec::new(),
                                    });
                                detail.current_step = step;
                                detail.ep_return = ep_return;
                                detail.metrics_history.push(metric_row);
                                detail.latest_policy = policy;
                                detail.latest_reward_breakdown = reward_breakdown;
                                if obs_feature.is_some() {
                                    detail.latest_obs = obs_feature;
                                }
                            }
                            OutFrame::Log {
                                task_id,
                                level,
                                message,
                            } => {
                                let log_line = format!("[{}] {}", level, message);
                                if let Some(detail) = sidebar.task_details.get_mut(&task_id) {
                                    detail.logs.push(log_line);
                                }
                            }
                            OutFrame::CheckpointMsg {
                                task_id,
                                checkpoint,
                            } => {
                                if let Some(item) =
                                    sidebar.task_list.iter_mut().find(|t| t.id == task_id)
                                {
                                    item.checkpoints_count += 1;
                                }
                                if let Some(detail) = sidebar.task_details.get_mut(&task_id) {
                                    detail.checkpoints.push(checkpoint);
                                }
                            }
                            OutFrame::CheckpointLoaded {
                                task_id,
                                checkpoint,
                            } => {
                                if let Some(detail) = sidebar.task_details.get_mut(&task_id) {
                                    detail.checkpoints.push(checkpoint.clone());
                                }
                                // Cache checkpoint path and trigger visual spawn from UI
                            }
                        },
                    }
                    cx.notify();
                });
            }
        }
    })
    .detach();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(_) => return,
        };

        rt.block_on(async move {
            let ws_url = format!("ws://{}", DEFAULT_RL_SERVER_ADDR);

            let (frame_tx, mut frame_rx) = mpsc::unbounded_channel::<InFrame>();

            // 将收到的外部 InFrame 转发到内部 frame_tx
            let bridge_task = async move {
                while let Some(frame) = rx.recv().await {
                    if frame_tx.send(frame).is_err() {
                        break;
                    }
                }
            };

            let connection_loop = async move {
                loop {
                    if let Ok((ws_stream, _)) = connect_async(&ws_url).await {
                        // 通知 UI 已连接
                        let _ = event_tx.send(WsEvent::Connected(true));

                        let (mut write, mut read) = ws_stream.split();

                        // 连接建立后自动拉取任务列表
                        let req_bytes =
                            bincode::serialize(&InFrame::GetTaskList).unwrap_or_default();
                        let _ = write.send(Message::Binary(req_bytes.into())).await;

                        // 读任务
                        let event_tx_read = event_tx.clone();
                        let read_task = async move {
                            while let Some(Ok(msg)) = read.next().await {
                                if let Message::Binary(bytes) = msg {
                                    if let Ok(out_frame) = bincode::deserialize::<OutFrame>(&bytes)
                                    {
                                        let _ = event_tx_read.send(WsEvent::Frame(out_frame));
                                    }
                                }
                            }
                        };

                        // 写任务
                        let write_task = async {
                            while let Some(in_frame) = frame_rx.recv().await {
                                if let Ok(msg_bytes) = bincode::serialize(&in_frame) {
                                    if write.send(Message::Binary(msg_bytes.into())).await.is_err()
                                    {
                                        break;
                                    }
                                }
                            }
                        };

                        futures_util::future::select(Box::pin(read_task), Box::pin(write_task))
                            .await;

                        // 通知 UI 断开连接
                        let _ = event_tx.send(WsEvent::Connected(false));
                    } else {
                        // 通知 UI 断开连接
                        let _ = event_tx.send(WsEvent::Connected(false));
                    }

                    // 2 秒后自动重试连接
                    sleep(Duration::from_secs(2)).await;
                }
            };

            futures_util::future::join(bridge_task, connection_loop).await;
        });
    });
}
