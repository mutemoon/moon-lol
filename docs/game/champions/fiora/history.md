# Fiora 开发历史

## 2026-07-09 - Phase 1+2：Q/R/被动/E 全面补全 + 修复 R 死代码

### 背景

用户要求做「Fiora 技能进一步实现计划」，确认范围 Phase 1+2 连做（Q、R、被动、E），并允许改 `lol_core`。W 不在本轮范围。排查中发现两个前置 bug 与一个框架缺口。

### 关键发现与前置修复

1. **R 是死的（高优先）**：`cast_fiora_r` 把 `BuffFioraR` 挂在 Fiora 自身（`commands.entity(entity).with_related::<BuffOf>(...)`，holder=Fiora），但 `on_r_damage_create` 用 `buff_of.get() == target_entity`（敌方）匹配，永远不成立 -> R 要害从不被消耗，`update_add_vital` 的「R 期间跳过该目标」也从不生效。
2. **被动/R 要害命中用 `GlobalTransform`，无头模式永不更新**：headless harness 用 `MinimalPlugins`（不含 `TransformPlugin`），`GlobalTransform` 保持 (0,0)，导致 `on_passive_damage_create` / `on_r_damage_create` 的 `is_in_direction` 永远算出 source=target=(0,0) -> 被动真伤从不触发。`update_add_vital`/`update_remove_vital` 与 Q 用的是 `Transform`，唯独这两个命中 observer 用 `GlobalTransform`，不一致。**统一改为 `Transform`**（与 `update_add_vital` 一致；对无父实体 `Transform.translation == GlobalTransform.translation`）。
3. **`get_skill_value` 只能读 `calculations`，读不了 `dataValues`**：`CDRefundPercent`、`ASPercent`、`AttackTwoPercentTAD`、`HealPerSecond` 都在 `dataValues`。在 `lol_core::skill::helpers` 暴露 `pub fn get_skill_data_value`（包装私有 `get_named_data_value`）并 re-export，避免硬编码。

### 各技能实现

**Q（`q.rs`）**：`FioraQPending` 增加 `skill_entity`；`cast_fiora_q` 加 `CommandAttackReset`（重置普攻）；`on_fiora_q_dash_end` 命中要害（活跃 `Vital` + `is_in_direction`）则 `amount *= 2`，命中后对 `skill_entity` 的 `CoolDown` 退还没收 `remaining *= (1 - CDRefundPercent)`（ron=0.5）。

**R（`r.rs`）**：`cast_fiora_r` 选 500 范围内朝向 cursor 的最近敌方英雄，把 `BuffFioraR` 挂到**目标**（修复发现 1，仿 Darius 出血）；`BuffFioraR` 增加 `level`。`on_r_damage_create` 击破要害时 `CommandDamageCreate { True, hp.max * [0.03,0.035,0.04][level] }`；四要害全破或目标死亡（已破≥1）时给 Fiora 挂 `BuffFioraRHeal` 治疗光环（550 范围，5s，每秒 [50,75,100]，`update_fiora_r_heal` 计时）。R 期间 30% 移速由共享 `BuffFioraMS` 提供。

**被动（`passive.rs`）**：新增共享 `BuffFioraMS { percent, timer, applied, applied_bonus }` + `update_fiora_ms_buff`（首次 tick 加 `speed * percent`，到期减回）。`on_passive_damage_create` 破绽命中后治疗 Fiora + 挂 `BuffFioraMS`（8%，1.5s）。治疗量 `FIORA_PASSIVE_HEAL=40.0` 为占位（wiki 无数值、ron `passive_heal_amount` 公式为空）。

**E（`e.rs`）**：`cast_fiora_e` 从 ron 读 `ASPercent`（攻速）与 `AttackTwoPercentTAD`（暴击比例），挂 `BuffAttack` + `BuffFioraE { left, crit_bonus_ratio, timer: 3s }`。`on_event_attack_end`：第一击（left 2->1）对 `trigger.target` 挂 `DebuffSlow(0.4, 1.0)`，第二击（left 1->0）追加 `CommandDamageCreate` 物理 = `(AttackTwoPercentTAD - 1) * AD`；两击耗尽或 3s 到期（`update_fiora_e_buff`）移除 buff 与 `BuffAttack`。

### 测试（TDD，先红后绿）

- `q_tests.rs` +3：`fiora_q_resets_attack_timer`（断言 Windup `end_time` 刷新）、`fiora_q_hit_refunds_cooldown`（命中剩余 CD < 未命中×0.6）、`fiora_q_damage_doubled_on_vital`（双 harness 对比匹配/非匹配方向，差 > 350 即翻倍+被动真伤）。
- `r_tests.rs` +4（新建）：目标挂载、要害真伤、四要害全破触发治疗光环、目标死亡触发治疗光环。
- `passive_tests.rs` +2：击破要害获得移速、击破要害治疗菲奥娜。
- `e_tests.rs` +4（新建）：等级攻速、第一击减速、第二击暴击额外伤害（用 `EventAttackEnd` 手动触发隔离）、3s 过期。
- 攻击重置测试要点：`CommandAttackReset` 的语义是「移除 `AttackState` 并以保存目标立即重起一段全新 Windup」，故断言 `end_time` 刷新而非 `AttackState` 消失；且需先移除 `AttackAuto` 隔离 `update_attack_auto` 的自动重起。

全量 `cargo check --all-targets` 干净，`cargo test -p lol_champions` 全绿（Fiora 21 项）。

### 局限性 / 未实现

- **W 招架未做**（本轮范围外）：受「只用列出 buff」约束，伤害招架需大额 `BuffShieldWhite`，CC 招架需 lol_core 拦截点；用户已允许改 lol_core，后续 W 可考虑加无敌组件。
- **`DebuffSlow` 无到期系统**：`lol_core` 仅有 `update_debuff_stun`/`update_debuff_knockup`，无 `update_debuff_slow` -> E 第一击减速实际永不过期（Darius 同款问题）。框架缺口，未在本轮处理。
- **被动治疗量为占位**（40.0）：待真实数值就绪后替换。
- **Vital 即将超时变红视觉**：`timeout_red_triggered` 标记已置位，但扇形颜色未随之变化（渲染相关，低优先）。
- **被动 MS 与 R MS 叠加**：R 期间若被动也触发，两者相加（wiki 语义是 R 替换为 30%）；未做互斥，近似处理。
- **被动击破回血方向**：`on_passive_damage_create` 用 `Without<Vital>` 划分 source/target 的 Health 查询集以避开可变冲突。

## 2026-07-04 - 被动破绽扇形视觉 + 修复被动从未启用

### 背景

用户反馈被动（破绽 Vital）没有视觉效果，要求加上简单的扇形指示器（见 `feedback.md`）。排查时发现一个更严重的前置问题：`update_add_vital` 通过 `With<AbilityFioraPassive>` 过滤被动技能实体，但 `AbilityFioraPassive` 标记**从未被任何代码挂上**，导致 Vital 根本不会生成--被动形同未启用，自然也没有任何东西可看。

### 决策与过程

1. **先修被动启用**：新增 `attach_fiora_passive_ability`（FixedUpdate）系统，把 `AbilityFioraPassive` 幂等地挂到 Fiora 英雄的被动技能实体上（通过 `PassiveSkillOf` 反查英雄，再用 `With<Fiora>` 限定只作用于 Fiora）。挂上后 `update_add_vital` 才真正开始给范围内敌方英雄标记 Vital。
   - 之所以用单独的对账系统而非在某处一次性 insert，是因为被动技能实体由场景（`config.ron`）加载生成，插件构建时尚不存在；用 `Without<AbilityFioraPassive>` 过滤的幂等系统可在加载完成后随时补挂。
2. **再加视觉**：新增 `FioraVitalVisual { target }` 标记组件与 `update_vital_visuals`（FixedUpdate）对账系统，与「Vital 如何被挂上」完全解耦--只看目标身上有没有 `Vital`：
   - 缺视觉 -> 生成一个平铺地面、指向要害方向的半透明扇形实体；
   - 方向 / 位置变化 -> 更新 Transform；
   - Vital 消失（目标死亡 / 离开 / 超时被回收）-> 回收视觉实体。
3. **扇形朝向**：扇形网格默认朝 +Z，按 `Direction` 旋转到对应象限（Up->+Z、Right->+X、Down->-Z、Left->-X），与 `is_in_direction` 的「要害方向 = 攻击者应从哪一侧接近」语义一致；张角 90°，与 `is_in_direction` 的象限判定范围对齐。
4. **网格构造**：手写三角扇（中心顶点 + 弧上顶点），朝向 +Y 为正面（索引顺序 `(center, next, current)`），`unlit` + `AlphaMode::Blend` 半透明，离地 0.3 避免 Z-fighting。
5. **无头 / 渲染兼容**：视觉系统签名带 `Option<ResMut<Assets<Mesh>>>` / `Option<ResMut<Assets<StandardMaterial>>>`，无头模式下这两个资源不存在 -> 不创建任何 Mesh，仅维护标记实体与 Transform，因此视觉的生命周期与朝向仍可在无头测试中断言。Mesh / 材质句柄用 `Local` 缓存，避免每次生成都重建资源。

### 测试（TDD）

`passive_tests.rs` 先红后绿，四个用例：

1. `fiora_passive_vital_visual_spawns_and_tracks_direction`：直接挂已知方向的 Vital，断言扇形生成且朝向匹配；改方向后断言朝向跟随更新。
2. `fiora_passive_vital_visual_despawns_when_target_gone`：目标 despawn 后视觉被回收。
3. `fiora_passive_marks_enemy_in_range_with_vital`：端到端--范围内敌人被被动标记 Vital 并生成视觉（同时验证 `attach_fiora_passive_ability` 修复生效）。
4. `fiora_passive_vital_sector_render`：用 `build_render` 录制扇形视频；未设 `MOON_LOL_RUN_RENDER_TESTS` 时回退无头，断言仍成立。

### 局限性 / 未实现

- 扇形为单色，未实现 Vital 即将超时变红（`timeout_red_triggered`）等过渡表现；用户要求「简单的扇形」，暂不展开。
- 录像测试在本机渲染模式下会命中 `lol_render::ui::element` 的既有 panic（与 Riven 既有渲染测试同款，非本英雄问题），故实际视频需在渲染环境正常时由 `npm run test:render` 产出。

## 2026-07-04 - Q 重构为「位移后戳刺 + 英雄优先」

### 背景

原 Q 实现（`mod.rs::cast_fiora_q`）在施法瞬间同步触发 `ActionDamage { Nearest, All }`，从施法者起点位置取最近敌人造成伤害，与「位移停止后戳刺」的语义不符，且没有英雄优先级。详见 `feedback.md`。

### 决策与过程

- 拆出 `q.rs` 模块，承载 Q 的施法逻辑与位移结束 observer（与 `e.rs` / `r.rs` / `passive.rs` 的拆分风格一致）。
- 施法时只做三件事：
  1. 播放 `spell1` 动画；
  2. `ActionDash`（`damage: None`）--不产生 `DashDamageComponent`，因此**不会像 Riven Q 那样对路径敌人造成碰撞伤害**；
  3. 挂 `FioraQPending` 标记（携带技能法术句柄与等级），等位移结束再戳刺。
- 新增 `on_fiora_q_dash_end` observer 监听 `EventMovementEnd`：
  - 仅在 `trigger.source == MovementSource::Dash` 且施法者带 `FioraQPending` 时触发，避免走位结束误触发；
  - 以位移终点为圆心、`FIORA_Q_STRIKE_RADIUS`（200）为半径索敌：分别记录「最近的敌方英雄」与「最近的任意敌方单位」，取 `nearest_champion.or(nearest_any)`；
  - **命中判定包含目标碰撞半径**：有效范围 = 戳刺半径 + 目标 `Bounding.radius`（即 `dist - target_radius <= STRIKE_RADIUS`，以敌人边缘是否进入范围为准，而非中心点）；最近目标也按「距边缘的距离」比较；
  - 用 `get_skill_value("total_damage")` 计算伤害（与 `ActionDamage` 内部一致，stat=2 取 AD），通过 `CommandDamageCreate` 对选定单体造成物理伤害；
  - 处理后立即移除 `FioraQPending`，保证一次 Q 只戳一次。

### 为什么不用 `ActionDamage` 的 `Nearest` 形状

`DamageShape::Nearest` 只能先取「最近的一个敌人」再叠加 `TargetFilter` 校验：若 `filter = Champion` 而最近的是小兵，则会直接空放（不造成任何伤害）。无法表达「有英雄优先英雄，否则取最近单位」的语义，因此索敌逻辑写在 observer 里，伤害用 `CommandDamageCreate` 直接对选定单体施加。

### 常量

- 位移最大距离 300、速度 1000（沿用原值，位移时长 ≈ 0.3s）。
- 戳刺半径 200（位移终点为圆心）。

### 局限性 / 未实现

本次仅完成用户指定的「位移后戳刺 + 英雄优先」。其余 Q 机制（命中减 CD、命中要害减 CD/翻倍、重置普攻、悬停/可击杀目标优先级）见 `todo.md`。

### 测试（TDD）

`q_tests.rs` 三个 headless 测试，先红后绿：

1. `fiora_q_strikes_after_dash_not_at_cast`：位移结束前（0.1s）无伤害，结束后（0.6s）有伤害。
2. `fiora_q_prioritizes_champion_over_closer_minion`：路径上更近的小兵不被碰撞伤害，英雄被优先戳中（小兵距终点更近但仍戳英雄，验证真·英雄优先）。
3. `fiora_q_strikes_nearest_unit_when_no_champion`：无英雄时戳最近小兵。
4. `fiora_q_strike_includes_target_bounding_radius`：敌人中心点刚好超出戳刺半径、但边缘（含碰撞半径）在范围内时仍应命中。

测试中用不带 `Champion` 组件的敌方实体模拟小兵（harness 的 `add_enemy` 只能生成英雄）。
