# Agent 选手与决策体系 — 架构设计

本文档精准描述 Agent 选手与决策体系的实现原理、数据库表、API 设计与文件引用，不涉及具体代码细节。

## 一、接口设计与业务流

Agent（选手）的生命周期包括：创建选手、修改策略、发布快照、报名排队、更新/Fork以及获取 ELO 评分。

### 1. 核心数据表结构说明

- **`agents`**: 存储选手的基础信息（如关联英雄、选手名称）及策略配置（Prompt, Preamble, 模型类型, 以及 `config_json` 字段）。
- **`agent_snapshots`**: 冻结的参赛快照。每次在 Rank 报名排队时，自动生成/提取该版本的只读快照，避免比赛中途策略变更导致的行为不一致。
- **`agent_fork_relations`**: 存储选手之间的 Fork 继承关系，记录上游 Agent ID 以便检测更新。
- 参见：[schema.sql](/crates/lol_web_server/migrations/schema.sql)

### 2. HTTP 接口列表

- `GET /api/agents` - 列出当前用户的选手预设。
- `POST /api/agents` - 创建新选手。
- `PUT /api/agents/:id` - 更新选手策略与配置。
- `POST /api/agents/:id/publish` - 发布一个不可变的参赛快照。
- `POST /api/agents/:id/fork` - Fork 社区中公开的选手。
- `POST /api/agents/:id/pull-upstream` - 从上游拉取最新的选手配置。
- 参见：[handlers.rs](/crates/lol_web_server/src/handlers.rs)

---

## 二、服务端架构分层设计

### 1. 领域层 (Domain)

- **`agent.rs`**: 定义 `Agent` 核心实体，并提供可见性校验、槽位数量控制以及策略配置合法性检查等纯函数规则。
- **`agent_snapshot.rs`**: 负责将 `Agent` 状态冻结并转换成不可变的快照属性。
- 参见：[agent.rs](/crates/lol_web_server/src/domain/agent.rs) 和 [agent_snapshot.rs](/crates/lol_web_server/src/domain/agent_snapshot.rs)

### 2. 服务层 (Service)

- **`agent_service.rs`**: 编排 Agent CRUD 操作。在创建前依赖 `SubscriptionService` 校验选手的槽位限额。
- **`agent_snapshot_service.rs`**: 负责生成 ELO 主体专用的 `v1/v2/v3...` 快照版本。
- **`community_service.rs`**: 实现公开选手的浏览、Fork 副本创建以及拉取上游合并的接口逻辑。
- 参见：[agent_service.rs](/crates/lol_web_server/src/service/agent_service.rs)

---

## 三、对局运行时决策驱动设计 (Bevy 引擎侧)

在 Bevy 游戏引擎运行时，针对 LLM, RL, Script 三种不同类型的 Agent，系统设计了统一的驱动分发层：

```
                    ┌─────────────────┐
                    │   AgentDriver   │ (驱动 Trait)
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
  │  LlmDriver  │     │  RlDriver   │     │ScriptDriver │
  └─────────────┘     └─────────────┘     └─────────────┘
```

1. **`AgentDriver` Trait**:
   - 规定了 `observe()` (接收游戏局势观测) 和 `action()` (下发 ECS 动作) 的契约。
2. **`LlmDriver`**:
   - 现有逻辑。通过 WebSocket 将观测数据发送给 LLM 执行器，并将推理的动作解析汇入 Bevy ECS 行动队列。
3. **`RlDriver`**:
   - 适配 Gym 环境接口 (`MoonLoLEnv`)。支持高频的 obs/action 二进制或 msgpack 张量传递，支持配置 Reward Shaper 解析权重对局内评分。
4. **`ScriptDriver`**:
   - 在 Rust 侧嵌入 JS 运行时（如 `rquickjs`）。运行于沙盒环境，拦截文件和网络 I/O，限制单 tick 的 CPU 执行时长以防止对局崩溃。
   - 参见 Bevy 引擎侧的系统驱动模块：[systems.rs](/crates/lol_agent/src/systems.rs)

---

## 四、前端选手管理模块

- **`heroes.vue`**: 选手的详情与管理页面。包含选手卡片网格、属性编辑（模型、Prompt、Preamble）、Monaco 脚本编辑器、快照发布控制及版本时间线。
- **LLM 模型配置为级联下拉**：模型字段不再手填，而是「模型供应商 + 模型名」两个下拉。供应商下拉列出设置页所有启用的供应商（按平台模型 / 预设 / 自定义分组，附图标与启用圆点）；模型名下拉随供应商切换刷新，候选来自该供应商的 `models` 列表，并保留「手填」兜底入口以覆盖预设外的新模型，手填值回写进供应商的 `models` 以便复用。选中后写入 `agents.model` 与 `config_json.provider_id`，运行时由 LLM 执行器按 `provider_id` 解析 `baseUrl / apiKey / apiFormat`。供应商目录与设置页详见 [模型供应商与模型设置](../llm-provider-setting/arch.md)。
- Monaco 脚本编辑器集成了 `.d.ts` 类型声明文件，为 JS Agent 的 `observe()` 和 `action()` 等 API 提供代码高亮与自动补全。
- **数据 100% 存储于云端**：去除了所有本地离线降级和冲突同步机制。前端直接读写云端服务，免去了本地缓存镜像，也删除了离线同步状态机（`useAgentSyncMachine.ts`）与冲突解决弹窗（`SyncConflictDialog.vue`）。若用户未登录，请求将被拦截并自动唤起登录弹窗。
- **`rl-training.vue`**: RL 选手的遥测监控面板。包含训练实时数据波形、Reward 构成比饼图以及 Value 估计的可视化展示。
