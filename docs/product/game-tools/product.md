# 游戏工具 — 产品设计

## 一、核心术语与定位

游戏工具是外部世界与运行中的 Bevy 游戏进程交互的统一入口，按使用对象分为两种并存形态：

1. **命令行 (CLI)**
   - 面向开发者 / 运维，对游戏进程提供**完全控制**：观测、动作、暂停、状态、切换英雄、上帝模式、冷却开关、重置位置、脚本热重载、RL 步进等完整命令面。
   - 现状实现：`lol_cli`（`crates/lol_cli/src/main.rs`），一个 Bevy-free 的 WebSocket 客户端。

2. **MCP 工具层**
   - 面向 rig agent（Tauri 后端内现有的、web server 中规划中的），仅作为 agent 与游戏交互的工具，**只暴露观察 (observe) 与动作 (action)** 两个能力。
   - 不以 http / stdio 形式独立部署为 MCP server；rig agent 直接使用 rmcp 提供的进程内内存 MCP client 的 tools。

**关键规则**：
- CLI 与 MCP 不各自实现协议，二者共享同一个 Bevy-free 的游戏客户端 `lol_client`，协议类型与 WS 会话逻辑只此一份。
- 命令面有意分层：CLI = 完全控制，MCP = agent 交互所需的最小子集（observe + action）。调试 / 作弊类指令（上帝模式、冷却开关等）不进入 MCP，避免 agent 越权。

---

## 二、CLI 与 MCP 的职责对比

| 维度 | 命令行 (CLI) | MCP 工具层 |
|---|---|---|
| 使用对象 | 开发者 / 运维 / 脚本 | rig agent（Tauri / web server） |
| 命令面 | 完整（observe / action / pause / state / switch_champion / god_mode / toggle_cooldown / reset_position / get_agents / set_script / rl_*） | 仅 observe + action |
| 形态 | 独立二进制，clap 子命令 | rmcp `#[tool]`，进程内内存消费 |
| 传输 | 直连游戏 WebSocket | rmcp 进程内 client/server（duplex），无端口无 stdio |
| 复用 | 薄前端，委托 `lol_client` | `GameToolServer` 委托 `lol_client` |

---

## 三、共享客户端 (lol_client)

为消除现状中协议类型三处重复定义（`lol_server::protocol` / `lol_cli` / Tauri `ws.rs`）与 WS 会话逻辑重复，抽取 Bevy-free 的 `lol_client` crate 作为唯一游戏客户端：

- **协议类型**：以 `crates/lol_server/src/protocol.rs` 为权威，统一 `WsRequest` / `WsResponse` / `WsEvent`。
- **WS 会话**：把 `apps/client/src-tauri/src/ws.rs` 的 `WsSession::send_cmd`（id 关联 + 5s 超时 + oneshot）去 Tauri 化后迁入。
- **类型化命令面**：`GameClient` 提供方法一一映射服务端 cmd 字符串，参数用纯 Rust 类型（`f32` / `u64` / `usize`）拼出与 `lol_cli` 现有一致的 JSON（如 `{"Move":[x,y]}`、`{"Skill":{"index":..,"point":[x,y]}}`），不引入 Bevy 的 `Action` 枚举。

> CLI、MCP、Tauri 后端三处此后都依赖 `lol_client`，不再各自持有一份协议或会话代码。

---

## 四、MCP 工具层的产品约束

- **只读 + 动作**：MCP 仅暴露 `observe(entity_id)` 与 `action(entity_id, action)`，对应服务端 `get_observe` / `action`。
- **进程内消费**：rig agent 在进程内创建 rmcp client/server 对，通过 `agent.rmcp_tools(tools, peer)` 注入；不存在独立 MCP server 进程，无网络端口、无 stdio。
- **替换现有子进程桥**：Tauri 后端现有 rig agent 通过 `BashTool` 子进程调用 `lol_cli` 间接控制游戏，改为内存 rmcp tools 后获得更低延迟、类型化 schema、无子进程开销。
- **web server 接入**：web server 中的 rig agent 为规划项，后续接入同一套 rmcp tools。
