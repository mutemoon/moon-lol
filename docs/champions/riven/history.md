# Riven 开发历史

## 2026-05-05 — 直线 missile 系统 + 完整 spell 导出

### 实现内容

- **数据结构扩展**: `lol_base::movement` 新增 `MissileBehavior`, `HeightSolver`, `VerticalFacing`, `MissileSpecification.missile_width`/`behaviors`/`height_solver`/`vertical_facing`，`MovementTypeFixedSpeed` 新增 `tracks_target`/`project_target_to_cast_range`/`use_height_offset_at_end`/`offset_initial_target_height`
- **DataSpell 扩展**: `lol_base::spell::DataSpell` 新增 16 个字段 (castRange, castRadius, castConeAngle, castConeDistance, lineWidth, castFrame, animationName, cooldownTime, cantCancelWhileWindingUp, spellRevealsChampion, affectsTypeFlags, alternateName, coefficient, hitEffectKey, selectionPriority, useAnimatorFramerate)
- **完整重新导出**: 更新 `league_to_lol::extract::spell` 的转换代码，运行 `pnpm extract` 重新导出所有英雄的 spell RON 文件（现在包含完整字段）
- **线性 missile 系统**: `lol_core::missile` 新增：
  - `CommandLinearMissileCreate` — 创建向目标点直线飞行的导弹
  - `LinearMissile` 组件 — 标记线性导弹实体
  - `linear_missile_fixed_update` — 每帧移动 + 碰撞检测（基于宽度过滤敌方）
- **Wind Slash 改为发射导弹**: 不再直接造成伤害，改为发射 3 枚直线导弹（左/中/右，7° 扩散），每枚导弹独立碰撞检测，宽度 100，速度 1600，射程 1100

### 测试

- 所有 12 个 Riven 测试通过

## 2026-05-04 — 重构：CommandMissileCreate 统一导弹系统 + Riven Wind Slash 适配

### 重构内容

- **合并导弹创建接口**: 废弃 `CommandLinearMissileCreate`，所有直线/追踪导弹统一使用 `CommandMissileCreate`
  - `target: Option<Entity>` — Some 为追踪导弹，None 为直线导弹
  - `destination: Option<Vec3>` — 直线导弹目标位置
  - 新增 `speed: Option<f32>` — 覆盖 spell data 的飞行速度
  - 新增 `particle_hash: Option<u32>` — 覆盖 spell data 的粒子效果
- **缺失 spell data 容错**: `on_command_missile_create` 不再对 `res_assets_spell_object.get()` 使用 `.unwrap()`，缺失 spell 或 spell_data 时优雅降级使用默认值
- **修复 damage bug**: `LinearMissile.damage` 从硬编码 `0.0` 改为使用 `trigger.damage`
- **Wind Slash 适配**: `cast_riven_wind_slash` 使用 `CommandMissileCreate` 发射 3 枚导弹，传入 `RivenFengShuiEngine` spell handle 和 `particle_hash` 覆盖

### 注意事项

- Wind Slash 伤害改为导弹创建时预计算（基于所有敌方目标的平均 HP 比例），不再按单个目标分别计算。这是导弹系统 pre-calculated damage 的固有特性，如需恢复 per-target 计算需要后续在 missile 系统中增加 on-hit 伤害公式回调

### 测试

- 12/12 Riven 测试通过

## 2026-05-04 — 初始技能实现与测试修复

### 实现内容

- **模块结构**: 创建 `mod.rs`、`q.rs`、`passive.rs`、`tests.rs`
- **Q - 折翼之舞**: 三段重施系统 (SkillRecastWindow)，每段 Dash 250 单位，附带伤害、动画、粒子效果
- **W - 震魂斩**: 圆形范围伤害 (半径 300)，通过 ActionDamage 系统对敌方单位造成伤害
- **E - 无畏跃**: Dash 250 单位 + BuffShieldWhite 护盾 (100 基础值)
- **R - 放逐之刃**: 粒子效果触发 (Riven_R_Indicator_Ring, Riven_R_ALL_Warning)，无伤害/buff 逻辑
- **被动 - 符文之刃**: EventAttackEnd 触发额外 20% AD 伤害，消耗一层 BuffRivenPassive

## 2026-05-04 — 完整技能实现（W 眩晕、Q3 击退、E 护盾数据驱动、R buff + Wind Slash）

### 实现内容

- **W 眩晕**: 对 300 半径内敌人施加 BuffStun（0.75s）+ MovementBlock，通过 `update_riven_stun` 系统到期自动移除
- **Q3 击退**: Q3 位移时添加 `RivenQ3Pending` 标记，`on_riven_dash_end` 观察 `EventMovementEnd`，将 250 半径内敌人沿径向推开 75 单位
- **E 护盾数据驱动**: 使用 `get_skill_value("total_shield", level, stat_fn)` 从 `RivenFeint.ron` spell 数据读取护盾值（50 + 110% bonus AD），不再硬编码 100.0
- **R 被动 Buff**: 初次 R 增加 25% AD + 75 攻击距离，通过 `BuffRivenR` 子实体 + `BuffOf` 管理，15 秒后 `update_riven_buffs` 系统回退属性
- **R 二段 Wind Slash**: 锥形范围（1100 半径，18° 角），基于已损失生命值线性增伤（0% HP → max_damage，100% HP → min_damage）
- **眩晕打断**: `on_riven_skill_cast` 顶部检查 `BuffStun`，眩晕中直接返回阻止施法

### 测试

- `riven_w_stuns_enemies_in_range` — W 眩晕、范围过滤、过期移除
- `riven_r_buff_increases_stats_and_allows_recast` — R 增伤、加攻击距离、连招窗口
- `riven_r_wind_slash_deals_damage_in_cone` — Wind Slash 锥形伤害、前后过滤
- `riven_r_wind_slash_deals_more_damage_to_low_hp_targets` — 低血量增伤验证
- `riven_r_buff_expires_after_15_seconds` — R buff 15 秒到期属性恢复
- `riven_stun_prevents_skill_cast` — 眩晕阻止施法
- `riven_q3_knocks_back_enemies` — Q3 击退距离、RivenQ3Pending 移除

### 注意事项

- Wind Slash 敌人放在 -Z 方向（默认 `Transform.forward()` 方向），+Z 方向敌人不被伤害
- R 冷却在初次施放时设置为真实冷却（120/90/60s），同时添加 SkillRecastWindow 允许 15 秒内重施 Wind Slash
- R buff 属性回退使用 `damage.0 * (1.0 - 0.25 / 1.25)` 公式：`total = base * 1.25` → `base = total / 1.25`

### Bug 修复

1. **Dash 被网格加载阻塞**: `apply_final_movement_decision` 通过 `Res<ResourceGrid>` 要求网格资源必须存在，但原测试轮询只检查 `ResourceGrid` 资源是否存在（`get_resource`），不检查内部 `Handle<ConfigNavigationGrid>` 是否已加载完成。导致 Dash 触发时网格 asset 尚未加载，路径规划失败。修复: 在 `test_utils.rs` 加载循环中改用 `asset_server.get_load_state(&grid.0).is_some_and(|s| s.is_loaded())` 确保网格 asset 完全加载后才开始测试。

2. **Team 组件缺失**: 测试 Harness 生成英雄时没有插入 `Team` 组件，导致 `on_action_damage` 中 `q_team.get(entity)` 失败并直接返回，W 技能无法造成伤害。修复: 在 `test_utils.rs` 的英雄 spawn 中加入 `Team::Order`。

3. **Asset handle 类型不匹配**: `spell()` 方法使用 `load_hash()` 将 path-based handle 转为 hash key 再查，但 path-based handle 的 AssetId 不是 Uuid 变体，导致 panic。修复: 改用 `Assets::get(asset_id)` 直接查询。

### 测试

- `riven_q_cycles_through_three_real_stages` — 验证三段 Q 的 RecWindow 状态转换、冷却、位移
- `riven_q_recast_window_expires_after_4_seconds` — 验证 4 秒窗口超时机制
- `riven_w_hits_only_enemies_in_range` — W 范围 (260 vs 420) 和友军过滤
- `riven_e_spawns_shield_and_dash_absorbs_damage` — E 护盾吸收和位移
- `riven_r_starts_cooldown_without_moving_or_damaging` — R 冷却和位置不变
