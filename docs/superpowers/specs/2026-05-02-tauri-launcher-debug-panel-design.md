# Tauri 2 启动器 + 调试面板

## 概述

使用 Tauri 2 作为 LoL 桌面启动器，统一 `apps/web` 项目同时支持桌面端（Tauri）和 Web 端（wasm）。
运行时通过 `window.__TAURI__` 判断平台，通过 `GameConnection` 抽象层屏蔽平台差异。
桌面端调试面板通过 WebSocket 与 Bevy 游戏进程通信，实现运行时游戏控制（切换英雄、无敌、暂停等）。

## 整体架构

```
┌─────────────────────────────────────────────────────────┐
│  apps/web (Vue 3 + Tauri 2)                            │
│                                                         │
│  ┌──────────┐  ┌───────────────┐  ┌──────────────────┐ │
│  │ Launcher │  │ Debug Panel   │  │ AI Battle Panel  │ │
│  │ 英雄选择  │  │ 运行时控制     │  │ (现有)            │ │
│  └──────────┘  └───────────────┘  └──────────────────┘ │
│                      │                                  │
│              ┌───────┴───────┐                          │
│              │ GameConnection │  ◄── 平台适配层          │
│              │  (抽象接口)     │                          │
│              └───────┬───────┘                          │
│         ┌────────────┴────────────┐                     │
│  ┌──────┴──────┐          ┌──────┴──────┐              │
│  │ Desktop     │          │ Web          │              │
│  │ WS Client   │          │ wasm direct  │              │
│  └──────┬──────┘          └──────┬──────┘              │
│         │                        │                      │
│  ┌──────┴──────┐          ┌──────┴──────┐              │
│  │Tauri backend│          │ 浏览器 wasm  │              │
│  │进程管理     │          │ in-process   │              │
│  └─────────────┘          └─────────────┘              │
│         │                                               │
└─────────┼───────────────────────────────────────────────┘
          │ 启动/管理子进程
          ▼
┌─────────────────┐
│ Bevy 游戏进程     │
│ ws://127.0.0.1   │
│     :9001        │
│                  │
│ PluginDebugPanel │
│ ← WS command     │
│   handlers       │
└─────────────────┘
```

### 运行模式判断

- 前端启动时检查 `window.__TAURI__` 是否存在 → 桌面模式 / Web 模式
- `GameConnection` 是 TypeScript 接口，桌面模式实例化 `WsGameConnection`，Web 模式实例化 `WasmGameConnection`
- 上层组件（Launcher、Debug Panel）只依赖 `GameConnection`，不关心平台

### 桌面端启动流程

1. 用户在 Launcher 选英雄/模式 → 点击启动
2. Tauri invoke `start_game` command → Rust 后端 `Command::new("moon_lol")` 启动 Bevy 子进程
3. Bevy 进程监听 `127.0.0.1:9001`，启动后前端 WS 连接
4. 前端收到 `game_loaded` 事件 → 游戏开始

---

## WS 通信协议

双端（TypeScript / Rust）各有一套完整的类型定义，用 `serde` tag 枚举对齐 JSON。

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
type WsResponseOk = {
  id: number;
  type: "result";
  ok: true;
  data?: unknown;
};

type WsResponseErr = {
  id: number;
  type: "result";
  ok: false;
  error: string;
};

type WsResponse = WsResponseOk | WsResponseErr;
```

### 命令列表

| 命令 | params schema | response |
|------|--------------|----------|
| `switch_champion` | `{name: string}` | `{ok: true}` 或 `{ok: false, error: string}` |
| `god_mode` | `{enabled: bool}` | `{ok: true}` |
| `toggle_cooldown` | `{enabled: bool}` | `{ok: true}` |
| `reset_position` | `{}` | `{ok: true}` |
| `toggle_pause` | `{}` | `{ok: true, paused: bool}` |
| `get_state` | `{}` | `{ok: true, data: GameState}` |

### 事件（游戏 → 面板）

```typescript
type GameLoadedEvent    = { type: "event"; event: "game_loaded";       data: {} };
type GamePausedEvent    = { type: "event"; event: "game_paused";       data: { paused: boolean } };
type ChampionChangedEvent = { type: "event"; event: "champion_changed";  data: { name: string } };
type EntitySelectedEvent  = { type: "event"; event: "entity_selected";   data: { entity_id: number; kind: "champion" | "minion" | "turret" | "hero"; name: string } };
type GameCloseEvent     = { type: "event"; event: "game_close";        data: { reason: "user_exit" | "crash" } };
type GameLogEvent       = { type: "event"; event: "log";               data: { level: "info" | "warn" | "error"; msg: string } };

type WsEvent = GameLoadedEvent | GamePausedEvent | ChampionChangedEvent
             | EntitySelectedEvent | GameCloseEvent | GameLogEvent;
```

### Rust 端对应定义

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsEvent {
    GameLoaded,
    GamePaused { data: GamePausedData },
    ChampionChanged { data: ChampionChangedData },
    EntitySelected { data: EntitySelectedData },
    GameClose { data: GameCloseData },
    Log { data: LogData },
}

#[derive(Serialize, Deserialize)]
struct GamePausedData { paused: bool }

#[derive(Serialize, Deserialize)]
struct ChampionChangedData { name: String }

#[derive(Serialize, Deserialize)]
struct EntitySelectedData {
    entity_id: u64,
    kind: EntityKind,
    name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EntityKind { Champion, Minion, Turret, Hero }

#[derive(Serialize, Deserialize)]
struct GameCloseData { reason: CloseReason }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CloseReason { UserExit, Crash }

#[derive(Serialize, Deserialize)]
struct LogData { level: LogLevel, msg: String }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LogLevel { Info, Warn, Error }

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsRequest {
    #[serde(rename = "cmd")]
    Command { id: u64, cmd: CmdKind, params: serde_json::Value },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CmdKind { SwitchChampion, GodMode, ToggleCooldown, ResetPosition, TogglePause, GetState }
```

---

## 平台检测与 GameConnection 抽象

### 平台检测

```typescript
const isTauri = !!(window as any).__TAURI__;
```

### GameConnection 接口

```typescript
interface GameConnection {
  send(cmd: string, params: Record<string, unknown>): Promise<WsResponse>;
  onEvent(handler: (event: WsEvent) => void): UnlistenFn;
  connect(): Promise<void>;
  disconnect(): void;
  readonly state: ConnectionState;
}
```

### Desktop: WsGameConnection

直接使用浏览器原生 `WebSocket` 连接 `ws://127.0.0.1:9001`，收到消息后按 `WsEvent` 类型分发。

### Web: WasmGameConnection

封装 wasm 内部调用，对外暴露相同接口：

```typescript
class WasmGameConnection implements GameConnection {
  private eventBus = mitt<WsEvent>();

  async connect() {
    registerWasmCallback((data) => this.eventBus.emit(data));
  }

  async send(cmd, params) {
    return lolDebugCommand(cmd, JSON.stringify(params));
  }

  onEvent(handler) {
    return this.eventBus.on("*", handler);
  }
}
```

### Vue 注入

```typescript
const connection = createConnection();
provide("gameConnection", connection);
```

---

## 前端路由与组件

### 路由

```
/           → Launcher（英雄选择 + 启动）
/debug      → Debug Panel（游戏运行中，仅桌面端）
/play       → AI Battle Panel（现有，不变）
/blog/:name → 博客（现有，不变）
```

### Launcher 页面（`/`）

- 英雄选择网格（图标网格，可选择）
- 模式下拉（Sandbox）
- 启动按钮
- 桌面端启动后自动跳转 `/debug`
- Web 端启动后自动跳转 `/play?mode=sandbox`

### Debug Panel（`/debug`，仅桌面端）

- 连接状态指示灯
- 当前英雄显示 + 切换下拉
- 开关按钮：无敌、无冷却、暂停
- 重置位置按钮
- 日志面板：监听 `log` 事件自动滚动
- Bevy 游戏窗口为独立原生窗口，不在 Tauri webview 内渲染

---

## 进程生命周期管理（桌面端）

### 启动

```
用户点击 [启动] → Tauri invoke("start_game", {mode, champion})
  → Rust: spawn moon_lol --ws-port 9001 --mode sandbox --champion Riven
  → 前端 WS 连接成功 → game_loaded 事件
```

### 关闭

```
用户关闭窗口 / 点击 [停止]
  → 前端 disconnect WS
  → Tauri invoke("stop_game")
  → Rust: kill child process (SIGTERM → 超时 3s → SIGKILL)
```

### 异常处理

- Bevy 进程意外退出 → 前端收到 `game_close {reason: "crash"}`
- 面板显示"游戏已崩溃"，可重新启动

### 端口

MVP 固定 `9001`，单实例。后续可加端口检测自动分配。

---

## Bevy 侧 — PluginDebugPanel

### 文件结构

```
crates/lol_core/src/debug/
├── mod.rs          # PluginDebugPanel, WS server 启动
├── server.rs       # tokio WS server，用 Bevy IoTaskPool
├── protocol.rs     # WsRequest / WsEvent / CmdKind 定义
└── handlers.rs     # 每个命令的处理函数
```

### WS server 集成

```rust
// 用 Bevy IoTaskPool（已绑定 tokio runtime）spawn WS server
fn start_ws_server(world: &mut World, port: u16) {
    let (tx, rx) = async_channel::unbounded();
    let io_pool = world.resource::<IoTaskPool>();
    io_pool.spawn(async move {
        let listener = TcpListener::bind(("127.0.0.1", port)).await?;
        while let Ok((stream, _)) = listener.accept().await {
            let tx = tx.clone();
            tokio::spawn(handle_connection(stream, tx));
        }
    }).detach();
    world.insert_resource(DebugWsChannel(rx));
}

// Bevy Update system poll channel
fn ws_process_commands(channel: ResMut<DebugWsChannel>, ...) {
    while let Ok(req) = channel.try_recv() {
        let result = dispatch(req.cmd, req.params, world);
        channel.send_response(req.id, result);
    }
}
```

依赖：`tokio`、`tokio-tungstenite`、`async-channel`

### 命令 → ECS 操作

| 命令 | 处理方式 |
|------|----------|
| `switch_champion` | despawn 当前 hero entity → spawn 新 hero → 相机切换 |
| `god_mode` | 修改 `DamageMultiplier.received = 0.0` 或恢复 |
| `toggle_cooldown` | 设置全局 `SkillCooldownMode` 标志 |
| `reset_position` | 将 hero `Transform` 设回出生点 |
| `toggle_pause` | 修改 `Time<Virtual>` 相对速度 |
| `get_state` | 查询当前各组件状态并组装返回 |

---

## Tauri Backend

### 文件结构

```
apps/web/src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── capabilities/
│   └── default.json
└── src/
    ├── main.rs       # 入口
    ├── lib.rs        # Tauri commands 注册
    ├── process.rs    # Bevy 子进程管理
    └── state.rs      # 应用全局状态
```

### 核心 Tauri commands

```rust
#[tauri::command]
async fn start_game(state: State<'_, Mutex<AppState>>, config: GameConfig) -> Result<(), String>;

#[tauri::command]
async fn stop_game(state: State<'_, Mutex<AppState>>) -> Result<(), String>;
```

### Bevy CLI 参数

```rust
#[derive(Parser)]
struct Args {
    #[arg(long)] ws_port: u16,
    #[arg(long)] mode: String,
    #[arg(long)] champion: String,
}
```

### 优雅关闭

Tauri `on_window_event` 处理 `CloseRequested` → 先 kill Bevy 子进程。

---

## 构建与分发

### 开发模式

```bash
# 终端 1：Tauri dev server
cd apps/web && cargo tauri dev

# 终端 2：手动启动 Bevy（方便调试日志）
cargo run --example lol -- --ws-port 9001 --mode sandbox --champion Riven
```

### 桌面端打包

```
cargo tauri build
  ├── 构建前端（pnpm build）
  ├── 构建 Bevy（cargo build --release --example lol）
  ├── Bevy 二进制放入 Tauri bundle resources
  └── 输出 .msi / .dmg / .deb / .AppImage
```

Bevy 二进制在打包版中从 `app.path().resource_dir()/bin/moon_lol` 定位，开发版从 `target/` 定位。

### Web 端构建（不变）

```bash
cargo build --release --example lol --target wasm32-unknown-unknown
wasm-bindgen ... --out-dir packages/lol
cd apps/web && pnpm build
```

---

## MVP 范围

| 模块 | 内容 |
|------|------|
| **Tauri 项目** | `apps/web/src-tauri/` 搭建，process 管理，2 个 invoke commands |
| **GameConnection 层** | TypeScript 接口 + `WsGameConnection` + `WasmGameConnection` 骨架 |
| **Launcher 页面** | 英雄选择网格 + 模式下拉 + 启动按钮 |
| **Debug Panel** | 英雄切换、无敌/冷却/暂停开关、日志面板、连接状态 |
| **Bevy PluginDebugPanel** | tokio WS server，6 个命令 handler |
| **CLI 参数** | `--ws-port`、`--mode`、`--champion` |
| **构建** | `cargo tauri dev` 开发，`cargo tauri build` 打包 |

### 不出现在 MVP

- sidecar 自动化构建
- 多实例/多端口
- 高级调试（ECS 查看器、Observer 追踪、技能树可视化）
- Web 端 debug 面板（Web 保持现有 AI panel）
- 账号管理、设置、插件管理
