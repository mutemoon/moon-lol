use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use lol_rl_protocol::{InFrame, OutFrame, VisualOutFrame, DEFAULT_RL_SERVER_ADDR};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::components::sidebar::{AppSidebar, VisualSession};
use crate::services::runtime::run_on_tokio;
use crate::services::visual_process::spawn_visual_env;
use crate::services::visual_ws::{spawn_visual_ws, VisualWsEvent};
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
                                let old_ids: Vec<String> =
                                    sidebar.task_list.iter().map(|t| t.id.clone()).collect();
                                let new_tasks = tasks.clone();
                                sidebar.task_list = tasks;
                                // 任务被删除后，若正在查看其详情则回到列表
                                if let Some(sel) = sidebar.selected_task_id.clone() {
                                    if !sidebar.task_list.iter().any(|t| t.id == sel) {
                                        sidebar.selected_task_id = None;
                                        sidebar.task_details.remove(&sel);
                                    }
                                }
                                // 若正在可视化被删除的任务，一并关闭子进程
                                if let Some(vid) = sidebar.visual_task_id.clone() {
                                    if old_ids.contains(&vid)
                                        && !sidebar.task_list.iter().any(|t| t.id == vid)
                                    {
                                        sidebar.visual_session = None;
                                        sidebar.visual_in_tx = None;
                                        sidebar.latest_visual_frame = None;
                                        sidebar.visual_error = None;
                                        sidebar.visual_ws_connected = false;
                                        sidebar.visual_task_id = None;
                                        sidebar.running_visual_model = None;
                                    }
                                }
                                // 同步 DataTable delegate 数据（不调 refresh，保留用户调宽的列宽）
                                if let Some(table) = &sidebar.table_state {
                                    let _ = table.update(cx, |state, cx| {
                                        state.delegate_mut().set_tasks(new_tasks);
                                        cx.notify();
                                    });
                                }
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
                                reward_formula,
                                reward_variables,
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
                                        reward_formula: None,
                                        latest_reward_variables: None,
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
                                if reward_formula.is_some() {
                                    detail.reward_formula = reward_formula;
                                }
                                if reward_variables.is_some() {
                                    detail.latest_reward_variables = reward_variables;
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
                                    if !detail
                                        .checkpoints
                                        .iter()
                                        .any(|c| c.id == checkpoint.id)
                                    {
                                        detail.checkpoints.push(checkpoint.clone());
                                    }
                                }
                                // 用户点过「运行可视 Env」，等到的就是这个 checkpoint → 拉起可视化子进程
                                if sidebar.running_visual_model.as_deref()
                                    == Some(checkpoint.id.as_str())
                                {
                                    let path = checkpoint.path.clone();
                                    let tid = task_id.clone();
                                    let weak = entity_weak_ui.clone();
                                    cx.spawn(move |_: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                                        let mut cx = cx.clone();
                                        async move {
                                            spawn_visual_session(&weak, &mut cx, tid, path).await;
                                        }
                                    })
                                    .detach();
                                }
                            }
                            OutFrame::TaskDetail {
                                task_id,
                                checkpoints,
                                metrics_history,
                                logs,
                            } => {
                                let detail = sidebar
                                    .task_details
                                    .entry(task_id.clone())
                                    .or_insert_with(|| LocalTaskDetail {
                                        id: task_id.clone(),
                                        name: task_id.clone(),
                                        agent_type: "PPO".to_string(),
                                        env_name: "FioraVsRiven".to_string(),
                                        status: "Running".to_string(),
                                        current_step: 0,
                                        ep_return: 0.0,
                                        checkpoints: Vec::new(),
                                        metrics_history: Vec::new(),
                                        latest_policy: Vec::new(),
                                        latest_reward_breakdown: Vec::new(),
                                        latest_obs: None,
                                        reward_formula: None,
                                        latest_reward_variables: None,
                                        logs: Vec::new(),
                                    });
                                detail.checkpoints = checkpoints;
                                if !metrics_history.is_empty() {
                                    detail.metrics_history = metrics_history;
                                    if let Some(last) = detail.metrics_history.last() {
                                        detail.current_step = last.step;
                                        detail.ep_return = last.ep_return;
                                    }
                                }
                                if !logs.is_empty() {
                                    detail.logs = logs;
                                }
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

/// 拉起 lol_rl_visual 子进程并连接视觉 WS，把事件转发到 sidebar 可视化状态。
async fn spawn_visual_session(
    weak: &gpui::WeakEntity<AppSidebar>,
    cx: &mut gpui::AsyncApp,
    task_id: String,
    checkpoint_path: String,
) {
    // 先关掉旧会话（kill_on_drop 已设，drop 即终止旧子进程）
    let _ = weak.update(cx, |sidebar, _| {
        sidebar.visual_session = None;
        sidebar.visual_in_tx = None;
        sidebar.latest_visual_frame = None;
        sidebar.visual_error = None;
        sidebar.visual_task_id = None;
    });

    let spawned = run_on_tokio(move || async move {
        spawn_visual_env(&checkpoint_path)
            .await
            .map_err(|e| e.to_string())
    })
    .await;

    let (child, port) = match spawned {
        Ok(ok) => ok,
        Err(e) => {
            let _ = weak.update(cx, |sidebar, _| {
                sidebar.visual_error = Some(format!("启动可视化子进程失败: {e}"));
            });
            return;
        }
    };

    let (vis_event_tx, mut vis_event_rx) = mpsc::unbounded_channel::<VisualWsEvent>();
    let cmd_tx = spawn_visual_ws(port, vis_event_tx);

    {
        let _ = weak.update(cx, |sidebar, _| {
            sidebar.visual_session = Some(VisualSession {
                child: Some(child),
                port,
                cmd_tx: Some(cmd_tx.clone()),
            });
            sidebar.visual_in_tx = Some(cmd_tx);
            sidebar.visual_ws_connected = false;
            sidebar.visual_error = None;
            sidebar.visual_task_id = Some(task_id);
        });
    }

    // 转发视觉 WS 事件到 UI；WS 线程退出（sender drop）后循环自终结
    while let Some(ev) = vis_event_rx.recv().await {
        let _ = weak.update(cx, |sidebar, cx| {
            match ev {
                VisualWsEvent::Connected => sidebar.visual_ws_connected = true,
                VisualWsEvent::Disconnected => sidebar.visual_ws_connected = false,
                VisualWsEvent::Frame(VisualOutFrame::Frame(f)) => {
                    if f.terminated {
                        sidebar.visual_paused = true;
                    }
                    sidebar.latest_visual_frame = Some(f);
                }
                VisualWsEvent::Frame(VisualOutFrame::Ready { .. }) => {}
                VisualWsEvent::Frame(VisualOutFrame::Log { .. }) => {}
                VisualWsEvent::Frame(VisualOutFrame::Exited { code }) => {
                    sidebar.visual_ws_connected = false;
                    sidebar.visual_session = None;
                    sidebar.visual_in_tx = None;
                    sidebar.visual_task_id = None;
                    sidebar.visual_error = Some(format!("可视化子进程已退出 (code {:?})", code));
                }
            }
            cx.notify();
        });
    }
}
