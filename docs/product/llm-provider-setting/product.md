# 模型供应商与模型设置 — 产品设计

本文档定义客户端「设置 → 模型设置」页面的产品形态，以及它如何作为 Agent 选手 LLM 配置的数据源。预设供应商目录参考开源项目 [farion1231/cc-switch](https://github.com/farion1231/cc-switch) 的供应商预设整理而成，按本产品计费模型裁剪。

## 一、产品定位

模型设置页统一管理用户的**模型供应商**及其**模型列表**。供应商配置好之后，用户在 Agent 选手编辑页只需「选供应商 + 选模型名」，而不必手填 Base URL、API Key 或模型字符串。

供应商分两类，对应计费文档中的两条模型动力源：

| 类别 | 来源 | API Key | 计费 |
|---|---|---|---|
| **预设供应商** | 平台预置的国内外厂商目录（智谱、DeepSeek、火山方舟、Kimi、MiniMax 等），仅预填 Base URL 与默认模型 | 用户自填 | BYO，平台不参与计费 |
| **自定义供应商** | 用户完全自定义的 API 端点 | 用户自填 | BYO，平台不参与计费 |
| **平台模型**（计费侧） | 管理员在服务端 env 配置的共享模型网关 | 管理员持有 | 按 Token 消耗以精粹结算 |

> 预设供应商仅是「填表捷径」：选中后自动带入 Base URL、API 格式与默认模型列表，API Key 仍由用户填写并保管。平台模型走独立网关，凭证由管理员在服务端 env 配置（用户不可见、不可改），不出现在供应商列表里，而是在选手编辑页的供应商下拉中以「平台模型」选项出现；其可选模型名也由管理员通过 `PLATFORM_MODELS` env 提供，用户只能选不能手填。选手的 LLM 配置必填：要么选平台模型，要么选一个自带供应商。

## 二、设置页布局

采用左侧供应商导航 + 右侧表单的双栏布局，与客户端整体设置页风格一致：

```
┌──────────────────────────────────────────────────────────┐
│ 管理模型供应商，配置后可在选手编辑页选择使用。      [刷新] │
├────────────────┬─────────────────────────────────────────┤
│ 平台模型        │  添加模型供应商                          │
│  · MoonLOL共享 │  选择预设可自动填入厂商参数，或自定义手填。│
│                │                                          │
│ 我的供应商      │  供应商类型 [智谱 BigModel          ▾]    │
│  · 智谱 BigModel●│  名称      [智谱 BigModel____________]   │
│  · 火山方舟 ●  │  Base URL  [https://open.bigmodel.cn/...]│
│  + 添加供应商   │  API Key   [______________________]      │
│                │  API 格式  [Anthropic Messages (/v1) ▾]   │
│                │                                          │
│                │  模型列表  [+ 添加模型]                   │
│                │             · glm-5.1                     │
│                │                                          │
│                │           [ 保存 ]  [ 刷新 ]  [ 删除 ]    │
└────────────────┴─────────────────────────────────────────┘
```

左侧导航只列出**已配置**的供应商，不铺开全部预设目录，避免列表过长：

- **平台模型**：只读项，平台托管的共享模型网关，不可编辑 Base URL / API Key，仅用于在选手编辑页被选中。
- **我的供应商**：用户已添加的供应商（无论预设还是自定义），每项显示启用状态圆点；尾部固定「+ 添加供应商」入口，选中后右侧渲染空表单。

右侧「添加供应商」表单顶部是**供应商类型下拉**：选项为「自定义供应商」加全部预设厂商（智谱、DeepSeek、火山方舟……）。选中预设即自动预填名称、Base URL、API 格式与默认模型，用户只需补 API Key；选「自定义」则全部手填。预设预填的字段仍可二次修改。

右侧表单字段：

| 字段 | 说明 |
|---|---|
| 供应商类型 | 仅新增表单出现：下拉选预设（预填厂商参数）或自定义 |
| 名称 | 供应商显示名，如「智谱 GLM」 |
| Base URL | API 端点，如 `https://open.bigmodel.cn/api/anthropic` |
| API Key | 密码型输入，明文不回显；已设置时 placeholder 提示「留空保持不变」 |
| API 格式 | 下拉：Anthropic Messages `/v1/messages` / OpenAI Chat Completions / OpenAI Responses / Gemini Native |
| 模型列表 | 可增删的模型名集合，每项一个字符串；「+ 添加模型」追加空行 |

「刷新」按钮对启用的供应商逐个探测 `{baseUrl}/v1/models`（兼容 `/models`），用成功返回的模型列表与本地模型列表合并去重，便于一键补齐。

设置页不再提供任何「全局默认凭证」：平台模型凭证由管理员 env 管理，BYO 供应商凭证各自独立，选手 LLM 配置必须显式选择，无静默回退。

## 三、预设供应商目录

下列为预置的国内厂商与官方预设，Base URL、默认模型、API 格式均预填，用户只需填 API Key。数据整理自 cc-switch 的 `claudeProviderPresets.ts`，按本产品场景裁剪。

| 厂商 | Base URL | 默认模型 | API 格式 |
|---|---|---|---|
| 智谱 BigModel | `https://open.bigmodel.cn/api/anthropic` | `glm-5.1` | Anthropic Messages |
| 智谱 z.ai（海外） | `https://api.z.ai/api/anthropic` | `glm-5.1` | Anthropic Messages |
| DeepSeek | `https://api.deepseek.com/anthropic` | `deepseek-v4-pro` | Anthropic Messages |
| 火山方舟 Agentplan | `https://ark.cn-beijing.volces.com/api/coding` | `ark-code-latest` | Anthropic Messages |
| 豆包 Seed | `https://ark.cn-beijing.volces.com/api/compatible` | `doubao-seed-2-1-pro` | Anthropic Messages |
| 百度千帆 Coding | `https://qianfan.baidubce.com/anthropic/coding` | `qianfan-code-latest` | Anthropic Messages |
| 阿里百炼 | `https://dashscope.aliyuncs.com/apps/anthropic` | — | Anthropic Messages |
| 阿里百炼 For Coding | `https://coding.dashscope.aliyuncs.com/apps/anthropic` | — | Anthropic Messages |
| Kimi | `https://api.moonshot.cn/anthropic` | `kimi-k2.7-code` | Anthropic Messages |
| StepFun | `https://api.stepfun.com/step_plan` | `step-3.5-flash-2603` | Anthropic Messages |
| MiniMax | `https://api.minimaxi.com/anthropic` | `MiniMax-M2.7` | Anthropic Messages |
| Longcat | `https://api.longcat.chat/anthropic` | `LongCat-Flash-Chat` | Anthropic Messages |
| 百灵 BaiLing | `https://api.tbox.cn/api/anthropic` | `Ling-2.5-1T` | Anthropic Messages |
| 小米 MiMo | `https://api.xiaomimimo.com/anthropic` | `mimo-v2.5-pro` | Anthropic Messages |
| KAT-Coder | `https://vanchin.streamlake.ai/.../claude-code-proxy` | `KAT-Coder-Pro V1` | Anthropic Messages |

完整清单与图标资源维护在前端 `config/providerPresets.ts`，结构与 cc-switch 的 `ProviderPreset` 对齐。所选预设均暴露 Anthropic 兼容端点。

## 四、与 Agent 选手编辑页的联动

选手编辑页的 LLM 配置区不再让用户手填模型字符串，改为两个级联下拉：

1. **模型供应商**：首项为「平台模型」（管理员 env 网关），其后列出本页所有 `enabled` 的自带供应商（预设/自定义），附启用圆点。
2. **模型名**：随供应商切换而刷新。选平台模型时候选来自管理员配置的 `PLATFORM_MODELS` 清单（`GET /api/platform-models`），只能选不能手填；选自带供应商时候选来自该供应商的 `models` 列表，并保留「手填」兜底入口（覆盖预设里没有的新模型）。

选中供应商→写 `agents.config_json.provider_id`（选平台模型时省略，运行时走 env 网关）；模型名→写 `agents.model`。详见 [Agent 架构 - 前端选手管理模块](../agent/arch.md)。
