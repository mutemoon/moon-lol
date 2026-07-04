---
name: champion-dev
description: 开发、实现英雄的技能、buff
---

# 背景

你是负责开发某个英雄的程序员，在 docs 中存放开发该英雄的信息：

- docs/game/champions/{champion}/wiki.md
  英雄技能介绍

- docs/game/champions/{champion}/history.md
  开发历史，记录开发过程，最重要的是记录开发时的决策的背景、过程、局限性等

- docs/game/champions/{champion}/todo.md
  尚未解决的问题

- docs/game/champions/{champion}/feedback.md
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

# 技能系统架构

## 施法链路

```
输入 CommandAction
  → 统一管线 on_skill_cast（校验等级/蓝量/CD/阻塞）
    → EventSkillCast（派发给英雄 observer）
      → on_xxx_skill_cast（按 SkillSlot 分发，调用原子动作）
```

统一管线（`lol_core::skill::observers::on_skill_cast`）负责：
- 查找技能实体、校验技能等级、校验蓝量、校验冷却
- 重施窗口（`SkillRecastWindow`）激活时跳过冷却检查
- `AfterCast` 模式自动启动冷却，`Manual` 模式由英雄代码自行控制

## 可用原子动作

实现技能时 **只使用以下原子动作**，不要自创新的系统：

| 原子 | 用途 | 来源 |
|---|---|---|
| `CommandAnimationPlay` | 播放技能动画 | `lol_base::render_cmd` |
| `ActionDamage` + `ActionDamageEffect` | 范围伤害（Circle/Sector/Nearest/Annular） | `lol_core::action::damage` |
| `ActionDash` | 位移/冲刺（Fixed/Pointer） | `lol_core::action::dash` |
| `CommandDamageCreate` | 直接对单体造成伤害 | `lol_core::damage` |
| `CommandMissileCreate` | 发射飞弹 | `lol_core::missile` |
| `CommandAttachedFieldCreate` | 附着在施法者身上的伤害场 | `lol_core::missile` |
| `CommandAttackReset` | 重置普攻计时器 | `lol_core::attack` |
| `with_related::<BuffOf>(...)` | 添加 buff 关系实体 | `lol_core::base::buff` |

## 可用 Buff/Debuff

| Buff | 用途 | 来源 |
|---|---|---|
| `BuffShieldWhite` | 护盾 | `lol_core::buffs::shield_white` |
| `DebuffStun` | 眩晕 | `lol_core::buffs::cc_debuffs` |
| `DebuffSlow` | 减速 | `lol_core::buffs::cc_debuffs` |
| `BuffCastBlock` | 施法阻塞（技能施法期间禁止操作） | `lol_core::buffs::common_buffs` |
| `MovementBlock` | 移动阻塞 | `lol_core::movement` |

## 伤害形状

`DamageShape` 枚举支持：
- `Circle { radius }` — 以施法者为圆心的圆形范围
- `Sector { radius, angle }` — 扇形范围
- `Nearest { max_distance }` — 最近单体
- `Annular { inner, outer }` — 环形范围（内圈不命中）

## 目标过滤

`TargetFilter` 枚举：
- `All` — 所有敌方单位
- `Champion` — 仅敌方英雄

# 技能实现模式

## 模式 A：单段技能

一个 observer 调用一个原子动作，最简单。冷却模式 `AfterCast`。

```rust
// 示例：Garen E
commands.trigger(CommandAnimationPlay { entity, hash: "spell3".to_string(), repeat: false, duration: None });
commands.trigger(ActionDamage {
    entity,
    skill: skill_spell,
    effects: vec![ActionDamageEffect {
        shape: DamageShape::Circle { radius: 200.0 },
        damage_list: vec![TargetDamage {
            filter: TargetFilter::All,
            amount: "total_damage".to_string(),
            damage_type: DamageType::Physical,
        }],
        particle: Some(hash_bin("Garen_E_Hit")),
    }],
});
```

## 模式 B：分段/重施技能

使用 `SkillCooldownMode::Manual` + `SkillRecastWindow`。

```rust
let stage = recast.map(|w| w.stage).unwrap_or(1);
match stage {
    1 => {
        // 第一段行为
        commands.entity(skill_entity).insert(SkillRecastWindow::new(2, max_stage, recast_duration));
    }
    2 => {
        // 第二段行为（二段技能在此结束）
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        cooldown.timer = Some(Timer::from_seconds(cooldown.duration, TimerMode::Once));
    }
    _ => {
        // 终段（三段技能）
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        cooldown.timer = Some(Timer::from_seconds(cooldown.duration, TimerMode::Once));
    }
}
```

参考实现：
- 三段技能 → `crates/lol_champions/src/riven/q.rs`
- 二段技能 → `crates/lol_champions/src/leesin/mod.rs` (Q/W/E)
- 多段冲刺 → `crates/lol_champions/src/ahri/mod.rs` (R)

## 模式 C：自增益 + 重施（R 大招）

R1 添加 buff 增益 + 开启重施窗口，R2 释放大招效果 + 关闭窗口 + 启动冷却。

参考实现：`crates/lol_champions/src/riven/mod.rs` (R1/R2) + `crates/lol_champions/src/riven/r.rs`

## 模式 D：被动联动

技能施放时添加被动 buff，攻击/伤害事件消耗 buff 触发额外效果。

参考实现：
- 简单被动 → `crates/lol_champions/src/riven/passive.rs`（攻击消耗层数造成额外伤害）
- 复杂被动 → `crates/lol_champions/src/fiora/passive.rs`（方向标记系统）

## 模式 E：伤害命中联动

监听 `EventDamageCreate` 或 `EventAttackEnd`，给目标附加额外效果。

```rust
fn on_xxx_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_xxx: Query<(), With<Xxx>>,
) {
    if q_xxx.get(trigger.source).is_err() { return; }
    let target = trigger.event_target();
    // 叠 buff / 减速 / 回血 / 标记
    commands.entity(target).with_related::<BuffOf>(SomeDebuff::new(...));
}
```

参考实现：`crates/lol_champions/src/darius/mod.rs` (on_darius_damage_hit)

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
