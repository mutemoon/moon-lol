# Client — 产品迭代路线与完成情况

以下是 MoonLOL 统一客户端 apps/client（在桌面与 Web 运行模式下）的开发路线图与完成情况，根据代码真实状态标记。

## Phase 1 — 本地 MVP
- [x] **LLM Agent 对战模拟与实时决策观察** — 已完成。在 `/debug` 页面中可实时观察 LLM Agent 的思考链（CoT）对话历史及游戏日志。
- [x] **红蓝双方多 Agent 编排** — 已完成。在主页（编排页）可以配置 Order 和 Chaos 两侧的槽位。
- [x] **出生点地图点选器** — 已完成。在 `/spawn-presets` 页面可通过交互式 SVG 地图点选并保存坐标。
- [x] **对局历史持久化与回放** — 已完成。在 `/history` 页面可查看已持久化的本地对局历史并加载回放。
- [x] **设置面板（模型配置、常规设置）** — 已完成。`/settings` 页面已支持主题、语言、模型 API Key/Base URL 等常规设置。
- [x] **布局重命名与侧边栏调整** — 已完成。将 `desktop.vue` 布局重命名为 `dashboard.vue`，并将"账号相关（电竞经理）"与"设置"选项调整至侧边栏的最底部。

## Phase 2 — Web 服务化与资产体系
- [x] **预设体系简化为选手 (Agent) 与独立出生点** — 已完成。整合为统一的选手管理（`heroes.vue` 作为“我的选手”管理页），去除了多层级“大脑”预设绑定，并在槽位层独立支持选择出生点。
- [x] **胜利条件配置系统 UI** — 已完成。在主页中集成了 `WinConditionBuilder.vue` 和 `WinNode.vue`，支持配置条件原子与组合逻辑，并随场景保存。
- [x] **Script Agent 编辑器** — 已完成。`heroes.vue` 在 `agent_type === 'script'` 时切换为 Textarea JSON 编辑器，支持输入和保存自定义脚本/RL JSON配置。
- [x] **RL Agent 训练与可视化面板** — 已完成。`/rl-training` 提供算法选择（PPO/SAC/DQN）、超参数（学习率/Gamma/熵系数/Clip/网络架构）、训练步骤（Max Timesteps/Rollout/Batch/Epochs）、奖励权重（Reward Shaper）配置，以及启动/暂停/恢复/终止流控；右侧面板实时呈现训练进度、Episodic Return / Loss / KL / Entropy 四联曲线、策略分布、Reward 分项与 Value 估计。

## Phase 3 — 房间与观战
- [x] **房间创建/邀请/大厅双入口 UI** — 已完成。`/rooms` 顶部提供「邀请码加入」与「创建房间」两个核心入口；下方 Tabs 切换公开大厅与我的房间，卡片网格展示房间名称、人数、阵营策略、状态。
- [x] **房间多人编排 UI** — 已完成。`/rooms/:id` 用红蓝两栏对称布局展示双阵营 Agent 槽位，标题栏单行 chip 展示约束（人数 / 每人 Agent 数上限 / 阵营策略 / 大厅可见性 / Prompt 可见性 / 邀请码），添加/移除/离开/解散/开始对局齐备。
- [x] **服务器对局池与算力监控后台 UI** — 已完成。`/admin` 上部用 stat 列呈现运行中对局数、总内存、平均内存/局、CPU 使用率（含 Progress 条），下部 Table 列出全部进行中对局并支持「强制中止」。
- [x] **操作流观战与回放 UI** — 已完成。`/observe/:id` 主区域分三块：摘要带（直播徽章 / 暂停刷新 / 结束对局）、阵营对照、事件时间线；预留 WASM/WebGPU 渲染容器供后续接入。
- [x] **BYO Agent 掉线异常可视化** — 已完成。`/observe/:id` 监听 `agent_stalled` / `agent_resumed` 事件，顶部展示"对局已暂停等待恢复"告警条，阵营对照中失联 Agent 的状态点变橙色。
- [x] **日志 24h 留存与 SQLite DB 加载分析 UI** — 已完成。`/logs-archive` 列出近 24h 的对局并展示剩余保留时间，支持下载服务端日志 SQLite DB；上部提供本地 `.sqlite` 加载入口，可一键交至 `/debug` 调试器分析。

## Phase 4 — Rank 竞技
- [x] **参赛快照发布机制 UI** — 已完成。`heroes.vue` 顶部新增「发布参赛快照」按钮与可见性切换（私有/好友/公开），底部以列表展示历史快照（版本号 + 发布时间 + "最新"标记）。
- [x] **Rank 全自动匹配队列与报名排队 UI** — 已完成。`/rank` 上部为报名区（模式 / Agent / 参赛快照三选下拉 + 入队按钮），下部为「我的排队状态」列表（模式 / ELO / 等待时长 / 退出）。
- [x] **ELO 排行榜 UI** — 已完成。`/leaderboard` Tabs 切换总排行 / 今日增量，按模式过滤；Table 列出排名、Agent、ELO/Δ、胜负、胜率，前三名以奖牌图标突出。
- [x] **精粹与订阅计费 UI** — 已完成。`/billing` 自上而下三段：精粹余额 hero stat + 每日签到；订阅套餐对比卡（含当前订阅高亮）；精粹流水明细表（签到 / Token 消耗 / 充值 / 订阅 / 槽位购买等类型）。

## Phase 5 — 社区与职业
- [x] **Agent 社区市场 UI** — 已完成。`/community` Tabs 切换最新 / 热门 / ELO 排序，搜索框过滤 Agent / 英雄；卡片网格展示公开 Agent，点击触发 Fork 对话框（自定义新名称，Fork 成功后跳回 `/heroes?focus=…`），Fork 出的副本卡片附 GitFork 徽章。
