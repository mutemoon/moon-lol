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
- **开始对局进程调度**: 房主点击开局后，`RoomService` 调用 `MatchService`。`MatchService` 从端口池动态获取可用端口，并通过 Rust `std::process::Command` 启动一个独立的 Bevy 引擎子进程，将快照与条件参数传递给 Bevy。
- 参见：[room_service.rs](/crates/lol_web_server/src/service/room_service.rs) 与 [match_service.rs](/crates/lol_web_server/src/service/match_service.rs)

### 2. Rank 匹配逻辑

- **`rank_service.rs`**: 实现了天梯排队出入队逻辑。
- **自动匹配调度器 (Matching Daemon)**: 后端常驻协程定期轮询 `rank_queues` 状态为 `queued` 的 Agent，基于 ELO rating 差值和等待时间（时间越长差值范围自动扩容）进行匹配。匹配成功后自动拉起 Bevy 对局。
- **ELO 计算**: 引入公式 $R_{new} = R_{old} + K \cdot (S - E)$。S 为胜负平积分 (1/0/0.5)，E 为基于双方分差计算的胜率期望。
- 参见：[rank_service.rs](/crates/lol_web_server/src/service/rank_service.rs)

### 3. WS 观战反向代理与操作流

- **WebSocket 代理**: 前端在观战或调试时连接 Axum 暴露的统一 WS 端点 `/api/ws/:match_id`。Axum 后端解析 Token 并校验权限后，通过网络中继将 WS 连接透明代理转发到 Bevy 子进程的具体运行端口。
- **操作流同步**: Bevy 对局每 tick 将产生的操作流事件写入 PostgreSQL 并通过 WS 广播。前端通过 `GET /api/matches/:id/events?from_seq=<last_seq+1>` 增量轮询或监听中继 WS 实时渲染。
- 参见：[handlers.rs](/crates/lol_web_server/src/handlers.rs) (WS 代理入口)

---

## 三、前端模块与接口对接

- **`/rooms` & `/rooms/:id`**: 房间大厅、邀请码弹窗与多人编排面板。展示阵营约束、用户身份（房主/普通成员/观众），支持槽位占位与修改。
- **`/observe/:id`**: 观战面板。整合了事件流时间线，左侧对比阵营，下部接入 Bevy 公共 `lol_render` 的 WebGPU/WASM 画布重放渲染容器，实现对操作流的本地实时重构渲染。
- **`/rank` & `/leaderboard`**: 选手天梯报名面板与积分榜。表格展示 ELO 总排行与日榜增量，并以奖牌图标标识前三名。
- **`/logs-archive`**: 近 24 小时对局日志归档页。支持打包并下载 SQLite DB 日志文件，由 `/debug` 组件载入还原分析。
- 参见：`apps/client` 下的 Vue 页面结构。
