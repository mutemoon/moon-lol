---
name: champion-dev
description: 开发、实现英雄的技能、buff
---

# 英雄开发

## 背景

你是负责开发某个英雄的程序员，在 `crates/lol_champions/src/{champion}/` 中存放了开发该英雄的文档和实现代码：

- wiki.md: 英雄技能介绍
- history.md: 开发历史，记录开发过程，最重要的是记录开发时的决策的背景、过程、局限性等
- todo.md: 尚未解决的问题
- feedback.md: 用户提出的需求、问题

英雄的配置、定义存放在 `assets/characters/{champion}/config.ron` 中

播放动画时，使用的动画名字必须与 `assets/characters/{champion}/animations/skin{N}.ron` 中的定义一致，区分大小写

你需要在 `crates/lol_champions/src/{champion}/` 中写代码，包括测试

## 实现技能 Feature 流程（TDD 强制）

实现一个新技能或修改技能行为时，必须遵循 TDD（测试驱动开发）：

1. **先在 `crates/lol_champions/src/{champion}/tests.rs` 写测试**
   - 使用 `ChampionTestHarness::build::<Champion>(name, HarnessMode::Headless, &config)`
   - 至少覆盖：施法成功、冷却生效、消耗扣除、目标筛选、多段/重施窗口
2. **运行测试确保失败**（红）
3. **再实现技能逻辑**，让测试通过（绿）
4. **重构**（如有必要）

测试写完后才允许实现或修改技能代码。

## 单元测试 Debug 流程

**首先要通过日志定位问题，不要一上来就盲目添加修复代码**

- 在代码执行路径上添加详细的 info! 的中文日志
- 如果还是找不到问题就继续添加更多日志输出
- 问题解决后日志可以留下，只需要将 info! 改为 debug!

## 验收标准

cargo check -p lol_champions
cargo test -p lol_champions

# 英雄技能系统

## 架构设计

- 架构类型：基于 `ECS` 与 `Observer` 的解耦技能架构。
- 核心设计原则：回归代码主导逻辑，施法校验由统一管线处理，技能的具体动作与状态变迁完全由代码观察者系统控制。
- 行为与状态：技能的运行逻辑以系统形式写在代码中，运行时状态挂载在 `ECS` 实体上。

## 英雄技能开发流程与核心组件

### 统一管线

- 管线定位：通用施法前置校验与扣减资源，校验通过后派发事件。详见 [skill-pipe.md](./references/skill-pipe.md)。

### 观察者与联动

- 英雄观察者：通过监听施法事件分发技能逻辑。
- 联动观察者：监听伤害造成与普攻结束等衍生事件。

### 原子动作与状态

- 原子动作：播放动画、冲刺位移、范围伤害场、发射飞弹等。详见 [skill-action.md](./references/skill-action.md)。
- 状态系统：提供护盾、眩晕、减速等通用效果。

## 英雄标准目录结构

- 路径规则：英雄目录位于 `crates/lol_champions/src/<hero>/`。
- `mod.rs`：注册插件与核心施法监听器。
- 技能文件：`q.rs`、`w.rs`、`e.rs`、`r.rs` 存放对应主动技能逻辑。
- `passive.rs`：存放被动技能逻辑。
- `buffs.rs`：声明英雄特有的状态组件。
- `tests.rs`：集成测试与测试配置。

## 数值读取机制

- 计算公式值：通过 `get_skill_value` 动态解析属性加成公式。详见 [skill-value.md](./references/skill-value.md)。
- 配置原始值：通过 `get_skill_data_value` 读取特定等级原始数组。

## 典型技能参考文档

- 单段技能：[skill-single-stage.md](./references/skill-single-stage.md)
- 分段重施技能：[skill-multi-stage.md](./references/skill-multi-stage.md)
- 被动联动技能：[skill-passive.md](./references/skill-passive.md)
- 飞弹与投射物：[skill-missile.md](./references/skill-missile.md)
- 位移系统：[skill-dash.md](./references/skill-dash.md)
- 冷却控制机制：[skill-cooldown.md](./references/skill-cooldown.md)
- 测试与调试实践：[skill-test.md](./references/skill-test.md)
- 搜索研究英雄技能的详细介绍：[skill-research.md](./references/skill-research.md)
