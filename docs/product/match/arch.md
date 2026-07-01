# 对局与房间/Rank/观战系统 — 架构设计

本文档精准描述对局、房间、Rank 匹配及观战系统的架构设计与实现原理，不涉及具体代码细节。

## 一、对局实体与核心数据库关系

对局及其编排资产由一系列相关联的数据表驱动：

### 1. 核心表结构定义

- **`spawn_presets`**: 记录出生点预设的名字、所属阵营（Order/Chaos）及具体地图二维坐标。
- **`scenarios` / `scenario_win_conditions`**: 储存自定义场景和嵌套胜利条件树的原子节点与逻辑节点（AND/OR/NOT）。
- **`rooms` / `room_members` / `room_agent_slots`**: 支撑房间系统。记录房间属性约束、进入的成员以及红蓝阵营的 Agent 槽位分配。
- **`matches` / `match_participants` / `match_events`**: 统一本地/房间/Rank的三形态对局。`matches` 记录类型（local/room/rank）、运行端口及状态；`match_participants` 保存快照和对战结果；`match_events` 保存按序号 (`seq`) 增量排序的客户端操控/行为事件。
- **`seasons` / `rank_queues` / `elo_ratings`**: 支撑 Rank 天梯匹配。记录当前赛季状态、匹配队列中的 Agent 状态以及具体的 ELO rating 积分。
- 参见：[schema.sql](/crates/lol_web_server/migrations/schema.sql)

---

## 二、Web Server 服务层编排设计

### 1. 房间生命周期与进程调度

- **`room_service.rs`**: 负责房间大厅加入、邀请码验证与阵营槽位分配。
- **开始对局进程调度**: 房主点击开局后，`RoomService` 调用 `LocalGameService::start`。进程托管（端口池分配 + spawn + 进程表）由共享 crate `lol_game_process_manager` 承担，spawn 命令构建复用 `lol_client::launch`；本层在其上叠加 match 记录、状态机与 supervisor。详见 [游戏进程托管](../game-host/)。
- 参见：[room_service.rs](/crates/lol_web_server/src/service/room_service.rs) 与 [match_service.rs](/crates/lol_web_server/src/service/match_service.rs)

### 2. Rank 匹配逻辑

- **`rank_service.rs`**: 实现了天梯排队出入队逻辑。
- **自动匹配调度器 (Matching Daemon)**: 后端常驻协程定期轮询 `rank_queues` 状态为 `queued` 的 Agent，基于 ELO rating 差值和等待时间（时间越长差值范围自动扩容）进行匹配。匹配成功后自动拉起 Bevy 对局。
- **ELO 计算**: 引入公式 $R_{new} = R_{old} + K \cdot (S - E)$。S 为胜负平积分 (1/0/0.5)，E 为基于双方分差计算的胜率期望。
- 参见：[rank_service.rs](/crates/lol_web_server/src/service/rank_service.rs)

### 3. 胜负判定与 Match Supervisor

- **判定位置在 web server，不在 Bevy 进程内**：Bevy 引擎只负责产出结构化对局事件（英雄击杀 `champion_kill`、推塔 `turret_destroyed`、补刀里程碑 `cs_threshold`、时间推进 `time_progress`），由 `lol_core` 的 `match_events` 插件经 `MatchEventChannel` 写出，`lol_server` 转发到 WS。
- **Match Supervisor**：`LocalGameService` 启动 Bevy 子进程后，为每个对局 spawn 一个 `match_supervisor` tokio task，订阅子进程 WS，按事件到达顺序维护 `SoloState`，每条事件后调用纯函数裁决器 `solo_rules::evaluate` 判定胜负。命中后调 `MatchService::finish_internal` 落库（`winner_team` + 参与者结果），并把每条事件 `append_event_internal` 写入 `match_events` 供观战轮询。
- **SOLO 规则**：先达成任一即胜——拿一血 / 推掉对方一塔 / 补刀满 100；若游戏超过 15 分钟仍未分胜负，则按补刀数判定胜负（多者胜，相等为平局）。"先到先得"由事件到达顺序天然保证。
- 参见：[solo_rules.rs](/crates/lol_web_server/src/domain/solo_rules.rs)、[match_supervisor.rs](/crates/lol_web_server/src/service/match_supervisor.rs)、[match_events.rs](/crates/lol_core/src/match_events.rs)

### 4. WS 观战事件流与操作流同步

- **WebSocket 观战订阅**: 前端在观战或回放时，连接 Axum 暴露的 WS 端点 `/api/matches/:id/events/ws?token=<jwt_token>`。Axum 后端校验 Token 与权限后，返回与该对局关联的内存广播通道（由对局 Match Supervisor 在拉起 Bevy 子进程后自动创建并管理会话）。
- **增量防丢包同步**: 客户端建立 WebSocket 链接后，Axum 处理器首先通过数据库读取从 `from_seq` 开始的所有历史事件，然后接入实时广播流并根据 `seq` 进行去重过滤，以推送给客户端完整的操作流事件包。
- 参见：[match_.rs](/crates/lol_web_server/src/handlers/match_.rs) (WS 观战订阅实现)

---

## 三、前端模块与接口对接

- **`/rooms` & `/rooms/:id`**: 房间大厅、邀请码弹窗与多人编排面板。展示阵营约束、用户身份（房主/普通成员/观众），支持槽位占位与修改。
- **`/observe/:id`**: 观战面板。整合了事件流时间线，左侧对比阵营，下部为重放渲染容器，使用统一的 `SpectatorDriver` 驱动底层渲染：
  - **Web 端** (`WasmSpectatorDriver`)：接入 Canvas 中的 Bevy WASM 画布。JS 缓存全部操作流事件，当用户拖拉时间轴 Seek 后退时，JS 重置 WASM Canvas，并从第 0 帧到目标帧一次性批量注入事件（Fast-forward），快速还原状态。
  - **Desktop 端** (`LocalSpectatorDriver`)：直接调本地 Tauri 命令控制游戏子进程，不走画布。
- **`/rank` & `/leaderboard`**: 选手天梯报名面板与积分榜。表格展示 ELO 总排行与日榜增量，并以奖牌图标标识前三名。
- **`/logs-archive`**: 近 24 小时对局日志归档页。支持打包并下载 SQLite DB 日志文件，由 `/debug` 组件载入还原分析。
- 参见：`apps/client` 下的 Vue 页面结构。
