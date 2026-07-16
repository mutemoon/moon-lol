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
- 延迟范围伤害（地面警示器）：[skill-delayed-aoe.md](./references/skill-delayed-aoe.md)
- 位移系统：[skill-dash.md](./references/skill-dash.md)
- 控制系统（CC）：[skill-cc.md](./references/skill-cc.md)
- 动画系统：[skill-animation.md](./references/skill-animation.md)
- 冷却控制机制：[skill-cooldown.md](./references/skill-cooldown.md)
- 测试与调试实践：[skill-test.md](./references/skill-test.md)
- 搜索研究英雄技能的详细介绍：[skill-research.md](./references/skill-research.md)

## 开发经验积累

### 数据驱动倾向：一切可配置的数值优先从 RON 读取

团队犯过最多的错误就是写硬编码常量替代 RON 数据。每次新技能必须遵循以下优先级：

1. **calculations（公式值）** → `get_skill_value(spell_obj, name, level, |stat| ...)`
   - 适合：基础伤害、加成系数等需要与攻击力/法强联动的数值
   - 例如：锐雯风斩 `min_damage` 公式 = `MinBase + total_AD * 0.6`

2. **dataValues（数据值）** → `get_skill_data_value(spell_obj, name, level)`
   - 适合：持续时间、百分比、固定数值等按等级查表的值
   - 例如：菲奥娜 R 的 `MarkDuration`、`PercentMS`、`VitalPercent`、`HealPerSecond`、锐雯 R 的 `Duration`、`PercentBonusAD`、`TooltipAttackRange`

3. **effectAmounts（效果值）** → 通过 `get_skill_value` 传 `1.0` 作为 stat 提取倍率
   - 适合：需要从 RON 公式中提取纯倍率的场景
   - 技巧：对 `stat(total_AD) * effectValue(N)` 这样的公式，传 `|stat| if stat == 2 { 1.0 } else { 0.0 }` 即得到 `effectValue[N]` 原始值
   - 例如：德莱厄斯 W 的总伤害倍率 = `get_skill_value(..., "empowered_attack_damage", ..., |stat| if stat == 2 { 1.0 } else { 0.0 })`，然后 `ratio = total_mult - 1.0`（减去基础普攻 100% AD）

4. **cooldownTime（冷却时间）** → 直接从 `spell_obj.spell_data.cooldown_time` 读取
   - `get_skill_data_value` 不能读 cooldownTime，因为它是 spell 顶层字段而非 dataValues
   - cooldownTime 数组索引 0 往往是 nil 占位，真正的值从索引 1 开始
   - 读取方式：`spell_obj.spell_data.as_ref().and_then(|d| d.cooldown_time.as_ref()).and_then(|v| v.get(skill.level).copied()).unwrap_or(default)`
   - 注意索引用 `skill.level` 而非 `skill.level - 1`

5. **保留为硬编码常量的情况**（仅限以下两类）：
   - **手感常数**：前摇延迟、缓冲区时间、触发阈值等游戏手感调优值。它们不来自 Riot 的数据导出，而是开发者的手感调优。例如：`FIORA_R_ACTIVE_DURATION = 0.5`（R 生效前延迟）、`VITAL_R_TIMEOUT = 1.5`（要害过期红色闪烁预警）
   - **命中框常数**：圆形 AoE 半径、Sweet Spot 起始距离等空间碰撞参数。它们决定的是技能"打不打得中"而非"打多少伤害"。例如：`AATROX_Q_RADIUS = 300.0`、`AATROX_Q_SWEET_SPOT_MIN = 200.0`
   - **判断标准**：如果 RON 中能找到对应字段 → 读 RON；如果 RON 中没有且是游戏设计/调优参数 → 硬编码；如果 RON 中有但值明显不对 → 以 RON 为准并检查配置

### Buff 属性修改的逆操作模式

技能开启时直接修改属性（伤害/攻击距离/速度），到期后必须准确恢复原始值。

**错误做法**：在恢复代码中硬编码回退值
```rust
// ❌ 硬编码 0.25 和 75.0，一旦 RON 修改就不同步
damage.0 = damage.0 * (1.0 - 0.25 / 1.25);
attack.range -= 75.0;
```

**正确做法**：在 Buff 组件中存储应用时的实际值，到期后按存储值恢复
```rust
// ✅ Buff 组件中记录原始加成
pub struct BuffRivenR {
    pub timer: Timer,
    pub bonus_ad_ratio: f32,  // 从 RON PercentBonusAD 读取的实际值
    pub bonus_range: f32,     // 从 RON TooltipAttackRange 读取的实际值
}
// ✅ 恢复时用存储值
damage.0 /= 1.0 + buff_r.bonus_ad_ratio;
attack.range -= buff_r.bonus_range;
```

### RON 数值索引规则

| 字段 | 索引方式 | 说明 |
|------|---------|------|
| `dataValues.values` | `level - 1` | `get_skill_data_value` 内部处理 |
| `effectAmounts.values` | `level - 1` | 通过 formula 的 effectIndex 间接访问 |
| `cooldownTime` | `level`（⚠️ 不是 level-1） | 索引 0 通常是 nil 占位，真正的各级数值从索引 1 开始 |
| `castRange` / `castRadius` | `level - 1` | `get_skill_cast_radius` 内部处理 |

### 测试必须检验的维度

- **施法成功**：技能管线应当通过（`SkillCastResult::Started`）
- **效果生效**：伤害/Debuff/Buff 已正确挂载
- **冷却扣除**：`can_cast(slot_index)` 变为 false
- **消耗扣除**：蓝量/能量正确减少
- **目标筛选**：友军不受伤害，敌军受到伤害
- **多段重施**：重施窗口正确，第二次施法走不同分支
- **Buff 过期**：属性恢复回初始值
- **数据驱动正确性**：所有从 RON 读取的数值应与硬编码时代的测试期望一致
