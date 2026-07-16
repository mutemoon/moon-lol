# Darius 待解决问题

## Q 技能 - 大杀四方

### 已完成
- [x] 内圈和外圈双重伤害形状实现
- [x] 内圈: Circle r=150
- [x] 外圈: Annular 150-350
- [x] Q 造成伤害（使用 ActionDamageEffect）
- [x] Q 触发出血效果（通过 on_darius_damage_hit observer）
- [x] 内圈伤害不叠加血怒，外圈叠加（`DARIUS_Q_INNER_TAG=2` 区分）
- [x] 从 Spell Asset 读取伤害数值（InnerDamage / OuterDamage）

### 待实现

#### 高优先级
1. **外圈命中回复生命值**
   - 每个英雄回复 17% 已损失生命值
   - 最多 51%（3 个英雄）

#### 中优先级
2. Q 技能粒子特效
3. Q 被打断的处理

#### 低优先级
4. Q 闪（Q Flash）机制
5. 内圈范围精细调整（实际约 150）
6. 外圈范围精细调整（实际约 350-400）

---

## W 技能 - 致残

### 已完成
- [x] 播放动画 (spell2)
- [x] 攻击重置 (CommandAttackReset)
- [x] 强化普攻：额外物理伤害 + 50% 减速 1 秒
- [x] 施法冷却

### 待实现
无

---

## E 技能 - 无情铁手

### 已完成
- [x] 播放动画 (spell3)
- [x] 拉回效果（锥形查询 + `CommandKnockback{Toward}` 拉到脚下，见 `e.rs`）
- [x] 击飞 0.75s（由 `CommandKnockback` 自动施加 `DebuffKnockup`）
- [x] 减速效果（40% 减速 1s，`DebuffSlow`）

### 待实现
1. **被动护甲穿透**
   - 被动: 获得 20/25/30/35/40% 护甲穿透
   - 需要护甲穿透系统支持

---

## R 技能 - 诺克萨斯断头台

### 已完成
- [x] 播放动画 (spell4)
- [x] 造成范围真实伤害（Nearest shape）
- [x] 每层出血 +20% 伤害（`RDamagePercentPerHemoStack`）

### 待实现
1. **斩杀后重置**
   - 3 级 R 击杀后完全刷新冷却
   - 其他等级 R 斩杀后 6 秒内可再次施放

2. **跃起不可选中**
   - R 跃起时有约 0.3 秒不可选中

---

## 被动 - 血怒

### 已完成
- [x] `BuffDariusBleed` 组件实现（叠加层数、DoT 伤害）
- [x] 出血叠加效果（通过 `on_darius_damage_hit`）
- [x] 5 层触发诺克萨斯之力（`BuffDariusMight`，+50% AD）
- [x] 出血 DoT（`DARIUS_BLEED_DOT_TAG` 标记避免无限叠层）
- [x] DoT 伤害公式：0.3×AD×层数 物理伤害

### 待实现
无

---

## 测试状态

### 已通过测试
- `darius_q_deals_damage`: Q 造成伤害 ✓
- `darius_q_applies_hemorrhage`: Q 施加出血 ✓
- `darius_w_goes_on_cooldown`: W 施法冷却 ✓
- `darius_w_applies_on_hit_buffs`: W 强化普攻效果 ✓
- `darius_e_cast_goes_on_cooldown`: E 施法冷却 ✓
- `darius_e_pulls_enemies_in_cone_to_feet`: E 锥形内拉回脚边 ✓
- `darius_e_does_not_pull_outside_cone`: E 锥形外不拉回 ✓
- `darius_e_applies_slow_to_pulled`: E 被拉敌人挂减速 ✓
- `darius_r_deals_damage`: R 造成伤害 ✓