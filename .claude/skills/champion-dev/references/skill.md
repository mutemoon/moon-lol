# 技能系统架构设计

- 架构类型：基于 `ECS` 与 `Observer` 的技能架构。
- 核心设计原则：摒弃静态配置，回归代码主导。由事件直接触发逻辑，由代码全权编排技能的动作、状态与时序。
- 行为与状态：技能的行为写在代码里，技能的状态挂载在 `ECS` 实体上。

# 英雄技能实现路径与框架

## 现有架构层次总览

- 统一管线：校验等级、资源、冷却以及阻塞并派发施法事件。详见 [skill-pipe.md](./skill-pipe.md)。
- 英雄观察者：监听施法事件，分发技能并管理状态。详见各英雄专属目录。
- 原子动作：动画、位移、伤害、飞弹、伤害场、Buff 等构建块。详见 [skill-action.md](./skill-action.md)。
- 联动观察者：伤害命中、攻击结束及位移结束的回调。
- 状态系统：提供通用计时 Buff、护盾、减速、眩晕、出血等状态。

## 典型实现模式

- 单段技能模式：详见 [skill-single-stage.md](./skill-single-stage.md)。
- 分段与二段技能模式：详见 [skill-multi-stage.md](./skill-multi-stage.md)。
- 被动联动模式：详见 [skill-passive.md](./skill-passive.md)。

## 标准模块目录结构

- `crates/lol_champions/src/<hero>/mod.rs`：存放 Plugin 注册、标记组件与施法监听。
- `q.rs`、`w.rs`、`e.rs`、`r.rs`：具体主动技能逻辑。
- `passive.rs`：被动技能逻辑。
- `buffs.rs`：英雄专属 Buff 与状态组件。
- `tests.rs`：跨技能集成测试。
- `q_tests.rs`：单元测试。

## 核心设计与组件

- 技能实体：技能作为 ECS 实体承担自身状态。详见 [skill-entity.md](./skill-entity.md)。
- 冷却模式：包含自动冷却和手动冷却。详见 [skill-cooldown.md](./skill-cooldown.md)。
