# Server — 产品迭代路线与完成情况

以下是 MoonLOL 服务端（lol_web_server & Bevy engine / lol_core）的开发路线图与完成情况，根据代码真实状态标记。

## Phase 2 — Web 服务化与资产体系
- [x] **Web 服务端骨架** — 已完成。`crates/lol_web_server` 实现了基础骨架，包含 Auth、Preset、Scenario、Game、History、Log 等路由及 WS 反向代理。
- [x] **用户鉴权与数据隔离** — 已完成。注册、登录和密码重置通过 JWT Bearer 进行身份校验，各 Service 层均支持按 `user_id` 进行分域和隔离。
- [x] **预设体系简化为选手 (Agent) 与独立出生点** — 已完成。整合为扁平选手预设（Agent = 英雄 + 决策类型 + 策略配置），由 `agent_service` 提供增删改查并剥离出生点至槽位级独立配置。
- [x] **胜利条件配置系统存储** — 已完成。Axum 后端 `scenario_service` 支持 win-condition 条件树的 CRUD。
- [ ] **胜利条件运行时引擎集成** — 未完成/进行中。Bevy 游戏核心运行时尚未完整接入胜利条件树的动态逻辑判定（如 kills >= N、补刀 >= N、推掉一塔等自动判胜），目前仍固定以 120 秒计时结束。
- [ ] **Script Agent 热重载引擎运行时** — 未完成。Rust 侧嵌入 JavaScript 运行时（如 Boa/QuickJS）并在 Bevy 引擎中热重载脚本的机制尚未实现。
- [ ] **RL Agent 训练桥接与 Gymnasium 接口** — 未完成。需实现 Bevy 侧的 Gymnasium 环境接口（`MoonLoLEnv`），并开发支持算法选择、超参数/步骤配置与动态奖励函数（Reward Shaper）权重解析的 Python 训练流控桥接服务。

## Phase 3 — 房间与观战
- [x] **房间创建/邀请/大厅双入口服务** — 已完成。`room_service.rs` 已实现房间创建、获取、成员加入/退出/解散，大厅列表以及通过邀请码加入等核心 API。
- [x] **房间多人编排服务** — 已完成。已实现 `validate_add_slot` 与 `validate_join` 约束逻辑，包括槽位上限、单阵营策略等，支持多成员在房间内编排 Agent。
- [x] **服务器对局池与算力监控后台服务** — 已完成。`admin_service.rs` 支持查询运行中对局、内存/算力监控指标快照，以及管理员强行中止（abort）指定对局的功能。
- [ ] **操作流观战与回放数据下发** — 未完成。尚未实现通过 WebSocket 向所有房间成员/观众广播实时操作流并在客户端同步渲染的管线。
- [ ] **BYO Agent 掉线异常挂起判定** — 未完成。尚未实现在房间对局中由于 BYO Agent 的推理服务不响应时自动挂起对局的逻辑。
- [ ] **日志 24h 留存与下载 API** — 未完成。服务器端日志存储在 PostgreSQL 中，但尚未支持将一局对局的日志直接打包为 SQLite 数据库文件并提供下载。

## Phase 4 — Rank 竞技
- [x] **参赛快照发布服务** — 已完成。`agent_snapshot_service.rs` 实现了将当前 Agent 的选手策略配置发布并冻结为不可变 Snapshot 的功能。
- [x] **ELO 评级系统服务** — 已完成。`rank_service.rs` 实现了 `get_elo` 与 `record_result`，支持在对局结束后根据 K-factor 进行 ELO 积分交换。
- [x] **精粹与订阅计费系统服务** — 已完成。`essence_service.rs` 实现了每日签到奖励发放（check-in）、精粹扣除（deduct）及套餐订阅管理。
- [x] **订阅制与 Agent 槽位限制服务** — 已完成。`SubscriptionServiceImpl` 实现了 `AgentLimitProvider`，支持根据用户当前订阅的计划（免费版限5个、Pro版限20个）限制创建 Agent 的数量。
- [ ] **上单 SOLO 模式固定规则引擎** — 未完成。上路一塔前出生的 1v1 对战模式与 Bevy 引擎内的固定胜利规则判定尚未开发。
- [ ] **Rank 7×24 全自动匹配调度器/后台进程** — 未完成/进行中。`rank_service.rs` 中已具备 `try_match` 匹配算法（根据等待时长自动扩展 rating 窗口），但 lol_web_server 尚未启动一个常驻的后台协程（Loop）来全自动轮询匹配池并开局。
- [ ] **掉线宽限期判定与中止重排** — 未完成。Rank 对局中的掉线宽限时间（30s）检测以及无责中止对局的逻辑尚未实现。

## Phase 5 — 社区与职业
- [x] **Agent 分享与 Fork 服务** — 已完成。`community_service.rs` 实现了浏览公开 Agent 列表（browse_public）以及 Fork 他人的 Agent 预设到自己名下（fork）的 API。
- [ ] **上游同步机制** — 未完成。尚未实现已 Fork 的 Agent 自动拉取并合并上游最新配置的后端逻辑。
