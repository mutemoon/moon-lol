# 游戏进程托管 — 待办

## 已完成

- 抽出共享 crate `lol_game_process_manager`：端口池、进程表、`ProcessLauncher` trait、`GameProcessManager`、`AgentRunner` 注入点。
- 云端 `LocalGameService` 重构为委托 `GameProcessManager`：进程托管下沉共享 crate，本层只留对局体系（match 记录、状态机、supervisor、AI 环显式 spawn）。行为不变。
- 云端 `CommandProcessLauncher` 支持生产部署：`MOON_LOL_BINARY` env 指向预编译二进制，缺省回退 `cargo run --bin moon_lol --`。
- spawn 命令构建归一到 `lol_client::launch`（`BevySpawnRequest` / `build_command` / `bevy_args` / `default_rust_log` / `workspace_root` / `binary_name`），桌面与云端共用。
- AI 决策环共享运行时 `lol_agent_runtime`（见游戏工具文档），桌面 / 云端各自实现凭证解析与副作用出口。

## 待办：桌面端多局化

桌面端仍为单进程模型（`AppState.bevy` 单值、写死端口 9001、硬限单局）。需接入 `GameProcessManager` 实现多局托管。

### 阶段 3：桌面后端

- [ ] `DesktopProcessLauncher` impl `ProcessLauncher`：迁现有 `process.rs` 的 spawn 逻辑（dev `cargo run --` / release 打包二进制、非 headless、cwd 为 workspace 根、RUST_LOG），改用 `tokio::process::Command::from(build_command(..))`，按 port 维护子进程表供 kill。
- [ ] `AppState` 重构：`bevy: Option<BevyProcess>` / `ws: Option<WsSession>` 替换为 `Arc<GameProcessManager>`。进程表由 manager 持有。
- [ ] 桌面 `AgentRunner`：封装现有 `agent.rs` 的 `DesktopCredentialResolver`（读 providers.json）+ `DesktopSink`（emit 对话历史 / 终结事件、写盘历史、停进程），注入 manager。
- [ ] Tauri 命令重写：
  - `start_game(config) -> { id, port }`：调 `manager.start`，端口由池分配。AI 环在 start 时自动起（决策已定：start 时自动起 AI，见下）。
  - `stop_game(id)`：调 `manager.stop(id)`，原全局 stop 改按 id。
  - `connect_ws(port)` / `connect_ws_observe(port)`：从前端传入 port（由 start_game 返回）。
  - `send_ws_cmd(port, cmd, params)`：加 port 参数，按端口选 WS 会话，保留实时调试通道。
- [ ] `process.rs` / `ws.rs` 清理：`start_game` / `stop_game` 删除，spawn 逻辑迁入 launcher；ws 桥接按 port 建立。

### 阶段 4：前端多局化

- [ ] `startGame` 返回 `{ id, port }`，store 持有当前局标识。
- [ ] `connectWs(port)` / `sendWsCmd(port, ..)` / `stopGame(id)` 带参。
- [ ] `useWsClient` / `gameStore` / `debug.vue` / `observe/[id].vue` 适配多局。
- [ ] 保留 `sendWsCmd` / `ws-event` 实时通道，不迁移到轮询契约。

### 关键决策（已定）

- **AI 环启动时机**：start 时自动起 AI（manager 注入桌面 `AgentRunner`），而非 connect_ws 时起。语义从原「connect 即 AI」改为「start 即 AI」。`connect_ws_observe`（回放）路径不起 AI。
- **实时调试通道**：保留按端口直连 Bevy WebSocket 的 `sendWsCmd`，桌面 debug 页实时调试能力不变。不迁移到云端事件轮询契约。
- **headless**：`BevySpawnRequest.game_config.headless` 按场景区分——AI 对局可 headless，回放 / 观战需窗口（非 headless）。
- **owner_id**：桌面本地调试无对局归属，`GameProcessManager` 不含 owner_id 概念。

### 验证

- `cargo check --all-targets`。
- `cargo test -p lol_game_process_manager`。
- 桌面手动：起多局本地调试对局，确认端口池分配（不再 9001）、多局并发、按 id 停局、AI 决策环、实时 sendWsCmd 调试、observe 回放（非 headless）。
