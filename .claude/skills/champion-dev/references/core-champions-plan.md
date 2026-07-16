# 上单九英雄 · 全功能实现指南

> **目标**: 不妥协、不偷懒、完整实现九英雄的全部 feature，同时重构底层框架使其优雅、可组合、面向未来
> **原则**: 通用机制下沉到 `lol_core`，英雄只做差异化组合；数据驱动优于硬编码；ECS 组合优于继承

---

## 一、框架层重构：五大通用机制

### 1.1 位移与控制统一体系 `action/displace.rs`

**现状问题**:

- `CommandKnockback` 只支持 Away/Toward 两种模式，无法表达"抱起后摔"(Sett R)、"拉回中心点"(Aatrox W)
- 锥形拉回 (Darius E / Sett E / Mordekaiser E) 每个英雄自己写锥形检测 + 逐个触发 `CommandKnockback`，大量重复
- 不区分"带位移的控制"和"纯控制"(Sett E 双侧眩晕 vs 单侧减速)

**重构方案**: 引入 `ActionDisplace` 事件，统一所有"对敌方造成位移"的表达

```rust
/// 统一位移动作：英雄技能只需描述「对谁、往哪移、附带什么效果」
#[derive(EntityEvent, Debug, Clone)]
pub struct ActionDisplace {
    pub entity: Entity,                    // 施法者
    pub targets: DisplaceTargetSelection,  // 目标选择策略
    pub motion: DisplaceMotion,            // 位移运动
    pub effects: Vec<DisplaceEffect>,      // 命中附带效果（伤害/CC/标记）
}

/// 目标选择策略
pub enum DisplaceTargetSelection {
    /// 锥形范围内所有敌方英雄（Darius E / Sett E / Mordekaiser E）
    Cone {
        range: f32,
        angle: f32,           // 全角，非半角
        direction: Vec2,      // 施法方向
    },
    /// 圆形范围内所有敌方英雄（Riven Q3 / Volibear R 落地）
    Circle {
        radius: f32,
        center: DisplaceCenter,
    },
    /// 最近的单个敌方英雄（Sett R 抓取）
    Nearest {
        range: f32,
    },
    /// 指定实体列表（由英雄自行筛选后传入）
    Explicit(Vec<Entity>),
}

pub enum DisplaceCenter {
    Caster,
    Point(Vec2),
}

/// 位移运动描述
pub enum DisplaceMotion {
    /// 向施法者拉回（Darius E / Mordekaiser E / Sett E / Aatrox W 引爆）
    PullToward {
        distance: f32,         // 最大拉回距离
        speed: f32,
        clamp_at_source: bool, // true: 不拉过施法者脚下
    },
    /// 从施法者击退（Riven Q3 / Camille E2 命中英雄）
    PushAway {
        distance: f32,
        speed: f32,
    },
    /// 拉回到指定点（Aatrox W 引爆拉回 W 中心）
    PullToPoint {
        point: Vec2,
        distance: f32,         // 最大拉回距离
        speed: f32,
    },
    /// 抓取并跟随施法者位移（Sett R 抱人）
    GrabAndFollow {
        /// 被抓取的目标跟随施法者直到施法者位移结束
        /// 落地后投掷方向由施法者朝向决定
        throw_distance: f32,   // 落地后投掷距离
        throw_speed: f32,
    },
    /// 无位移，仅施加效果（用于纯 CC 场景）
    None,
}

/// 命中附带效果
pub enum DisplaceEffect {
    /// 击飞（含位移期间不可操作）
    Knockup { duration: f32 },
    /// 眩晕（不含位移）
    Stun { duration: f32 },
    /// 减速
    Slow { percent: f32, duration: f32 },
    /// 伤害
    Damage {
        amount: f32,
        damage_type: DamageType,
        tag: Option<u32>,
    },
    /// 标记（通用 debuff 挂载）
    Mark {
        component: Box<dyn FnOnce(&mut Commands, Entity) + Send + Sync>,
    },
}
```

**Sett E 双侧检测**: `ActionDisplace` 的 observer 在处理 `Cone` 目标时，自动检测锥形前后是否都有敌人，若双侧均命中则 effect 升级为 `Stun`。这个行为由英雄侧的 `SettEMode` enum 控制：

```rust
/// Sett E 特殊模式：双侧检测
pub enum ConeHitPolicy {
    /// 默认：所有命中目标施加相同效果
    Uniform,
    /// Sett E：检测前后两个锥形，若均命中则升级为 Stun
    DualSide {
        stun_duration: f32,    // 双侧命中时的眩晕时长
        slow_percent: f32,     // 单侧命中时的减速比例
        slow_duration: f32,
    },
}
```

**Sett R 抱人实现**:

```
on_sett_r:
  1. 用 Nearest{range} 选最近敌方英雄 -> grabbed_target
  2. 对 grabbed_target 挂 GrabbedBy(sett_entity) 组件 + DebuffKnockup(duration)
  3. 施法者自身触发 ActionDash 向 cast_point
  4. update_grabbed_entities 系统：每帧把 grabbed 目标的 Transform 同步到施法者位置
  5. on_sett_r_dash_end (EventMovementEnd):
     a. 移除 GrabbedBy
     b. 将目标投掷到落地点对面（PushAway）
     c. 落地圆形 AoE 伤害 + 减速
```

关键组件：

```rust
/// 被抓取标记：挂在被抓者身上，每帧同步位置
#[derive(Component, Debug)]
pub struct GrabbedBy {
    pub grabber: Entity,
    pub offset: Vec3,  // 相对于 grabber 的偏移
}
```

新增系统 `update_grabbed_entities`（在 `FixedUpdate`）：

```rust
fn update_grabbed_entities(
    q_grabbed: Query<(Entity, &GrabbedBy)>,
    q_transform: Query<&Transform>,
    mut q_target_transform: Query<&mut Transform, Without<GrabbedBy>>,  // 避免冲突
) {
    // 每帧把被抓者位置同步到 grabber 位置 + offset
}
```

**英雄侧代码对比（重构前后）**:

```rust
// ========== 重构前（Darius E，41 行手写锥形检测 + 逐个 knockback）==========
let half_angle = DARIUS_E_CONE_ANGLE.to_radians() / 2.0;
for (enemy, enemy_transform) in q_enemies.iter() {
    // ... 12 行锥形检测 ...
    commands.entity(enemy).trigger(|e| CommandKnockback { ... });
    commands.entity(enemy).with_related::<BuffOf>(DebuffSlow::new(...));
}

// ========== 重构后（Darius E，5 行组合声明）==========
commands.trigger(ActionDisplace {
    entity,
    targets: DisplaceTargetSelection::Cone {
        range: DARIUS_E_RANGE,
        angle: DARIUS_E_CONE_ANGLE,
        direction: forward,
    },
    motion: DisplaceMotion::PullToward {
        distance: DARIUS_E_RANGE,
        speed: DARIUS_E_PULL_SPEED,
        clamp_at_source: true,
    },
    effects: vec![
        DisplaceEffect::Knockup { duration: 0.75 },
        DisplaceEffect::Slow { percent: 0.4, duration: 1.0 },
    ],
});
```

**使用此机制的英雄**:

| 英雄          | 技能         | TargetSelection      | Motion                 | Effects                       |
| ------------- | ------------ | -------------------- | ---------------------- | ----------------------------- |
| Darius E      | 无情立场     | Cone{535, 90°}       | PullToward{535, clamp} | Knockup(0.75) + Slow(0.4, 1s) |
| Sett E        | 迎面痛击     | Cone{490, 90°}       | PullToward{490, clamp} | DualSide(Stun/Slow)           |
| Mordekaiser E | 断魂一拽     | Cone{550, 100°}      | PullToward{250, clamp} | Damage(魔法)                  |
| Riven Q3      | 折翼第三段   | Circle{250, Caster}  | PushAway{75}           | Knockup(0.75)                 |
| Aatrox W 引爆 | 冥府之链     | Explicit(marked)     | PullToPoint{center}    | Knockup(0.5) + Damage         |
| Sett R        | 消防官       | Nearest{475}         | GrabAndFollow          | AoE Damage + Slow             |
| Camille E2    | 钩索命中英雄 | Circle{150, DashEnd} | PushAway{small}        | Stun(1.0)                     |
| Volibear R    | 风暴之怒落地 | Circle{300, DashEnd} | None                   | Damage + Slow                 |

---

### 1.2 飞弹碰撞体系增强 `missile.rs`

**现状**:

- `LinearMissile` 只做实体碰撞 → 伤害，碰撞后直接 `CommandDamageCreate`
- `sticky: true` 做地形碰撞 → `EventMissileHit`（已实现，青钢影 E1 可用）
- 但 Aatrox W / 剑姬 W 的「导弹碰撞英雄后触发特殊效果」无通用支持

**增强方案**: 扩展 `LinearMissile` 的碰撞回调

```rust
/// 飞弹碰撞目标策略
pub enum MissileCollisionTarget {
    /// 碰撞敌方实体（默认）
    Enemy,
    /// 只碰撞地形（sticky 飞弹，青钢影 E1）
    WallOnly,
    /// 碰撞敌方实体但不造成伤害，改为发事件（Aatrox W / Fiora W 飞弹模式）
    EnemyNoDirectDamage,
}

/// 飞弹命中实体事件（新增）：触发于 source（施法者），携带命中目标和飞弹信息
#[derive(EntityEvent, Debug, Clone)]
pub struct EventMissileHitEntity {
    #[event_target]
    pub source: Entity,
    pub target: Entity,
    pub spell: Handle<Spell>,
    pub hit_point: Vec3,
}
```

**Aatrox W 重构为导弹模式**:

```
on_aatrox_w:
  1. CommandMissileCreate { destination, collision: EnemyNoDirectDamage }
  2. 导弹飞行并检测碰撞英雄
  3. 碰撞后发 EventMissileHitEntity { source: aatrox, target: enemy }

on_aatrox_w_missile_hit (观察 EventMissileHitEntity):
  1. 对 target 造成伤害 + 减速
  2. 在 target 脚下生成 W 区域（矩形/线条）
  3. 挂 DebuffAatroxWMark { center: hit_point, timer: 1.5s }

update_aatrox_w_marks:
  到期引爆：检测 target 是否仍在 W 区域内
    → 在区域内：PullToPoint{center} + 二次伤害
    → 已走出：无效果
```

这样 Aatrox W 的「导弹飞行 → 碰撞英雄 → 标记 → 延迟引爆 → 拉回中心」的完整链路用通用原语组合实现。

**青钢影 E1（钩索碰墙）已有支持**:

现有的 `sticky: true` + `EventMissileHit` 已经实现了钩索碰墙锚定的核心机制。但 Camille E 的完整流程需要补齐：

```
on_camille_e (stage 1):
  1. CommandMissileCreate { sticky: true, destination: cast_direction * range }
     → 飞弹飞行，遇墙自动锚定并发 EventMissileHit

on_camille_e_missile_hit (观察 EventMissileHit):
  1. 读取 hit_point（墙点）
  2. ActionDash { entity: camille, move_type: WorldPoint(hit_point) }
     → 把青钢影拉到墙上
  3. 挂 SkillRecastWindow 开启 E2 窗口
  4. 挂 BuffCamilleWallCling { timer, wall_point } 组件

on_camille_e (stage 2):
  1. 从 BuffCamilleWallCling 读取 wall_point，确定冲刺方向
  2. 检测冲刺路径上是否有英雄 → 有则用 DashMoveType::Entity{target}
  3. 冲刺伤害 + 攻速加成
  4. 命中英雄时触发 ActionDisplace 眩晕
```

**架构评估**: `EventMissileHit`（碰墙）和新增 `EventMissileHitEntity`（碰英雄）双轨制，足以覆盖当前和未来所有飞弹碰撞场景：

| 碰撞类型          | 事件                                  | 使用英雄                              |
| ----------------- | ------------------------------------- | ------------------------------------- |
| 碰墙锚定          | `EventMissileHit`                     | 青钢影 E1、贾克斯 E（未来）、瑟庄妮 Q |
| 碰英雄 + 伤害     | `linear_missile_collision` 内直接处理 | 锐雯 R Wind Slash                     |
| 碰英雄 + 特殊效果 | `EventMissileHitEntity`               | Aatrox W、Fiora W（可选导弹模式）     |

---

### 1.3 治疗 / 吸血 / 回血通用体系

**现状问题**:

- 治疗分散在各处：直接 `health.value += heal`（Mordekaiser W、Volibear W2）、`BuffSelfHeal`（通用）
- 无「伤害转治疗」机制（Aatrox E 被动、Darius Q 外圈、Fiora 被动）
- 无「区域持续治疗」机制（Fiora R 光环）

**方案**: 三层治疗体系

#### 层 1: 瞬时治疗 `CommandHeal`（新增 action）

```rust
#[derive(EntityEvent, Debug, Clone)]
pub struct CommandHeal {
    pub entity: Entity,     // 被治疗者
    pub source: Entity,     // 治疗来源
    pub amount: f32,        // 治疗量
    pub tag: Option<u32>,   // 标签（用于区分来源，如 Aatrox 被动/Q 外圈）
}
```

统一治疗入口，便于未来接入治疗减少（重伤）、治疗统计。

#### 层 2: 伤害转治疗 `ActionVampirism`（新增 buff）

```rust
/// 伤害转治疗 buff：持有期间，造成伤害时按比例治疗自身
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Vampirism" })]
pub struct BuffVampirism {
    pub ratio: f32,         // 治疗比例（如 0.16 = 16%）
    pub damage_filter: VampirismFilter,
    pub timer: Option<Timer>,  // None = 永久（如 Aatrox E 被动）
}

pub enum VampirismFilter {
    /// 所有伤害（Aatrox R 增强治疗）
    All,
    /// 仅对英雄（Darius Q 外圈）
    ChampionOnly,
    /// 仅技能伤害（不含普攻 DoT）
    SkillOnly,
}
```

由 `EventDamageCreate` 的全局观察者驱动：伤害结算后检查 source 是否有 `BuffVampirism`，有则触发 `CommandHeal`。

#### 层 3: 区域持续治疗 `HealingField`（新增组件）

```rust
/// 持续治疗场：以某点为中心，每 tick 治疗范围内友军
#[derive(Component, Debug)]
pub struct HealingField {
    pub center: Vec2,
    pub radius: f32,
    pub heal_per_tick: f32,
    pub tick_timer: Timer,
    pub duration_timer: Timer,
    pub team: Team,
}
```

用于 Fiora R 治疗光环。

**使用场景映射**:

| 英雄          | 技能       | 治疗类型                   | 实现                                                  |
| ------------- | ---------- | -------------------------- | ----------------------------------------------------- |
| Aatrox E      | 被动       | 对英雄伤害转治疗 16%       | `BuffVampirism { ratio: 0.16, filter: ChampionOnly }` |
| Aatrox R      | 增强自疗   | 全伤害转治疗增强           | 修改已有 `BuffVampirism.ratio`                        |
| Darius Q      | 外圈回血   | 每命中英雄回复已损生命 12% | `CommandHeal` 在 AoE 结算后触发                       |
| Fiora P       | 要害击破   | 瞬时 + 移速                | `CommandHeal` + `BuffMoveSpeed`                       |
| Fiora R       | 大招治疗场 | 区域持续治疗               | `HealingField`                                        |
| Mordekaiser W | 重施回血   | 护盾转治疗                 | `CommandHeal`（已有逻辑保留）                         |
| Volibear W2   | 咬回血     | 基于已损生命               | `CommandHeal`                                         |

---

### 1.4 `BuffMoveSpeed` 修复 (F3)

**现状**: `BuffMoveSpeed` 已有完整的 `update_move_speed_buff` 系统，但 `PluginCommonBuffs` 的 `build` 方法已经注册了此系统。

**实际问题排查**: 检查 `PluginCommonBuffs` 是否被 `lol_core` 的顶层 Plugin 注册。如未注册，则在 `lol_core/src/lib.rs` 中补齐：

```rust
app.add_plugins(buffs::common_buffs::PluginCommonBuffs);
```

这将立即修复 Aatrox R / Sett Q / Volibear Q / Fiora P 的移速加成。

---

### 1.5 `ActionDamageEffect` 增强：tag 支持 + 回调

**现状问题**:

- `ActionDamageEffect` 的 `tag` 字段是 `Option<u32>`，但 Camille 的 `on_camille_damage_hit` 全局观察者未按 tag 过滤
- Darius Q 的内外圈需要不同伤害倍率和不同效果（外圈回血），但当前双 effect 用同一个 `blade_damage`

**增强方案**:

```rust
pub struct ActionDamageEffect {
    pub shape: DamageShape,
    pub damage_list: Vec<TargetDamage>,
    pub tag: Option<u32>,
    pub exclude: Vec<DamageShape>,
    /// 命中后额外执行的效果（不在 ActionDamageEffect 内做 CC/治疗，改用标签路由）
    pub on_hit_tag: Option<u32>,  // 新增：为此 effect 的伤害打上 tag
}
```

英雄侧的 `on_xxx_damage_hit` 观察者严格按 tag 过滤即可：

```rust
// Camille: 只在 W 外圈减速
fn on_camille_damage_hit(...) {
    match trigger.event().tag {
        Some(CAMILLE_W_OUTER_TAG) => { /* 减速 + 回血 */ }
        _ => {}  // 其他伤害不减速！
    }
}
```

---

## 二、英雄层实现指南

### 2.1 Aatrox (暗裔魔剑)

#### 被动 - 死亡镰刀

```
修改 passive.rs:
  1. DAMAGE_RATIO → 按等级从 RON passive_damage_calc 读取
  2. 冷却 → 从 RON PassiveCooldown 读取（按等级递减 22→8.59s）
  3. 被动触发时 +50 攻击距离（临时 BuffAttack 或直接修改 Attack.range）
  4. 命中英雄/大型野怪减少冷却（检查 target 是否 Champion/LargeMonster）
  5. 对小兵治疗降低为 25%
```

#### E - 暗影冲锋 · 被动

```
新增 e.rs 被动:
  1. 挂载 BuffVampirism { ratio: 0.16, filter: ChampionOnly } 为永久 buff
  2. Aatrox R 激活时，修改 ratio *= (1 + r_heal_amp)
  3. E 主动增加 CommandAttackReset
```

#### R - 世界终结者

```
修改 r.rs:
  1. BuffMoveSpeed 加成（修复 F3 后自动生效）
  2. 恐惧附近小兵/野怪：对范围内非英雄施加 DebuffFlee(3s)
  3. 击杀延长 5s：监听 EventDamageCreate，若 target 死亡且 R 激活中，刷新 R timer
  4. 自疗增强：修改 BuffVampirism.ratio（R 增幅从 RON 读取 50/75/100%）
  5. 体型增大 5%：修改 Transform.scale（到期恢复）
```

---

### 2.2 Darius (诺克萨斯之手)

#### Q - 大杀四方

```
修改 q.rs:
  1. 内圈伤害改用 inner_damage（50% 伤害），外圈用 blade_damage（完整伤害）
  2. 外圈回血：在 ActionDelayedDamage 的结算回调中：
     - 统计外圈命中的英雄数量
     - 每命中一个英雄，CommandHeal { amount: missing_hp * 0.12 }
     - 上限从 RON 读取
  3. 内圈不叠出血 → 已有 INNER_TAG 过滤
```

**核心**: `ActionDelayedDamage` 结算后需要一个「命中报告」回调，让英雄知道哪些目标被哪个 effect 命中：

```rust
/// AoE 命中报告：延迟伤害结算后发出，英雄可观察此事件执行后续效果
#[derive(EntityEvent, Debug, Clone)]
pub struct EventAoEHitReport {
    #[event_target]
    pub caster: Entity,
    pub effect_index: usize,      // 哪个 ActionDamageEffect
    pub hit_targets: Vec<Entity>,  // 被命中的目标列表
    pub tag: Option<u32>,
}
```

Darius Q 观察此事件，当 tag 匹配外圈时执行回血。

#### E - 无情立场

```
重构为 ActionDisplace:
  Cone{535, 90°} + PullToward{535, clamp} + Knockup(0.75) + Slow(0.4, 1s)
新增被动护甲穿透:
  E 等级系统 → 挂载 BuffArmorPen { percent } 到 Darius 自身
```

#### R - 诺克萨斯断头台

```
新增击杀重置:
  on_darius_r 结算伤害后，监听目标 Health <= 0
    → 重置 R 冷却，开启 6s 重置窗口
    → 窗口内满层出血自动叠加（诺克萨斯之力刷新）
```

---

### 2.3 Mordekaiser (铁铠冥魂)

#### E - 断魂一拽

```
重构为 ActionDisplace:
  Cone{550, 100°} + PullToward{250, clamp} + Damage(魔法)
被动法术穿透:
  待法穿系统实现后接入
```

#### R - 死亡领域

```
空间隔离核心实现:
  1. 标记双方进入领域: InsideRealm { realm_id } 组件
  2. 修改全局系统的 Query 过滤:
     - 攻击系统：只攻击同 realm 或无 realm 的目标
     - 技能目标检测：同上
     - 小兵 AI：忽略领域内英雄
  3. 视觉隔离：领域内外实体互相不可见（渲染层级）
  4. 领域结束时移除 InsideRealm
```

---

### 2.4 Sett (荒蔓之拳)

#### Q - 屈人之威

```
修改 q.rs:
  1. SETT_Q_MAX_HP_RATIO 改为从 RON PercentDamage 读取
  2. BuffMoveSpeed 修复 F3 后自动生效
  3. 移速加成朝向英雄时加倍 → 需检测 Movement.direction 与最近英雄方向的夹角
```

#### E - 迎面痛击

```
重构为 ActionDisplace + DualSide 策略:
  1. Cone{490, 90°} 前方 + Cone{490, 90°} 后方分别检测
  2. 前后均命中 → 全部 Stun(1s)
  3. 仅一侧命中 → 该侧 Slow(0.5, 1s)
  4. 所有命中者 PullToward{490, clamp} + Damage
```

#### R - 消防官 (抱人重构)

```
on_sett_r:
  1. Nearest{475} 选取抓取目标
  2. 对目标挂 GrabbedBy + DebuffKnockup(R持续时间)
  3. 对自身 ActionDash 向 cast_point（抱着人飞）
  4. update_grabbed_entities 同步位置

on_sett_r_dash_end:
  1. 移除 GrabbedBy
  2. 投掷目标到落地点（PushAway 方向由冲刺方向决定）
  3. 以落地点为中心 Circle AoE:
     - 基础伤害 + 目标最大生命 % 额外伤害
     - 伤害计算从 RON 读取
     - Slow(0.4, 1.5s)
```

---

### 2.5 Volibear (雷霆咆哮)

#### 被动 - 风暴之力

```
修改 passive.rs:
  1. 连锁闪电 tag → Some(VOLIBEAR_P_CHAIN_TAG)（避免触发减速）
  2. 数值从 RON 读取而非硬编码
```

#### E - 落雷

```
修改 e.rs:
  1. 护盾值从 RON calculated_shield 读取（替代硬编码 100.0）
  2. 护盾条件：Volibear 在落雷区域内时才获得护盾
     → AoE 结算后检查 caster 是否在圆心 radius 内
```

#### R - 风暴之怒

```
修改 r.rs:
  1. 落地后禁用范围内防御塔（需 TurretDisabled 系统支持，已有模块）
  2. R 期间移速加成：BuffMoveSpeed（修复 F3 后生效）
  3. 增加最大生命值有持续时间 → 到期回退
```

---

### 2.6 Camille (青钢影)

#### 被动 - 自适应防御

```
修改 passive.rs:
  1. 自适应护盾类型：
     - 检查目标最近的伤害类型（物理/魔法）
     - 物理多 → BuffShieldWhite（物理护盾）；魔法多 → BuffShieldMagic
  2. 护盾冷却 16-10s（按等级）：
     - 挂 CamillePassiveCooldown 组件在 Camille 自身
     - 未冷却完成时不触发护盾
  3. 护盾比例/持续时间从 RON 读取
```

#### Q - 精密协议

```
修改 q.rs:
  1. Q1 命中后移速加成 40%（BuffMoveSpeed）
  2. Q2 延迟判定 0.75s（SkillRecastWindow 的 min_delay 字段）
  3. Q2 部分伤害转真伤：
     - 从 RON 读取 TrueDamagePercent（40-100%）
     - Q2 的 BuffOnHitBonusDamage 拆分为两部分：
       物理部分 = ratio * (1 - true_percent)
       真伤部分 → 额外 CommandDamageCreate { damage_type: True }
```

#### W - 战术横扫

```
重构 w.rs:
  1. 双 ActionDamageEffect：
     inner_effect: Sector{radius, 60°} → 物理伤害
     outer_effect: Annular{inner, outer} ∩ Sector{80°} → 物理伤害 + 最大生命 %
  2. 外圈命中英雄回血 75%：在 EventAoEHitReport 中按 tag 过滤
  3. 外圈减速 → tag: CAMILLE_W_OUTER_TAG, on_camille_damage_hit 按 tag 施加减速
```

#### E - 钩索 (完整两段)

```
重构 e.rs（基于现有 sticky 飞弹体系）:

on_camille_e (stage 1):
  1. CommandMissileCreate {
       sticky: true,
       destination: cast_direction * E_RANGE,
       collision: WallOnly,
     }
  2. 挂 SkillRecastWindow 但不立即开启（等碰墙后开启）

on_camille_e_missile_hit (观察 EventMissileHit):
  1. 检查 spell 是否为 Camille E → 是则处理
  2. ActionDash { move_type: WorldPoint(hit_point) }（拉到墙上）
  3. 到达墙面后挂 BuffCamilleWallCling { wall_point, timer: 0.75s }
  4. 此时开启 SkillRecastWindow（E2）

on_camille_e (stage 2):
  1. 从 wall_point 向 cast_point 方向冲刺
  2. 检测路径上最近英雄 → 有则 DashMoveType::Entity{target, stop_radius}
  3. 冲刺沿途伤害：DashDamageIntent（已有机制）
  4. 命中英雄时：ActionDisplace { Stun(1.0) }
  5. 攻速加成 BuffAttack + BuffCamilleE

update_camille_wall_cling:
  wall_cling timer 到期 → 自动执行 E2 冲刺（或直接跌落）
```

#### R - 海克斯科技绝杀

```
修改 r.rs:
  1. 施法时短暂不可选中（TargetImmune 组件 0.5s）
  2. 区域锁定：
     - 以目标位置为中心生成 Hextech Arena 组件
     - 其他英雄触碰边界时 PushAway（击退出区域）
     - 目标触碰边界时 PullToward（推回区域内）
  3. 持续时间到期后移除 Arena
  4. 施法者死亡 → 提前移除 Arena
```

---

### 2.7 Fiora (剑姬)

#### W - 招架

```
当前实现可保留（计时器模式），但需补齐:
  1. ImmuneToCC + BuffDamageReduction(1.0) 的免疫不含真实伤害
     → BuffDamageReduction 需区分 physical/magic/all，true_damage 不减免
  2. 反刺可选导弹模式（未来扩展）或保留当前矩形检测（已够用）
  3. 格挡硬控 vs 软控区分：
     on_fiora_w_parried_cc 已实现 ControlTag 检测
     但需确保只有硬控（Stun/Knockup/Root）才标记 parried_hard_cc
     → 检查 ControlTag 是否区分硬/软控，如不区分需增加 ControlSeverity 字段
```

#### R - 决斗

```
修改 r.rs:
  1. R 期间移速加成 30%：BuffMoveSpeed
  2. 击破 4 要害后治疗光环：
     - 触发 HealingField { center: target_pos, radius: 600, ... }
     - 治疗友军 heal_per_tick 从 RON 读取
  3. 等级数值改为从 RON 读取
```

---

### 2.8 Irelia (刀妹)

#### Q - 利刃冲击

```
修改 q.rs:
  1. DashMoveType::Pointer → DashMoveType::Entity { target, stop_radius: 25 }
     （追踪型冲刺，不稳标记/可击杀目标优先选中后传入）
  2. 命中治疗：CommandHeal { amount: 12 + 0.12 * ad }
  3. 对小兵伤害 60%：检查 target 是否 Minion，是则 amount *= 0.6
  4. 击杀小兵也刷新 CD（当前只检查 unsteady）
```

#### R - 先锋之刃

```
重构 r.rs:
  1. 第一段：导弹形态
     - CommandMissileCreate { destination, collision: Enemy }
     - 飞弹穿透命中所有敌人（hit_enemies 不去重 → 不对，应保留去重但穿透不销毁）
     - 需 LinearMissile 增加 pass_through: bool 字段
  2. 命中后施加标记 + 减速（通过 tag → on_irelia_damage_hit）
  3. 飞弹终点生成禁锢区域：
     - IreliaRWall { boundary_points, timer: 2.5s }
     - 敌人触碰边界时受伤 + 减速 + 消耗墙体
```

---

### 2.9 Riven (锐雯)

#### Q - 折翼之舞

```
修改 q.rs:
  Q3 落地改用 ActionDisplace:
    Circle{250, Caster} + PushAway{75} + Knockup(0.75) + Damage
  R 激活期间 Q 半径扩大（从 BuffRivenR 读取 radius_bonus）
```

#### R - 放逐之刃

```
修改 r.rs:
  1. AD 恢复公式：到期时 damage.0 -= bonus_ad（直接减去加成值，而非除法）
     → BuffRivenR 存储 bonus_ad 绝对值而非 ratio
  2. R2 Wind Slash 伤害逐目标计算：
     对每个命中目标分别计算 missing_hp_ratio
     → 修改 cast_riven_wind_slash 中的循环逻辑
  3. 被动日志 info! → debug!
```

---

## 三、数据驱动清理

将以下硬编码值迁移到 RON 文件：

| 英雄       | 数值项                | 当前         | 目标                                                  |
| ---------- | --------------------- | ------------ | ----------------------------------------------------- |
| Aatrox P   | `DAMAGE_RATIO` 15%    | 硬编码 const | `get_skill_value("passive_damage", level, ...)`       |
| Aatrox P   | 冷却 22s              | 硬编码 const | `get_skill_data_value("PassiveCooldown", level)`      |
| Camille P  | 护盾比例 6% / 2s      | 硬编码 const | `get_skill_data_value("ShieldRatio/Duration", level)` |
| Sett Q     | `SETT_Q_MAX_HP_RATIO` | 硬编码数组   | `get_skill_data_value("PercentDamage", level)`        |
| Volibear E | 护盾 100.0            | 硬编码       | `get_skill_value("calculated_shield", level, ...)`    |
| Volibear P | 攻速/链式系数         | 硬编码 const | `get_skill_data_value(...)`                           |
| Riven P    | 30%-46.76%            | 硬编码插值   | `get_skill_data_value("PassivePercent", level)`       |
| Darius E   | 范围/角度             | 硬编码 const | `get_skill_data_value(...)`                           |
| Fiora R    | 真伤比例数组          | 硬编码       | `get_skill_data_value("VitalDamagePercent", level)`   |

---

## 四、实现顺序（依赖关系排序）

### Phase 0: 框架层修复（阻塞所有后续）

```
0.1  验证并修复 BuffMoveSpeed 系统注册（F3）
0.2  CommandHeal 治疗 action（供后续英雄使用）
0.3  ActionDisplace 统一位移体系（包含 GrabbedBy）
0.4  EventMissileHitEntity 飞弹碰英雄事件
0.5  EventAoEHitReport AoE 命中报告
0.6  LinearMissile.pass_through 穿透支持
```

### Phase 1: 位移类技能（依赖 Phase 0.3）

```
1.1  Darius E → ActionDisplace（最简单，验证 API）
1.2  Sett E → ActionDisplace + DualSide 策略
1.3  Mordekaiser E → ActionDisplace
1.4  Riven Q3 → ActionDisplace
1.5  Sett R → GrabAndFollow 完整流程
1.6  Aatrox W → 导弹模式 + PullToPoint 引爆
```

### Phase 2: 飞弹类技能（依赖 Phase 0.4）

```
2.1  Camille E → sticky 飞弹 + EventMissileHit + WorldPoint dash + E2 冲刺
2.2  Aatrox W 导弹化（如 Phase 1.6 未完成）
2.3  Irelia R → 穿透飞弹 + 禁锢区域
```

### Phase 3: 治疗 & 吸血（依赖 Phase 0.2）

```
3.1  Aatrox E 被动 → BuffVampirism
3.2  Darius Q 外圈回血 → EventAoEHitReport + CommandHeal
3.3  Fiora P 击破治疗 → CommandHeal（已部分实现）
3.4  Fiora R 治疗光环 → HealingField
3.5  Aatrox R 自疗增强 → BuffVampirism ratio 修改
```

### Phase 4: 其余功能补全

```
4.1  Aatrox P 等级成长 + 冷却缩减
4.2  Camille P 自适应护盾 + 冷却
4.3  Camille Q 真伤转换 + 移速
4.4  Camille W 内外圈 + 回血 + tag 过滤
4.5  Camille R 区域锁定
4.6  Darius R 击杀重置
4.7  Irelia Q 追踪冲刺 + 治疗 + 小兵判定
4.8  Mordekaiser R 空间隔离
4.9  Volibear E 数据驱动护盾 + P 链式闪电 tag 修复
4.10 Riven R AD 恢复公式修正 + R2 逐目标伤害
```

### Phase 5: 数据驱动清理

```
5.1  RON 文件补充缺失字段
5.2  硬编码 const 替换为 get_skill_data_value
5.3  测试验证数值读取正确
```

---

## 五、测试策略

每个 Phase 完成后用 `ChampionTestHarness` 验证：

```rust
#[test]
fn test_darius_e_pull_displace() {
    let mut harness = ChampionTestHarness::new();
    let darius = harness.spawn_champion::<Darius>(Vec3::ZERO);
    let enemy = harness.spawn_enemy(Vec3::new(300.0, 0.0, 0.0));
    harness.cast_skill(darius, SkillSlot::E, Vec2::new(300.0, 0.0));
    harness.advance_frames(15);
    // 敌人应被拉到 Darius 脚下附近
    let pos = harness.position(enemy);
    assert!(pos.x < 50.0, "E 应拉回到脚下");
}

#[test]
fn test_sett_e_dual_side_stun() {
    let mut harness = ChampionTestHarness::new();
    let sett = harness.spawn_champion::<Sett>(Vec3::ZERO);
    let front = harness.spawn_enemy(Vec3::new(200.0, 0.0, 0.0));
    let back = harness.spawn_enemy(Vec3::new(-200.0, 0.0, 0.0));
    harness.cast_skill(sett, SkillSlot::E, Vec2::new(200.0, 0.0));
    harness.advance_frames(10);
    // 双侧命中应眩晕
    assert!(harness.has_debuff::<DebuffStun>(front));
    assert!(harness.has_debuff::<DebuffStun>(back));
}

#[test]
fn test_sett_r_grab_and_slam() {
    let mut harness = ChampionTestHarness::new();
    let sett = harness.spawn_champion::<Sett>(Vec3::ZERO);
    let target = harness.spawn_enemy(Vec3::new(200.0, 0.0, 0.0));
    harness.cast_skill(sett, SkillSlot::R, Vec2::new(-300.0, 0.0));
    harness.advance_frames(30);
    // target 应被带到 Sett 落点附近
    let sett_pos = harness.position(sett);
    let target_pos = harness.position(target);
    assert!(sett_pos.distance(target_pos) < 100.0);
}
```

---

## 六、架构前瞻性评估

### 已有机制复用能力

| 通用机制                | 当前覆盖             | 扩展到                                                    |
| ----------------------- | -------------------- | --------------------------------------------------------- |
| `ActionDisplace`        | 9 英雄的 12 个技能   | 未来所有位移控制（如 Lee Sin R、Alistar W、Blitzcrank Q） |
| `EventMissileHit`       | 青钢影 E1            | 瑟庄妮 Q、魄罗投掷、诺提勒斯 Q                            |
| `EventMissileHitEntity` | Aatrox W             | 锐雯 R、Ezreal Q、Jinx W                                  |
| `BuffVampirism`         | Aatrox E/R           | 所有吸血装备、征服者符文                                  |
| `CommandHeal`           | 治疗统一入口         | 重伤减疗、治疗统计、奶妈技能                              |
| `HealingField`          | Fiora R              | 索拉卡 W、奥巴马 W                                        |
| `GrabbedBy`             | Sett R               | 泰坦 R、锤石 Q（未来）                                    |
| `DualSide`              | Sett E               | 独此一用（但模式化为枚举，不增加复杂度）                  |
| `DashMoveType::Entity`  | Irelia Q、Camille E2 | 阿卡丽 R、亚索 E                                          |

### 不需要重构的

- **`CommandKnockback`**: 保留为低层原语，`ActionDisplace` 的 observer 内部调用它
- **`ActionDelayedDamage`**: AoE 延迟伤害体系完善，补齐 `EventAoEHitReport` 即可
- **`BuffOf` 关系体系**: ECS buff 架构优秀，无需改动
- **`SkillRecastWindow`**: 重施窗口通用且稳定

### 需要关注的未来需求

1. **法术穿透 / 护甲穿透**：Mordekaiser E 被动、Darius E 被动 → 需全局伤害管线支持 `ArmorPen`/`MagicPen` 字段
2. **真实伤害免疫的 `BuffDamageReduction`**：当前 ratio=1.0 减免所有伤害，但 Fiora W 应不减免真伤
3. **区域困锁 / 空间隔离**：Camille R、Mordekaiser R → 需通用的 `Arena`/`Realm` 组件 + 位移边界检测系统
