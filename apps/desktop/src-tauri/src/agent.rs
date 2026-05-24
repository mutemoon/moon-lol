use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::completion::Chat;
use rig::providers::anthropic;
use rig::providers::anthropic::completion::CompletionModel;
use tauri::{Emitter, Manager};

use crate::tools::BashTool;

pub enum AgentState {
    // 游戏暂停中，AI 思考下一步行动
    Thinking,
    // 游戏继续执行，执行 N 帧后再次暂停
    Playing,
}

pub fn create_agent(api_key: String, base_url: String, model: String, preamble: String) -> Agent<CompletionModel> {
    let client = anthropic::Client::builder()
        .api_key(&api_key)
        .base_url(&base_url)
        .build()
        .expect("初始化 rig Anthropic 客户端失败");

    client
        .agent(&model)
        .max_tokens(200 * 1000)
        .default_max_turns(10)
        .preamble(&preamble)
        .tool(BashTool)
        .build()
}

pub async fn on_thinking(
    agent: &Agent<CompletionModel>,
    local_chat_history: &mut Vec<rig::completion::Message>,
    cycle: u64,
) -> Result<String, String> {
    let prompt = format!(
        "这是你的决策周期第 {} 次。请立刻调用 bash 工具运行 `cargo run --bin lol_cli -- obs` 以获取最新局势观测，分析后做出行动命令（例如：`cargo run --bin lol_cli -- act move --x 7500 --y 7500`）！",
        cycle
    );
    let response = agent
        .chat(&prompt, local_chat_history)
        .await
        .map_err(|e| e.to_string())?;
    Ok(response)
}

/// 后台协调器循环：在 40s 时暂停游戏，运行 Agent 决策，继续游戏 1s，周而复始，在 120s（2分钟）时终止并统计
pub async fn run_agent_orchestrator(app: tauri::AppHandle, ws: crate::ws::WsSession) {
    println!("[Agent Orchestrator] 启动 AI Agent 后台生命周期循环");

    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    let base_url = std::env::var("ANTHROPIC_BASE_URL").unwrap_or_default();
    let model =
        std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "deepseek-v4-flash".to_string());
    let preamble = std::env::var("ANTHROPIC_PREAMBLE").unwrap_or_default();

    if api_key.is_empty() {
        println!("[Agent Orchestrator] 未检测到 ANTHROPIC_API_KEY，无法开启 AI Agent 决策环");
        return;
    }

    let agent = create_agent(api_key, base_url, model, preamble);
    let mut chat_history = Vec::new();
    let mut cycle_count = 0;

    loop {
        // 检查 WebSocket 状态，如果连接已被主动关闭或游戏已终止，则立即退出控制环
        let is_running = {
            if let Some(state_mutex) = app.try_state::<std::sync::Mutex<crate::state::AppState>>() {
                if let Ok(s) = state_mutex.lock() {
                    s.ws.is_some()
                } else {
                    false
                }
            } else {
                false
            }
        };

        if !is_running {
            println!("[Agent Orchestrator] 游戏进程已销毁或连接会话已被清空，退出 AI 决策生命周期控制环。");
            break;
        }

        // Step 1: 查询实时观测以检查游戏时间
        let obs_res = ws
            .send_cmd("get_observe".to_string(), serde_json::Value::Null)
            .await;
        let Ok(resp) = obs_res else {
            // 连接断开或异常时睡眠 500ms 后重试
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            continue;
        };

        let Some(data) = resp.data else {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            continue;
        };

        let Some(time) = data.get("time").and_then(|t| t.as_f64()) else {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            continue;
        };

        println!("[Agent Orchestrator] 游戏内实时时间: {:.2}s", time);

        // Step 2: 在 120秒（2分钟）时结束游戏，并展示成绩
        if time >= 120.0 {
            println!("[Agent Orchestrator] 游戏时间已达 2 分钟，正在终结并进行成绩统计...");

            let minion_kills = data
                .get("myself")
                .and_then(|m| m.get("minion_kills"))
                .and_then(|k| k.as_u64())
                .unwrap_or(0);

            let gold = data
                .get("myself")
                .and_then(|m| m.get("gold"))
                .and_then(|g| g.as_f64())
                .unwrap_or(0.0);

            #[derive(serde::Serialize, Clone)]
            struct AgentFinishedPayload {
                minion_kills: u64,
                gold: f64,
            }

            // 广播统计数据至前端展示
            let _ = app.emit(
                "agent-finished",
                AgentFinishedPayload { minion_kills, gold },
            );

            // 终结并关闭游戏
            if let Some(state) = app.try_state::<std::sync::Mutex<crate::state::AppState>>() {
                let _ = crate::process::stop_game(&state);
            }
            break;
        }

        // Step 3: 开始运行 AI 决策（>= 40秒时开始循环）
        if time >= 40.0 {
            println!("[Agent Orchestrator] 游戏时间 >= 40s，正在暂停游戏准备 AI 观测与行动...");

            // 暂停游戏
            let state_resp = ws
                .send_cmd("get_state".to_string(), serde_json::Value::Null)
                .await;
            let mut is_paused = false;
            if let Ok(sr) = state_resp {
                if let Some(sd) = sr.data {
                    is_paused = sd.get("paused").and_then(|p| p.as_bool()).unwrap_or(false);
                }
            }
            if !is_paused {
                let _ = ws
                    .send_cmd("toggle_pause".to_string(), serde_json::Value::Null)
                    .await;
            }

            cycle_count += 1;
            println!(
                "[Agent Orchestrator] 触发第 {} 次 AI 思考决策决策环...",
                cycle_count
            );
            match on_thinking(&agent, &mut chat_history, cycle_count).await {
                Ok(reply) => {
                    println!("[Agent Orchestrator] Agent 决策回复内容:\n{}", reply);
                }
                Err(e) => {
                    println!("[Agent Orchestrator] Agent 决策执行失败: {}", e);
                }
            }

            // 继续/恢复游戏
            println!("[Agent Orchestrator] AI 决策执行完毕，继续运行游戏 1s 后再次进行决策...");
            let state_resp = ws
                .send_cmd("get_state".to_string(), serde_json::Value::Null)
                .await;
            let mut is_paused = false;
            if let Ok(sr) = state_resp {
                if let Some(sd) = sr.data {
                    is_paused = sd.get("paused").and_then(|p| p.as_bool()).unwrap_or(false);
                }
            }
            if is_paused {
                let _ = ws
                    .send_cmd("toggle_pause".to_string(), serde_json::Value::Null)
                    .await;
            }

            // 恢复运行 1秒钟
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        } else {
            // 未到 40s，保持运行，每 500ms 轮询一次
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
}
