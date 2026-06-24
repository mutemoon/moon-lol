# 算力、精粹与订阅计费 — 架构设计

本文档精准描述精粹余额、扣费流水及订阅系统的架构设计与实现原理，不涉及具体代码细节。

## 一、计费数据模型

计费体系由用户精粹钱包和订阅计划配置相关表驱动：

### 1. 核心表结构定义

- **`essence_balances`**: 存储用户当前账户的精粹余额（单钱包设计）。
- **`essence_transactions`**: 记录每一笔精粹流水（`delta`），并记录账单变化原因 (`reason`) 如：签到、扣除 Token 费、扩充选手槽位等，支持流水审计。
- **`billing_plans`**: 定义不同的订阅档位（如 `free`, `pro`, `elite`），配置价格、赠送精粹额度及选手槽位上限 (`max_agents`)。
- **`subscriptions`**: 记录用户当前激活的订阅状态、开始/结束日期以及是否开启自动续期。
- 参见：[schema.sql](/crates/lol_web_server/migrations/schema.sql)

---

## 二、Web Server 计费服务层设计

### 1. 签到与精粹存取

- **`essence_service.rs`**: 负责实现每日签到的幂等性校验。限制同一自然日内单用户只能签到一次。
- 提供 `deduct` 扣费和 `add` 充值接口。扣减精粹时强制包含行级乐观锁，并在扣减至负数时抛出 `INSUFFICIENT_ESSENCE` 错误。
- 参见：[essence_service.rs](/crates/lol_web_server/src/service/essence_service.rs)

### 2. 选手限额检查

- **`SubscriptionServiceImpl`**: 实现了 `AgentLimitProvider` 契约。
- 当用户尝试创建新选手时，`AgentService` 会首先调用 `SubscriptionServiceImpl` 统计该用户在 `agents` 表中已存在的选手总数，并与 `billing_plans` 对应的 `max_agents` 进行比对。若超出限额则阻断创建。
- 参见：[user_service.rs](/crates/lol_web_server/src/service/user_service.rs)

### 3. 平台模型 Token 消耗扣费链路

- 在对局 Bevy 实例中，`LlmDriver` 每次调用平台 LLM API 时，记录请求和响应消耗的 Prompt Token 和 Completion Token。
- 对局结束后，通过上报通道把整局累加的 Token 使用量发送给 Web Server。Web Server 计算费用（按配置定价表）并通过 `EssenceService` 执行扣减，生成对应的流水凭证。

---

## 三、前端计费中心模块

- **`/billing`**: 计费管理页面。
- 展示当前精粹余额，集成了每日签到交互按钮。
- 展示历史精粹收支明细列表（支持分页拉取）。
- 展示套餐订阅对比卡片，用户点选后调用 mock 支付流完成购买或套餐升级。
- 参见：`apps/client` 下的 billing 页面结构。
