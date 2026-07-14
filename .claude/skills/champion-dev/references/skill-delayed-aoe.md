# 延迟范围伤害系统

- 适用场景：施法后有显式延迟、需要地面警示器（telegraph）再结算伤害的范围技能，如落雷、蓄力锥形、砸地、内外圈分区的延迟判定。
- 核心思想：一条命令 `ActionDelayedDamage`（`lol_core::action::delayed_damage`）同时驱动伤害逻辑与三阶段视觉生命周期，消除各英雄手写 Timer + 查询 + 画圈的重复。
- 不适用：瞬发范围伤害仍用 `ActionDamage`（如锐雯 W）；投射物用 `CommandMissileCreate`（如锐雯 R），不进本通路。

# 三阶段生命周期

命令派发后在世界空间生成一个指示器实体（挂 `DelayedDamageInstance` + `AoEVisual`），由 `update_delayed_damage`（FixedUpdate）推进三阶段：

- **Delay**：指示器出现，可生长/脉动/淡入，不造成伤害。`delay_timer` 计时。
- **Impact**：延迟结束瞬间结算伤害（复用 `apply_damage_effects`），指示器缩放到 `impact_burst_scale` 并满 alpha。
- **Fade**：指示器从爆发尺寸缩到 0、alpha 从 1 淡到 0，到期 `despawn`。

伤害在施法瞬间快照的位置/朝向上结算，不跟随施法者移动。

# 命令字段

`ActionDelayedDamage`：

- `entity`：施法者。
- `skill` / `skill_level`：技能资产句柄与等级（用于伤害公式与 dataValue 读取）。
- `delay`：延迟秒数。来源见下文「延迟时间来源」。
- `point`：施法目标点（XZ）。方向性形状的朝向由 `point - 施法者位置` 决定，重合时退回 Transform 面向方向（与 Darius E 锥形一致）。
- `effects`：伤害特效列表，每条含 `shape` + `damage_list` + `exclude`。
- `indicator`：视觉配置，见下文。
- `origin`：伤害原点模式，见下文。

# 原点模式 AoEOrigin

- `Caster`（默认）：以施法者为中心。矩形/扇形从施法者向外，圆形以施法者为圆心。如万豪 W、青钢影 W。
- `CastPoint`：以施法目标点为中心（地面靶向 AoE）。如狗熊 E 落雷在指针处生成。

# 伤害变异的组合原语

技能的伤害差异应映射到以下三个框架原语，而非手写分支：

- **空间分区**（同一形状内不同区域伤害不同）-> `ActionDamageEffect.exclude: Vec<DamageShape>`。被 `exclude` 命中的目标从该 effect 中排除，交给另一个 effect 处理。如万豪 W：外圈 75° 扇形物理 `exclude:[中心 30° 扇形]`，另起一条中心 30° 扇形真伤 effect。
- **聚合条件**（仅当目标孤立/唯一时加成）-> `TargetDamage.modifier: DamageModifier::Isolation { scalar_data_value }`。从 spell dataValues 读标量乘算伤害，仅当该 effect 命中唯一目标时生效。如铁男 Q 孤立加成。
- **多维形状**（多个独立几何同时判定）-> 多个 `ActionDamageEffect`。如诺手 Q：内圈 `Circle{150}` + 外圈 `Annular{150,350}` 两条 effect。

`exclude` 是每个 effect 独立的；多 effect 之间互不影响各自的目标收集。

# 视觉指示器配置 AoEIndicator

- `color`：延迟阶段基础颜色（RGB；alpha 由生命周期系统单独驱动）。
- `pulse`：延迟阶段是否脉动（scale 微胀缩 + alpha 闪烁）。
- `grow_from_zero`：是否从零尺寸/零 alpha 生长到完整（如狗熊 E 从天而降的落雷）。
- `impact_burst_scale`：爆发阶段缩放倍数（相对完整尺寸）。
- `fade_duration`：褪去持续秒数。

mesh 由 `lol_render` 的 `PluginAoEVisual` 按 `DamageShape` 构建（Circle/Sector/Annular/Rectangle/Nearest），双面 winding 避免背面剔除。`Transform.scale` 只承载生命周期相位因子，不乘半径。

# 延迟时间来源

- 施法帧：`delay_from_cast_frame(spell_obj)` = `castFrame / 30.0`。如万豪 W castFrame 5.265/30、狗熊 E castFrame 25/30、诺手 Q castFrame 7.5/30。
- 蓄力时长：读 spell dataValues，如 `get_skill_data_value(spell_obj, "ChargeDuration", level)`，青钢影 W 用此作为 `delay`。

# 典型实现参考

- 铁男 Q：`Rectangle { width, length, start_distance }` + `DamageModifier::Isolation`，从 ron 读 MaceStartDistance/MaceLength/RectangleWidth/IsolationScalar。
- 万豪 W：`AoEOrigin::Caster`，双 effect —— 外圈 `Sector{350,75°}` 物理 `exclude:[内圈]` + 内圈 `Sector{350,30°}` 真伤；castFrame 5.265/30；金色脉动。
- 狗熊 E：`AoEOrigin::CastPoint`，`Circle{radius}` 魔法；castFrame 25/30；`grow_from_zero: true`。
- 青钢影 W：`AoEOrigin::Caster`，`Sector{radius,angle}` 物理；delay 读 ChargeDuration 蓄力；脉动。
- 诺手 Q：双形 `Circle{150}` 内圈 + `Annular{150,350}` 外圈，均物理；castFrame 7.5/30。

相关：[原子动作](./skill-action.md) 的 `ActionDamage` 为瞬发版本；[测试实践](./skill-test.md)。
