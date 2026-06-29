# 游戏工具 — 待办与完成情况

本页面记录游戏工具 (CLI + MCP 并存) 的已完成和未完成任务，整合了客户端与服务端的技术指标。

## 一、已完成任务列表

- [x] **基础 CLI**：`lol_cli` 作为 Bevy-free WebSocket 客户端，支持 Observe / Action / Pause / Unpause / State 子命令。（`crates/lol_cli/src/main.rs`）
- [x] **游戏 WS Server 命令分发**：`lol_server` 经 Bevy observer 事件按 cmd 字符串分发，`lol_agent` / `lol_debug` 各自处理。（`crates/lol_agent/src/systems.rs`、`crates/lol_debug/src/lib.rs`）
- [x] **Tauri rig agent 子进程桥**：Tauri 后端 rig agent 通过 `BashTool` 子进程调用 `lol_cli` 控制游戏。（`apps/client/src-tauri/src/tools.rs`、`agent.rs`）

---

## 二、待完成任务列表

### 1. 共享客户端 lol_client 抽取

#### 服务端 (Server)
- [x] **抽取 lol_client crate**：新建 Bevy-free 的 `crates/lol_client`，作为 CLI / MCP / Tauri 共用游戏客户端。
- [x] **统一协议类型**：以 `lol_server/src/protocol.rs` 为权威迁入 `lol_client`，`lol_server` re-export；消除 `lol_cli`、Tauri `ws.rs` 与 web server `match_supervisor` 三份副本。
- [x] **WsSession 去 Tauri 化迁移**：把 Tauri `ws.rs` 的 `WsSession::send_cmd` 迁入 `lol_client`，事件回调改为可选 `event_tx` 注入。
- [x] **类型化 GameClient 命令面**：方法映射全部服务端 cmd 字符串，参数用纯 Rust 类型拼 JSON。

---

### 2. CLI 完全控制面

#### 客户端 (Client)
- [x] **lol_cli 重构为 lol_client 薄前端**：依赖 `lol_client`，保留 Pause / Unpause 幂等预检测。
- [x] **扩展完整命令面**：补 switch_champion / god_mode / toggle_cooldown / reset_position / get_agents / set_script / rl_reset / rl_step 子命令。

---

### 3. MCP 工具层与 rig agent 接入

#### 客户端 (Client)
- [x] **GameToolServer**：用 rmcp `#[tool]` 定义，仅暴露 observe / action，委托 `GameClient`。
- [x] **Tauri agent 替换子进程桥**：`agent.rs` 用进程内 rmcp tools 取代 `BashTool` → `lol_cli` 子进程调用。
- [x] **进程内内存消费**：`tokio::io::duplex` 建立 rmcp client/server 对，`agent.rmcp_tools(tools, peer)` 注入。

#### 服务端 (Server)
- [x] **依赖引入**：workspace 新增 `rmcp` (client/macros/server)；Tauri 与 web server 经 `lol_client` 间接受益。
- [x] **web server rig agent 接入**：`crates/lol_web_server` 新建 `agent_orchestrator` 决策环，复用 `serve_inprocess` 注入 rmcp tools。
  - `LocalStartInput` 增 `scenario_agents` 字段：非空且存在 LLM 凭据时由 `local_game_service.start` 与 match supervisor 并行 spawn 编排环。
  - LLM 凭据来自环境变量（ANTHROPIC_*），与 Tauri 编排环一致；无凭据 / 无 agent 时静默跳过。
