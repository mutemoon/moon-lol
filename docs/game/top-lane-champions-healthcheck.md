# 上单四姐妹五虎 · 代码体检报告

> 范围：camille / fiora / irelia / riven（四姐妹）+ aatrox / darius / mordekaiser / sett / volibear（五虎）
> 视角：组合优先（优先复用 `lol_core` 共享原语，而非 per-champion 手写）
> 日期：2026-07-14 ｜ 基线：9 英雄全部编译，`cargo test -p lol_champions` = 179 passed / 0 failed

## 基线结论

全绿，但「全绿」含水：至少 5 处 pass-for-wrong-reason / 空壳测试在反向断言 bug 或零断言。真实健康度远低于绿灯所示。问题分三层：框架级根因（修一处救多英雄）→ 跨英雄系统性模式 → 各英雄局部缺陷。

---

## 第一层：框架级根因（`lol_core`，最高杠杆）

### F1 · AP 伤害解析返回 0 —— 严重
`crates/lol_core/src/action/damage.rs:244-253`，`apply_damage_effects` 的 stat 闭包只对 `stat==2`（AD）返回 `damage.0`，其余（`stat==0` AP、`stat==1`）一律返回 `0.0`。所有走 `ActionDamage` + `get_skill_value` 的 AP 加成伤害**丢失全部 AP 加成**。
- 直接受害者：mordekaiser Q（丢 70% AP）、irelia 等含 AP 系数的技能。
- 修复：闭包补 `stat==0` 分支查 `AbilityPower`（或统一 stat→属性查询表）。

### F2 · `ActionDamageEffect`/`TargetDamage` 无 tag，`apply_damage_effects` 硬编码 `tag: None` —— 严重
`damage.rs:272` 永远传 `tag: None`，而 `CommandDamageCreate` 已有 `tag: Option<u32>` 字段。后果：英雄无法区分一次伤害来自哪个技能/形状。
- 直接受害者（全局伤害观察者误触）：
  - volibear `on_volibear_damage_hit`（`volibear/mod.rs:533-543`）：tag:None 通配，Q 命中 / 连锁闪电也触发被动减速。
  - camille `on_camille_damage_hit`（`camille/mod.rs:447-460`）：对 camille 的**所有**伤害（Q/E/R/平 A）施加 W 减速，wiki 仅 W 外圈减速。
  - darius `on_darius_damage_hit`（`darius/mod.rs:269-309`）：Q **内圈**也叠出血，wiki 仅外圈叠层。
  - fiora passive/R 观察者（`passive.rs:265`、`r.rs:185`）：未过滤 `With<Fiora>`，**友军**攻击带 Vital 的敌人也会触发要害击破（治疗/移速给友军）。
- 修复：给 `ActionDamageEffect` 加 `tag: Option<u32>`，`apply_damage_effects` 透传到 `CommandDamageCreate`；英雄给内圈/特定 effect 打 tag，观察者据此过滤。（fiora 的友军误触同时需加 `With<Fiora>` 源过滤。）

### F3 · `BuffMoveSpeed` / `BuffSelfHeal` / `BuffResist` 无处理系统 —— 死组件 —— 严重
`crates/lol_core/src/buffs/common_buffs.rs:28/45/64`。全仓库无任何系统读取它们的 `bonus_percent`/`amount`/`armor` 字段，也无通用 buff 生命周期系统让它们过期（对比 `BuffShieldWhite`/`BuffShieldMagic`/`BuffAttack`/CC debuff/on_hit 各有专属 tick/apply 系统）。aatrox 代码注释「移速增益由 BuffMoveSpeed 自管（到期自移除）」是**错误的**。
- 后果：挂上的移速/治疗/双抗 buff **既不生效也不过期**，且 buff 实体泄漏。
- 受影响英雄（9 个）：masteryi、volibear、hecarim、renekton、sylas、jax、aatrox、kayn、sett。
- 本批 9 英雄直接受影响：volibear（R MS）、sett（Q MS）、aatrox（R MS）。
- 修复：在 `lol_core` 为三者各加 tick+apply+despawn 系统（仿 `update_shield_white`）。

### F4 · `update_attack_state` 覆写 `buff_bonus_attack_speed` —— 中
`crates/lol_core/src/attack.rs:220-226`：每帧从 `BuffAttack` 聚合重算 `buff_bonus_attack_speed`，直接写 field 的 buff 会被覆盖。
- 受害者：volibear 被动（`volibear/mod.rs:571`）直接写 `buff_bonus_attack_speed` → 下一帧被清零。
- 修复：volibear 被动改走 `BuffAttack` 聚合（与框架一致），或框架为「直接写」留独立通道。

### F5 · 无共享治疗 / 周期 DoT 原语 —— 中
- `CommandHeal` 缺失：darius Q 外圈回血、fiora R 治疗光环均手写。
- `BuffPeriodicDamage` 缺失：darius 出血、mordekaiser DoT、fiora R heal 三处重复 `Timer::Repeating` tick + `CommandDamageCreate` 模式。
- 修复：提取 `BuffPeriodicDamage { source, tick_timer, duration_timer, formula, tag }` + `update_periodic_damage`；加 `CommandHeal`。

### F6 · hard-CC 不取消位移 —— 中
位移系统当前「暂停」而非「取消」dash。riven Q3 落点（`RivenQ3Pending`）若被硬 CC 打断会残留，且 `on_riven_dash_end` 未校验 `MovementSource::Dash`（`q.rs:92-99`，对比 fiora Q 正确校验），后续非 dash 位移结束会误触发 Q3 伤害。
- 修复：位移系统支持硬 CC 取消并广播取消事件；观察者补 `MovementSource::Dash` 校验。

---

## 第二层：跨英雄系统性模式

| # | 模式 | 表现 | 涉及英雄 |
|---|---|---|---|
| S1 | pass-for-wrong-reason / 空壳测试 | riven Wind Slash 断言两伤害**相等**（反向断言平均 HP bug）；darius W 零断言；darius E 断言**施放前**可施放；darius q.rs 硬编码 150/75 断言比值 2.0（数学恒等）；fiora E 手动触发 `EventAttackEnd` 绕过攻击管道 | riven, darius, fiora |
| S2 | 硬编码常量 vs spell data | darius（W 比例 `0.35+lvl*0.05`、R 范围 400 vs 475、R 每层 0.2）；riven（W 半径 100 vs 250、R 冷却、R 导弹路径小写 `riven`）；camille（W 减速 0.6 vs 0.8、E recast 4.0 vs 1.0）；aatrox（Q 甜点距离分支）；fiora（R `[0.03,0.035,0.04]` 数组） | 多数 |
| S3 | 手管冷却 vs AfterCast 管线 | riven R2 手动重置冷却为满时长（`mod.rs:258-260`），管线本应自管；R1 已起算，R2 不该重置 | riven |
| S4 | 无蓝量 / 冷却测试 | darius 全技能 `give_mana(1000)` 使蓝量恒满无法检测消耗；无任何技能测冷却进入/恢复 | darius（+疑似其他） |
| S5 | 全局伤害观察者不过滤源 | 见 F2 | volibear, camille, darius, fiora |
| S6 | 手写 AoE 几何 vs `collect_targets_in_shape`/`is_in_shape` | riven W 眩晕手写循环（伤害/眩晕双筛选路径，半径改一处即分叉）；fiora W 反刺手写矩形投影；aatrox Q 手写甜点距离分支（应 `Circle{200}`+`Annular{200,300}`） | riven, fiora, aatrox |
| S7 | 死代码 / 过期 todo | darius（`DariusQInnerDamage` pub 死结构、todo.md 标已实现项为 TODO）；riven（`PluginRivenQ/W/E/R` 未注册且 Q 重复注册观察者、`let _ = entity;`、`build_render`、unused imports）；fiora（`PluginFioraPassive`、`_parry_duration` 读后丢弃、`.max(2.0)` 冗余） | darius, riven, fiora |
| S8 | CC 框架违规 | fiora W 手动 `insert((ImmuneToCC, MovementBlock, CastBlock))`（`w.rs:92-94`），违反 skill-cc.md 禁令；残留 CC 到期会被 `sync_cc_markers` 覆盖清除导致 W 锁提前失效 | fiora |
| S9 | 阶段 4.2 stub | camille E 二段钩锁纯 stub（E1 无导弹/墙体/附着，E2 无眩晕、距离/速度错）；camille R 区域锁定/击退/不可选中未实现 | camille |

---

## 第三层：各英雄优先修复清单

> P0=功能错误/框架违规 ｜ P1=组合偏离/数值错误 ｜ P2=清理/补测

### aatrox
| 级 | 项 | 位置 |
|---|---|---|
| P1 | Q 甜点改为 `Circle{200}`+`Annular{200,300}` 组合，删手写距离分支 | `aatrox/mod.rs:243-273` |
| P1 | R 额外 AD 改走 stat-modifier buff，勿直接 `Damage.0 +=` | `aatrox/mod.rs:530-534` |
| P2 | R 移速 buff 受 F3 影响（当前死组件，不生效） | `aatrox/mod.rs:537` |

### camille
| 级 | 项 | 位置 |
|---|---|---|
| P0 | `on_camille_damage_hit` 仅对 W 外圈减速（依赖 F2 tag） | `camille/mod.rs:447-460` |
| P1 | W 补内/外圈划分 + maxHP 伤害 + 治疗（双 effect + exclude） | `camille/mod.rs` |
| P1 | W 减速 0.6→读 ron 0.8 | — |
| P1 | Q 补移速、Q2 补真伤转换 | — |
| P1 | E recast 窗口 4.0→读 ron 1.0 | — |
| P2 | E 二段钩锁 / R 区域锁定实现（阶段 4.2） | `camille/mod.rs:372-403` |
| P2 | `clear_camille_on_hit` 轮询改 `Buffs` 组件查询（同文件 R 已正确示范） | — |

### darius
| 级 | 项 | 位置 |
|---|---|---|
| P0 | R 伤害类型 `Physical`→`True`（wiki 明确真实伤害） | `darius/mod.rs:261` |
| P0 | Q 内圈不叠出血（依赖 F2 tag） | `darius/mod.rs:269-309` |
| P1 | W 伤害比例读 spell 公式（`on_darius_w` 补 `res_spells`），删 `0.35+lvl*0.05` | `darius/mod.rs:232` |
| P1 | Q 内外圈伤害区分（内圈约 50%） | `darius/q.rs:66-90` |
| P1 | R 范围 400→读 475；R 每层 0.2→读 dataValue | `darius/mod.rs:56,58` |
| P2 | 补 Q 外圈回血、W 击杀返蓝减 CD、R 斩杀重置/不可选中 | — |
| P2 | 删 `DariusQInnerDamage`；更新 todo.md | `darius/q.rs:21-31` |
| P2 | 修空壳测试（W 零断言、E 断言前置）；删 q.rs 常量抄写测试 | `tests.rs:100-128`、`q.rs:119-133` |

### fiora
| 级 | 项 | 位置 |
|---|---|---|
| P0 | W 手挂 CC 标记改 `BuffCastBlock::new(dur)` + ImmuneToCC 作 buff+`On<Remove>` | `fiora/w.rs:92-94` |
| P0 | passive/R 观察者加 `With<Fiora>` 源过滤；R 光环挂 Fiora 而非 `trigger.source` | `passive.rs:265`、`r.rs:185` |
| P1 | W 招架区分硬/软控（减速不应触发眩晕反刺） | `fiora/w.rs:232-249` |
| P1 | 提取被动方向选择共享函数（24 行重复） | `passive.rs:174-198/306-330` |
| P1 | R 等级数组改 `get_skill_data_value` | `fiora/r.rs:21-23` |
| P2 | passive `.unwrap()`→`else continue` 防 panic | `passive.rs:144,226` |
| P2 | `_parry_duration` 传参生效；删 `PluginFioraPassive`/`.max(2.0)` | `mod.rs:116,121` |
| P3 | W 反刺复用 `is_in_shape` | `fiora/w.rs:158-199` |

### irelia
| 级 | 项 | 位置 |
|---|---|---|
| P1 | Q 改用 `DashMoveType::Entity{target,stop_radius}`（源码注释称本为 irelia Q 设计），当前用 `Pointer{max:250}` | `irelia/q.rs:87-94` |

### mordekaiser
| 级 | 项 | 位置 |
|---|---|---|
| P0 | Q AP 加成丢失（依赖 F1） | — |

### riven
| 级 | 项 | 位置 |
|---|---|---|
| P0 | R2 删手动重置冷却，仅 `remove::<SkillRecastWindow>()` | `riven/mod.rs:258-260` |
| P0 | `on_riven_dash_end` 补 `MovementSource::Dash` 校验（依赖 F6） | `riven/q.rs:92-99` |
| P1 | W 半径 100→读 ron 250 | `riven/w.rs:12` |
| P1 | W 改 `ActionDamage` 替代 `CommandAttachedFieldCreate`，统一目标收集 | `riven/w.rs:32-39` |
| P1 | 修 pass-for-wrong-reason 测试（Wind Slash 断言改为 `low > full` 或重命名） | `r_tests.rs:75-114` |
| P2 | 删 `PluginRivenQ/W/E/R`、`let _ = entity;`、`build_render`、unused imports | 多处 |
| P2 | R buff 回退改存 `bonus_ad`/`bonus_range`，勿用反推公式 | `r.rs:120-121` |
| P2 | `info!`→`debug!`；E 护盾 `DebugSphere`→正式视觉；R 冷却/导弹路径数据驱动 | `passive.rs:115`、`e.rs:43`、`mod.rs:237` |

### sett
| 级 | 项 | 位置 |
|---|---|---|
| P0 | R 缺 `ActionDash`，违反位移框架规范 | `sett/mod.rs:424-473` |

### volibear
| 级 | 项 | 位置 |
|---|---|---|
| P0 | R 落地 AoE 应在 `EventMovementEnd` 结算，而非施放时 | `volibear/mod.rs:449-511` |
| P0 | `on_volibear_damage_hit` 仅对普攻/特定源减速（依赖 F2 tag） | `volibear/mod.rs:533-543` |
| P1 | 被动 AS 改走 `BuffAttack` 聚合（受 F4 覆写） | `volibear/mod.rs:571` |

---

## 第四层：测试质量缺口汇总

1. **pass-for-wrong-reason（最高优先）**：riven Wind Slash 反向断言 bug；darius W 零断言 / E 断言前置 / q.rs 数学恒等。这些绿灯是虚假安全感。
2. **无蓝量测试**：darius 全技能 `give_mana(1000)` 恒满，无法检测消耗（skill-test.md 要求优先断言蓝量扣减）。
3. **无冷却测试**：无技能验证冷却进入/恢复（riven R2 重置 bug 正因此无测试捕获）。
4. **手写常量断言 vs 读配置**：fiora W/E/R、darius 等多处断言硬编码值，ron 改动即脆裂。
5. **绕过管道**：fiora E 手动 `trigger(EventAttackEnd)` 跳过攻击 windup/damage。
6. **区间缺口**：riven W 只测 100（内）/420（外），无 100–250 区间目标，半径修正后无法发现。
7. **覆盖缺口**：W 伤害+眩晕同批命中一致性、Q 内外圈伤害差、5 层 DoT 倍率、目标筛选（盟友不命中）均无测试。

---

## 建议执行顺序

> 原则：先框架后英雄（框架根因是多英雄放大器）；先修功能错误再补测；每步 TDD。

**Phase A · 框架根因（救多英雄）**
1. F1 AP 解析 —— 补 `stat==0` 分支（+ mordekaiser Q 回归测试）
2. F2 tag 透传 —— `ActionDamageEffect.tag` + `apply_damage_effects` 透传（+ 内圈不叠出血/仅 W 减速回归测试）
3. F3 死组件 —— `BuffMoveSpeed`/`BuffSelfHeal`/`BuffResist` tick+apply+despawn 系统（+ aatrox/sett/volibear MS 生效测试）
4. F4 AS 覆写 —— volibear 被动改 `BuffAttack` 聚合
5. F6 dash 取消 —— 位移硬 CC 取消 + `MovementSource::Dash` 校验

**Phase B · 各英雄 P0 功能错误**
- darius R True / Q 内圈不叠层；fiora W CC 框架化 + 源过滤；riven R2 冷却 + dash 源校验；sett R 补 dash；volibear R 落地时机 + 减速源过滤；mordekaiser Q AP（随 F1）

**Phase C · 系统性模式**
- S1 修虚假测试 → S4 补蓝量/冷却测试 → S2 常量数据驱动 → S6 AoE 复用 → S7 死代码/todo 清理

**Phase D · 不完整功能（阶段 4.2 等）**
- camille E 钩锁 / R 区域；darius Q 回血/W 返蓝/R 重置；riven W per-target HP；fiora R 治疗光环

**Phase E · 框架原语提取**
- F5 `BuffPeriodicDamage` / `CommandHeal`（消除三处 DoT/heal 重复）
