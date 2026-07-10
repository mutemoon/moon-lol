---
name: champion-dev
description: 开发、实现英雄的技能、buff
---

# 背景

你是负责开发某个英雄的程序员，在 docs 中存放开发该英雄的信息：

- ./references/champions/{champion}/wiki.md
  英雄技能介绍

- ./references/champions/{champion}/history.md
  开发历史，记录开发过程，最重要的是记录开发时的决策的背景、过程、局限性等

- ./references/champions/{champion}/todo.md
  尚未解决的问题

- ./references/champions/{champion}/feedback.md
  用户提出的需求、问题

英雄的配置、定义存放在 assets/characters/{champion}/config.ron 中

播放动画时，使用的动画名字必须与 assets/characters/{champion}/animations/skin{N}.ron 中的定义一致，区分大小写

你需要在 crates/lol_champions/src/{champion} 中写代码，包括测试

# 在开始一切之前

1. 学习英雄 Riven 的测试代码范例（`crates/lol_champions/src/riven/tests.rs`），这是所有英雄测试的标准范式
2. 学习 docs/game/skill_test.md 文档
3. 学习 docs/game/skill.md 文档
4. 学习 docs/game/skill_impl.md 文档
5. 学习本英雄的 wiki 文档

# 技能系统

见 [skill.md](./references/skill.md)。

# 标准目录结构

```
crates/lol_champions/src/<hero>/
├── mod.rs           # Plugin + 标记 Component + on_xxx_skill_cast observer
├── q.rs             # Q 技能逻辑（复杂技能才拆分）
├── w.rs             # W 技能逻辑
├── e.rs             # E 技能逻辑
├── r.rs             # R 技能逻辑
├── passive.rs       # 被动技能
├── buffs.rs         # 英雄专属 buff/状态 Component
├── tests.rs         # 跨技能集成测试 + config/build_headless/build_render
├── q_tests.rs       # Q 技能测试
└── ...
```

简单英雄（纯单段技能）不需要拆分子文件，所有逻辑写在 mod.rs 即可。

# mod.rs 标准骨架

```rust
pub mod buffs;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

#[derive(Default)]
pub struct PluginXxx;

impl Plugin for PluginXxx {
    fn build(&self, app: &mut App) {
        app.add_observer(on_xxx_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Xxx"))]
#[reflect(Component)]
pub struct Xxx;

fn on_xxx_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_xxx: Query<(), With<Xxx>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_xxx.get(entity).is_err() { return; }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else { return; };

    match skill.slot {
        SkillSlot::Q => { /* ... */ }
        SkillSlot::W => { /* ... */ }
        SkillSlot::E => { /* ... */ }
        SkillSlot::R => { /* ... */ }
        _ => {}
    }
}
```

如果需要分段/重施，Query 改为：

```rust
q_skill: Query<(&Skill, &mut CoolDown, Option<&SkillRecastWindow>)>,
```

# tests.rs 标准骨架

```rust
#![cfg(test)]
use bevy::math::{Vec2, Vec3};
use crate::test_utils::*;

pub fn xxx_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "xxx",
        config_path: "characters/Xxx/config.ron",
        skin_path: "characters/Xxx/skins/skin0.ron",
        add_champion_plugin: |app| { app.add_plugins(crate::xxx::PluginXxx); },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<crate::xxx::Xxx>(name, HarnessMode::Headless, &xxx_config())
}

pub fn build_render(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<crate::xxx::Xxx>(name, HarnessMode::Render, &xxx_config())
}

#[test]
fn xxx_q_deals_damage() {
    let mut h = build_headless("xxx_q");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    h.cast_skill(0, Vec2::new(200.0, 0.0)).advance(0.5);
    assert!(h.health(enemy) < hp_before, "Q 应造成伤害");
    h.finish();
}
```

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

# 并行开发规则

每个英雄模块之间 **零依赖**，可以安全并行开发。

**必须遵守**：

1. 只修改 `crates/lol_champions/src/<分配的英雄>/` 下的文件
2. **不碰** `lib.rs`（Plugin 已注册好）
3. **不碰** `lol_core`（框架层）
4. **不碰** `test_utils.rs`（共享测试工具）
5. 只使用上面列出的原子动作和 Buff，不要自创新的 Event/Command
6. 每个 buff Component 使用 `#[require(Buff = Buff { name: "XxxBuff" })]` 宏

# 验收标准

cargo check -p lol_champions
cargo test -p lol_champions
