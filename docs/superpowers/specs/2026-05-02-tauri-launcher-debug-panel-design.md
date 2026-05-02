# Tauri 2 启动器 + 调试面板

## 概述

新增 `apps/desktop` Tauri 2 桌面应用，作为 LoL 启动器 + 运行时调试面板。
通过 WebSocket 与 Bevy 游戏进程通信，实现运行时游戏控制（切换英雄、无敌、暂停等）。
现有 `apps/web` 保持纯 Web 端不变（wasm 模式）。

## 整体架构

```
┌─────────────────────────────────┐       ┌────────────────────┐
│  apps/desktop (Tauri 2 + Vue 3) │       │  apps/web (Vue 3)  │
│                                 │       │                    │
│  ┌──────────┐ ┌──────────────┐  │       │  AI Battle Panel   │
│  │ Launcher │ │ Debug Panel  │  │       │  (wasm, 不变)       │
│  │ 英雄选择  │ │ 运行时控制   │  │       └────────────────────┘
│  └──────────┘ └──────┬───────┘  │
│                      │          │
│              ┌───────┴───────┐  │
│              │  WS Client    │  │
│              └───────┬───────┘  │
│                      │          │
│  ┌───────────────────┴───────┐  │
│  │ Tauri Rust Backend        │  │
│  │ 进程管理 (start/stop game) │  │
│  └───────────────┬───────────┘  │
│                  │              │
└──────────────────┼──────────────┘
                   │ 启动/管理子进程
                   ▼
         ┌─────────────────┐
         │ Bevy 游戏进程     │
         │ ws://127.0.0.1   │
         │     :9001        │
         │                  │
         │ PluginDebugPanel │
         └─────────────────┘
```

### 桌面端启动流程

1. 用户在 Launcher 选英雄/模式 → 点击启动
2. Tauri invoke `start_game` → Rust 后端 `Command::new("moon_lol")` 启动 Bevy 子进程
3. Bevy 进程监听 `127.0.0.1:9001`，就绪后发 `game_loaded` 事件
4. 前端 WS 连上 → 自动跳转 Debug Panel

### apps/web（Web 端）

保持现有功能不变：`/play` AI 对战面板，wasm in-process 渲染。

---

## WS 通信协议

桌面端 `apps/desktop` 前端通过 WebSocket 直连 Bevy 进程。

### 请求（面板 → 游戏）

```typescript
type WsRequest = {
  id: number;
  type: "cmd";
  cmd: "switch_champion" | "god_mode" | "toggle_cooldown"
     | "reset_position" | "toggle_pause" | "get_state";
  params: Record<string, unknown>;
};
```

### 响应

```typescript
type WsResponseOk = { id: number; type: "result"; ok: true; data?: unknown };
type WsResponseErr = { id: number; type: "result"; ok: false; error: string };
type WsResponse = WsResponseOk | WsResponseErr;
```

### 命令列表

| 命令 | params | response |
|------|--------|----------|
| `switch_champion` | `{name: string}` | `{ok: true}` 或 `{ok: false, error: string}` |
| `god_mode` | `{enabled: bool}` | `{ok: true}` |
| `toggle_cooldown` | `{enabled: bool}` | `{ok: true}` |
| `reset_position` | `{}` | `{ok: true}` |
| `toggle_pause` | `{}` | `{ok: true, paused: bool}` |
| `get_state` | `{}` | `{ok: true, data: GameState}` |

### 事件（游戏 → 面板）

```typescript
type GameLoadedEvent     = { type: "event"; event: "game_loaded";      data: {} };
type GamePausedEvent     = { type: "event"; event: "game_paused";      data: { paused: boolean } };
type ChampionChangedEvent = { type: "event"; event: "champion_changed"; data: { name: string } };
type EntitySelectedEvent = { type: "event"; event: "entity_selected";  data: { entity_id: number; kind: string; name: string } };
type GameCloseEvent      = { type: "event"; event: "game_close";       data: { reason: "user_exit" | "crash" } };
type GameLogEvent        = { type: "event"; event: "log";              data: { level: "info" | "warn" | "error"; msg: string } };

type WsEvent = GameLoadedEvent | GamePausedEvent | ChampionChangedEvent
             | EntitySelectedEvent | GameCloseEvent | GameLogEvent;
```

### Rust 端（已实现）

`crates/lol_core/src/debug/protocol.rs` — `WsRequest`, `CmdKind`, `WsEvent`, `WsResponse` 类型。
`crates/lol_core/src/debug/server.rs` — tokio WS server。
`crates/lol_core/src/debug/handlers.rs` — 6 个命令 handler + `ChampionSwitchQueue`。

---

## 桌面端：`apps/desktop`

### 前端路由

```
/         → Launcher（英雄选择 + 启动）
/debug    → Debug Panel（游戏运行中）
```

### Launcher 页面（`/`）

- 英雄选择（Riven / Fiora 等）
- 模式选择（Sandbox）
- 启动按钮 → Tauri invoke `start_game`

### Debug Panel（`/debug`）

- 连接状态指示灯（WS 状态）
- 当前英雄显示 + 切换下拉
- 开关：无敌、无冷却、暂停
- 重置位置按钮
- 日志面板：监听 `log` 事件自动滚动
- Bevy 游戏窗口为独立原生窗口，不在 Tauri webview 内渲染

### Tauri Rust Backend

```
apps/desktop/src-tauri/src/
├── main.rs       # 入口
├── lib.rs        # Tauri commands: start_game, stop_game
├── process.rs    # Bevy 子进程管理（spawn/kill/find binary）
└── state.rs      # AppState { bevy: Option<BevyProcess> }
```

#### Commands

```rust
#[tauri::command]
fn start_game(app: AppHandle, state: State<'_, Mutex<AppState>>, config: GameConfig) -> Result<(), String>;

#[tauri::command]
fn stop_game(state: State<'_, Mutex<AppState>>) -> Result<(), String>;
```

#### Bevy 二进制定位

- 开发模式：`target/debug/examples/lol.exe`
- 打包模式：`resource_dir/bin/lol.exe`

#### 进程生命周期

```
start_game → spawn Bevy → 前端连 WS → game_loaded
stop_game  → kill Bevy
窗口关闭   → 自动 kill Bevy
进程崩溃   → game_close {reason: "crash"} 推给面板
```

---

## 构建与分发

### 开发模式

```bash
# 终端 1：启动 Bevy 进程
cargo run --example lol -- --ws-port 9001 --mode sandbox --champion Riven

# 终端 2：Tauri dev
cd apps/desktop && pnpm tauri dev
```

### 打包

```bash
cd apps/desktop
pnpm tauri build
# 输出 .msi / .dmg / .deb
```

---

## MVP 范围

| 模块 | 位置 | 内容 |
|------|------|------|
| **Bevy PluginDebugPanel** | `crates/lol_core/src/debug/` | tokio WS server + 6 个命令 handler（已完成） |
| **CLI 参数** | `examples/lol.rs` | `--ws-port`, `--mode`, `--champion`（已完成） |
| **Champion 切换** | `crates/lol_champions/src/lib.rs` | ChampionSwitchQueue 处理 + spawn（已完成） |
| **Tauri Backend** | `apps/desktop/src-tauri/src/` | 进程管理，2 个 invoke commands |
| **Launcher 页面** | `apps/desktop/src/pages/` | 英雄选择 + 启动按钮 |
| **Debug Panel** | `apps/desktop/src/pages/debug.vue` | 英雄切换/无敌/冷却/暂停开关 + 日志 |
| **WS Client** | `apps/desktop/src/composables/` | WebSocket 通信封装 |

### 不出现在 MVP

- apps/web 任何改动
- 多实例/多端口
- 高级调试（ECS 查看器、Observer 追踪）
- 打包构建（先跑通 dev 模式）
