# 技能原子动作与状态

- 行为解耦：观察者不直接修改实体属性或触发复杂行为，而是向世界派发特定的原子动作命令或事件。
- 底层实现：原子动作为共享构建块，在框架层统一实现，以降低代码冗余。

# 核心原子动作指令

- `CommandAnimationPlay`：播放技能动画，需提供实体、动画哈希名称、是否循环及强制覆盖的持续时间。
- `ActionDamage`：范围伤害，会自动读取技能关联的公式配置。需要指定实体、技能句柄与包含伤害特效的列表。
- `ActionDash`：冲刺位移，控制实体向目标点冲刺（固定距离 / 指针最大值 / 绝对世界点 / 追踪实体），只含运动字段；沿途伤害通过 `DashDamageIntent` 组件挂载，详见 [位移系统](./skill-dash.md)。
- `CommandDamageCreate`：直接造成固定数值的单体物理、魔法或真实伤害。
- `CommandKnockback`：击退或拉回，包含目标、来源实体、距离、速度、击飞持续时间与方向（`Away` 击退 / `Toward` 拉回），详见 [位移系统](./skill-dash.md)。
- `CommandAttachedFieldCreate`：创建随身附着的圆形持续伤害力场，支持设置最终半径、基础伤害、持续时间及可选的力场半径成长动画。
- `CommandAttackReset`：重置普攻后摇计时器，从而允许无缝衔接下一次普通攻击。

# 状态控制与限制组件

- `BuffShieldWhite`：挂载于 Buff 实体的吸收类型白色护盾。
- `DebuffStun`：眩晕效果组件，禁止一切玩家操作与走位。
- `DebuffSlow`：减速效果组件，降低移动速度百分比。
- `CastBlock`：施法阻塞标记，禁止释放其他主动技能。
- `MovementBlock`：移动阻塞标记，禁止通过寻路移动。

# 范围伤害几何与过滤参数

- 圆形区域：`DamageShape::Circle`，需要提供判定半径。
- 扇形区域：`DamageShape::Sector`，需要提供判定半径与夹角。
- 环形区域：`DamageShape::Annular`，内圈以内不造成伤害。
- 最近单体：`DamageShape::Nearest`，在最大距离内筛选最靠近的单个目标。
- 过滤类型：通过 `TargetFilter::All` 过滤所有敌方小兵与英雄，`TargetFilter::Champion` 仅过滤敌方英雄，`TargetFilter::Minion` 仅过滤小兵。

# 状态关系注册

- `with_related`：通过在实体上以特定方法附加标记，系统会关联创建 Buff 关系实体并在英雄销毁时级联回收。
