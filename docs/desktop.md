# Desktop 启动器 + 调试面板

## 架构

```
apps/desktop (Tauri 2 + Vue 3)        crates/lol_core/src/debug/
┌────────────────────────────┐        ┌──────────────────────┐
│  Launcher → Debug Panel    │  WS    │  PluginDebugPanel     │
│         ↓ invoke           │ ←───→  │  tokio WS server      │
│  Tauri Rust Backend        │ :9001  │  6 个命令 handler     │
│    start_game / stop_game  │        └──────────────────────┘
└──────────┬─────────────────┘
           │ spawn / kill (cargo run)
           ▼
┌─────────────────┐
│  Bevy 游戏进程     │
│  examples/lol    │
└─────────────────┘
```

- **前端**: Vue 3, 两个视图 — Launcher（`/`） + Debug Panel（`/debug`）
- **后端**: Tauri Rust, `start_game` / `stop_game` 两个 invoke command
- **Bevy 侧**: `PluginDebugPanel` 启动 tokio WS server，监听 `127.0.0.1:9001`

## 文件结构

```
apps/desktop/
├── src/
│   ├── App.vue                     # Launcher + Debug Panel 切换
│   ├── main.ts                     # 入口
│   ├── composables/
│   │   └── useWsClient.ts          # WebSocket 客户端封装
│   └── components/
│       └── DebugPanel.vue          # 调试面板 UI
├── src-tauri/
│   ├── Cargo.toml                  # 独立 crate，不在 workspace 中
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   └── src/
│       ├── main.rs                 # 入口
│       ├── lib.rs                  # start_game / stop_game 注册
│       ├── process.rs              # Bevy 子进程管理
│       └── state.rs                # AppState
├── package.json
└── vite.config.ts

crates/lol_core/src/debug/
├── mod.rs          # PluginDebugPanel
├── protocol.rs     # WsRequest, CmdKind, WsEvent, WsResponse
├── server.rs       # tokio WS server
└── handlers.rs     # 6 个命令 handler + ChampionSwitchQueue
```

## WS 协议

所有消息均为 JSON，通过 `ws://127.0.0.1:9001` 通信。

### 命令（panel → 游戏）

```json
{"id": 1, "type": "cmd", "cmd": "switch_champion", "params": {"name": "Riven"}}
```

| 命令 | params |
|------|--------|
| `switch_champion` | `{name: string}` |
| `god_mode` | `{enabled: bool}` |
| `toggle_cooldown` | `{enabled: bool}` |
| `reset_position` | `{}` |
| `toggle_pause` | `{}` |
| `get_state` | `{}` |

### 响应

```json
{"id": 1, "type": "result", "ok": true}
{"id": 1, "type": "result", "ok": false, "error": "msg"}
```

### 事件（游戏 → panel）

```json
{"type": "event", "event": "game_loaded", "data": {}}
{"type": "event", "event": "champion_changed", "data": {"name": "Riven"}}
{"type": "event", "event": "game_paused", "data": {"paused": true}}
{"type": "event", "event": "game_close", "data": {"reason": "crash"}}
{"type": "event", "event": "log", "data": {"level": "info", "msg": "..."}}
```

## Dev 模式 vs Release

| | Dev | Release |
|---|---|---|
| 启动方式 | `cargo run --example lol` | 直接运行 `lol.exe` |
| `dynamic_linking` | ✅ 默认 feature | ❌ `--no-default-features` |
| DLL 依赖 | bevy_dylib.dll + std-*.dll | 无（静态链接） |
| binary 来源 | cargo 自动编译 | bundled in resource_dir/bin/ |

### 开发

```bash
# 终端 1：Tauri dev（自动 cargo run 启动 Bevy）
cd apps/desktop && pnpm tauri dev
```

### 打包

```bash
cd apps/desktop && pnpm tauri build
# beforeBuildCommand = pnpm build ; cargo build --release --example lol --no-default-features
```

`dynamic_linking` 通过 root crate 的 `[features]` 控制：

```toml
# Cargo.toml (root)
[features]
default = ["bevy/dynamic_linking"]  # dev 开启，release --no-default-features 关闭
```

`tauri.conf.json` 的 `beforeBuildCommand` 用 `;` 串联前端构建 + Bevy release 构建（去 dynamic_linking）。

## Tauri 子进程管理

`process.rs` 启动逻辑：

```
start_game(config)
  → is_dev? 
    → YES: cargo run --example lol -- --ws-port 9001 --mode ... --champion ...
    → NO:  resource_dir/bin/lol.exe --ws-port 9001 --mode ... --champion ...
  → cwd 设为 workspace root（确保 assets/ 能找到）
```

`stop_game()` 直接 `child.kill()`。

## Bevy 调试命令实现

### handlers.rs

| 命令 | 实现方式 |
|------|----------|
| `switch_champion` | despawn 当前 Champion → 推入 ChampionSwitchQueue → lol_champions 下一帧 spawn |
| `god_mode` | 插入/移除 `BuffDamageReduction { percentage: 1.0 }` |
| `toggle_cooldown` | 切换所有 Skill 的 `SkillCooldownMode` (AfterCast ↔ Manual) |
| `reset_position` | 将 Champion 的 `Transform.translation` 设回 `Vec3::ZERO` |
| `toggle_pause` | 设置 `Time<Virtual>::relative_speed` 为 0.0 / 1.0 |
| `get_state` | 查询 GlobalDebugState + 当前英雄名 |

### ChampionSwitchQueue

`handlers.rs` 定义 `ChampionSwitchQueue(pub Vec<String>)` 资源。handler 写入，`lol_champions/src/lib.rs` 的 `process_champion_switch_queue` 系统读取并 spawn 对应英雄（Riven / Fiora）。

## apps/web（Web 端）

不变。保持现有 wasm 模式 + AI Battle Panel。
