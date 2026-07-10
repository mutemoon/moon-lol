# 技能原子动作与状态

## 核心设计

- 行为封装：原子动作函数是对已有事件或动作的二次封装，不作为独立的数据存储层。
- 主要职责：降低观察者逻辑中的模板代码，提升技能实现代码的可读性，且避免引入静态配置层。

## 常用原子动作

- `CommandAnimationPlay`：播放技能动画，来源于渲染模块。
- `ActionDamage` 与 `ActionDamageEffect`：处理范围伤害。
- `ActionDash`：实现位移冲刺逻辑，支持固定位置与指针方向。
- `CommandDamageCreate`：直接对单一目标造成伤害。
- `CommandMissileCreate`：向特定方向或目标发射飞弹。
- `CommandAttachedFieldCreate`：创建并附着在施法者身上的持续伤害力场。
- `CommandAttackReset`：重置普通攻击间隔计时器。
- `with_related::<BuffOf>`：给目标实体附加特定状态关系实体。

## 可用状态与控制效果

- `BuffShieldWhite`：吸收特定伤害的护盾。
- `DebuffStun`：禁止一切操作的眩晕效果。
- `DebuffSlow`：降低移动速度的减速效果。
- `BuffCastBlock`：在技能施放期间禁止其他主动操作的施法阻塞。
- `MovementBlock`：禁止移动的移动阻塞。

## 伤害形状

- `Circle`：以施法者为圆心的圆形区域。
- `Sector`：特定半径与夹角的扇形区域。
- `Nearest`：指定最大距离内的最近单一目标。
- `Annular`：指定内外圈半径的环形区域，内圈不受伤害。

## 目标过滤

- `All`：筛选所有敌方单位。
- `Champion`：仅筛选敌方英雄单位。
