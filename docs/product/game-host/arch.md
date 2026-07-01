# 游戏进程托管 — 架构设计

本文档精准描述 Bevy 游戏子进程的托管分层、共享 crate `lol_game_process_manager` 的职责边界、桌面端与云端各自叠加的能力，以及 spawn 命令构建的复用方式与文件引用，不涉及具体代码细节。

## 一、为什么需要这层

桌面端本地调试与云端竞技对局都要托管 Bevy 子进程：分配端口、spawn、记录进程、停止。两端原本各持一份劣化副本——桌面写死端口 9001、硬限单局；云端在 `LocalGameService` 里把进程托管与对局体系焊在一起。本次把**进程托管**抽成共享 crate，两端复用；对局体系仅云端需要，留云端叠加。

两端职责由此清晰：

- 桌面本地调试 = 进程托管 + 与游戏 WebSocket 交互 + AI 决策环。不碰对局记录、胜负判定。
- 云端竞技 = 进程托管 + 对局记录 + 胜负判定 supervisor + AI 决策环。

## 二、分层与数据流

```
┌─────────────────────────────┐    ┌──────────────────────────────┐
│  桌面 Tauri 后端             │    │  云端 lol_web_server          │
│  本地调试：起多游戏 + WS 交互 │    │  LocalGameService             │
│  DesktopProcessLauncher      │    │  + match 记录 / 状态机         │
│  + AI 环（start 时自动起）    │    │  + match_supervisor 胜负判定   │
│  + GameClient 实时调试通道    │    │  + AI 环（start 时显式 spawn） │
└──────────┬──────────────────┘    └──────────┬───────────────────┘
           │                                  │
           └──────────────┬───────────────────┘
                          ▼
        ┌─────────────────────────────────────────┐
        │  lol_game_process_manager（共享 crate）  │
        │  端口池 + 进程表 + ProcessLauncher trait │
        │  + GameProcessManager（start/stop/...） │
        └─────────────────────┬───────────────────┘
                              │ spawn 命令构建复用
                              ▼
        ┌─────────────────────────────────────────┐
        │  lol_client::launch                      │
        │  BevySpawnRequest / build_command /      │
        │  bevy_args / default_rust_log /          │
        │  workspace_root / binary_name            │
        └─────────────────────────────────────────┘
```

进程托管层不含对局语义、不依赖 Bevy / Postgres / HTTP。spawn 命令的构建纯知识归一到 `lol_client::launch`，各端只决定 program 与前缀（桌面 dev `cargo run --` / release 打包二进制；云端 `cargo run --bin moon_lol --` 或 `MOON_LOL_BINARY` env 指向的二进制）。

## 三、共享 crate：lol_game_process_manager

### 1. 端口池

- 范围 9100 至 9200，半开区间，最多 100 个并发游戏进程。
- `allocate_port(used)` 返回池内首个空闲端口，池满返回 None；`is_valid_port(port)` 校验是否在范围内。
- 纯逻辑，无 IO，从原 `lol_web_server::domain::local_game` 迁入。web_server 该模块改为 re-export。

### 2. ProcessLauncher trait

抽象 spawn 与 kill，可 mock。各端提供实现：

- 桌面 `DesktopProcessLauncher`：dev `cargo run --` / release 打包二进制、非 headless、cwd 为 workspace 根、设 RUST_LOG；按 port 维护子进程表供 kill。
- 云端 `CommandProcessLauncher`：`cargo run --bin moon_lol --` headless；优先读 `MOON_LOL_BINARY` env 跑预编译二进制（生产部署），缺省回退 cargo run（dev / CI）。

trait 方法签名 `launch(port, &BevySpawnRequest)` / `kill(port)`，端口由 manager 从池分配后传入。

### 3. GameProcessManager

持有 launcher、可选的 `AgentRunner`、内存进程表（已用端口 HashSet + 托管进程列表）。方法：

- `start(StartGameInput) -> (id, port)`：分配端口 → spawn（失败回滚端口）→ 登记进程 → 若有 `scenario_agents` 且注入了 `AgentRunner` 则 spawn AI 环。
- `stop(id)` / `stop_by_port(port)`：kill + 释放端口 + 移除记录。
- `find_by_port(port)` / `list_processes` / `cleanup`。

`ManagedProcess` 字段为通用 `id`（桌面用作停止 / 选择标识，云端对齐 match_id）、port、status。

### 4. AgentRunner

AI 决策环启动器，类型为闭包 `Fn(port, agents) -> Future`，解耦凭证解析。桌面注入桌面 runner（读 providers.json），云端不注入（云端在 `LocalGameService::start` 内按每请求的 owner_id 显式 spawn，因凭证解析需 owner_id）。

- 参见：[manager.rs](/crates/lol_game_process_manager/src/manager.rs)
- 参见：[port_pool.rs](/crates/lol_game_process_manager/src/port_pool.rs)

## 四、云端 LocalGameService 叠加层

`LocalGameServiceImpl` 持有 `GameProcessManager` + `MatchRepo` + `MatchService` + `ModelProviderService`，在其上叠加对局体系：

1. `match_repo.insert` 建 match 记录。
2. `manager.start` 起进程（端口池分配 + spawn + 进程表）。失败时 `match_repo.update_abort` 回滚。
3. `match_repo.update_ports` / `update_status` 落端口与 running 状态。
4. spawn `match_supervisor::run_supervisor` 订阅 Bevy WS，套用 SOLO 胜负规则落库。
5. 若有场景 agent，spawn `agent_orchestrator::run_agent_orchestrator`（按 owner_id 解析凭证）。

`stop`：校验 match 归属 → `manager.stop_by_port` → `match_repo.update_abort`。行为与重构前一致。

- 参见：[local_game_service.rs](/crates/lol_web_server/src/service/local_game_service.rs)
- 参见：[match_supervisor.rs](/crates/lol_web_server/src/service/match_supervisor.rs)
- 参见：[solo_rules.rs](/crates/lol_web_server/src/domain/solo_rules.rs)

## 五、spawn 命令构建复用

`lol_client::launch` 归一 spawn 纯知识：

- `BevySpawnRequest`：program、prefix_args、port、game_config（mode / champion / scene / headless）、cwd、rust_log。
- `build_command(req) -> std::process::Command`：配 stdio=null、可选 cwd、可选 RUST_LOG、program、前缀、`bevy_args`。桌面同步 `.spawn()` 得 `std::process::Child`；云端 `tokio::process::Command::from(cmd).spawn()` 得异步 Child。
- `bevy_args` / `default_rust_log` / `workspace_root` / `binary_name` 共享。

各端差异仅收敛为 `BevySpawnRequest` 的 program / prefix_args / cwd / rust_log / headless。

- 参见：[launch.rs](/crates/lol_client/src/launch.rs)

## 六、依赖与复用映射

| 层 | 复用来源 | 去向 |
|---|---|---|
| 端口池 | `lol_web_server::domain::local_game` | `lol_game_process_manager`（权威），web_server re-export |
| 进程托管 | `LocalGameServiceImpl` 的进程部分 | `lol_game_process_manager::GameProcessManager` |
| spawn 命令 | 桌面 `process.rs` + 云端 `CommandProcessLauncher` | `lol_client::launch::build_command` |
| AI 决策环 | 桌面 / 云端各一份 Orchestrator | `lol_agent_runtime::run_orchestrator`（见游戏工具文档） |
| 对局体系 | 留云端 | `LocalGameService` + `match_supervisor` + `solo_rules` |
