# Darius 待解决问题

## Q 技能 - 大杀四方

### 已完成
- [x] 内圈和外圈双重伤害形状实现
- [x] 内圈: Circle r=150
- [x] 外圈: Annular 150-350
- [x] Q 造成伤害（使用 ActionDamage）
- [x] Q 触发出血效果（通过 on_darius_damage_hit observer）

### 进行中
- [ ] 内圈伤害不叠加血怒，外圈叠加（需要区分伤害来源）

### 待实现

#### 高优先级
1. **区分内圈/外圈出血效果**
   - 当前 `on_darius_damage_hit` 对所有伤害都叠加血怒
   - 需要一种方式区分内圈和外圈伤害事件
   - 可能方案: 添加 effect_tag 到 EventDamageCreate

2. **从 Spell Asset 读取伤害数值**
   - 当前伤害值硬编码为 150/75
   - 需要实现从 `hash_bin("InnerDamage")` 和 `hash_bin("OuterDamage")` 读取

3. **外圈命中回复生命值**
   - 每个英雄回复 17% 已损失生命值
   - 最多 51%（3个英雄）

#### 中优先级
4. Q 技能粒子特效
5. Q 被打断的处理

#### 低优先级
6. Q 闪（Q Flash）机制
7. 内圈范围精细调整（实际约150）
8. 外圈范围精细调整（实际约350-400）

---

## W 技能 - 致残

### 已完成
- [x] 播放动画 (spell2)
- [x] 攻击重置 (CommandAttackReset)
- [x] 造成额外物理伤害
- [x] 对命中目标施加 50% 减速，持续 1 秒

### 待实现
无

---

## E 技能 - 无情立场

### 已完成
- [x] 播放动画 (spell3)

### 待实现
1. **拉回效果（位移系统）**
   - E 将敌人拉向自己
   - 需要位移系统支持

2. **减速效果**
   - 40% 减速，持续 1 秒
   - 对拉回的敌人施加

3. **被动护甲穿透**
   - 被动: 获得 20/25/30/35/40% 护甲穿透
   - 需要护甲穿透系统支持

---

## R 技能 - 诺克萨斯断头台

### 已完成
- [x] 播放动画 (spell4)
- [x] 造成范围伤害（Nearest shape）

### 待实现
1. **血怒层数增伤**
   - 每层血怒增加 30% 伤害（effectAmounts 中 RDamagePercentPerHemoStack）
   - 需要在伤害计算中考虑血怒层数

2. **斩杀后重置**
   - 3级 R 击杀后完全刷新冷却
   - 其他等级 R 斩杀后 6 秒内可再次施放

3. **跃起不可选中**
   - R 跃起时有约 0.3 秒不可选中

---

## 被动 - 血怒

### 已完成
- [x] BuffDariusBleed 组件实现
- [x] 出血叠加效果（通过 on_darius_damage_hit）

### 待实现
1. **5 层触发狂热状态**
   - 叠满 5 层时获得 +50% 总 AD
   - 需要 BuffDariusMight 实现和触发系统

2. **出血伤害计算**
   - 每层造成 12 物理伤害（随等级变化）
   - 需要周期性伤害系统

---

## 测试状态

### 已通过测试
- darius_q_deals_damage: Q 造成伤害 ✓
- darius_q_applies_hemorrhage: Q 施加出血 ✓
- darius_w_resets_attack: W 重置攻击 ✓
- darius_w_applies_slow: W 施加减速 ✓
- darius_e_cast_success: E 可以施放 ✓
- darius_r_deals_damage: R 造成伤害 ✓
