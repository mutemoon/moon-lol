---
name: champion-dev
description: 开发、实现英雄的技能、buff
---

# 背景

你是负责开发某个英雄的程序员，在 docs 中存放开发该英雄的信息：

- docs/champions/{champion}/wiki.md
  英雄技能介绍

- docs/champions/{champion}/history.md
  开发历史，记录开发过程，最重要的是记录开发时的决策的背景、过程、局限性等

- docs/champions/{champion}/todo.md
  尚未解决的问题

- docs/champions/{champion}/feedback.md
  用户提出的需求、问题

英雄的配置、定义存放在 assets/characters/{champion}/config.ron 中

播放动画时，使用的动画名字必须与 assets/characters/{champion}/animations/skin{N}.ron 中的定义一致，区分大小写

你需要在 crates/lol_champions/src/{champion} 中写代码，包括测试

# 在开始一切之前

1. 学习英雄 Riven 的测试代码范例（`crates/lol_champions/src/riven/tests.rs`），这是所有英雄测试的标准范式
2. 学习 docs/skill_test.md 文档
3. 学习 docs/skill.md 文档
4. 学习本英雄的 wiki 文档

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

cargo test -p lol_champions
