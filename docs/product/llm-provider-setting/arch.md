# 模型供应商与模型设置 — 架构设计

本文档描述模型供应商子系统的数据模型、接口、运行时贯通与文件引用，以及前后端实现细节。对应产品形态见 [产品设计](product.md)。

## 一、数据模型

### 服务端 `model_providers` 表

新增于 [schema.sql](/crates/lol_web_server/migrations/schema.sql)，按 `user_id` 隔离：

```sql
CREATE TABLE IF NOT EXISTS model_providers (
    id          UUID PRIMARY KEY,
    user_id     INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    category    TEXT NOT NULL DEFAULT 'custom',   -- preset | custom | platform
    preset_type TEXT NOT NULL DEFAULT '',
    base_url    TEXT NOT NULL DEFAULT '',
    api_key     TEXT NOT NULL DEFAULT '',
    api_format  TEXT NOT NULL DEFAULT 'anthropic', -- anthropic | openai_chat | openai_responses | gemini_native
    models      JSONB NOT NULL DEFAULT '[]',       -- 字符串数组
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    website_url TEXT NOT NULL DEFAULT '',
    api_key_url TEXT NOT NULL DEFAULT '',
    icon        TEXT NOT NULL DEFAULT '',
    icon_color  TEXT NOT NULL DEFAULT '',
    sort_order  INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (user_id, name)
);
```

`agents` 表无需迁移：`provider_id` 存进既有 `config_json` JSONB，`model` 列已存在。

**密钥处理**（存明文，UI 不回显）：`GET` 列表时 `api_key` 置空、附 `has_api_key: bool`；`PUT` 时若 `api_key` 为空串则用 `COALESCE(NULLIF(...,''), api_key)` 保留旧值。密钥加密留作后续 todo。

### 前端类型

[types.ts](/apps/client/src/services/types.ts) 定义 `ModelProvider` / `ModelProviderInput` / `ApiFormat` / `ProviderCategory`，字段 snake_case 与服务端 DTO 对齐。`FrontAgentConfig` 增 `model?` / `provider_id?` / `config_json?`，使场景装配能携带每选手的供应商与模型。

## 二、服务端六层子系统

model_provider 子系统按 domain→repository→cache→service→handler 分层：

| 层 | 文件 | 说明 |
|---|---|---|
| domain | [model_provider.rs](/crates/lol_web_server/src/domain/model_provider.rs) | `ModelProvider` / `ModelProviderInput` / `ModelProviderDto`（脱敏，`has_api_key`）+ 校验 |
| repository | [model_provider_repo.rs](/crates/lol_web_server/src/repository/model_provider_repo.rs) | `ModelProviderRepo` trait + `PgModelProviderRepo`；`find_for_runtime` 校验归属；`update` 空串保留旧 key |
| cache | [model_provider_cache.rs](/crates/lol_web_server/src/cache/model_provider_cache.rs) | `MokaModelProviderCache`（key=user_id，存含明文列表供运行时）+ `NoopModelProviderCache` |
| service | [model_provider_service.rs](/crates/lol_web_server/src/service/model_provider_service.rs) | `list` 脱敏 / `create` / `update` / `delete` / `resolve_for_runtime`（缓存优先） |
| handler | [model_provider.rs](/crates/lol_web_server/src/handlers/model_provider.rs) | `list` / `create` / `update` / `delete`，均 `AuthUser` + `State<AppState>` |
| 组合根 | [main.rs](/crates/lol_web_server/src/main.rs) | 注入 repo→cache→service→`AppState.model_provider_service` |

路由注册在 [handlers/mod.rs](/crates/lol_web_server/src/handlers/mod.rs)：`/api/model-providers`（get+post）、`/api/model-providers/:id`（put+delete）。

### 平台模型端点

[handlers/platform_model.rs](/crates/lol_web_server/src/handlers/platform_model.rs) 的 `GET /api/platform-models`（`AuthUser`）读取管理员 env `PLATFORM_MODELS`（逗号分隔），返回可选模型名数组。平台模型不走 DB、无 service 层——凭证与模型清单都由管理员在服务端 env 配置，用户只读。

## 三、运行时按 provider 解析凭证

每个选手的 LLM 凭证按其 `provider_id` 解析：选自带供应商时取该供应商的 `{api_key, base_url}`；选「平台模型」（`provider_id` 为空）时走管理员在服务端 env 配置的平台网关（`ANTHROPIC_API_KEY` / `ANTHROPIC_BASE_URL` / `ANTHROPIC_MODEL`），按 Token 消耗以精粹结算。`model` 取选手选定的模型名，平台模型缺省回退 `ANTHROPIC_MODEL`。

> 不再有「全局默认凭证」回退：删除了 `ai_config` 表及其六层子系统、Tauri `get/set_ai_config` 命令与启动期 env 注入。选手 LLM 配置必须显式选择平台模型或自带供应商；拿不到 api_key 的选手被跳过。全局 Preamble（曾由 `ANTHROPIC_PREAMBLE` 拼到每个选手 prompt 前）一并移除——系统提示词即选手自身的 `prompt`。

### 服务端编排器 [agent_orchestrator.rs](/crates/lol_web_server/src/service/agent_orchestrator.rs)

- `SceneAgentConfig` 含 `model: Option<String>` / `provider_id: Option<Uuid>`（serde default，向后兼容）。
- `Orchestrator::new` 接收 `owner_id` 与 `Arc<dyn ModelProviderService>`；每个 agent 调 `resolve_for_runtime(provider_id, owner_id)` 取 `{api_key, base_url, model}`，`provider_id` 为空时走平台 env。
- `run_agent_orchestrator` 签名含 `owner_id` + `providers`；调用方 [local_game_service.rs](/crates/lol_web_server/src/service/local_game_service.rs) 从 `AppState` 注入 owner_id 与服务。
- 非 anthropic 格式暂回退 anthropic 兼容路径并 `warn!` 告警（预设供应商均走 anthropic 兼容端点，已覆盖）。

### 桌面端编排器

- 桌面端运行时由前端将所有模型供应商配置随 `start_game` 传递给 Tauri 后端，在内存中解析并映射每个选手的模型与凭证，避免了本地配置文件的读写与多份存储的不一致性。

### 快照冻结（修复既有缺口）

[handlers/agent_snapshot.rs](/crates/lol_web_server/src/handlers/agent_snapshot.rs) 此前把 `config_freeze` 存空对象。改为加载 Agent 后调用既有 `agent_snapshot_service::build_config_freeze`（已存在但未被调用），使参赛快照冻结 `model` + `config_json`（含 `provider_id`）。**API Key 不进快照**——服务器对局只允许平台模型或用户预先授权的 BYO 凭证，避免把用户密钥冻结到服务端。

## 四、场景装配贯通

[useSlotConfig.ts](/apps/client/src/composables/useSlotConfig.ts) 的 `expandSlot` 此前丢弃 `model`/`config_json`。改为从选中 `HeroPreset` 透传 `model` / `provider_id` / `config_json` 到 `FrontAgentConfig`，使 `buildAgents()`→`saveCustomScenario`→场景 json→两个编排器拿到每选手的 `model` + `provider_id`。

## 五、前端实现

| 模块 | 文件 | 说明 |
|---|---|---|
| 云端服务接口 | [cloud.ts](/apps/client/src/services/cloud.ts) | `listModelProviders` / `create/update/deleteModelProvider` / `listPlatformModels` |
| 云端实现 | [cloudImpl.ts](/apps/client/src/services/cloudImpl.ts) | 对接 `/api/model-providers` 与 `/api/platform-models` |
| 预设目录 | [providerPresets.ts](/apps/client/src/config/providerPresets.ts) | 整理自 cc-switch 的厂商预设；`PLATFORM_PROVIDER_ID` 哨兵 |
| 状态 | [providersStore.ts](/apps/client/src/stores/providersStore.ts) | `useProviders`：统一走云端模型供应商 CRUD 接口 |
| 设置页 | [settings.vue](/apps/client/src/pages/settings.vue) | 左导航仅列已配置供应商 + 右表单；新增时「供应商类型」下拉选预设预填厂商参数或自定义手填，CRUD、刷新探测。不再有全局默认凭证卡片 |
| 选手编辑页 | [heroes.vue](/apps/client/src/pages/heroes.vue) | LLM 模型字段改为供应商 + 模型名级联下拉：平台模型候选来自 `/api/platform-models`（只选不填），自带供应商候选来自其 `models`（保留手填兜底）；写 `model` + `config_json.provider_id` |

### 存储策略

- **Web 与桌面端**：均 100% 走云端数据库存储（`/api/model-providers`），去除了本地 `providers.json`。

## 六、数据流总览

```
设置页 ──CRUD──> providersStore ──> /api/model-providers ─> model_providers 表
选手编辑页 ──级联下拉──> agents.model + config_json.provider_id
            └─ 平台模型候选 ──> /api/platform-models ─> 管理员 PLATFORM_MODELS env
场景装配 expandSlot ──透传──> FrontAgentConfig.{model, provider_id}
启动对局 ──> 传入当前 model_providers 配置 ──> 按 provider_id 解析凭证
            ├── BYO: 匹配前端传入的供应商配置获取 api_key 与 base_url
            └── 平台: provider_id 为空 ─> 管理员 env 网关（精粹计费）
发布快照 ──> build_config_freeze 冻结 model + config_json.provider_id（不含密钥）
```

## 七、验证

- `cargo check --all-targets`：0 错误（全工作区）。
- `cargo test -p lol_web_server`：lib + handler + local_game + model_provider 测试通过（已移除 ai_config / config_repo 测试）。
- 前端 `vue-tsc -b && vite build`：类型检查与构建通过。
- 手测要点：设置页增删供应商与刷新模型列表（无全局默认凭证卡片）；heroes.vue 选平台模型时下拉为管理员 `PLATFORM_MODELS` 清单、选自带供应商时为其 models 列表并回显；桌面本地对局按 provider 解析凭证（看 `debug!` 日志），选平台模型时走 env 网关；发布快照 `config_freeze` 非空且含 `provider_id`。
