# 模型供应商与模型设置 — 待办与完成情况

本页面记录模型供应商子系统的已完成与未完成任务。

## 一、已完成任务列表

- [x] **服务端六层子系统**：新增 `model_providers` 表与 domain/repository/cache/service/handler 六层，镜像 config 子系统；`AppState` + `main.rs` 注入；路由 `/api/model-providers`；service 单测覆盖脱敏 / 空 key 保留旧值 / resolve_for_runtime 缓存命中与回退。
- [x] **服务端编排器按 provider 解析凭证**：`SceneAgentConfig` 增 `model`/`provider_id`，`Orchestrator::new` 按 `provider_id`+owner 调 `resolve_for_runtime`，缺省回退 env；`local_game_service` 注入 owner_id 与服务。
- [x] **桌面端编排器贯通**：`agent.rs` 的 `AgentConfig` 增字段、`load_providers` 读 providers.json、`resolve_credentials` 按 provider 解析凭证、缺省回退 env。
- [x] **快照冻结接线**：`publish_snapshot` 改为加载 Agent 并调用 `build_config_freeze`，使参赛快照冻结 `model` + `config_json.provider_id`。
- [x] **前端服务层**：types.ts 加 `ModelProvider`/`ModelProviderInput` + `FrontAgentConfig` 字段；cloud/cloudImpl 加 CRUD；backend.ts 三映射；Tauri `get/set_model_providers` 命令读写 providers.json。
- [x] **预设目录**：`config/providerPresets.ts` 落地国内外厂商预设（智谱、DeepSeek、火山方舟、Kimi、MiniMax 等），数据整理自 cc-switch。
- [x] **设置页供应商管理 UI**：`model_settings` tab 重写为左侧导航（仅列平台 + 已配置供应商）+ 右侧表单；新增供应商时通过「供应商类型」下拉选预设自动预填厂商参数或选自定义手填，CRUD、刷新探测。
- [x] **选手编辑页级联下拉**：`heroes.vue` LLM 模型字段改为供应商 + 模型名级联下拉，写 `model` + `config_json.provider_id`，保留手填兜底。
- [x] **场景装配贯通**：`useSlotConfig.expandSlot` 透传 `model`/`provider_id`/`config_json`，使场景 json→编排器拿到每选手凭证。
- [x] **移除平台默认凭证**：删除 `ai_config` 表及其六层子系统、`/api/config` 路由、Tauri `get/set_ai_config` 命令与启动期 env 注入、前端 `getAiConfig`/`setAiConfig`/`AiConfig` 类型、设置页全局默认凭证卡片与 `settings.model.*` 文案。选手 LLM 配置必填，无静默全局回退；编排器拿不到凭证的选手被跳过。
- [x] **移除全局 Preamble**：编排器不再读 `ANTHROPIC_PREAMBLE`、不再拼到每个选手 prompt 前；系统提示词即选手自身 `prompt`。
- [x] **平台模型为管理员 env 网关**：平台模型凭证由管理员在服务端 env 配置（`ANTHROPIC_API_KEY/BASE_URL/MODEL`），用户不可见不可改；可选模型名由管理员 `PLATFORM_MODELS` env 提供，新增 `GET /api/platform-models` 端点 + cloud `listPlatformModels`；`heroes.vue` 选平台模型时下拉只选不填，按 Token 消耗以精粹结算。

---

## 二、待完成任务列表

### 1. 安全与凭证

- [ ] **API Key 加密存储**：当前 `model_providers.api_key` 存明文。改为存储时加密、镜像云端时单独加密，本地缓存仅存于桌面端受控目录。
- [ ] **可见性即权限**：导出/查看供应商密钥的权限由对应资产可见性设置决定（与 Agent Prompt / 模型配置同规则），需在导出 / Fork 路径上落地校验。

### 2. 运行时格式扩展

- [ ] **openai_chat 客户端**：`create_agent` 按 `api_format` 选 rig 客户端，目前仅 anthropic 生效；补 `rig::providers::openai` 路径。
- [ ] **openai_responses / gemini_native**：当前回退 anthropic 兼容路径并告警；补对应 rig 客户端或格式转换。

### 3. 存储与同步

- [ ] **桌面端云端同步**：桌面端供应商目前仅存本地 `providers.json`，未镜像云端。复用选手资产的云优先同步机制（在线写云端 + 镜像本地、离线降级）。
- [ ] **刷新探测健壮性**：`{baseUrl}/v1/models` 探测兼容剥离已知子路径后的变体（`/models` 等），并处理无 api_key 或鉴权失败的优雅降级。

### 4. 体验打磨

- [ ] **自定义供应商拖拽排序**：左侧导航自定义项支持 `grip-vertical` 拖拽调整 `sort_order`。
- [ ] **手填模型回写**：选手编辑页手填的模型名回写进该供应商的 `models` 列表以便复用。
- [ ] **图标资源**：预设供应商的厂商图标目前仅存名称占位，补 SVG 图标资源与渲染。
