# Agent 选手与决策体系 — 待办与完成情况

本页面记录 Agent 选手与决策体系的已完成和未完成任务，整合了客户端与服务端的技术指标。

## 一、🔴 当前优先项 — 三类决策驱动类型的端到端贯通

LLM 与 Script 选手的对局运行时已贯通（Script 经 `lol_agent::driver` 的 `ScriptDriver` rquickjs 沙盒驱动）。剩余优先级最高的是把 **RL 选手**（Gym 接口 / 推理端点 / Reward 分配）端到端打通。

---

## 二、已完成任务列表

- [x] **选手资产体系重构**：整合原有多层级“大脑”预设，合并为统一的选手管理，支持独立配置英雄、策略与决策类型。
- [x] **可见性与快照发布服务**：实现了选手的私有/公开可见性控制以及不可变参赛快照的发布与提取。
- [x] **快照管理 UI**：`heroes.vue` 中集成了发布快照按钮、可见性切换以及快照版本时间线。
- [x] **社区 Fork 功能**：在 `community_service` 中实现了浏览公开选手、Fork 生成自身副本的 API 以及前端一键克隆的交互。
- [x] **LLM 实时决策流观察**：支持在 `/debug` 中展示 LLM Agent 的 CoT (思考链) 以及观测/动作的实时流。
- [x] **常用英雄技能框架**：实现了锐雯 (Riven) 和菲奥娜 (Fiora) 的核心战斗与招式机制。
- [x] **桌面端云优先与冲突解决**：选手以云端为主存储，桌面端在线云优先 + 本地镜像、离线降级；本地与云端差异以 `冲突 / 未同步 / 仅云端` 徽标呈现，冲突态只改本地，同步对话框逐项选择保留哪边（详见第 4 节）。

---

## 三、待完成任务列表

### 1. Script Agent 脚本驱动与编辑器

#### 客户端 (Client)
- [x] **集成 Monaco Editor**：在选手编辑页中接入 Monaco 替换普通 Textarea，提供 JS 语法高亮与智能提示。（`components/agent/ScriptEditor.vue` + `lib/monaco.ts` 本地化 worker）
- [x] **添加 TypeScript 类型定义**：引入 `.d.ts` 声明文件，使 Monaco editor 能为 `observe()` / `action()` / `log()` 提供精确的类型推导。（`services/scriptAgentTemplates.ts` 的 `SCRIPT_API_DTS`，与服务端 `lol_agent::models` 对齐）
- [x] **内置脚本模版库**：提供常用动作脚本（如一键走A、智能补刀、特定技能连招等）的点选快速填充。（`SCRIPT_TEMPLATES`：走A / 补刀 / 连招）
- [x] **热重载与编译状态指示**：在编辑器中显示运行时当前加载的脚本状态、上次热重载时间及报错日志。（编译状态取自 Monaco 静态诊断；运行时活动脚本状态待服务端 ScriptDriver 接入）
- [x] **调试面板与日志流**：展示脚本的单独日志流，并提供断点、单步执行和局部变量观测的操控入口。（日志流 + 控制入口已就绪，运行时联动待 ScriptDriver）

#### 服务端 (Server)
- [x] **按 AgentType 运行时分发器**：设计 `AgentDriver` trait，实现 `LlmDriver` / `RlDriver` / `ScriptDriver` 的分流，并在 Bevy 开局时按类型实例化。（`lol_agent::driver`：`AgentDriver` trait + `AgentKind` + `create_driver` 工厂）
- [x] **嵌入 JS 沙盒运行时**：使用 `rquickjs` 库在 Bevy 中运行 JS 选手代码，禁用网络与硬盘 I/O。（QuickJS 默认无 fs/net，仅注入显式宿主绑定）
- [x] **时间片监控与安全熔断**：限制 JS 脚本单次 tick 执行的 CPU 时长（如超出 5ms 强制挂起），防止因死循环挂起 Bevy 引擎。（`Runtime::set_interrupt_handler` + `DEFAULT_TICK_BUDGET=5ms`，单测覆盖死循环熔断）
- [x] **暴露宿主 API**：提供 `observe()`、`action()`、`log()`、`wait_ticks(n)` 等原生 Rust 到 JS 的绑定。（`__observe/__push_action/__log/__wait` 原生绑定 + JS 前导封装）
- [x] **运行时热重载**：监听脚本变化或 WS 推送，无缝替换正在对局中的 JS 执行函数，保留状态数据。（`ScriptDriver::reload` 保留 `globalThis.state`；`ScriptAgent` 变更检测 + `set_script` WS 指令热重载）

---

### 2. RL Agent 训练、遥测与可视化

#### 客户端 (Client)
- [x] **编辑页配置扩展**：支持为 RL 选手提供模型权重路径 (`.pth`)、BYO 推理端点 URL 以及 Reward Shaper 权重表配置。（`heroes.vue` RL 结构化表单，写入 `config_json.{model_path,inference_endpoint,reward_shaper}`）
- [x] **遥测数据 WS 接入**：在 `/rl-training` 页面对接真实的 Python 训练守护进程，展现 reward、loss、KL 散度等曲线。（`useRlTelemetry` 原生 WS 客户端 + 无守护进程时本地模拟回落）
- [x] **策略分布与动作概率可视化**：在训练面板展示 Agent 当前状态的动作概率分布。（`policyDist` 概率条，随遥测帧更新）
- [x] **模型快照与权重切换**：支持保存训练中的 checkpoint，并提供”一键应用为当前策略配置”的功能。（保存 Checkpoint + 选 RL 选手一键写入 `config_json.model_path` 并通知守护进程）

#### 服务端 (Server)
- [x] **Gymnasium 接口包装**：在 Bevy 侧实现 `MoonLoLEnv` 环境的 standard 重置与单步交互。（`lol_agent::rl::MoonLoLEnv` reset/step + `rl_reset`/`rl_step` WS 指令，单测覆盖终止/截断）
- [x] **高频张量 WS 传递**：实现支持二进制或 msgpack 格式的 obs/action 高速状态流传递，用于高频推理。（`rmp-serde` 编解码 + base64 包装：`get_observe_packed`/`action_packed`，obs/action msgpack 往返单测）
- [x] **Reward 动态分配器**：在 `step()` 过程中解析 `config_json` 里的 Reward Shaper 配置，动态拼装最终 reward。（`RewardShaper::from_config_json` + 分项 `RewardBreakdown`，支持部分配置回落默认）
- [x] **训练守护进程与 API**：用 Python 封装训练控制中心，暴露启动/停止训练、checkpoint 存取的 REST 接口。（`services/rl-trainer/` 纯标准库：REST `/api/train/*` + `/api/checkpoints/*`、WS 遥测/控制、`SimulatedEnv`/`BevyEnv`，11/11 单测通过）

---

### 3. Fork 溯源与上游同步机制

#### 客户端 (Client)
- [x] **Fork 详情展示**：在选手页展示”Fork 自 [作者/原选手名称]”的链接和版本。（发布页「上游同步」区，解析上游 Agent 名称与作者 `owner_id`）
- [x] **差异对比与合并预览**：提供「拉取上游更新」入口，展示 Prompt / config_json 差异，让用户选择覆盖或保留。（`ForkDiffDialog` 用 Monaco DiffEditor 并排对比 Prompt/config_json，确认即调 `pullUpstream` 覆盖、取消即保留）

#### 服务端 (Server)
- [x] **上游拉取 API**：实现 `POST /api/agents/:id/pull-upstream`，将上游最新快照策略拉取并覆盖当前选手的编辑态，标记为待发布。（`community_service::pull_upstream` + Fork 时持久化 `upstream_agent_id`；`update` 刷新 `updated_at` 触发未发布指示）

---

### 4. 账号与同步优化 (云优先 + 冲突解决)

#### 客户端 (Client)
- [x] **桌面端云优先 + 本地镜像**：选手以云端为主存储，桌面端在线时 CRUD 走云端、写后镜像本地缓存，离线降级 Tauri 本地；Web 端始终云端。（`backend.ts` 共享 `cloudHeroPresetHandlers` + `mirrorHeroPresetToLocal`，桌面在线分支优先云端、失败回退本地）
- [x] **去除静态默认选手**：移除 `BUILTIN_HERO_PRESETS` 演示假数据，列表为空即真的没有选手。（`stores/gameStore.ts`；`BUILTIN_SPAWN_PRESETS` 保留）
- [x] **本地 ↔ 云端冲突解决**：回到在线后若本地与云端有差异，卡片显示 `冲突 / 未同步 / 仅云端` 徽标；冲突态下保存只写本地；点「同步」弹出对话框左右两列对比云端/本地，逐项选择保留哪边（保留本地=推送覆盖云端，保留云端=拉取覆盖本地）。（`heroes.vue` `computeDivergences`/`displayPresets` + `SyncConflictDialog.vue`，复用 `uploadPresetToCloud`/`pullCloudToLocal`）
- [x] **同步生命周期状态机**：用 xstate 集中管理 `offline / synced / divergent / dialogOpen / applying` 流程态，差异态禁止写云端、离线态无同步按钮由显式 guard 表达。（`composables/useAgentSyncMachine.ts`；Web 端 stub）
- [x] **选手脏状态与未发布指示**：选手策略被修改后，卡片和发布面板应有视觉高亮（如”有未发布改动”），提醒重新发布快照。（`hasUnpublishedChanges`：云端 `updated_at` 晚于最新快照即标记）
- [x] **Thinking Depth 调节组件**：在选手配置中直接提供 LLM 思考深度的调节滑块。（写入 `config_json.thinking_depth`）
- [x] **JSON 导入导出**：在选手列表页支持将选手策略导出为 JSON/RON 文件或从本地文件导入。

#### 服务端 (Server)
- [x] **创建/更新端点规范化**：实现 `/api/agents` 的 `POST` 和 `PUT` API，统一校验 `AgentLimitProvider` 规定的选手槽位限额（免费版上限为 5 个）。（`agent_service.rs` create/update + `assert_within_slot_limit`）

---

### 5. LLM 模型配置改为供应商/模型下拉选择

模型字段不再手填，改为「供应商 + 模型名」级联下拉，供应商目录与设置页详见 [模型供应商与模型设置](../llm-provider-setting/todo.md)。

#### 客户端 (Client)
- [ ] **供应商 + 模型级联下拉**：`heroes.vue` 的 LLM 模型字段改为两个下拉——供应商下拉列出设置页所有启用供应商（平台模型 / 预设 / 自定义分组），模型名下拉随供应商切换刷新其 `models` 列表；选中写入 `agents.model` 与 `config_json.provider_id`。
- [ ] **手填兜底与回写**：模型名下拉保留「手填」入口以覆盖预设外新模型，手填值回写进该供应商的 `models` 以便复用。
- [ ] **运行时凭证解析联动**：LLM 执行器按 `config_json.provider_id` 在本地供应商表查 `baseUrl / apiKey / apiFormat`，配合 `agents.model` 发起请求。
