# 控制系统（CC）

- 设计主旨：CC 效果以「独立 Buff 实体 + 角色轻量标记」解耦。Buff 实体持有倒计时与具体类型；角色只挂极速查询的标记；各系统（移动/施法/选中）只认标记、互不干涉；净化按标签批量销毁 Buff，标记随 Buff 死亡自动清。
- 四原则：①标记在角色，逻辑在 Buff ②Buff 自己管自己（生成加标记、消亡删标记）③系统只认标记 ④净化即杀人。
- 核心约束：**不要手动 `insert(MovementBlock/CastBlock/MovementSlow)`**。这些标记由 `PluginCc` 的观察者从 CC Buff 自动桥接。英雄代码只负责生成/销毁 Buff 实体。

# CC Debuff 组件

均位于 `lol_core::buffs::cc_debuffs`，`#[require(Buff, ControlTag)]`，带 `Timer`，由全局 `update_cc_buff_timers`（FixedUpdate）tick，过期自动 despawn。

- `DebuffStun::new(duration)`：眩晕，禁移动 + 禁施法。
- `DebuffSlow::new(percent, duration)`：减速，`percent` 0.0-1.0，取角色身上最强值。`MovementSlow{percent}` 标记接入 `movement.rs`，`speed *= 1.0 - slow.percent`。
- `DebuffSilence::new(duration)`：沉默，仅禁施法。
- `DebuffFear::new(duration)`：恐惧，禁移动 + 禁施法。
- `DebuffKnockup::new(duration)`：击飞，仅禁施法（**不加 MovementBlock**，保留击退位移通路，见 [位移系统](./skill-dash.md) 的 `CommandKnockback`）。

# 自施法锁 BuffCastBlock

- 位于 `lol_core::buffs::common_buffs`，`BuffCastBlock::new(duration)`。
- 非.ControlTag、**不可被净化**（自身技能 windup 不受免控影响）。
- 同样由 `PluginCc` 观察者桥接 `MovementBlock`+`CastBlock`，过期自动 despawn。

# CC -> 标记映射

| Buff | MovementBlock | CastBlock | MovementSlow | ControlTag(可净化) |
|---|---|---|---|---|
| DebuffStun | ✓ | ✓ | | ✓ |
| DebuffSilence | | ✓ | | ✓ |
| DebuffFear | ✓ | ✓ | | ✓ |
| DebuffSlow | | | ✓(最强%) | ✓ |
| DebuffKnockup | ✗(保留位移) | ✓ | | ✓ |
| BuffCastBlock | ✓ | ✓ | | ✗(不可净化) |

# 标记组件（系统只认这些）

- `MovementBlock`：移动系统 `update_path_movement` 用 `Without<MovementBlock>` 跳过，禁寻路移动。
- `CastBlock`：施法管线 `on_skill_cast` 用 `With<CastBlock>` 拦截，禁释放主动技能。
- `MovementSlow { percent }`：移动系统按 `1 - percent` 降速。
- `ImmuneToCC`：CC 施加观察者据此阻断新 CC（见下）。

# 标记桥接机制（PluginCc，挂于 PluginCore）

- `ControlTag`：所有可净化 CC debuff 的 `#[require]`。`CommandCleanse` 据此批量销毁。
- `sync_cc_markers(char, excluding, &各 CC Query, &BuffOf Query)`：遍历指向 `char` 的 Buff 实体（排除 `excluding`），按映射表幂等取并集 set/clear 三类标记。一处实现，所有观察者调用。
- `on_add_control`（`On<Add, ControlTag>`）：经 `BuffOf.0` 找角色；若角色有 `ImmuneToCC` 则 despawn 该 Buff（CC 不沾身）并 return；否则 `sync_cc_markers(char, None)`。
- `on_remove_control`（`On<Remove, ControlTag>`）：`sync_cc_markers(char, Some(触发实体))`--多 Buff 叠加时移除其一不会误清标记。
- `BuffCastBlock` 用独立 `On<Add/Remove, BuffCastBlock>` 观察者桥接（无 ControlTag）。
- `update_cc_buff_timers`（FixedUpdate）：tick 所有 CC debuff + BuffCastBlock 的 timer，过期 despawn -> 触发 `On<Remove>` -> 标记自动重算。

# 净化 CommandCleanse

- `CommandCleanse { entity }`（EntityEvent，`lol_core::buffs::cc_debuffs`）：observer 销毁目标 `Buffs` 中所有带 `ControlTag` 的 Buff 实体 -> 触发 `On<Remove, ControlTag>` -> 标记自动清。
- 不认识任何具体控制类型，只按标签杀人。
- 用法：`commands.entity(target).trigger(|e| CommandCleanse { entity: e });`

# 免控 ImmuneToCC

- 免控 buff（如 Olaf R） granting 免疫时直接 `commands.entity(caster).insert(ImmuneToCC)`（施法者已知）。
- 免控 buff 消亡时用 `On<Remove, 免控Buff>` 观察者经 `BuffOf` 找角色、`remove::<ImmuneToCC>()`（带多 buff 安全检查：仍有其它存活免控 buff 则保留）。
- 激活即解控：granting 免控的同时对自身触发 `CommandCleanse`，解除已有 CC。
- `on_add_control` 见到 `ImmuneToCC` 会立即 despawn 新施加的 CC buff，故免控期间新 CC 不沾身。

# 组合模式

- 直接施加 CC：技能命中目标后 `commands.entity(target).with_related::<BuffOf>(DebuffX::new(duration))`（如 Riven W 眩晕、Tryndamere W 减速）。
- 位移端点 CC：观察 `EventMovementEnd { source: Dash }` 在端点施加 CC。
- 强化普攻命中 CC：技能起手挂 `BuffXAttack`（带 duration 字段），观察 `EventAttackEnd` 找该 Buff -> 对 `trigger.target` 施加 CC -> despawn `BuffXAttack`（如 Garen Q 沉默）。
- 免控：granting 免控 buff + `insert(ImmuneToCC)` + 自身 `CommandCleanse`（如 Olaf R）。
- 击退/拉回：`CommandKnockback` 内部已插 `DebuffKnockup`，无需额外施加（见 [位移系统](./skill-dash.md)）。
- 自施法锁：技能 windup 期间 `.with_related::<BuffOf>(BuffCastBlock::new(duration))` 锁住自身移动/施法（如 Riven W 蓄力）。

# 关键坑

- **Buff↔标记桥接必须用 `On<Remove>` 观察者，不要用轮询系统**。轮询读 `Buffs(Vec<Entity>)` 会在 Buff despawn 当帧看到陈旧实体（despawn 命令未应用 + 帧边界竞态），标记清不掉。`On<Remove>` 在 despawn 时同步触发，当帧清除。
- **`On<Add, 主组件>` 触发时 `BuffOf` 尚未就绪**（`with_related` 先 spawn 主组件再插 `BuffOf`）。免控 buff 的「加标记」侧改在施法 handler 里直接 `insert`，「删标记」侧用 `On<Remove>`（此时 `BuffOf` 仍在）。`On<Add, ControlTag>` 能找到 `BuffOf` 是因为 ControlTag 为 required 组件、插入时序不同。
- **测试 harness 64Hz**（delta=15.625ms），技能有施法 windup，Buff 实际创建延后 -> 计时器过期常落在 `advance` 末帧。故 expiry 测试依赖 `On<Remove>` 当帧清除（轮询会漏末帧）。测试断言标记（`MovementBlock`/`CastBlock`）而非具体 Debuff 组件。
- **施加 CC 必须用 `with_related::<BuffOf>(...)`**，不能直接 `entity.insert(DebuffX)`--后者不建立 Buff 关系，观察者找不到角色、标记不会桥接。
- `Buffs::iter()` 仅在 `use bevy::prelude::*` 时 yield 拥有的 `Entity`，否则 yield `&Entity`（`.copied()` 会编译错）。

# 局限性

- 无韧性 tenacity 减免（Garen W 等的 CC 时长缩减待后续）。
- 不可选 / 免伤标记是独立维度，不在 CC 系统内（本系统聚焦控制）。
- 仓库内非四姐妹五虎英雄的自定义 CC buff（Soraka 等）仍用旧逻辑，待单独迁移到 `ControlTag`。
- 免控 buff（如 `BuffOlafR`）的 timer 在各英雄 `mod.rs` 自维护，未纳入全局 `update_cc_buff_timers`（因其非 CC debuff）。
