//! 操作动作：注入消息 / 模拟决策 / 重置。

use gpui::*;

use crate::components::agent_chat_history::AgentChatMessage;
use crate::components::sidebar::AppSidebar;

use super::types::{current_round, initial_messages, update_state, with_state, AGENT_ID};

/// 注入一条用户消息（来自 user 输入框）。
pub(super) fn inject_user(cx: &mut Context<AppSidebar>) {
    let text = with_state(|s| s.user_input.trim().to_string());
    if text.is_empty() {
        return;
    }
    let round = with_state(|s| current_round(&s.messages));
    update_state(|s| {
        s.messages.push(AgentChatMessage {
            agent_id: AGENT_ID.to_string(),
            role: "user".to_string(),
            kind: "message".to_string(),
            content: text,
            round: Some(round),
        });
        s.user_input.clear();
    });
    cx.notify();
}

/// 注入一条 AI 消息（来自 AI 输入框）。
pub(super) fn inject_ai(cx: &mut Context<AppSidebar>) {
    let text = with_state(|s| s.assistant_input.trim().to_string());
    if text.is_empty() {
        return;
    }
    let round = with_state(|s| current_round(&s.messages));
    update_state(|s| {
        s.messages.push(AgentChatMessage {
            agent_id: AGENT_ID.to_string(),
            role: "assistant".to_string(),
            kind: "message".to_string(),
            content: text,
            round: Some(round),
        });
        s.assistant_input.clear();
    });
    cx.notify();
}

/// 模拟一次 AI 决策：追加一条决策消息并开启新轮次。
pub(super) fn simulate_decision(cx: &mut Context<AppSidebar>) {
    let round = with_state(|s| current_round(&s.messages)) + 1;
    let content = format!(
        "## 模拟决策动作\n\n思考过程：\n1. 观测返回：自身 HP=180（低于 30% 安全线）。\n2. 敌方小兵 103 处于攻击距离，但安全优先，必须立即后撤。\n3. 撤退目标点选择己方防御塔防区 (1500, 2000)。\n\n决策动作：\n```bash\nlol_cli action -e {} move 1500 2000\n```",
        AGENT_ID
    );
    update_state(|s| {
        s.messages.push(AgentChatMessage {
            agent_id: AGENT_ID.to_string(),
            role: "assistant".to_string(),
            kind: "public_decision".to_string(),
            content,
            round: Some(round),
        });
    });
    cx.notify();
}

/// 模拟一次工具调用结果（追加一条 tool 消息）。
pub(super) fn add_tool_result(cx: &mut Context<AppSidebar>) {
    let round = with_state(|s| current_round(&s.messages));
    let content = r#"【执行工具 game_action 结果】
stdout:
```json
{
  "status": "success",
  "damage_dealt": 84.5,
  "gold_gained": 21,
  "current_position": [2712, 12308]
}
```
stderr: "" "#;
    update_state(|s| {
        s.messages.push(AgentChatMessage {
            agent_id: AGENT_ID.to_string(),
            role: "user".to_string(),
            kind: "tool_call".to_string(),
            content: content.to_string(),
            round: Some(round),
        });
    });
    cx.notify();
}

/// 重置回初始 mock 数据。
pub(super) fn reset_all() {
    update_state(|s| {
        s.messages = initial_messages();
        s.user_input.clear();
        s.assistant_input.clear();
    });
}
