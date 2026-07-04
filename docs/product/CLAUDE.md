# 产品架构总览

本文件是 `docs/product` 的入口，简短介绍整个 MoonLOL 产品的架构。各子系统的详细产品 / 架构 / 待办见对应子目录。

## 一、产品定位

MoonLOL 是基于 Bevy 引擎复刻《英雄联盟》之上的 **AI Agent 电竞平台**。用户扮演"电竞经理"，配置、培养、发布 Agent 选手（英雄 + 决策配置），让 Agent 在召唤师峡谷自动对战、自动上分。

服务三类人群：研究者（调试 Agent 策略）、普通玩家（社交观战）、竞技作者（冲天梯排名）。

## 二、技术栈

| 层级 | 选型 |
|---|---|
| 游戏引擎 | Bevy (Rust ECS)，定制支持 WebGPU 与 Basis Universal 纹理压缩 |
| Web 服务端 | Axum + SQLx + PostgreSQL + JWT 鉴权 |
| 桌面客户端壳 | Tauri (Rust 后端 + WebView 前端) |
| 客户端前端 | Vue 3 + TypeScript + Pinia + TailwindCSS + shadcn-vue |
| 通信协议 | WebSocket |
| LLM 框架 | Rig SDK / Anthropic API |
| 模型训练 | Python + PyTorch + CleanRL (PPO) + Gymnasium |

## 三、物理架构与 crate 分层

```
┌─────────────────────────────────────────────────────────┐
│  apps/client/src-tauri   桌面客户端壳 (Tauri + Vue)       │
│    rig agent 编排环 · 本地对局进程托管                  │
└───────────────┬──────────────────────────┬──────────────┘
                │ HTTP / WS                │ 进程内 rmcp
┌───────────────▼──────────┐   ┌───────────▼──────────────┐
│  lol_web_server          │   │  lol_client  共享游戏客户端 │
│  Axum 服务端              │   │  协议 + WsSession + GameClient │
│  handlers→service→repo→  │   │  + GameToolServer (MCP)   │
│  cache→domain→infra      │   └───────────┬──────────────┘
└───────────────┬──────────┘               │ WS
                │ GameProcessManager 托管   │ ws://127.0.0.1:{port}
                │ (端口池 + lol_client::launch)│
                ▼                          ▼
┌─────────────────────────────────────────────────────────┐
│  Bevy 游戏进程 (子进程)                                   │
│  lol_core · lol_champions · lol_agent · lol_debug        │
│  lol_render · lol_server (WS cmd 字符串分发)              │
└─────────────────────────────────────────────────────────┘
```

- **`lol_client`**：Bevy-free 的共享游戏客户端。协议类型、WS 会话、类型化 `GameClient` 命令面与 MCP 工具层 `GameToolServer`，以及 `launch` 模块的 spawn 命令构建（`BevySpawnRequest` / `build_command`）。CLI、MCP、Tauri、web server 四处共用，不各自持有一份协议或会话代码。
- **`lol_game_process_manager`**：游戏进程托管共享 crate。端口池、进程表、`ProcessLauncher` trait、`GameProcessManager`。桌面本地调试与云端 `LocalGameService` 共用，对局体系仅云端叠加。详见 [游戏进程托管](game-host/)。
- **`lol_cli`**：clap 薄前端，依赖 `lol_client`，提供完整控制面：observe / action / pause / state / switch_champion / god_mode / toggle_cooldown / reset_position / get_agents / set_script / rl_*。
- **`lol_web_server`**：六层依赖倒置服务端。常驻 Rank 匹配守护协程、每局一个 match_supervisor、以及 web server 侧的 agent 决策环。
- **Bevy 进程**：`lol_server` 经 observer 事件按 cmd 字符串分发到 `lol_agent` / `lol_debug` 处理；`lol_core::match_events` 产出结构化对局事件经 WS 转发。

## 四、对局三形态

长期并存、共享同一套 Agent 资产：

| 形态 | 游戏运行位置 | 登录 | 侧重点 |
|---|---|---|---|
| 本地多 Agent 对局 | 本机，desktop 独占 | 必须登录 | 研究调试，配置统一走云端 |
| 房间多用户对局 | 服务器 | 必须登录 | 社交娱乐，实时观战 |
| Rank 竞技 | 服务器 | 必须登录 | 竞技排名，7×24 自动上分 |

房间是 Rank 的基础，Rank 本质是系统当房主的全自动房间；Desktop 是 Web 的超集，Web 是轻量消费端。

## 五、Agent 选手体系

**选手 = 英雄 + 决策类型 + 策略配置**。三种类型：

- **LLM Agent**：大语言模型推理，经 WS 收 `get_observe` 观测、下发 `action` 动作。
- **RL Agent**：强化学习模型，经 Gymnasium 环境接口 `MoonLoLEnv` 高频微操。
- **Script Agent**：内嵌 JS 运行时执行用户脚本，支持热重载。

ELO 主体是 Agent 而非用户，按 Agent × 模式分别计算。Rank 队列使用发布冻结的参赛快照，进行中的对局不受配置变更影响。

## 六、游戏工具层（外部与游戏交互的统一入口）

- **CLI**：面向开发者 / 运维，提供完全控制。
- **MCP 工具层**：面向 rig agent，进程内内存消费，仅暴露 observe / action 两个能力（调试 / 作弊类指令不进入，避免 agent 越权）。
- 二者共享 `lol_client`，协议与会话逻辑只此一份。Tauri 与 web server 的 rig agent 均经 `serve_inprocess` 注入同一套 rmcp tools，取代子进程桥。

## 七、计费

单局 Bevy 实例约 50 MB 内存，并发对局数受物理内存约束。模型动力源两条路径：平台模型按 Token 消耗以**精粹**结算；BYO 模型自行提供 API Key，平台不参与计费。精粹来源为每日签到与充值，用途为抵扣 Token 与购买额外选手槽位。免费用户上限 5 个 Agent，订阅档位提供更多槽位与定期精粹。

## 子系统文档索引

- [账号、平台与客户端](platform/) — 双端形态、Web 服务端六层架构、前端三层服务、离线在线同步。
- [对局与房间 / Rank / 观战](match/) — 三形态对局、胜负判定、Match Supervisor、WS 观战代理。
- [Agent 选手与决策体系](agent/) — 英雄 vs 选手、三种 Agent 类型、快照发布、社区 Fork。
- [游戏工具](game-tools/) — CLI + MCP 并存、共享客户端 lol_client、rig agent 接入。
- [游戏进程托管](game-host/) — 共享 crate lol_game_process_manager、端口池、桌面本地调试与云端竞技分层、spawn 命令复用。
- [算力、精粹与订阅计费](billing/) — 服务器算力约束、模型动力源、精粹规则、订阅槽位。
- [模型供应商与模型设置](llm-provider-setting/) — 供应商管理、预设目录、运行时按 provider 解析凭证。
- [游戏控制 RPC](rpc-bridge/) — cmd 字符串手搓分发改 `CommandWsRequest<T>` 泛型事件 + 类型化入参 + 单一 dispatch 注册表，消除双重分发与类型擦除，handler 类型化、样板归零。
