export interface Post {
  title: string;
  desc: string;
  date: string;
  tag: string;
  content: string;
}

export const posts: Record<string, Post> = {
  architecture: {
    title: "工程架构",
    desc: "Moon LoL 的高层系统设计：Rust Core 与 Web Frontend.",
    date: "2025.11.28",
    tag: "ARCHITECTURE",
    content: `
# 工程架构

\`\`\`mermaid
graph TD
    subgraph "Rust Core (Bevy)"
        App[Bevy App]
        ECS[ECS World]
        Plugins[Core Plugins]
    end

    subgraph "Frontend (Vue)"
        Web[Vue App]
        Render[Renderer]
    end

    App -->|Update| ECS
    ECS -->|State/Events| Web
    Web -->|Input| ECS
\`\`\`

## 核心设计理念

Moon LoL 采用高性能的分层架构。 系统的核心是基于 Rust 的 Bevy 引擎，它提供了极致的性能和类型安全的 ECS 架构。

## 核心支柱

- **Rust Core (Bevy):** 负责所有游戏逻辑、物理模拟和状态管理。利用 ECS 模式实现高并发和内存友好的数据处理。
- **Frontend (Vue 3):** 通过 WebSocket 或 HTTP 轮询获取游戏状态，利用 WebGL/Canvas 进行实时渲染，主要用于调试和观察。

## 高性能通信

前端与后端通过高效的数据协议进行通信，确保在渲染大量实体时依然保持流畅的帧率。
    `,
  },
  "data-flow": {
    title: "数据流转",
    desc: "从 Bevy ECS 到 Web 前端的数据管线。",
    date: "2025.11.28",
    tag: "DATA",
    content: `
# 数据流转

\`\`\`mermaid
sequenceDiagram
    participant Bevy as Bevy System
    participant Frontend as Vue Client

    loop Game Loop
        Bevy->>Bevy: Run Systems (Movement, Attack, etc.)
        Bevy->>Bevy: Update State
    end
    
    loop Render Loop
        Frontend->>Bevy: Request State
        Bevy-->>Frontend: JSON Snapshot
        Frontend->>Frontend: Render Frame
    end
\`\`\`

## 游戏循环

系统的数据流转完全由 Bevy 的 Schedule 驱动。

## 核心循环 (Update)

Bevy 引擎每帧执行一次 Update Schedule：

1. 处理输入事件 (Input Events)。
2. 运行所有的 Systems (移动、攻击、伤害计算)。
3. 更新组件状态 (Components)。

## 渲染循环 (Render)

前端 Vue 应用并不控制游戏逻辑，它只是一个观察者。它通过 API 定期拉取最新的游戏状态快照（通常是 JSON 格式），然后更新 DOM 或 Canvas。这种解耦设计允许后端以最大速度运行，而前端只负责可视化。
    `,
  },
  ecs: {
    title: "ECS 组件与系统",
    desc: "深入解析游戏核心逻辑：插件系统与实体组件设计。",
    date: "2025.11.28",
    tag: "CORE",
    content: `
# ECS 组件与系统

## 一切皆实体 (Everything is an Entity)

在 Moon LoL 中，无论是英雄、小兵、防御塔，还是飞行的技能弹道，本质上都是 ECS 世界中的一个 Entity。 它们的行为差异仅仅来自于它们挂载了不同的 Component 集合。

## 核心组件 (Components)

- **Health:** 存储当前生命值和最大生命值。当生命值归零时，触发死亡逻辑。
- **Controller:** 标记该实体受外部控制（如 RL Agent 或 玩家输入）。
- **Transform:** Bevy 内置组件，定义实体在 3D 空间中的位置、旋转和缩放。
- **Skill:** 管理技能冷却、等级和施放状态。

## 系统插件 (Plugins)

我们将功能模块化为 Bevy Plugins。每个 Plugin 注册相关的 Systems 和 Resources。 以下是当前系统注册的所有核心插件：

- PluginFioraPassive
- PluginFioraE
- PluginFioraR
- PluginBarrack
- PluginChampion
- PluginCharacter
- PluginDebugSphere
- PluginFiora
- PluginHwei
- PluginMinion
- PluginTurret
- PluginAction
- PluginAnimation
- PluginAttack
- PluginAttackAuto
- PluginAggro
- PluginBase
- PluginCamera
- PluginController
- PluginDamage
- PluginGame
- PluginLife
- PluginLifetime
- PluginMap
- PluginMissile
- PluginMovement
- PluginNavigaton
- PluginParticle
- PluginResource
- PluginRotate
- PluginRun
- PluginSkill
- PluginSkin
- PluginState
- PluginUI
    `,
  },
};
