# MoonLOL — 接口设计文档

> 配套 [PRODUCT.md](../PRODUCT.md) 的技术实现规范。本文档定义从数据模型到前后端契约的完整接口层，作为各 Phase 实现的契约依据。

## 目录

- [一、概述与分层](#一概述与分层)
- [二、数据模型](#二数据模型)
- [三、Web Server 接口](#三web-server-接口)
- [四、Bevy 协议升级](#四bevy-协议升级)
- [五、前端契约](#五前端契约)
- [六、测试策略](#六测试策略)
- [七、实现路线](#七实现路线)

---

## 一、概述与分层

### 1.1 服务端六层架构

```
┌──────────────────────────────────────────────────────────┐
│  handlers        HTTP/WS 薄层                              │
│                  参数解析 + 鉴权 → 调 service → 序列化        │
│                  (无业务逻辑，可轻量 HTTP 集成测试)            │
└────────────────────────┬─────────────────────────────────┘
                         │ 调用
┌────────────────────────┴─────────────────────────────────┐
│  service (interface)  业务编排 trait 层                    │
│  AgentService / RoomService / MatchService / RankService … │
│  ── 编排多个 repository / cache，跨子系统协作               │
│  ── 持有 Arc<dyn XxxRepo> + Arc<dyn XxxCache>             │
└────────────────────────┬─────────────────────────────────┘
                         │ 调用
          ┌──────────────┴──────────────┐
          │                             │
┌─────────┴──────────┐      ┌──────────┴───────────────┐
│  repository        │      │  cache                   │
│  (持久层 trait)     │      │  (缓存层 trait)           │
│  XxxRepo trait      │      │  XxxCache trait           │
│  ── SQL/IO 边界     │      │  ── 读多写少热点          │
│  ── impl: PgXxxRepo │      │  ── impl: MokaXxxCache    │
│     (持 PgPool)     │      │     NoopCache (测试默认)  │
└─────────┬──────────┘      └──────────┬───────────────┘
          │                            │
          │ 共享                       │ 共享
┌─────────┴────────────────────────────┴───────────────────┐
│  domain           领域模型 + 纯业务规则                     │
│                   ELO 计算 / 可见性判定 / 槽位限制校验等      │
│                   (纯函数，无 IO，单测覆盖)                 │
└──────────────────────────────────────────────────────────┘
                         │
┌────────────────────────┴─────────────────────────────────┐
│  infra            基础设施 (DB 连接池 / Moka 客户端 / 进程池) │
└──────────────────────────────────────────────────────────┘
```

**层间依赖规则**（强制，编译期可保证）：

| 层 | 可依赖 | 不可依赖 |
|---|---|---|
| `handlers` | service trait | repository/cache/infra |
| `service` | repository trait + cache trait + domain | handlers/infra |
| `repository` | domain + infra(PgPool) | service/cache/handlers |
| `cache` | domain + infra(Moka) | service/repository/handlers |
| `domain` | （仅 std + serde） | 任何其他层 |
| `infra` | （基础设施库） | 任何业务层 |

**关键约束**：
- **依赖指向 trait，不指向 impl**：service 持 `Arc<dyn AgentRepo>`，不持 `Arc<PgAgentRepo>`。impl 在 `main.rs` 装配。
- **domain 是纯函数库**：ELO 计算、可见性判定、槽位限制等业务规则放 domain，输入领域类型、输出领域类型，零 IO，可单测覆盖到 100%。
- **repository 是 IO 边界**：trait 方法对应数据访问语义（`find_agent_by_id`、`save_match_participants`），impl 翻译为 SQL。一个表族一个 repo trait。
- **cache 是可选旁路**：service 调 cache 先查、miss 再查 repo 并回填。cache 层永远有一个 `NoopCache` 实现作为测试默认。

### 1.2 各子系统分层清单

每个子系统按上述六层拆开，下表列出全部子系统的 trait 清单（每个 trait = 一个可 mock 边界）：

| 子系统 | domain 模块 | repository trait | cache trait | service trait |
|---|---|---|---|---|
| Auth | `auth` | `UserRepo` | — | `UserService` |
| 大脑（策略配置） | `agent_config` | `AgentConfigRepo` | `AgentConfigCache` | `AgentConfigService` |
| Agent（选手） | `agent` | `AgentRepo` | `AgentCache` | `AgentService` |
| Agent 快照 | `agent_snapshot` | `AgentSnapshotRepo` | — | `AgentSnapshotService` |
| 出生点预设 | `spawn_preset` | `SpawnPresetRepo` | — | `SpawnPresetService` |
| 场景 | `scenario` | `ScenarioRepo` | — | `ScenarioService` |
| 房间 | `room` | `RoomRepo` | — | `RoomService` |
| 对局实例 | `match_` | `MatchRepo` + `MatchEventRepo` | — | `MatchService` |
| 本地对局 | `local_game` | — | — | `LocalGameService` |
| Rank 队列 | `rank` | `RankQueueRepo` | — | `RankService` |
| ELO/赛季 | `elo`、`season` | `EloRepo`、`SeasonRepo` | `LeaderboardCache` | `LeaderboardService` |
| 精粹 | `essence` | `EssenceRepo` | — | `EssenceService` |
| 订阅 | `subscription` | `SubscriptionRepo` | `BillingPlanCache` | `SubscriptionService` |
| 社区 | `community` | (复用 AgentRepo) | `CommunityCache` | `CommunityService` |
| 管理后台 | `admin` | (复用 MatchRepo) | — | `AdminService` |
| WS 代理 | `ws_proxy` | — | — | `WsProxyService` |
| 配置 | `config` | `ConfigRepo` | — | `ConfigService` |
| 日志 | `log` | `LogRepo` | — | `LogService` |

> 同一子系统内：service 编排 repo + cache + domain；repo 和 cache 只对接 infra。跨子系统协作（如 RankService 开局时要拍快照 + 托管 Bevy）走 service ↔ service 引用，**绝不**跨层直连别子的 repo。

### 1.3 设计原则

1. **统一对局实例**：三种对局形态（本地/房间/Rank）共用 `matches` 表，差异在 `form` 字段
2. **配置快照隔离**：对局进行中用 `agent_snapshots` 冻结配置，与编辑态分离
3. **操作流即观战**：所有观战/回放基于 `match_events` 操作流，客户端本地渲染
4. **鉴权统一**：HTTP 走 JWT Bearer，WS 走 query 参数（兼容浏览器无法自定义 WS header）
5. **依赖倒置**：上层定义并依赖下层 trait，impl 在装配期注入；任何一层可被 mock 替换
6. **domain 纯净**：业务规则集中在 domain 层纯函数，不接触 IO，保证可测性
7. **热点只读缓存**：缓存仅覆盖读多写少、强一致性要求低的数据（排行榜、赛季、模式定义、用户公开资料）；余额、队列状态、对局进行态不缓存

### 1.3 全局约定

| 项 | 约定 |
|---|---|
| API 前缀 | `/api` |
| 请求/响应格式 | JSON（`Content-Type: application/json`） |
| 时间戳 | ISO 8601 字符串（UTC，如 `2026-06-22T10:00:00Z`） |
| ID 形态 | UUID v4（字符串） |
| 错误模型 | `{ "error": { "code": "STRING", "message": "中文描述", "details"?: any } }`，HTTP 状态码语义化 |
| 分页 | `?offset=0&limit=20`，响应 `{ items: [...], total: number }` |
| 枚举值 | snake_case 字符串（如 `visibility=private`、`form=local`） |
| 字段命名 | 后端 snake_case，前端 TS 镜像保持 snake_case（与 Rust struct 对齐，避免转换层） |

### 1.4 鉴权策略

| 通道 | 鉴权方式 |
|---|---|
| HTTP `/api/*`（除 `auth/*`） | `Authorization: Bearer <jwt>` header |
| WebSocket `/api/ws/:match_id` | `?token=<jwt>&role=controller\|spectator` query 参数 |
| Tauri IPC（desktop 本地） | 无需 token（进程内调用） |

JWT claims（沿用现有）：
```json
{ "user_id": 123, "exp": 1719500000 }
```
有效期 30 天，密钥由 `JWT_SECRET` 环境变量控制。

---

## 二、数据模型

### 2.1 表结构总览

```
── 用户与配置（保留）──
users                       用户账号
ai_config                   BYO 模型配置

── Agent 资产三件套（重构）──
spawn_presets               出生点预设
agent_configs               「大脑」（原 agent_presets 演化，曾称 Agent 配置）
agents                      「选手」= 英雄 + 大脑（替代 hero_presets，本地/房间对局中可额外关联出生点）

── Agent 资产扩展 ──
agent_snapshots             参赛快照（Rank 专用，§2.5）
agent_fork_relations        Fork 关系图谱

── 对局编排（保留）──
scenarios                   场景预设（原 custom_scenarios 重命名）
scenario_win_conditions     胜利条件

── 房间子系统（新建）──
rooms                       房间元数据 + 房主约束
room_members                房间成员
room_agent_slots            房间内 Agent 槽位
room_invites                邀请码

── 对局实例（新建，核心）──
matches                     对局实例（跨三种形态统一）
match_participants          对局参与者（快照 + 阵营 + 结果）
match_events                操作流事件（观战/回放素材）

── Rank 子系统（新建）──
seasons                     赛季
rank_queues                 匹配队列状态
elo_ratings                 ELO 评分（Agent × 模式 × 赛季）

── 计费/订阅（新建）──
essence_balances            精粹余额
essence_transactions        精粹流水
subscriptions               订阅记录
billing_plans               订阅档位定义
```

### 2.2 表定义详述

#### 2.2.1 users（保留）

```sql
CREATE TABLE users (
    id           SERIAL PRIMARY KEY,
    phone        VARCHAR(20) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at   TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

#### 2.2.2 ai_config（保留）

```sql
CREATE TABLE ai_config (
    user_id  INT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    api_key  TEXT NOT NULL DEFAULT '',
    base_url TEXT NOT NULL DEFAULT '',
    preamble TEXT NOT NULL DEFAULT ''
);
```

#### 2.2.3 spawn_presets（保留 + 加字段）

```sql
CREATE TABLE spawn_presets (
    id         UUID PRIMARY KEY,
    owner_id   INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    x          REAL NOT NULL,
    z          REAL NOT NULL,
    team       TEXT NOT NULL,          -- 'order' | 'chaos'
    visibility TEXT NOT NULL DEFAULT 'private',  -- 'private'|'friends'|'public'
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);
```

#### 2.2.4 agent_configs（原 agent_presets 演化，「大脑」）

> 语义：英雄无关的"策略大脑"（大脑配置）——含 Agent 类型、Prompt、模型/权重等。可被多个 Agent 引用。

```sql
CREATE TABLE agent_configs (
    id           UUID PRIMARY KEY,
    owner_id     INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    agent_type   TEXT NOT NULL,        -- 'llm' | 'rl' | 'script'
    prompt       TEXT NOT NULL DEFAULT '',
    preamble     TEXT NOT NULL DEFAULT '',
    model        TEXT NOT NULL DEFAULT '',         -- LLM 模型名 / RL 权重路径 / 脚本标识
    config_json  JSONB NOT NULL DEFAULT '{}',      -- 类型专属扩展配置（如 RL 推理端点、脚本内容）
    visibility   TEXT NOT NULL DEFAULT 'private',
    forked_from  UUID NULL REFERENCES agent_configs(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);
```

**config_json 按类型的字段约定**：

| agent_type | config_json 内容 |
|---|---|
| `llm` | `{ thinking_depth?: int, tools?: [...] }` |
| `rl` | `{ inference_endpoint: string, model_version: string }` |
| `script` | `{ script_body: string, language: "javascript" }` |

#### 2.2.5 agents（替代 hero_presets，「选手」）

> 语义：一个 Agent = 英雄 + 大脑，是参赛/ELO/排行榜的主体。在本地/房间对局中，Agent 可以额外关联一个出生点预设。

```sql
CREATE TABLE agents (
    id               UUID PRIMARY KEY,
    owner_id         INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name             TEXT NOT NULL,            -- 「锐雯 · 激进压制」
    champion         TEXT NOT NULL,            -- 'Riven' | 'Fiora' | ...
    agent_config_id  UUID NOT NULL REFERENCES agent_configs(id) ON DELETE RESTRICT,
    spawn_preset_id  UUID NULL REFERENCES spawn_presets(id) ON DELETE SET NULL,
                                                   -- NULL：Rank 模式由规则覆盖
    visibility       TEXT NOT NULL DEFAULT 'private',
    forked_from      UUID NULL REFERENCES agents(id) ON DELETE SET NULL,
                                                   -- 若为 Fork 别人来的，指向原 Agent
    upstream_agent_id UUID NULL REFERENCES agents(id) ON DELETE SET NULL,
                                                   -- Fork 的"上游"（拉取更新的来源）
    created_at       TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);
```

**Fork 关系**：
- 用户 A 把公开 Agent X Fork 成自己的 Agent Y → `Y.forked_from = X.id`，`Y.upstream_agent_id = X.id`
- 后续 X 有更新，A 在 Y 上点"拉取上游" → 把 X 的当前配置拷贝到 Y 的关联 agent_config（创建新 agent_config 版本或覆盖）
- Fork 链可多层，但 `upstream_agent_id` 始终指向"可拉取更新的源"

#### 2.2.6 agent_snapshots（参赛快照，Rank 专用）

> 语义：Agent 在某时刻配置的不可变冻结版本。Rank 队列只消费快照。

```sql
CREATE TABLE agent_snapshots (
    id            UUID PRIMARY KEY,
    agent_id      UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    version       INT NOT NULL,                -- 1, 2, 3, ...
    config_freeze JSONB NOT NULL,
                  -- {
                  --   champion, agent_config_snapshot: {...完整 agent_configs 行...},
                  --   spawn_snapshot: {...} | null,
                  --   win_condition: {...} | null
                  -- }
    published_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (agent_id, version)
);
```

#### 2.2.7 scenarios / scenario_win_conditions（保留）

```sql
CREATE TABLE scenarios (
    id         UUID PRIMARY KEY,
    owner_id   INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    agents     JSONB NOT NULL,          -- 完整阵容编排（沿用 FrontAgentConfig 数组）
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);

CREATE TABLE scenario_win_conditions (
    owner_id   INT NOT NULL,
    scenario_id UUID NOT NULL REFERENCES scenarios(id) ON DELETE CASCADE,
    condition  JSONB NOT NULL,
    PRIMARY KEY (owner_id, scenario_id)
);
```

#### 2.2.8 rooms（房间元数据）

```sql
CREATE TABLE rooms (
    id                     UUID PRIMARY KEY,
    owner_id               INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name                   TEXT NOT NULL,
    invite_code            TEXT UNIQUE NOT NULL,     -- 6 位邀请码，用于 join-by-code
    max_members            INT NOT NULL DEFAULT 10,
    max_agents_per_member  INT NOT NULL DEFAULT 3,
    team_policy            TEXT NOT NULL DEFAULT 'free',  -- 'single_team' | 'free'
    lobby_visible          BOOLEAN NOT NULL DEFAULT TRUE,
    prompt_visible         BOOLEAN NOT NULL DEFAULT FALSE, -- 他人能否查看 Prompt/模型配置
    status                 TEXT NOT NULL DEFAULT 'lobby', -- 'lobby'|'running'|'closed'
    created_at             TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

#### 2.2.9 room_members

```sql
CREATE TABLE room_members (
    room_id    UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id    INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_ready   BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (room_id, user_id)
);
```

#### 2.2.10 room_agent_slots

```sql
CREATE TABLE room_agent_slots (
    id         UUID PRIMARY KEY,
    room_id    UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id    INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,  -- 槽位归属成员
    agent_id   UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    team       TEXT NOT NULL,            -- 'order' | 'chaos'，成员自由选
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

#### 2.2.11 matches（对局实例，统一三形态——核心表）

```sql
CREATE TABLE matches (
    id               UUID PRIMARY KEY,
    form             TEXT NOT NULL,        -- 'local' | 'room' | 'rank'
    room_id          UUID NULL REFERENCES rooms(id) ON DELETE SET NULL,
    rank_queue_id    UUID NULL,            -- 指向 rank_queues（见下）
    owner_id         INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                              -- 本地：发起者；房间：房主；Rank：系统
    mode             TEXT NOT NULL,        -- 'top_solo' | 'mid_solo' | ...（Rank 用）
    scenario_id      UUID NULL REFERENCES scenarios(id) ON DELETE SET NULL,
    win_condition    JSONB NULL,           -- 本局胜利条件（可来自 scenario 或 Rank 模式规则）
    status           TEXT NOT NULL DEFAULT 'pending',
                                             -- 'pending'|'running'|'paused'|'finished'|'aborted'
    bevy_port        INT NULL,             -- 分配的 Bevy 监听端口（端口池管理）
    ws_port          INT NULL,             -- 暴露给客户端的 WS 端口（通常 = bevy_port）
    started_at       TIMESTAMPTZ NULL,
    finished_at      TIMESTAMPTZ NULL,
    winner_team      TEXT NULL,            -- 'order'|'chaos'|'none'（none：中止或平局）
    abort_reason     TEXT NULL,            -- 中止原因（如 'agent_timeout'、'manual'）
    created_at       TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_matches_owner ON matches(owner_id, created_at DESC);
CREATE INDEX idx_matches_room ON matches(room_id) WHERE room_id IS NOT NULL;
CREATE INDEX idx_matches_status ON matches(status);
```

#### 2.2.12 match_participants

```sql
CREATE TABLE match_participants (
    id                UUID PRIMARY KEY,
    match_id          UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    agent_snapshot_id UUID NOT NULL REFERENCES agent_snapshots(id) ON DELETE RESTRICT,
    agent_id          UUID NOT NULL REFERENCES agents(id) ON DELETE RESTRICT,
                                              -- 冗余字段，便于排行榜查询
    user_id           INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    team              TEXT NOT NULL,            -- 'order' | 'chaos'
    bevy_entity_id    BIGINT NULL,              -- Bevy 侧实体 ID（运行时回填）
    result            TEXT NULL,                -- 'win'|'loss'|'draw'|'none'
    final_stats       JSONB NULL,               -- { kills, deaths, assists, minion_kills, gold, ... }
    UNIQUE (match_id, agent_snapshot_id)
);

CREATE INDEX idx_match_part_match ON match_participants(match_id);
CREATE INDEX idx_match_part_agent ON match_participants(agent_id);
```

#### 2.2.13 match_events（操作流，观战/回放素材）

```sql
CREATE TABLE match_events (
    id           BIGSERIAL PRIMARY KEY,
    match_id     UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    seq          INT NOT NULL,                -- 局内事件序号（从 0 递增）
    event_type   TEXT NOT NULL,               -- 'game_started'|'game_ended'|'action_executed'|
                                              -- 'kill'|'turret_destroyed'|'agent_status'
    agent_id     UUID NULL,                   -- 关联的 Agent（部分事件无）
    payload      JSONB NOT NULL,              -- 事件负载（见 §4.1 各类型 schema）
    game_time_ms BIGINT NOT NULL,             -- 游戏内时间戳（毫秒）
    occurred_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (match_id, seq)
);

CREATE INDEX idx_match_events_match ON match_events(match_id, seq);
```

#### 2.2.14 seasons

```sql
CREATE TABLE seasons (
    id         UUID PRIMARY KEY,
    name       TEXT NOT NULL,                 -- '2026 夏季赛'
    mode       TEXT NOT NULL,                 -- 'top_solo'
    starts_at  TIMESTAMPTZ NOT NULL,
    ends_at    TIMESTAMPTZ NOT NULL,
    status     TEXT NOT NULL DEFAULT 'scheduled',  -- 'scheduled'|'active'|'concluded'
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_seasons_mode_status ON seasons(mode, status);
```

#### 2.2.15 rank_queues（匹配队列状态）

```sql
CREATE TABLE rank_queues (
    id                UUID PRIMARY KEY,
    agent_id          UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    agent_snapshot_id UUID NOT NULL REFERENCES agent_snapshots(id) ON DELETE CASCADE,
    user_id           INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mode              TEXT NOT NULL,          -- 'top_solo'
    season_id         UUID NOT NULL REFERENCES seasons(id) ON DELETE RESTRICT,
    status            TEXT NOT NULL DEFAULT 'queued',  -- 'queued'|'matching'|'in_match'|'paused'|'removed'
    enqueued_at       TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_match_at     TIMESTAMPTZ NULL,
    UNIQUE (agent_snapshot_id, season_id)     -- 同一快照同赛季只能排一次
);

CREATE INDEX idx_rank_queue_status ON rank_queues(mode, status);
```

#### 2.2.16 elo_ratings

```sql
CREATE TABLE elo_ratings (
    id         UUID PRIMARY KEY,
    agent_id   UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    mode       TEXT NOT NULL,
    season_id  UUID NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    rating     DOUBLE PRECISION NOT NULL DEFAULT 1200,
    wins       INT NOT NULL DEFAULT 0,
    losses     INT NOT NULL DEFAULT 0,
    draws      INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (agent_id, mode, season_id)
);

CREATE INDEX idx_elo_leaderboard ON elo_ratings(mode, season_id, rating DESC);
```

**ELO 计算**：
- 初始值 `1200`，K 因子 `32`
- 胜负更新：`R_new = R_old + K * (S - E)`，其中 S 为实际得分（胜=1/平=0.5/负=0），E 为预期胜率
- 预期胜率：`E = 1 / (1 + 10^((R_opp - R_self) / 400))`

#### 2.2.17 essence_balances / essence_transactions

```sql
CREATE TABLE essence_balances (
    user_id  INT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    amount   BIGINT NOT NULL DEFAULT 0,        -- 精粹数量（整数，最小单位）
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE essence_transactions (
    id           BIGSERIAL PRIMARY KEY,
    user_id      INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    delta        BIGINT NOT NULL,              -- 正为入账，负为消耗
    reason       TEXT NOT NULL,                -- 'checkin'|'recharge'|'token_deduction'|'agent_slot'
    reference    TEXT NULL,                    -- 关联 ID（如订单号、对局 ID）
    balance_after BIGINT NOT NULL,             -- 流水后余额（审计用）
    created_at   TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_essence_tx_user ON essence_transactions(user_id, created_at DESC);
```

#### 2.2.18 subscriptions / billing_plans

```sql
CREATE TABLE billing_plans (
    id            TEXT PRIMARY KEY,            -- 'free' | 'pro' | 'elite'
    name          TEXT NOT NULL,
    price_cents   INT NOT NULL,                -- 月费（分）
    essence_per_month BIGINT NOT NULL,         -- 每月发放精粹
    max_agents    INT NOT NULL,                -- Agent 槽位上限
    features_json JSONB NOT NULL DEFAULT '{}', -- 其他增值特性
    sort_order    INT NOT NULL DEFAULT 0
);

CREATE TABLE subscriptions (
    id              UUID PRIMARY KEY,
    user_id         INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id         TEXT NOT NULL REFERENCES billing_plans(id),
    status          TEXT NOT NULL,             -- 'active'|'expired'|'cancelled'
    period_start    TIMESTAMPTZ NOT NULL,
    period_end      TIMESTAMPTZ NOT NULL,
    auto_renew      BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_subscriptions_user ON subscriptions(user_id, status);
```

### 2.3 数据模型与 PRODUCT.md 的映射

| PRODUCT.md 章节 | 主要表 |
|---|---|
| §2.1 Agent（英雄+配置+出生点） | `agents` + `agent_configs` + `spawn_presets` |
| §2.4 可见性/Fork | `agents.visibility` + `agents.forked_from/upstream_agent_id` |
| §2.5 参赛快照 | `agent_snapshots` |
| §3.A 本地对局 | `matches (form='local')` |
| §3.B 房间 | `rooms` + `room_members` + `room_agent_slots` + `matches (form='room')` |
| §3.C Rank | `seasons` + `rank_queues` + `elo_ratings` + `matches (form='rank')` |
| §5 观战/回放 | `match_events` |
| §6 精粹/订阅 | `essence_*` + `subscriptions` + `billing_plans` |

---

## 三、Web Server 接口

### 3.1 错误模型

所有非 2xx 响应统一格式：

```json
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent 不存在或无权访问",
    "details": { "agent_id": "..." }
  }
}
```

常用错误码：

| code | HTTP | 说明 |
|---|---|---|
| `UNAUTHORIZED` | 401 | 未登录或 token 失效 |
| `FORBIDDEN` | 403 | 无权操作该资源 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `VALIDATION_FAILED` | 400 | 请求参数校验失败 |
| `CONFLICT` | 409 | 状态冲突（如房间已开始、Agent 超槽位上限） |
| `RATE_LIMITED` | 429 | 触发限流 |
| `AGENT_SLOT_LIMIT` | 402 | Agent 槽位达上限（需订阅/购买） |
| `INSUFFICIENT_ESSENCE` | 402 | 精粹不足 |
| `INTERNAL` | 500 | 服务器内部错误 |

### 3.2 Auth 子系统

**保留现有接口**：

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/auth/register` | 注册（phone/password/code，code 固定 111111 直至接入短信） |
| POST | `/api/auth/login` | 登录，返回 `{token, user}` |
| POST | `/api/auth/reset-password` | 重置密码 |
| GET | `/api/auth/me` | **新增**：当前用户信息 + 订阅状态 + 精粹余额 |

**`GET /api/auth/me` 响应**：
```json
{
  "user": { "id": 123, "phone": "138****1234" },
  "subscription": { "plan_id": "pro", "period_end": "..." } | null,
  "essence": { "amount": 15000 },
  "agent_count": 3,
  "agent_limit": 10
}
```

### 3.3 Agent 资产子系统

#### 3.3.1 大脑（"策略大脑"，原 Agent 配置）

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/agent-configs` | 列出我的配置（`?agent_type=` 可选过滤） |
| POST | `/api/agent-configs` | 创建配置 |
| GET | `/api/agent-configs/:id` | 详情（按可见性校验访问） |
| PUT | `/api/agent-configs/:id` | 更新（仅 owner） |
| DELETE | `/api/agent-configs/:id` | 删除（被 Agent 引用时 RESTRICT） |

**POST/PUT 请求体**：
```json
{
  "name": "激进压制",
  "agent_type": "llm",
  "prompt": "你是一个上单锐雯...",
  "preamble": "全局策略：...",
  "model": "claude-sonnet-4",
  "config_json": { "thinking_depth": 2, "tools": [] },
  "visibility": "private"
}
```

#### 3.3.2 Agent（"选手"）

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/agents` | 列出我的 Agent（`?visibility=&forked_only=`） |
| POST | `/api/agents` | 创建 Agent（绑定 champion + config + spawn） |
| GET | `/api/agents/:id` | 详情 |
| PUT | `/api/agents/:id` | 更新（改 champion/config/spawn/visibility） |
| DELETE | `/api/agents/:id` | 删除 |
| PATCH | `/api/agents/:id/visibility` | 单独改可见性 |
| POST | `/api/agents/:id/publish` | **发布参赛快照**，返回新 snapshot |
| GET | `/api/agents/:id/snapshots` | 快照版本列表 |
| GET | `/api/agents/:id/elo` | ELO（`?mode=top_solo&season=<id>`） |

**POST `/api/agents` 请求体**：
```json
{
  "name": "锐雯 · 激进压制",
  "champion": "Riven",
  "agent_config_id": "<uuid>",
  "spawn_preset_id": "<uuid>",
  "visibility": "private"
}
```

**POST `/api/agents/:id/publish` 响应**：
```json
{
  "snapshot_id": "<uuid>",
  "agent_id": "<uuid>",
  "version": 3,
  "published_at": "2026-06-22T10:00:00Z"
}
```

**创建/发布约束**：
- 创建 Agent 前校验 `agent_count < agent_limit`（由订阅档位决定，免费=5）；否则返回 `AGENT_SLOT_LIMIT`（可购买扩槽）

#### 3.3.3 社区 Agent（可见性/Fork）

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/agents/community` | 浏览公开 Agent（`?champion=&sort=elo\|forks\|recent&page=`） |
| GET | `/api/agents/:id/full` | 完整详情（按可见性决定是否返回 prompt/model 等敏感字段） |
| POST | `/api/agents/:id/fork` | Fork 公开 Agent 为自己的副本 |
| POST | `/api/agents/:id/pull-upstream` | 从上游拉最新配置 |
| POST | `/api/agents/import-snapshot` | 引入社区分享的快照（§3.C.6） |

**Fork 请求**（POST `/api/agents/:id/fork`）：
```json
{ "new_name": "锐雯 · 激进压制 (我的副本)" }
```
响应：新创建的 Agent（含 `forked_from`、`upstream_agent_id` 指向原 Agent）。

**可见性 = 权限**：
- `private`：仅 owner
- `friends`：好友可见（好友系统 Phase 5 落地前等同 private）
- `public`：任何人可浏览基础信息；**敏感字段（prompt、model、config_json）只在 `friends` 或显式授权时返回**
- 房间内的可见性由房间的 `prompt_visible` 开关覆盖（§3.B.3）

### 3.4 出生点预设（保留）

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/spawn-presets` | 列出我的（`?team=` 可选） |
| POST | `/api/spawn-presets` | 创建 |
| PUT | `/api/spawn-presets/:id` | 更新 |
| DELETE | `/api/spawn-presets/:id` | 删除 |

### 3.5 Scenario（保留 + 重命名）

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/scenarios` | 列出我的 |
| POST | `/api/scenarios` | 创建（含 agents 阵容） |
| GET | `/api/scenarios/:id` | 详情 |
| PUT | `/api/scenarios/:id` | 更新 |
| DELETE | `/api/scenarios/:id` | 删除 |
| GET | `/api/scenarios/:id/win-condition` | 胜利条件 |
| PUT | `/api/scenarios/:id/win-condition` | 保存胜利条件 |

### 3.6 房间子系统

#### 3.6.1 房间生命周期

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/rooms` | 创建房间（含房主约束） |
| GET | `/api/rooms` | 我加入的房间列表 |
| GET | `/api/rooms/lobby` | 大厅公开房间列表（`lobby_visible=true` 的） |
| GET | `/api/rooms/:id` | 房间详情（成员/槽位/约束） |
| POST | `/api/rooms/join-by-code` | 凭邀请码加入 |
| POST | `/api/rooms/:id/join` | 直接加入（大厅路径） |
| POST | `/api/rooms/:id/leave` | 离开（房主离开 = 解散） |
| DELETE | `/api/rooms/:id` | 房主解散 |
| PATCH | `/api/rooms/:id` | 房主改约束 |
| POST | `/api/rooms/:id/kick` | 房主踢人 |

**POST `/api/rooms` 请求体**：
```json
{
  "name": "锐雯内战房",
  "max_members": 10,
  "max_agents_per_member": 3,
  "team_policy": "free",
  "lobby_visible": true,
  "prompt_visible": false
}
```
**响应**：房间详情 + `invite_code`（6 位）。

#### 3.6.2 房间内 Agent 槽位

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/rooms/:id/agents` | 成员添加 Agent 槽位（指定 team） |
| DELETE | `/api/rooms/:id/agents/:slot_id` | 移除槽位（房主或槽位拥有者） |
| PATCH | `/api/rooms/:id/agents/:slot_id` | 改 team |

**POST `/api/rooms/:id/agents` 请求体**：
```json
{ "agent_id": "<uuid>", "team": "order" }
```

**约束校验**：
- 房间状态必须为 `lobby`
- 成员槽位数 < `max_agents_per_member`
- 若 `team_policy='single_team'`，该成员已有槽位的 team 必须一致
- 全房间总槽位数 < `max_members`（按需调整语义）

#### 3.6.3 开局

**POST `/api/rooms/:id/start`**（仅房主）：
- 为每个槽位的 Agent 各拍一份**参赛快照**（冻结配置）
- 创建 `matches` 记录（`form='room'`）
- MatchService 从端口池分配端口，启动 Bevy 子进程（§4.5）
- 写入 `match_participants`（snapshot + team）
- 房间状态 → `running`
- 响应：
```json
{ "match_id": "<uuid>", "ws_port": 9142 }
```

### 3.7 对局实例 matches（统一三形态）

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/matches` | 列表（`?form=&status=&room_id=&offset=&limit=`） |
| GET | `/api/matches/:id` | 详情（含 participants/stats） |
| GET | `/api/matches/:id/events` | 操作流事件（`?from_seq=&limit=&type=`） |
| GET | `/api/matches/:id/events/download` | 下载完整事件流（SQLite 文件，§PRODUCT 5.2） |
| POST | `/api/matches/:id/stop` | 停止/中止对局（本地/房主） |
| GET | `/api/matches/:id/logs` | 结构化日志查询（沿用 LogService，按 match_id 过滤） |

**`GET /api/matches/:id/events` 响应**（分页 + 增量拉取）：
```json
{
  "items": [
    {
      "seq": 0,
      "event_type": "game_started",
      "agent_id": null,
      "payload": { "mode": "top_solo", "participants": [...] },
      "game_time_ms": 0
    },
    {
      "seq": 1,
      "event_type": "action_executed",
      "agent_id": "<uuid>",
      "payload": { "entity_id": 4294967185, "action": { "Move": [1200.0, 3400.0] } },
      "game_time_ms": 150
    }
  ],
  "total": 1024,
  "has_more": true
}
```

**观战拉取策略**：客户端首次拉 `from_seq=0`，之后用 `?from_seq=<last+1>` 轮询增量（或走 WS，§3.11）。

### 3.8 本地对局（desktop 专用）

> desktop 在本机跑 Bevy，不走服务器托管，但仍记录到 `matches` 表（`form='local'`）以便历史回看。

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/local/start` | 本机启动 Bevy，返回 match_id + ws_port |
| POST | `/api/local/stop` | 停止本机 Bevy |
| GET | `/api/local/status` | 当前本地对局状态 |

**POST `/api/local/start` 请求体**：
```json
{
  "mode": "custom",                    // 或 top_solo
  "scenario_id": "<uuid>" | null,
  "agents": [                          // 内联阵容（不强制走 Agent 资产）
    { "agent_id": "<uuid>", "team": "order" }
  ],
  "win_condition": { ... } | null
}
```
**响应**：
```json
{ "match_id": "<uuid>", "ws_port": 9100 }
```

> desktop 通过 Tauri IPC 时此接口实际不经过 HTTP，而是 Tauri command（详见 §5.1）。Web Server 实现用于 web 端调试。

### 3.9 Rank 子系统

#### 3.9.1 模式与赛季

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/rank/modes` | 可用模式列表（初期仅 `top_solo`） |
| GET | `/api/rank/seasons` | 赛季列表 |
| GET | `/api/rank/seasons/current?mode=top_solo` | 当前赛季 |

**`GET /api/rank/modes` 响应**：
```json
{
  "items": [
    {
      "mode": "top_solo",
      "name": "上单 SOLO",
      "description": "上路 1v1，先达成一血/一塔/100 补即胜",
      "win_condition_template": {
        "or": [
          { "kills": { ">=": 1 } },
          { "turret_destroyed": { "lane": "top", "tier": 1 } },
          { "minion_kills": { ">=": 100 } }
        ]
      },
      "players_per_match": 2
    }
  ]
}
```

#### 3.9.2 匹配队列

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/api/rank/queue` | Agent 入队（需已 publish 快照） |
| DELETE | `/api/rank/queue/:agent_id` | Agent 出队 |
| GET | `/api/rank/queue/status` | 我所有 Agent 的队列状态 |

**POST `/api/rank/queue` 请求体**：
```json
{
  "agent_id": "<uuid>",
  "snapshot_id": "<uuid>",          // 指定参赛的快照版本
  "mode": "top_solo"
}
```
**响应**：
```json
{
  "queue_id": "<uuid>",
  "status": "queued",
  "estimated_wait_seconds": 30
}
```

**队列约束**：
- 必须先 publish 快照（§3.3.2）
- 同一 agent 同一赛季只能排一次（UNIQUE 约束）
- 入队时若匹配池内有 ELO 接近的对手，立即配对 → 创建 match

#### 3.9.3 排行榜

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/rank/leaderboard` | 排行榜 |

**参数**：
- `mode=top_solo`（必填）
- `scope=total|daily`（默认 total）
- `season=<id>`（默认当前赛季）
- `offset=0&limit=50`

**响应**（total scope）：
```json
{
  "items": [
    {
      "rank": 1,
      "agent_id": "<uuid>",
      "agent_name": "锐雯 · 激进压制",
      "owner": { "id": 123, "name": "玩家A" },
      "champion": "Riven",
      "rating": 2345,
      "wins": 120,
      "losses": 30,
      "win_rate": 0.8
    }
  ],
  "total": 1024,
  "scope": "total",
  "season": { "id": "...", "name": "2026 夏季赛" }
}
```

**日榜**：按当日（UTC+8 0 点重置）新打的对局 ELO 增量排序，从 `match_participants` + `elo_ratings` 联合查询。

### 3.10 精粹与订阅

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/essence/balance` | 余额 |
| GET | `/api/essence/transactions` | 流水（`?offset=&limit=`） |
| POST | `/api/essence/check-in` | 每日签到（幂等：同一天重复请求返回已签到） |
| POST | `/api/essence/redeem` | 兑换码兑换 |
| POST | `/api/essence/recharge` | 充值（接入支付后，回调确认） |
| GET | `/api/subscriptions` | 我的订阅 |
| POST | `/api/subscriptions/subscribe` | 订阅/升级（`{ plan_id, period: 'month'\|'year' }`） |
| GET | `/api/billing/plans` | 档位列表 |

**精粹用途与计费**：
- 平台模型 Token 消耗：每局结束后按实际 Token 数扣精粹
- 默认汇率（Phase 4 再定）：`1 元 = 100 精粹`，`1 精粹 ≈ 1000 token`
- Agent 槽位扩展：花精粹购买额外槽（数量与价格见 `billing_plans.features_json`）
- BYO 模型不消耗精粹

### 3.11 WebSocket（统一观战/操控通道）

**路由**：`GET /api/ws/:match_id?token=<jwt>&role=controller|spectator&agent_id=<uuid>`

#### 握手流程

1. 服务端校验 `token`（JWT），失败 → 401 关闭
2. 校验 `match_id` 存在且 `status='running'`
3. `role` 校验：
   - `controller`：必须有 `agent_id` 且校验该用户在该 match 拥有该 agent（match_participants.user_id）
   - `spectator`：任意已登录用户可连；房间对局额外校验是否为该房间成员（若房主关闭旁观则拒绝）
4. 握手成功后，web server 内部 `connect_async` 到 `ws://127.0.0.1:<matches.bevy_port>`
5. **双向代理 + hello 注入**（见 §4.2）：web server 在转发前先注入 `hello` 帧

#### 转发策略

| 方向 | 处理 |
|---|---|
| client → Bevy | 校验 controller 身份与 entity_id 权限；spectator 的 action 帧丢弃并回 error |
| Bevy → client | 透传所有 event；controller 与 spectator 都能收 |

#### 帧格式（沿用 + 扩展）

见 §4 Bevy 协议升级。

### 3.12 管理后台（算力监控）

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/api/admin/matches/running` | 运行中 Bevy 实例 + 内存占用 |
| GET | `/api/admin/metrics` | 聚合指标（总对局数、并发数、平均时长） |
| POST | `/api/admin/matches/:id/abort` | 强制中止某局 |
| GET | `/api/admin/queues` | Rank 队列状态总览 |

> 管理后台鉴权：仅 `users.role='admin'`（users 表加 `role` 字段）。Phase 3 可先靠环境变量配置 admin user_id 列表。

### 3.13 分层装配（对应 §1.1 / §1.2）

§3.2–3.12 的每条路由背后，调用链都是 `handler → service → {repo, cache} + domain`。下表是每个子系统的四层装配一览（trait 列即 mock 边界）：

| 子系统 | domain 模块 | repository trait (impl) | cache trait (impl) | service trait (impl) |
|---|---|---|---|---|
| Auth | `auth` | `UserRepo` (`PgUserRepo`) | — | `UserService` (`UserServiceImpl`) |
| 配置 | `config` | `ConfigRepo` (`PgConfigRepo`) | — | `ConfigService` (`ConfigServiceImpl`) |
| 大脑（大脑配置） | `agent_config` | `AgentConfigRepo` (`PgAgentConfigRepo`) | `AgentConfigCache` (`MokaAgentConfigCache`/`Noop`) | `AgentConfigService` (`AgentConfigServiceImpl`) |
| Agent | `agent` | `AgentRepo` (`PgAgentRepo`) | `AgentCache` (`MokaAgentCache`/`Noop`) | `AgentService` (`AgentServiceImpl`) |
| Agent 快照 | `agent_snapshot` | `AgentSnapshotRepo` (`PgAgentSnapshotRepo`) | — | `AgentSnapshotService` (`AgentSnapshotServiceImpl`) |
| 出生点预设 | `spawn_preset` | `SpawnPresetRepo` (`PgSpawnPresetRepo`) | — | `SpawnPresetService` (`SpawnPresetServiceImpl`) |
| 场景 | `scenario` | `ScenarioRepo` (`PgScenarioRepo`) | — | `ScenarioService` (`ScenarioServiceImpl`) |
| 房间 | `room` | `RoomRepo` (`PgRoomRepo`) | — | `RoomService` (`RoomServiceImpl`) |
| 对局实例 | `match_` | `MatchRepo`+`MatchEventRepo` (`PgMatchRepo`/`PgMatchEventRepo`) | — | `MatchService` (`MatchServiceImpl`) |
| 本地对局 | `local_game` | — | — | `LocalGameService` (`LocalGameServiceImpl`) |
| Rank 队列 | `rank` | `RankQueueRepo` (`PgRankQueueRepo`) | — | `RankService` (`RankServiceImpl`) |
| ELO/赛季 | `elo`、`season` | `EloRepo`+`SeasonRepo` (`PgEloRepo`/`PgSeasonRepo`) | `LeaderboardCache` (`MokaLeaderboardCache`/`Noop`) | `LeaderboardService` (`LeaderboardServiceImpl`) |
| 精粹 | `essence` | `EssenceRepo` (`PgEssenceRepo`) | — | `EssenceService` (`EssenceServiceImpl`) |
| 订阅 | `subscription` | `SubscriptionRepo` (`PgSubscriptionRepo`) | `BillingPlanCache` (`MokaBillingPlanCache`/`Noop`) | `SubscriptionService` (`SubscriptionServiceImpl`) |
| 社区 | `community` | (复用 `AgentRepo`) | `CommunityCache` (`MokaCommunityCache`/`Noop`) | `CommunityService` (`CommunityServiceImpl`) |
| 管理后台 | `admin` | (复用 `MatchRepo`/`RankQueueRepo`) | — | `AdminService` (`AdminServiceImpl`) |
| WS 代理 | `ws_proxy` | — | — | `WsProxyService` (`WsProxyServiceImpl`) |
| 日志 | `log` | `LogRepo` (`SqliteLogRepo`) | — | `LogService` (`LogServiceImpl`) |

#### 3.13.1 模块划分（crate 内）

```
crates/lol_web_server/
├── src/
│   ├── main.rs              # 装配：构造各 impl，注入到 service/handler
│   ├── domain/              # 纯业务规则（无 IO）
│   │   ├── mod.rs
│   │   ├── agent.rs         # Agent 领域类型 + 可见性判定 + 槽位限制
│   │   ├── elo.rs           # ELO 计算（K=32 公式）
│   │   ├── room.rs          # 房间约束校验
│   │   ├── rank.rs          # 匹配配对算法
│   │   └── essence.rs       # 精粹扣减规则
│   ├── repository/          # 持久层 trait + impl
│   │   ├── mod.rs
│   │   ├── user_repo.rs         # trait UserRepo + struct PgUserRepo
│   │   ├── agent_repo.rs        # trait AgentRepo + struct PgAgentRepo
│   │   ├── match_repo.rs        # ...
│   │   └── ...
│   ├── cache/               # 缓存层 trait + impl
│   │   ├── mod.rs
│   │   ├── agent_cache.rs       # trait AgentCache + MokaAgentCache + NoopAgentCache
│   │   ├── leaderboard_cache.rs
│   │   └── ...
│   ├── service/             # 业务编排 trait + impl
│   │   ├── mod.rs
│   │   ├── agent_service.rs     # trait AgentService + struct AgentServiceImpl { repo, cache }
│   │   ├── room_service.rs
│   │   └── ...
│   ├── handlers.rs          # HTTP/WS 薄层（参数 → service → 序列化）
│   ├── models.rs            # API DTO（请求/响应 struct，与 domain 类型分离）
│   └── interfaces.rs        # 【deprecated】旧 trait，迁移后删除
└── tests/                   # 集成测试（testcontainers 真 PG）
    ├── agent_service_test.rs
    ├── room_service_test.rs
    └── ...
```

> **领域类型 vs DTO 分离**：domain 层定义 `Agent`/`Match`/`EloRating` 等纯领域类型；models.rs 定义 API 请求/响应 DTO（如 `CreateAgentRequest`、`AgentResponse`）。handler 做 domain ↔ DTO 转换，service 内部只流转 domain 类型。

#### 3.13.2 一个子系统的四层示例（Agent 资产）

下例展示单个子系统（Agent）从 domain 到 service 的完整分层，其他子系统照此模板：

```rust
// ── domain/agent.rs ── 纯业务规则，无 IO，单测覆盖 ──

pub struct Agent { /* 领域字段：id/owner_id/name/champion/visibility/forked_from/... */ }
pub enum Visibility { Private, Friends, Public }
pub enum AgentError { NotFound, Forbidden, SlotLimitExceeded { current: usize, limit: usize } }

/// 可见性判定（纯函数）
pub fn can_view(agent: &Agent, viewer_user_id: i32, is_friend: bool) -> bool { ... }

/// 槽位限制校验（纯函数）
pub fn assert_within_slot_limit(current_count: usize, limit: usize) -> Result<(), AgentError> { ... }

// ── repository/agent_repo.rs ── 持久层 trait + impl ──

#[async_trait]
pub trait AgentRepo: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<Agent>, RepoError>;
    async fn list_by_owner(&self, owner_id: i32, filter: AgentFilter) -> Result<Vec<Agent>, RepoError>;
    async fn count_by_owner(&self, owner_id: i32) -> Result<usize, RepoError>;
    async fn insert(&self, agent: Agent) -> Result<Agent, RepoError>;
    async fn update(&self, agent: Agent) -> Result<(), RepoError>;
    async fn delete(&self, id: &str) -> Result<(), RepoError>;
}

pub struct PgAgentRepo { pub pool: PgPool }   // impl 翻译为 SQL
#[async_trait]
impl AgentRepo for PgAgentRepo { ... }

// ── cache/agent_cache.rs ── 缓存层 trait + impl ──

#[async_trait]
pub trait AgentCache: Send + Sync {
    async fn get(&self, id: &str) -> Option<Agent>;
    async fn put(&self, agent: Agent);
    async fn invalidate(&self, id: &str);
}

pub struct MokaAgentCache { inner: moka::future::Cache<String, Agent> }
#[async_trait]
impl AgentCache for MokaAgentCache { ... }

pub struct NoopAgentCache;                     // 测试默认：不缓存
#[async_trait]
impl AgentCache for NoopAgentCache { /* 全 no-op */ }

// ── service/agent_service.rs ── 业务编排 trait + impl ──

#[async_trait]
pub trait AgentService: Send + Sync {
    async fn get_agent(&self, viewer: i32, id: &str) -> Result<AgentResponse, ServiceError>;
    async fn create_agent(&self, owner: i32, input: CreateAgentInput) -> Result<AgentResponse, ServiceError>;
    async fn list_my_agents(&self, owner: i32, filter: AgentFilter) -> Result<Vec<AgentResponse>, ServiceError>;
    // ... update/delete/publish/elo/fork ...
}

pub struct AgentServiceImpl {
    pub repo: Arc<dyn AgentRepo>,
    pub cache: Arc<dyn AgentCache>,
    pub config_repo: Arc<dyn AgentConfigRepo>,   // 跨子系统：校验 agent_config 存在
    pub spawn_repo: Arc<dyn SpawnPresetRepo>,
    pub snapshot_service: Arc<dyn AgentSnapshotService>,  // publish 时调
    pub essence_service: Arc<dyn EssenceService>,         // 扩槽时调
    pub subscription_service: Arc<dyn SubscriptionService>, // 取 agent_limit
}

#[async_trait]
impl AgentService for AgentServiceImpl {
    async fn get_agent(&self, viewer: i32, id: &str) -> Result<AgentResponse, ServiceError> {
        // 1. 查缓存
        if let Some(agent) = self.cache.get(id).await {
            if domain::can_view(&agent, viewer, self.is_friend(viewer, agent.owner_id).await?) {
                return Ok(AgentResponse::from(agent));
            }
            return Err(ServiceError::Forbidden);
        }
        // 2. miss → 查 repo
        let agent = self.repo.find_by_id(id).await?
            .ok_or(ServiceError::NotFound)?;
        self.cache.put(agent.clone()).await;   // 回填（注意：仅缓存公开 Agent）
        // 3. domain 规则判定
        if !domain::can_view(&agent, viewer, ...) { return Err(ServiceError::Forbidden); }
        Ok(AgentResponse::from(agent))
    }

    async fn create_agent(&self, owner: i32, input: CreateAgentInput) -> Result<AgentResponse, ServiceError> {
        // 1. 取订阅档位 → agent_limit
        let limit = self.subscription_service.get_agent_limit(owner).await?;
        // 2. 查当前数量
        let current = self.repo.count_by_owner(owner).await?;
        // 3. domain 规则校验（纯函数，可单测）
        domain::assert_within_slot_limit(current, limit)?;
        // 4. 校验引用的 agent_config/spawn 存在
        self.config_repo.find_by_id(&input.agent_config_id).await?
            .ok_or(ServiceError::InvalidAgentConfig)?;
        // 5. 落库
        let agent = self.repo.insert(Agent::new(owner, input)).await?;
        Ok(AgentResponse::from(agent))
    }
}
```

#### 3.13.3 装配（main.rs）

装配期决定 trait → impl 的绑定，业务代码完全感知不到具体实现：

```rust
// main.rs
let pool = PgPoolOptions::new()...connect(...).await?;
let moka = moka::future::Cache::builder().max_capacity(10_000).build();

// repository impl
let agent_repo: Arc<dyn AgentRepo> = Arc::new(PgAgentRepo { pool: pool.clone() });
let agent_config_repo: Arc<dyn AgentConfigRepo> = Arc::new(PgAgentConfigRepo { pool: pool.clone() });
// ... 其他 repo

// cache impl（生产用 Moka，测试用 Noop）
let agent_cache: Arc<dyn AgentCache> = Arc::new(MokaAgentCache { inner: moka.clone() });
let leaderboard_cache: Arc<dyn LeaderboardCache> = Arc::new(MokaLeaderboardCache { inner: moka.clone() });

// service impl（注入 repo + cache + 跨子系统 service）
let subscription_service: Arc<dyn SubscriptionService> = Arc::new(SubscriptionServiceImpl {
    repo: Arc::new(PgSubscriptionRepo { pool: pool.clone() }),
    billing_plan_cache: Arc::new(MokaBillingPlanCache { inner: moka.clone() }),
});
let agent_service: Arc<dyn AgentService> = Arc::new(AgentServiceImpl {
    repo: agent_repo, cache: agent_cache,
    config_repo: agent_config_repo, spawn_repo: spawn_preset_repo,
    snapshot_service: agent_snapshot_service,
    essence_service: essence_service,
    subscription_service: subscription_service.clone(),
});

let state = AppState { agent_service, room_service, match_service, ... };
```

> **循环依赖处理**：跨子系统 service 互相引用时（如 `AgentService.publish` 调 `AgentSnapshotService`，而 `AgentSnapshotService` 又要校验 Agent 存在），用 `Arc<dyn XxxService>` 注入；若编译期出现真环，用 lazy 初始化（`OnceLock`）或重构拆出第三个 service 承载共享逻辑。

#### 3.13.4 旧 interfaces.rs 的迁移路径

现有 `interfaces.rs` 里的 7 个 trait（`ConfigService`/`PresetService`/...）本质是 **service trait + repository trait 合一**，且 impl 直接持 `PgPool` 写 SQL。迁移分三步：

1. **拆分**：每个旧 trait 拆成 `XxxRepo`（持久）+ `XxxService`（编排）两个 trait
2. **提取 domain**：把 impl 里的业务规则（如槽位限制、可见性、ELO 公式）提到 `domain/` 纯函数
3. **删除**：迁移完成后删除 `interfaces.rs`，由 `repository/mod.rs` + `service/mod.rs` 取代

---

## 四、Bevy 协议升级

> 当前 Bevy WS 协议（`crates/lol_server/src/protocol.rs`）只支持调试命令 + 5 个事件，无连接身份、无操作流广播、无游戏结束事件。本节定义升级后的协议。

### 4.1 帧类型总览

沿用三种帧 + 新增能力：

| 帧类型 | 方向 | 结构 |
|---|---|---|
| `WsRequest` | client → Bevy | `{id, cmd, params}` |
| `WsResponse` | Bevy → client | `{id, type:"result", ok, data?, error?}` |
| `WsEvent` | Bevy → client | `{type:"event", event, data}` |

### 4.2 连接身份握手（hello）

升级 WS 后，**controller 连接的第一条消息必须是 hello**：

```json
{
  "id": 1,
  "cmd": "hello",
  "params": {
    "role": "controller",
    "agent_id": "<uuid>",
    "allowed_entity_ids": [4294967185],
    "match_id": "<uuid>"
  }
}
```

Bevy 侧为该连接记录：
```rust
struct Connection {
    role: Role,             // Controller { agent_id, allowed_entity_ids } | Spectator
}
```

**权限校验**：
- 后续 `action` 命令校验 `params.entity_id ∈ allowed_entity_ids`，否则回 `error: "entity not controlled by this connection"`
- spectator 连接发任何写命令（action/debug）都回 error
- hello 由 web server 在代理握手后**自动注入**（客户端不感知），携带从 match_participants 查到的映射

### 4.3 新增事件类型（protocol.rs 扩展）

```rust
impl WsEvent {
    // ── 对局生命周期 ──
    pub fn game_started(mode: &str, participants: &[ParticipantInfo]) -> Self;
    pub fn game_ended(winner_team: Option<&str>, reason: &str, stats: &[AgentStats]) -> Self;

    // ── 操作流（观战核心）──
    pub fn action_executed(agent_id: &str, entity_id: u64, action: &Action, game_time_ms: u64) -> Self;

    // ── 游戏内事件 ──
    pub fn kill(killer_agent_id: Option<&str>, victim_agent_id: &str, game_time_ms: u64) -> Self;
    pub fn turret_destroyed(team: &str, lane: &str, tier: u32, by_agent_id: Option<&str>, game_time_ms: u64) -> Self;
    pub fn inhibitor_destroyed(team: &str, lane: &str, game_time_ms: u64) -> Self;
    pub fn nexus_destroyed(team: &str, game_time_ms: u64) -> Self;

    // ── Agent 状态（异常可见性，§PRODUCT 3.B.6 / 3.C.4）──
    pub fn agent_status(agent_id: &str, status: &str, detail: Option<&str>) -> Self;
    // status: "ok" | "stalled" | "disconnected" | "recovered"
}
```

**事件 payload schema**：

```jsonc
// game_started
{
  "mode": "top_solo",
  "participants": [
    { "agent_id": "<uuid>", "entity_id": 4294967185, "team": "order", "champion": "Riven" }
  ]
}

// game_ended
{
  "winner_team": "order",           // null 表示中止无胜负
  "reason": "first_blood",          // 'first_blood'|'turret'|'cs_100'|'nexus'|'aborted'|'timeout'
  "stats": [
    { "agent_id": "<uuid>", "kills": 1, "deaths": 0, "assists": 0,
      "minion_kills": 45, "gold": 3200, "damage_dealt": 8200 }
  ]
}

// action_executed（关键：操作流）
{
  "agent_id": "<uuid>",
  "entity_id": 4294967185,
  "action": { "Move": [1200.0, 3400.0] },
  "game_time_ms": 150
}

// kill
{ "killer_agent_id": "<uuid>", "victim_agent_id": "<uuid>", "game_time_ms": 45200 }

// turret_destroyed
{ "team": "chaos", "lane": "top", "tier": 1, "by_agent_id": "<uuid>", "game_time_ms": 98000 }

// agent_status
{ "agent_id": "<uuid>", "status": "stalled", "detail": "no action for 30s" }
```

**Action 类型**（沿用 `lol_core::action::Action`，序列化为带标签的 enum）：
```jsonc
{ "Attack": 4294967290 }
{ "Move": [1200.0, 3400.0] }
{ "Stop": null }
{ "Skill": { "index": 0, "point": [1500.0, 3000.0] } }
{ "SkillLevelUp": 0 }
```

### 4.4 操作流广播实现

**触发点**：当 Bevy 处理一条 `action` 命令（`CommandAction` 事件被触发）后，新增一个 observer：
- 读取该 action 的来源 agent_id（从 Connection 上下文或 entity → agent_id 映射）
- 广播 `action_executed` 事件给**所有连接**（含 spectators）
- 同时 INSERT 一行到 `match_events`（由 web server 在代理层落库，避免 Bevy 直连 DB）

**架构选择**：Bevy 不直连 Postgres。事件流由 web server 的 WS 代理层做"嗅探落库"：
- Bevy → web server → client 的方向上，web server 解析每条 `WsEvent`，按 `event_type` 写入 `match_events`
- 这样 Bevy 保持纯粹的游戏引擎职责，DB 持久化由 web server 承担

### 4.5 胜利条件判定系统（新建）

**位置**：新 crate `lol_wincondition`，或 `lol_core` 内新模块。

**输入**：
- `win_condition: JSONB`（从 matches 表传入，Bevy 启动时作为资源配置）
- 游戏状态：击杀数、塔摧毁、补刀数（数据已存在于 `ObserveMyself`）

**实现**：
- 新增 Bevy 系统 `evaluate_win_condition`，每帧或事件触发时求值
- 求值引擎支持 §PRODUCT 4.5 的条件原子 + AND/OR/NOT 组合
- 命中条件 → 触发 `CommandGameEnd { winner_team, reason }` → 广播 `game_ended` 事件 → Bevy 停止游戏循环

**解决**：替代当前 `GAME_END_TIME=120s` 硬编码在 Tauri 客户端（`agent.rs:22`）的逻辑。

### 4.6 Bevy 进程生命周期（MatchService）

`MatchService`（替代 `GameServiceImpl`）职责：

| 职责 | 说明 |
|---|---|
| 端口池管理 | 维护 `[9100, 9200)` 端口池，分配/回收；`HashMap<match_id, (Child, port)>` |
| 启动 Bevy | 用端口池端口启动 `cargo run -- --ws-port <port> --match-id <uuid> --win-condition <json>` |
| 实体映射 | 启动后通过 `get_agents` 拉取 entity_id，回填 `match_participants.bevy_entity_id` |
| 事件落库 | WS 代理层嗅探 `WsEvent`，写入 `match_events` |
| 结束处理 | 监听 `game_ended` → 更新 `matches.status='finished'` + `winner_team` + `match_participants.result/final_stats` + 触发 ELO 更新（Rank） |
| 异常处理 | 子进程异常退出 → `matches.status='aborted'` + `abort_reason` |
| 心跳监控 | 周期检查子进程存活 + Bevy 内 `agent_status` stalled 检测（§PRODUCT 3.B.6 / 3.C.4） |
| 资源回收 | 结束/中止后释放端口、回收子进程 |

**启动参数扩展**（`src/main.rs` CLI）：
```
--ws-port <port>          # WS 监听端口
--match-id <uuid>         # 对局 ID（写入日志、便于关联）
--mode <mode>             # 对局模式
--scene <scene_uri>       # 场景文件
--win-condition <json>    # 胜利条件（JSON 字符串）
```

### 4.7 异常处理协议（§PRODUCT 3.B.6 / 3.C.4）

**房间模式**：
- web server 检测 controller 连接断开 → 广播 `agent_status: stalled`
- 房间对局进入 `paused` 状态，全员可见哪个 agent 异常
- 等待 controller 重连（30 秒宽限）或房主踢出

**Rank 模式**：
- 同样检测 → 广播 `agent_status: stalled` + 对局 `paused`
- 30 秒宽限内重连 → 继续
- 超时仍无动作 → `matches.status='aborted'`，`winner_team='none'`，双方回队列
- 频繁中止的 agent → `rank_queues.status='paused'`，待动力源恢复再 `queued`

**平台模型驱动的 Agent 不存在此问题**（web server 直接代为发 action）。

---

## 五、前端契约

### 5.1 IBackendClient 扩展点

`apps/client/src/services/backend.ts` 的 `IBackendClient` 接口需扩展。**新增方法分组**：

```ts
// ── Auth（当前缺失，必须补）──
login(phone: string, password: string): Promise<{ token: string; user: User }>
register(phone: string, password: string, code: string): Promise<{ token: string; user: User }>
logout(): Promise<void>
getMe(): Promise<MyProfile>

// ── Agent 资产 ──
listAgents(filter?: { visibility?: string; forkedOnly?: boolean }): Promise<Agent[]>
createAgent(input: CreateAgentInput): Promise<Agent>
getAgent(id: string): Promise<Agent>
updateAgent(id: string, input: UpdateAgentInput): Promise<Agent>
deleteAgent(id: string): Promise<void>
publishAgentSnapshot(agentId: string): Promise<AgentSnapshot>
listAgentSnapshots(agentId: string): Promise<AgentSnapshot[]>
getAgentElo(agentId: string, mode?: string): Promise<EloRating>

listAgentConfigs(filter?: { agentType?: string }): Promise<AgentConfig[]>
createAgentConfig(input: CreateAgentConfigInput): Promise<AgentConfig>
// ... CRUD

// ── 社区 ──
browseCommunityAgents(filter: CommunityFilter): Promise<PaginatedResult<Agent>>
forkAgent(agentId: string, newName: string): Promise<Agent>
pullUpstream(agentId: string): Promise<Agent>
importSnapshot(snapshotToken: string): Promise<Agent>

// ── 房间 ──
createRoom(input: CreateRoomInput): Promise<Room>
listMyRooms(): Promise<Room[]>
listLobbyRooms(): Promise<Room[]>
getRoom(id: string): Promise<RoomDetail>
joinRoomByCode(code: string): Promise<Room>
joinRoom(id: string): Promise<Room>
leaveRoom(id: string): Promise<void>
dissolveRoom(id: string): Promise<void>
updateRoomConstraints(id: string, input: Partial<RoomConstraints>): Promise<Room>
addRoomAgentSlot(roomId: string, agentId: string, team: Team): Promise<RoomAgentSlot>
removeRoomAgentSlot(roomId: string, slotId: string): Promise<void>
kickRoomMember(roomId: string, userId: number): Promise<void>
startRoomMatch(roomId: string): Promise<{ matchId: string; wsPort: number }>

// ── 对局实例 ──
listMatches(filter?: MatchFilter): Promise<PaginatedResult<MatchSummary>>
getMatch(id: string): Promise<MatchDetail>
getMatchEvents(id: string, fromSeq?: number, limit?: number): Promise<PaginatedResult<MatchEvent>>
stopMatch(id: string): Promise<void>

// ── 本地对局（desktop 通过 Tauri IPC，web 通过 HTTP）──
startLocalMatch(input: LocalStartInput): Promise<{ matchId: string; wsPort: number }>
stopLocalMatch(): Promise<void>

// ── Rank ──
listRankModes(): Promise<RankMode[]>
listSeasons(mode?: string): Promise<Season[]>
getCurrentSeason(mode: string): Promise<Season>
enqueueRank(input: EnqueueInput): Promise<RankQueueEntry>
dequeueRank(agentId: string): Promise<void>
getRankQueueStatus(): Promise<RankQueueEntry[]>
getLeaderboard(filter: LeaderboardFilter): Promise<PaginatedResult<LeaderboardEntry>>

// ── 精粹/订阅 ──
getEssenceBalance(): Promise<{ amount: number }>
listEssenceTransactions(filter?: Pagination): Promise<PaginatedResult<EssenceTransaction>>
checkInDaily(): Promise<{ awarded: number; newBalance: number }>
getMySubscription(): Promise<Subscription | null>
subscribe(planId: string, period: 'month' | 'year'): Promise<Subscription>
listBillingPlans(): Promise<BillingPlan[]>

// ── WebSocket（扩展，加 matchId + role）──
connectWs(matchId: string, role: 'controller' | 'spectator', agentId?: string): Promise<void>
disconnectWs(): Promise<void>
sendWsCmd(cmd: string, params?: Record<string, any>): Promise<any>
onWsEvent(callback: (event: WsEvent) => void): Promise<UnsubscribeFn>
```

**实现分层**：
- **Tauri 实现**（`tauriBackend.ts`）：auth/agent/room/match 等通过 `invoke` 调 Tauri command；本地对局直接进程托管
- **HTTP 实现**（新增 `httpBackend.ts`）：fetch + `Authorization: Bearer` header；WS 走 `/api/ws/:match_id?token=...`
- **Mock 实现**（`webBackend.ts`）：保留 localStorage，用于纯前端开发

### 5.2 数据类型（TS 镜像）

新增的核心 TS 类型（与 Rust models 对齐）：

```ts
type Team = 'order' | 'chaos';
type Visibility = 'private' | 'friends' | 'public';
type AgentType = 'llm' | 'rl' | 'script';

interface Agent {
  id: string;
  owner_id: number;
  name: string;
  champion: string;
  agent_config_id: string;
  spawn_preset_id: string | null;
  visibility: Visibility;
  forked_from: string | null;
  upstream_agent_id: string | null;
  created_at: string;
  updated_at: string;
}

interface AgentConfig {
  id: string;
  owner_id: number;
  name: string;
  agent_type: AgentType;
  prompt: string;
  preamble: string;
  model: string;
  config_json: Record<string, any>;
  visibility: Visibility;
  forked_from: string | null;
}

interface AgentSnapshot {
  id: string;
  agent_id: string;
  version: number;
  config_freeze: Record<string, any>;
  published_at: string;
}

interface Room {
  id: string;
  owner_id: number;
  name: string;
  invite_code: string;
  max_members: number;
  max_agents_per_member: number;
  team_policy: 'single_team' | 'free';
  lobby_visible: boolean;
  prompt_visible: boolean;
  status: 'lobby' | 'running' | 'closed';
  created_at: string;
}

interface MatchSummary {
  id: string;
  form: 'local' | 'room' | 'rank';
  mode: string;
  status: string;
  winner_team: Team | 'none' | null;
  started_at: string | null;
  finished_at: string | null;
}

interface MatchEvent {
  seq: number;
  event_type: string;
  agent_id: string | null;
  payload: Record<string, any>;
  game_time_ms: number;
}

interface EloRating {
  agent_id: string;
  mode: string;
  season_id: string;
  rating: number;
  wins: number;
  losses: number;
  draws: number;
}

interface WsEvent {
  type: 'event';
  event: string;       // 'game_started' | 'action_executed' | ...
  data: Record<string, any>;
}
```

### 5.3 OpenAPI 与类型同步

**方案**：Rust 端集成 [`utoipa`](https://docs.rs/utoipa)：
- 每个 handler 加 `#[utoipa::path(...)]` 注解
- 每个 model struct 加 `#[derive(ToSchema)]`
- 启动时生成 `openapi.json`，前端用 [`openapi-typescript`](https://github.com/drwpow/openapi-typescript) codegen TS types

**收益**：
- 消除当前"前端 TS types 手工镜像 Rust、易漂移"的问题
- 自动生成 API client（配合 `openapi-fetch` 或 `orval`）

**落地节奏**：Phase 3a 先用手工镜像，Phase 3b 起引入 utoipa。

### 5.4 Token 存储策略

| 客户端 | 存储位置 |
|---|---|
| Web（浏览器） | `localStorage`（key: `moon_lol_token`），fetch 拦截器自动注入 header |
| Desktop（Tauri） | Tauri IPC 不需要 token；若 webview 内也用 HTTP，则同 web |
| Desktop（keychain 可选） | Phase 5 可迁移到 `tauri-plugin-stronghold` 或系统 keychain |

---

## 六、测试策略

### 6.1 测试金字塔

```
        ▲
        │   ◆◆◆◆◆  E2E / 手工冒烟（少量）
        │   ◆◆◆◆◆
        │  ███████  Handler HTTP 集成测试（每路由 1-3 个）
        │ █████████
        │██████████████ Service 单元测试（每业务规则 1 个，全 mock repo+cache，不碰 DB）
        │██████████████
        │████████████████████ Domain 单元测试（纯函数，覆盖每条业务规则的所有分支）
        │████████████████████
        │████████████████████████████ Repository 集成测试（testcontainers 真 PG，验证 SQL/约束/索引）
        │████████████████████████████
```

| 层 | 工具 | 跑什么 | 触碰 IO |
|---|---|---|---|
| **domain 单测** | `#[test]`（纯同步） | ELO 公式、可见性判定、槽位限制、匹配配对算法 | 无 |
| **repository 集成测** | `#[tokio::test]` + testcontainers | 真 PG 上验证 SQL/约束/索引/事务 | 真 PostgreSQL（Docker） |
| **service 单测** | `#[tokio::test]` + mockall | 业务编排逻辑，repo 和 cache 全 mock | 无（mock） |
| **handler HTTP 测** | `#[tokio::test]` + axum `TestServer` | 路由/参数解析/鉴权/错误码，service 层 mock | 无（mock） |
| **E2E** | 手工 / 脚本 | 全链路（DB + Bevy 子进程 + WS） | 全真 |

**核心目标**：service 单测和 domain 单测合起来覆盖 100% 业务规则，且这层测试**零 IO、毫秒级**——因为业务逻辑的 bug 应该在 mock 测试里被抓到，而不是等到打 DB 才暴露。

### 6.2 dev-dependencies

`crates/lol_web_server/Cargo.toml` 需新增：

```toml
[dev-dependencies]
mockall = "0.13"              # 自动生成 repo/cache/service trait 的 mock
testcontainers = "0.23"       # 集成测试起真 PG 容器
tokio-test = "0.4"            # async 测试辅助
pretty_assertions = "1.4"     # 可读的 assert_eq diff
serde_json = "1"              # 构造测试 fixture
```

### 6.3 Domain 单元测试（纯函数，零 IO）

放在 `src/domain/<module>_tests.rs`，`#[cfg(test)]` 内联或同级文件。

**示例：ELO 计算**（`src/domain/elo.rs` + `elo_tests.rs`）：
```rust
#[test]
fn new_agent_has_1200_rating() {
    let r = EloRating::new();
    assert_eq!(r.rating, 1200.0);
}

#[test]
fn expected_score_winner_higher_rated() {
    // 1800 vs 1500：高分方预期胜率约 0.849
    let e = expected_score(1800.0, 1500.0);
    assert!((e - 0.849).abs() < 0.01);
}

#[test]
fn rating_update_upset_deltas_larger() {
    // 1200 击败 1800：低分方涨分多于稳赢场景
    let winner = apply_outcome(1200.0, 1800.0, Outcome::Win, k=32);
    let loser  = apply_outcome(1800.0, 1200.0, Outcome::Loss, k=32);
    assert!(winner - 1200.0 > 16.0);  // 涨 16+
    assert!(1800.0 - loser > 16.0);
}

#[test]
fn draw_updates_toward_expected() {
    // 1800 vs 1500 平局：高分方掉分，低分方涨分
    let high = apply_outcome(1800.0, 1500.0, Outcome::Draw, k=32);
    assert!(high < 1800.0);
}
```

**示例：可见性判定**（`src/domain/agent.rs`）：
```rust
#[test]
fn private_agent_only_owner_can_view() {
    let agent = test_agent(Visibility::Private, owner_id=1);
    assert!(can_view(&agent, viewer=1, is_friend=false));     // owner
    assert!(!can_view(&agent, viewer=2, is_friend=false));    // 他人
    assert!(!can_view(&agent, viewer=2, is_friend=true));     // 好友也不行
}

#[test]
fn friends_visible_to_friends() {
    let agent = test_agent(Visibility::Friends, owner_id=1);
    assert!(can_view(&agent, viewer=1, is_friend=false));     // owner
    assert!(!can_view(&agent, viewer=2, is_friend=false));    // 非好友
    assert!(can_view(&agent, viewer=2, is_friend=true));      // 好友
}
```

**示例：槽位限制**（`src/domain/agent.rs`）：
```rust
#[test]
fn slot_limit_blocks_creation_at_cap() {
    assert_within_slot_limit(current=5, limit=5)
        .expect_err("应拒绝：已达上限");
}

#[test]
fn slot_limit_allows_below_cap() {
    assert_within_slot_limit(current=4, limit=5)
        .expect("应放行：未达上限");
}
```

> **覆盖要求**：domain 层每条业务规则至少覆盖正常路径 + 边界 + 异常路径。新增 domain 规则时同步加测试。

### 6.4 Service 单元测试（mock repo + cache，零 IO）

**核心原则**：service 测试**绝不碰 DB**。所有 `XxxRepo` 和 `XxxCache` 用 mockall 自动生成 mock，注入到 service。

**mockall 自动生成**：给 trait 加 `#[automock]` 或在测试模块用 `mock!` 宏：
```rust
// src/service/agent_service.rs 末尾
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::*;            // trait AgentRepo 的 mock
    use mockall::mock;          // 或对已存在 trait 用 mock! 宏

    mock! {
        pub AgentRepo {}
        #[async_trait]
        impl AgentRepo for AgentRepo {
            async fn find_by_id(&self, id: &str) -> Result<Option<Agent>, RepoError>;
            async fn count_by_owner(&self, owner_id: i32) -> Result<usize, RepoError>;
            async fn insert(&self, agent: Agent) -> Result<Agent, RepoError>;
            // ... 全方法
        }
    }

    mock! {
        pub AgentCache {}
        #[async_trait]
        impl AgentCache for AgentCache {
            async fn get(&self, id: &str) -> Option<Agent>;
            async fn put(&self, agent: Agent);
            async fn invalidate(&self, id: &str);
        }
    }

    fn build_service(
        repo: MockAgentRepo,
        cache: MockAgentCache,
        subscription: MockSubscriptionService,
    ) -> AgentServiceImpl {
        AgentServiceImpl {
            repo: Arc::new(repo),
            cache: Arc::new(cache),
            subscription_service: Arc::new(subscription),
            // 其他依赖可按用例选择性 mock
            ...
        }
    }
}
```

**测试用例示例：`create_agent` 的槽位限制路径**：
```rust
#[tokio::test]
async fn create_agent_rejected_when_slot_limit_reached() {
    let mut repo = MockAgentRepo::new();
    repo.expect_count_by_owner()
        .returning(|_| Ok(5));              // 已达免费上限 5
    repo.expect_insert().times(0);          // 断言：不应落库

    let mut sub = MockSubscriptionService::new();
    sub.expect_get_agent_limit()
        .returning(|_| Ok(5));              // 免费档

    let service = build_service(repo, MockAgentCache::new(), sub);
    let err = service.create_agent(1, sample_input()).await.unwrap_err();
    assert!(matches!(err, ServiceError::AgentSlotLimit { .. }));
}

#[tokio::test]
async fn create_agent_succeeds_below_limit() {
    let mut repo = MockAgentRepo::new();
    repo.expect_count_by_owner().returning(|_| Ok(3));
    repo.expect_insert()
        .with(predicate::function(|a: &Agent| a.champion == "Riven"))
        .returning(|a| Ok(a));              // 模拟落库回写

    let mut sub = MockSubscriptionService::new();
    sub.expect_get_agent_limit().returning(|_| Ok(5));

    let service = build_service(repo, MockAgentCache::new(), sub);
    let result = service.create_agent(1, sample_input()).await.unwrap();
    assert_eq!(result.champion, "Riven");
}

#[tokio::test]
async fn get_agent_uses_cache_hit() {
    let mut repo = MockAgentRepo::new();
    repo.expect_find_by_id().times(0);      // 断言：缓存命中则不查 DB

    let mut cache = MockAgentCache::new();
    cache.expect_get()
        .returning(|_| Some(test_public_agent()));  // 命中

    let service = build_service(repo, cache, MockSubscriptionService::new());
    let result = service.get_agent(viewer=2, id="abc").await.unwrap();
    assert_eq!(result.id, "abc");
}

#[tokio::test]
async fn get_agent_cache_miss_falls_back_to_repo_and_backfills() {
    let mut repo = MockAgentRepo::new();
    repo.expect_find_by_id()
        .returning(|_| Some(test_public_agent()));  // DB 命中
    let mut cache = MockAgentCache::new();
    cache.expect_get().returning(|_| None);         // 缓存未命中
    cache.expect_put().times(1);                    // 断言：回填缓存

    let service = build_service(repo, cache, MockSubscriptionService::new());
    service.get_agent(2, "abc").await.unwrap();
}
```

**测试用例示例：Rank 匹配编排**（`rank_service.rs`，跨子系统协作）：
```rust
#[tokio::test]
async fn rank_match_makes_pair_when_elo_within_window() {
    let mut queue_repo = MockRankQueueRepo::new();
    queue_repo.expect_find_opponents()
        .returning(|agent_elo, window| {
            // 模拟池里有 ELO 接近的对手
            Ok(vec![test_queued_agent(rating=1210)])  // 窗口 50 内
        });
    queue_repo.expect_mark_matching().times(2);       // 标记双方进入匹配

    let mut match_service = MockMatchService::new();
    match_service.expect_create_rank_match()
        .with(predicate::eq(2))                       // 断言：创建一个 2 人对局
        .returning(|_| Ok(test_match()));

    let service = RankServiceImpl { queue_repo, match_service, ... };
    service.try_match(test_queued_agent(rating=1200)).await.unwrap();
}

#[tokio::test]
async fn rank_match_waits_when_no_opponent_in_window() {
    let mut queue_repo = MockRankQueueRepo::new();
    queue_repo.expect_find_opponents()
        .returning(|_, _| Ok(vec![]));                 // 池里没人
    queue_repo.expect_mark_matching().times(0);       // 断言：不配对

    let service = RankServiceImpl { queue_repo, match_service: MockMatchService::new(), ... };
    let result = service.try_match(test_queued_agent(rating=1200)).await.unwrap();
    assert!(result.is_none());                         // 未配对
}
```

**service 单测覆盖要求**：
- 每个公开方法覆盖：正常路径 + 每个错误分支（NotFound/Forbidden/SlotLimit/...）
- 缓存相关方法覆盖：命中、未命中回填、失效
- 跨子系统调用覆盖：被调方 mock，断言调用次数和参数

### 6.5 Repository 集成测试（testcontainers 真 PG）

**核心原则**：repo 测试**必须碰真 DB**，验证 SQL 正确性、约束触发、索引效率。不 mock 任何东西。

**基建**（`tests/common/mod.rs`）：
```rust
use testcontainers::{clients::Cli, images::postgres::Postgres};
use sqlx::PgPool;

pub async fn setup_pg() -> PgPool {
    let docker = Cli::default();
    let container = docker.run(Postgres::default());  // 拉 postgres:16
    let port = container.get_host_port_ipv4(5432);
    let url = format!("postgres://postgres:postgres@localhost:{port}/postgres");
    let pool = PgPoolOptions::new().connect(&url).await.unwrap();
    run_migrations(&pool).await;    // 建全部表（§2 schema）
    pool
}
```

**测试用例示例**（`tests/agent_repo_test.rs`）：
```rust
#[tokio::test]
async fn insert_and_find_by_id_roundtrip() {
    let pool = setup_pg().await;
    let repo = PgAgentRepo { pool };

    let agent = repo.insert(sample_agent(name="锐雯", owner_id=1)).await.unwrap();
    let found = repo.find_by_id(&agent.id).await.unwrap().unwrap();
    assert_eq!(found.name, "锐雯");
    assert_eq!(found.owner_id, 1);
}

#[tokio::test]
async fn unique_constraint_owner_name_rejects_duplicate() {
    let pool = setup_pg().await;
    let repo = PgAgentRepo { pool };

    repo.insert(sample_agent(name="同名的 Agent", owner_id=1)).await.unwrap();
    let err = repo.insert(sample_agent(name="同名的 Agent", owner_id=1)).await.unwrap_err();
    // 断言：触发 UNIQUE(owner_id, name) 约束
    assert!(matches!(err, RepoError::UniqueViolation { .. }));
}

#[tokio::test]
async fn delete_agent_cascades_to_snapshots() {
    let pool = setup_pg().await;
    let agent_repo = PgAgentRepo { pool: pool.clone() };
    let snap_repo = PgAgentSnapshotRepo { pool: pool.clone() };

    let agent = agent_repo.insert(sample_agent()).await.unwrap();
    snap_repo.insert(sample_snapshot(agent.id.clone())).await.unwrap();
    agent_repo.delete(&agent.id).await.unwrap();

    let snaps = snap_repo.list_by_agent(&agent.id).await.unwrap();
    assert!(snaps.is_empty());  // 断言：ON DELETE CASCADE 生效
}

#[tokio::test]
async fn list_by_owner_applies_visibility_filter() {
    let pool = setup_pg().await;
    let repo = PgAgentRepo { pool };

    repo.insert(sample_agent_with(visibility=Private, owner=1)).await.unwrap();
    repo.insert(sample_agent_with(visibility=Public,  owner=1)).await.unwrap();
    repo.insert(sample_agent_with(visibility=Friends, owner=2)).await.unwrap();

    let public_only = repo.list_public().await.unwrap();
    assert_eq!(public_only.len(), 1);
}
```

**集成测试覆盖要求**：
- 每个 repo 方法：正常 CRUD roundtrip
- 每个约束（UNIQUE/FK/CHECK）：触发约束的负例
- 关键索引：用 `EXPLAIN` 验证查询走索引（可选，性能测试）
- 事务边界：并发场景下的行为（如 ELO 更新不可串行化导致的异常）

> **CI 环境要求**：集成测试需要 Docker。GitHub Actions 用 `services: postgres:16` 或 docker-in-docker；本地需 Docker Desktop。

### 6.6 Handler HTTP 集成测试

用 axum `TestServer` + mock service trait，验证路由/参数解析/鉴权/错误码映射。service 层全 mock，不碰 DB。

```rust
#[tokio::test]
async fn get_agent_returns_404_when_service_returns_not_found() {
    let mut agent_service = MockAgentService::new();
    agent_service.expect_get_agent()
        .returning(|_, _| Err(ServiceError::NotFound));

    let app = build_test_router(Arc::new(agent_service));
    let response = TestServer::new(app)
        .get("/api/agents/abc")
        .with_header("authorization", "Bearer <test_jwt>")
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json();
    assert_eq!(body["error"]["code"], "NOT_FOUND");
}

#[tokio::test]
async fn create_agent_without_token_returns_401() {
    let app = build_test_router(Arc::new(MockAgentService::new()));
    let response = TestServer::new(app)
        .post("/api/agents")
        .json(&sample_request_body())
        .await;
    response.assert_status(StatusCode::UNAUTHORIZED);
}
```

### 6.7 缓存层测试

**Moka 实现的单元测试**：用真 Moka 实例（内存内，无外部依赖），验证 get/put/invalidate 行为。

```rust
#[tokio::test]
async fn moka_cache_put_then_get_roundtrip() {
    let cache = MokaAgentCache::test_instance();
    cache.put(test_agent(id="abc")).await;
    let got = cache.get("abc").await;
    assert!(got.is_some());
}

#[tokio::test]
async fn moka_cache_invalidate_removes_entry() {
    let cache = MokaAgentCache::test_instance();
    cache.put(test_agent(id="abc")).await;
    cache.invalidate("abc").await;
    assert!(cache.get("abc").await.is_none());
}
```

**缓存一致性测试（service 层）**：写操作（update/delete）后验证 cache 被失效：
```rust
#[tokio::test]
async fn update_agent_invalidates_cache_entry() {
    let mut repo = MockAgentRepo::new();
    repo.expect_update().returning(|_| Ok(()));
    let mut cache = MockAgentCache::new();
    cache.expect_invalidate()
        .with(predicate::eq("abc"))
        .times(1);  // 断言：update 后必须 invalidate

    let service = build_service(repo, cache, ...);
    service.update_agent(1, "abc", sample_update()).await.unwrap();
}
```

### 6.8 测试组织与运行

**目录结构**：
```
crates/lol_web_server/
├── src/
│   ├── domain/
│   │   ├── elo.rs
│   │   └── elo_tests.rs          # domain 单测（#[cfg(test)] 同级文件）
│   └── service/
│       ├── agent_service.rs
│       └── agent_service_tests.rs  # service 单测（mock，可内联或同级）
└── tests/                         # 集成测试（独立二进制，能访问 pub API）
    ├── common/
    │   └── mod.rs                 # setup_pg 等 fixture
    ├── agent_repo_test.rs         # repo 集成测（testcontainers）
    ├── agent_handler_test.rs      # handler HTTP 测
    └── ...
```

**运行命令**：
```bash
# 只跑 domain + service 单测（毫秒级，零 IO）
cargo test --lib -- --skip integration

# 只跑 repo 集成测（需 Docker，秒级）
cargo test --test '*' -- --include-ignored

# 全量
cargo test --all
```

**CI 流水线**：
1. `cargo fmt --check` + `cargo clippy`
2. `cargo test --lib`（domain + service，不需要 Docker，每次 PR 必跑）
3. `cargo test --test '*'`（repo 集成测，需 Docker，PR 必跑）
4. （可选）性能回归：关键查询的 `EXPLAIN` 基线对比

### 6.9 测试覆盖率目标

| 层 | 目标 | 说明 |
|---|---|---|
| domain | **100%** | 纯函数，分支少，必须全覆盖 |
| service | **≥ 90%** | 每方法覆盖正常 + 错误分支；跨子系统调用覆盖 |
| repository | **≥ 80%** | 每 CRUD 方法 roundtrip + 关键约束负例 |
| cache | **≥ 90%** | get/put/invalidate 全覆盖 |
| handlers | **≥ 70%** | 关键路由的鉴权/参数/错误码 |

**优先级**：domain > service > repository > cache > handlers。业务逻辑的 bug 应在 domain/service 层被抓到，而非依赖集成测或手工测。

---

## 七、实现路线

> 与 PRODUCT.md Phase 对齐，每个阶段产出可独立验证的增量。

### Phase 3a — 架构分层 + 数据模型 + Agent 资产重构

**目标**：建立六层分层架构与测试基建，把现有三表体系升级为 §2 的资产模型，前端能管理 Agent。

**架构与测试前置（必须最先做）**：
- [ ] 按照目录结构（§3.13.1）建立 `domain/` `repository/` `cache/` `service/` 四个模块
- [ ] 引入 dev-dependencies（§6.2）：mockall、testcontainers、pretty_assertions
- [ ] 建立 `tests/common/mod.rs` 的 testcontainers PG fixture（§6.5）
- [ ] 迁移首个子系统（建议从 Config 开始，最简单）走通完整的 domain → repo → service → handler 四层 + 各层测试，作为后续子系统的模板

**数据模型与业务**：
- [ ] 落地 §2.2.3 ~ §2.2.6 表结构（spawn_presets / agent_configs / agents / agent_snapshots）
- [ ] 拆分旧 `PresetService` → `SpawnPresetRepo`+`SpawnPresetService` + `AgentConfigRepo`+`AgentConfigService` + `AgentRepo`+`AgentService`（§3.13.4 迁移路径）
- [ ] 提取 domain 纯函数：可见性判定（§6.3）、槽位限制校验
- [ ] 实现 §3.3 / §3.4 路由
- [ ] 补 §5.1 的 auth 方法 + agent/config CRUD
- [ ] Agent 槽位上限校验（`agent_count < agent_limit`，免费=5，走 SubscriptionService）
- [ ] 删除旧的 `hero_presets` 表与 `interfaces.rs`（开发阶段无迁移负担）

**配套测试**：
- [ ] domain：可见性、槽位限制 100% 覆盖
- [ ] service：create/get/list/delete 含缓存命中/未命中/失效路径
- [ ] repository：CRUD roundtrip + UNIQUE(owner_id, name) + FK CASCADE
- [ ] handler：鉴权 401、NotFound 404、SlotLimit 402

### Phase 3b — 房间 + matches + Bevy 协议升级

**目标**：房间能开起来、对局能托管、操作流能观战。

- [ ] 落地 §2.2.8 ~ §2.2.10（rooms 系列表）
- [ ] 落地 §2.2.11 ~ §2.2.13（matches 系列表）
- [ ] 实现 `RoomRepo`+`RoomService`、`MatchRepo`+`MatchEventRepo`+`MatchService`
- [ ] domain：房间约束校验（team_policy、max_agents_per_member）、对局状态机
- [ ] 实现 §3.6 / §3.7 / §3.8 路由
- [ ] Bevy 协议升级：§4.2 hello + §4.3 新事件 + §4.4 操作流广播
- [ ] §4.5 胜利条件判定系统（domain 内纯函数：条件树求值 + 单测）
- [ ] §4.6 MatchService 进程托管（端口池）
- [ ] §3.11 WS 鉴权 + hello 注入 + 嗅探落库
- [ ] 日志 24h 留存 + 下载接口（§PRODUCT 5.2）
- [ ] **配套测试**：房间约束违规负例、胜利条件树各分支、对局状态机迁移、match_events 落库

### Phase 4a — Rank 队列 + ELO + 赛季

**目标**：Agent 7×24 自动上分。

- [ ] 落地 §2.2.14 ~ §2.2.16（seasons / rank_queues / elo_ratings）
- [ ] 实现 `RankQueueRepo`+`RankService`、`EloRepo`+`LeaderboardService`（含 `LeaderboardCache`）
- [ ] domain：ELO 计算公式、匹配配对算法（纯函数 + 全分支单测）
- [ ] 实现 §3.9 路由
- [ ] 匹配算法（ELO 接近配对，宽限窗口逐步扩大）
- [ ] 对局结束后 ELO 自动更新 + 总榜/日榜查询
- [ ] §4.7 Rank 异常处理（30 秒宽限 → 中止 → 重排）
- [ ] 赛季调度（自动 active/concluded）
- [ ] **配套测试**：ELO 公式 upset/draw/blowout 各场景、配对命中/未命中、缓存失效（ELO 更新后排行榜缓存必须 invalidate）

### Phase 4b — 精粹/订阅 + 社区 Fork

**目标**：商业化闭环 + 社区资产流通。

- [ ] 落地 §2.2.17 ~ §2.2.18（essence / subscriptions）
- [ ] 实现 `EssenceRepo`+`EssenceService`、`SubscriptionRepo`+`SubscriptionService`（含 `BillingPlanCache`）
- [ ] domain：精粹扣减规则（余额不可负）、订阅档位权益映射
- [ ] 实现 §3.10 路由
- [ ] 平台模型 Token 计费（对局结束按实际 Token 扣精粹）
- [ ] BYO 模型不计费路径
- [ ] 社区 Agent 浏览 + Fork + pull-upstream（§3.3.3）
- [ ] 社区快照分享/引入
- [ ] **配套测试**：精粹扣减幂等性、余额不足拒绝、Fork 关系链、并发扣款的事务隔离

### Phase 5 — 管理后台 + 社交 + 扩展模式

- [ ] §3.12 管理后台（算力监控、强制中止）
- [ ] OpenAPI codegen 落地（§5.3）
- [ ] 好友/关注/战队（§PRODUCT 7.2）
- [ ] 更多 Rank 模式（中单 SOLO / 下路 2v2 / 5v5）
- [ ] 战队职业比赛模式

---

## 附录 A：开放参数默认值

| 参数 | 默认值 | 说明 |
|---|---|---|
| 对局 ID | UUID v4 | 字符串 |
| 端口池范围 | `[9100, 9200)` | 100 局并发上限 |
| WS 鉴权方式 | query 参数 `?token=` | 浏览器 WS 无法自定义 header |
| ELO 初始值 | 1200 | |
| ELO K 因子 | 32 | |
| 日榜重置时区 | UTC+8 0 点 | 北京时间 |
| 精粹汇率 | 1 元 = 100 精粹，1 精粹 ≈ 1000 token | Phase 4 复核 |
| 免费用户 Agent 上限 | 5 | |
| 默认房间人数上限 | 10 | |
| 默认每人 Agent 数 | 3 | |
| Rank 掉线宽限 | 30 秒 | |
| JWT 有效期 | 30 天 | |
| 服务器日志留存 | 24 小时 | |

## 附录 B：术语对照（PRODUCT.md ↔ API_DESIGN.md）

| PRODUCT.md 术语 | 数据库表 | API 资源 |
|---|---|---|
| 大脑（"策略大脑"） | `agent_configs` | `/api/agent-configs` |
| Agent（"选手"） | `agents` | `/api/agents` |
| 参赛快照 | `agent_snapshots` | `/api/agents/:id/snapshots` |
| 出生点预设 | `spawn_presets` | `/api/spawn-presets` |
| 场景 | `scenarios` | `/api/scenarios` |
| 房间 | `rooms` + `room_members` + `room_agent_slots` | `/api/rooms` |
| 对局实例 | `matches` + `match_participants` + `match_events` | `/api/matches` |
| 赛季 | `seasons` | `/api/rank/seasons` |
| ELO | `elo_ratings` | `/api/agents/:id/elo`、`/api/rank/leaderboard` |
| 精粹 | `essence_balances` + `essence_transactions` | `/api/essence` |
