# 位移系统

- 设计主旨：位移只管「运动形状 + 目标解析 + 速度」并发生命周期事件；伤害/CC/击退/抓取作为挂在事件上的观察者组合--灵活组合大于上帝参数。
- 核心约束：`ActionDash` 只含运动字段，**不携带伤害**。需要伤害时由独立组件 + 生命周期事件观察者挂载。

# 冲刺位移 ActionDash

- `ActionDash { entity, move_type, speed, point }`：EntityEvent。`point` 为施法指针点（世界 XZ 坐标）。
- `DashMoveType::Fixed(distance)`：朝 `point` 方向固定距离冲刺；`point` 与自身重合时退回面向方向。
- `DashMoveType::Pointer { max }`：朝 `point` 冲刺，距离 clamp 到 `max`（近则到点，远则只走 max）。
- `DashMoveType::WorldPoint(Vec2)`：冲向绝对世界点（忽略 `point` 字段），用于锚点拉拽、传送落点。
- `DashMoveType::Entity { target, stop_radius }`：冲向实体并追踪，接触（距离 <= `stop_radius`）即停。
- 底层：`on_action_dash` 解析目的地后触发 `CommandMovement`（priority 100，`MovementSource::Dash`，`MovementWay::Path`），并发出 `EventDashStart`。`Entity` 变体额外插入 `TrackingDash` 组件。

# 追踪位移 TrackingDash

- `TrackingDash { target, stop_radius }`：由 `DashMoveType::Entity` 起手时插入。
- `update_tracking_dash`（FixedUpdate）：每帧把路径终点重设为 target 当前位置；接触或目标消失时清空路径、发 `EventMovementEnd`、移除自身。优先级 100，不被普通走位打断。

# 位移生命周期事件

- `EventDashStart { entity, start, destination }`：位移开始时由 `on_action_dash` 发出，携带起点与终点，供伤害等副作用观察者挂载。
- `EventMovementEnd { entity, source }`：位移结束时发出，`source = MovementSource::Dash` 标识来自冲刺，供端点伤害/CC 观察者使用。

# 沿途伤害（解耦原语）

- `DashDamageIntent { damage, skill }`：组件，champion 在触发 `ActionDash` **前**插入以声明沿途伤害意图。
- `DashDamage { radius_end, damage: TargetDamage }`：膨胀圆终点半径与伤害定义（`TargetDamage` 复用 `ActionDamage` 的过滤/数值/类型）。
- `on_dash_start_attach_damage`：观察 `EventDashStart`，读 `DashDamageIntent` 挂载 `DashDamageComponent`（携带 start/destination）并移除意图。
- `update_dash_damage`（FixedUpdate）：沿路径膨胀圆（半径 65 -> `radius_end`，按进度插值），对范围内未命中敌人触发 `CommandDamageCreate` 并去重--每个敌人每次位移只命中一次。
- `on_dash_end`：观察 `EventMovementEnd` 移除 `DashDamageComponent`。
- 数值来源：伤害数值由 `update_dash_damage` 经 `get_skill_value` 从 `skill` 资产按技能等级解析。

# 组合模式

- 纯位移：仅 `ActionDash`（如 Fiora Q 走位、Riven E）。
- 位移 + 沿途伤害：触发 `ActionDash` 前插入 `DashDamageIntent`（如 Irelia Q、Aatrox Q、Sett R、Volibear R、Camille E2/R）。
- 位移 + 端点效果：观察 `EventMovementEnd { source: Dash }`，在端点触发伤害/CC（如 Fiora Q 戳刺、Riven Q3 端点击退）。
- 位移 + 附着伤害场：搭配 `CommandAttachedFieldCreate` 生成随施法者移动的力场（如 Riven Q）。
- 拉回（自身不位移）：范围/锥形查询敌人 + 对每个敌人 `CommandKnockback { direction: Toward }`（如 Darius E）。
- 钩墙位移：sticky 飞弹碰墙 -> `EventMissileHit` -> `ActionDash { WorldPoint(anchor) }`（如 Camille E1）。

# 击退/拉回 CommandKnockback

- `CommandKnockback { entity, source, distance, speed, duration, direction }`：EntityEvent。
- `direction: DisplaceDirection`：`Away`（默认，背离 source 击退）/ `Toward`（朝 source 拉回，`distance` 钳制不越过 source，故传大于间距的值即可拉到脚下）。
- 底层：触发 `CommandMovement`（`MovementSource::Knockback`，priority 100），并插入 `DebuffKnockup(duration)` + `CastBlock`。`duration` 为 `None` 时按 `distance/speed` 计算。
- 关键约束：击退**不**加 `MovementBlock`（否则位移系统会跳过该实体），故击飞期间位移仍进行；自身 CC 不会取消自身击退位移。

# 粘性飞弹钩墙

- `CommandMissileCreate { ..., sticky: true }`：sticky 直线飞弹每步用 `is_walkable_by_xy` 检测地形，碰墙时销毁飞弹、留下 `WallAnchor`（默认 5s）、向 source 发 `EventMissileHit`。sticky 飞弹**不做**实体碰撞。
- `EventMissileHit { source, spell, point }`：`source` 为施法者（event_target），`point` 为墙点，`spell` 携带技能句柄。英雄观察者按 `spell` 分发二段逻辑（如 Camille E1 拉墙后开重施窗口）。

# 局限性

- `update_dash_damage` 起始半径 65.0 硬编码，暂无法按技能配置。
- 沿途伤害去重为「每次位移每敌人一次」，无法表达单次位移内多次命中。
- CC 与位移交互：当前硬 CC 为暂停（跳过但不清路径），CC 结束继续滑完；「硬 CC 取消位移」待后续修正。
