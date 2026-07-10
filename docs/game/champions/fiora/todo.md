# Fiora 待办

## 被动（破绽 Vital）

- [x] Vital 扇形视觉指示器（2026-07-04）
- [x] 修复被动从未启用：补挂 `AbilityFioraPassive` 标记（2026-07-04）
- [x] 修复被动/R 命中 observer 误用 `GlobalTransform`（无头模式永不更新）-> 改 `Transform`（2026-07-09）
- [x] 击破要害后的移速加成（8%，1.5s，`BuffFioraMS`）（2026-07-09）
- [x] 击破要害后的治疗（占位常量 `FIORA_PASSIVE_HEAL=40.0`，待真实数值）（2026-07-09）
- [ ] Vital 即将超时变红（`timeout_red_triggered`）的视觉过渡（标记已置位，扇形颜色未跟随）
- [ ] 被动治疗量对齐真实数据（ron `passive_heal_amount` 公式为空，当前占位）

## Q（前刺）

- [x] 位移停止后戳刺最近单位（2026-07-04）
- [x] 有敌方英雄时优先戳英雄（2026-07-04）
- [x] 不对路径上的敌人造成碰撞伤害，区别于 Riven Q（2026-07-04）
- [x] Q 命中敌人时冷却退还没收（`CDRefundPercent`，ron=0.5）（2026-07-09）
- [x] Q 命中要害（Vital）时伤害翻倍（2026-07-09）
- [x] Q 重置普攻计时器（`CommandAttackReset`）（2026-07-09）
- [ ] Q 目标优先级细化：悬停目标 > 可击杀目标 > 最近目标（当前仅「英雄优先 > 最近」）

## E（利刃之舞）

- [x] 按等级赋予额外攻速（ron `ASPercent`，经 `get_skill_data_value` 读取）（2026-07-09）
- [x] 第一击减速 40%（1s，`DebuffSlow`）（2026-07-09）
- [x] 第二击必暴（`(AttackTwoPercentTAD - 1) × AD` 额外物理伤害）（2026-07-09）
- [x] E 持续 3s 过期，移除攻速加成（`update_fiora_e_buff`）（2026-07-09）
- [ ] E 第二击「暴击」目前用额外物理伤害近似，未走原生暴击路径（`FioraCritAttack`）

## R（决斗）

- [x] 修复 R 死代码：`BuffFioraR` 改挂到目标敌方英雄（此前挂 Fiora 自身导致 `on_r_damage_create` 永不匹配）（2026-07-09）
- [x] 每个要害最大生命值真实伤害（3/3.5/4%）（2026-07-09）
- [x] 四要害全破 / 目标死亡（已破≥1）触发治疗光环（`BuffFioraRHeal`，550 范围，5s）（2026-07-09）
- [x] R 期间 30% 移速（`BuffFioraMS`）（2026-07-09）
- [ ] R 期间移速与被动移速的互斥（wiki：R 替换为 30%，当前两者叠加）
- [ ] R 治疗光环治疗量按等级从 ron `HealPerSecond` 读取（当前已硬编码 [50,75,100]，可改用 `get_skill_data_value`）

## W（招架）— 本轮未做

- [ ] W 招架：格挡即将到来的伤害 0.75s（受「只用列出 buff」约束，需大额 `BuffShieldWhite`；真伤穿透）
- [ ] W 招架控制 / 减益（需 lol_core 拦截点；用户已允许改 lol_core，可考虑加无敌组件）
- [ ] W 反刺：命中第一个敌方英雄造成魔法伤害 + 减速（招架硬控则眩晕）
- [ ] W 期间施法阻塞（`BuffCastBlock` + `MovementBlock`）

## 框架缺口（非 Fiora 专属，影响本英雄）

- [ ] `lol_core` 缺 `update_debuff_slow` -> E 第一击减速永不过期（Darius 同款）
- [ ] 飞弹 on-hit 回调缺失 -> W 反刺若用飞弹（`FioraWMissile`）难施加 on-hit 效果（本轮 W 用近战查询规避）
