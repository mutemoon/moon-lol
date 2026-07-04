# 游戏工具 — 架构设计

本文档精准描述游戏工具 (CLI + MCP 并存) 的实现原理、共享客户端分层、MCP 工具暴露与 rig agent 消费方式及文件引用，不涉及具体代码细节。

## 一、总体架构与数据流

```
                ┌──────────────────────────────┐
                │   游戏 WS Server (Bevy 内)    │
                │  lol_server / lol_agent /     │
                │  lol_debug  (cmd 字符串分发)   │
                └───────────────┬──────────────┘
                        WebSocket (ws://127.0.0.1:9001)
                                │
                ┌───────────────▼──────────────┐
                │      lol_client (共享)        │
                │  protocol types + WsSession   │
                │  + GameClient (类型化命令面)   │
                └───────┬───────────────┬───────┘
                        │               │
            ┌───────────▼──┐    ┌───────▼────────────────┐
            │   lol_cli    │    │  GameToolServer (rmcp)  │
            │ 完全控制      │    │  仅 observe + action    │
            │ (clap 前端)   │    │  #[tool] → GameClient   │
            └──────────────┘    └───────────┬────────────┘
                                            │ 进程内内存 (duplex)
                                ┌───────────▼───────────┐
                                │  rig Agent (rmcp_tools) │
                                │  Tauri 后端 / web server │
                                └───────────────────────┘
```

游戏 WS Server 读取 `WsRequest { id, cmd, params }`，经 Bevy observer 事件分发到各插件按 `cmd` 字符串 match 处理，返回 `WsResponse`。CLI 与 MCP 工具层都是该 server 的客户端，差异仅在命令面与消费形态。

---

## 二、共享客户端层 (lol_client)

### 1. 协议类型统一

- 以 `crates/lol_server/src/protocol.rs` 的 `WsRequest` / `WsResponse` / `WsEvent` 为权威定义，迁入 `lol_client` 并由 `lol_server` 反向依赖或共同引用，消除 `lol_cli/src/main.rs` 与 `apps/client/src-tauri/src/ws.rs` 两份副本。
- 参见：[protocol.rs](/crates/lol_server/src/protocol.rs)

### 2. WS 会话迁移

- 把 `apps/client/src-tauri/src/ws.rs` 的 `WsSession::send_cmd` 去 Tauri 化（移除 `app.emit("ws-event", ...)` 耦合，事件回调改为可选 channel 或 trait 注入）后迁入 `lol_client`，保留 id 关联、oneshot 应答与 5s 超时。
- 参见：[ws.rs](/apps/client/src-tauri/src/ws.rs)

### 3. 类型化命令面 GameClient

- `GameClient` 内持有 `WsSession`，方法一一映射服务端 cmd 字符串，参数用纯 Rust 类型拼 JSON：
  - `observe(entity_id)` → `get_observe`
  - `action(entity_id, action)` → `action`（action 序列化为 `{"Move":[x,y]}` / `{"Attack":id}` / `"Stop"` / `{"Skill":{"index":..,"point":[x,y]}}` / `{"SkillLevelUp":idx}`）
  - `pause()` / `unpause()` → `toggle_pause`（保留幂等预检测）
  - `state()` → `get_state`
  - `switch_champion` / `god_mode` / `toggle_cooldown` / `reset_position` / `get_agents` / `set_script` / `rl_reset` / `rl_step` …
- 命令面的服务端处理位于：
  - 参见：[systems.rs](/crates/lol_agent/src/systems.rs)（`get_agents` / `get_observe` / `action` / `set_script` / `rl_reset` / `rl_step`）
  - 参见：[lib.rs](/crates/lol_debug/src/lib.rs)（`switch_champion` / `god_mode` / `toggle_cooldown` / `reset_position` / `toggle_pause` / `get_state`）
  - 参见：[action.rs](/crates/lol_core/src/action.rs)（`Action` 枚举，JSON 形状的权威来源）

---

## 三、CLI 层 (lol_cli)

- `lol_cli` 重构为薄 clap 前端，依赖 `lol_client`，子命令覆盖完整命令面（现有 Observe / Action / Pause / Unpause / State + 扩展 switch_champion / god_mode / …）。
- 保留 Pause / Unpause 的幂等预检测（先 `get_state` 再决定是否 `toggle_pause`）等现有逻辑，迁移为调用 `GameClient`。
- 参见：[main.rs](/crates/lol_cli/src/main.rs)

---

## 四、MCP 工具层 (GameToolServer)

### 1. 工具定义

- 用 rmcp `#[server]` + `#[tool]` 定义 `GameToolServer`，内部持有 `GameClient`，只暴露两个 tool：
  - `observe(entity_id: u64)` → 委托 `GameClient::observe`
  - `action(entity_id: u64, action: ActionArgs)` → 委托 `GameClient::action`
- 命令面有意收窄到 observe + action，调试 / 作弊类指令不暴露。

### 2. 进程内内存消费

- rig agent 在进程内用 `tokio::io::duplex` 建立内存传输，`GameToolServer` 与 rmcp client 各 `serve` 一端并 spawn 为 tokio task；再 `client.list_tools()` 取得 tools，`agent.rmcp_tools(tools, client.peer())` 注入 rig agent。
- 不开端口、不走 stdio、无独立 MCP server 进程。
- 参见：[agent.rs](/apps/client/src-tauri/src/agent.rs)（现有 rig agent 编排循环）

### 3. 替换现有子进程桥

- Tauri 后端现有 `BashTool`（`apps/client/src-tauri/src/tools.rs`）通过子进程调用 `lol_cli` 控制游戏，改为进程内 rmcp tools：更低延迟、类型化 schema、无子进程开销。
- 参见：[tools.rs](/apps/client/src-tauri/src/tools.rs)

---

## 五、依赖与复用映射

- 新增依赖：`rmcp`（features: `client,macros,server`）、`rig-core -F rmcp`（Tauri 已有 rig 0.37，按需补 rmcp feature）。
- web server 中的 rig agent 为规划项（`crates/lol_web_server` 目前无 rig 依赖），后续接入同一套 rmcp tools。

| 层 | 复用来源 | 去向 |
|---|---|---|
| 协议类型 | `lol_server/src/protocol.rs` | `lol_client`（权威），CLI / Tauri / MCP 共用 |
| WS 客户端 | `tauri/src/ws.rs::WsSession` | `lol_client`（去 Tauri 化） |
| 命令 → JSON | `lol_cli/src/main.rs` 手拼逻辑 | `lol_client::GameClient` 类型化方法 |
| rig Tool 模式 | `tauri/src/tools.rs::BashTool` | rmcp `#[tool]`（observe / action） |
