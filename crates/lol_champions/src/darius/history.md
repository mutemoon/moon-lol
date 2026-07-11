# Darius 开发历史

## 2026-07-10 - E 技能拉回 + 击飞 + 减速实现

### 背景
位移框架组合化重构（Phase 1 已完成启用新原语）。E 此前是纯 stub（只播动画，
`mod.rs` 注释"拉回效果待实现"）。Phase 4.1 用 Darius E 验证"灵活组合大于上帝参数"
的论点：拉回技能不需要专属字段，由通用原语组合而成。

### 实现内容
1. 新建 `crates/lol_champions/src/darius/e.rs`，`mod.rs` 的 stub `cast_darius_e` 删除。
2. TDD：先写 `e_tests.rs`（锥形内拉到脚边 / 锥形外不动 / 被拉挂 40% 减速），红→绿。

### 组合表达（核心决策）
E 是纯 CC（wiki 无伤害值），**不走 `ActionDamage`**（否则会触发出血），而是三原语组合：
- **锥形查询**：复用 `DamageShape::Sector` 的几何（朝向、半径、半角），但直接空间
  查询敌人。朝向取施法点方向（`trigger.point - pos`），施法点与自身重合时退回
  `Transform::forward()`。
- **拉回 + 击飞**：`CommandKnockback { direction: Toward, distance: 535, duration: 0.75 }`。
  `Toward` 自动钳制不越过 source，故 distance 传范围上限即可拉到脚下；击飞
  （`DebuffKnockup`）由 `on_command_knockback` 自动施加，无需另写。
- **减速**：`DebuffSlow(0.4, 1.0)`，作为副作用挂在每个被拉敌人上。
- Darius 自身不位移（只对敌人触发 `CommandKnockback`）。

### 数值
- 范围 535、锥角 90°、击飞 0.75s、拉回速度 1200、减速 40%/1s。

### 局限性 / 待解决
- 护甲穿透被动未实现（待护甲穿透系统）。
- 锥角 90° 是估算值，wiki 未给出精确角度。
- 拉到 source 精确重叠（Toward 语义），未保留小偏移；后续若需"拉到身前一段"可改
  `CommandKnockback` 的 distance 或引入 offset。
- 测试中 `darius_e_does_not_pull_outside_cone` 在 stub 阶段也过（stub 谁都不拉），
  实现后靠锥形过滤继续通过。

## 2026-05-11 - 技能测试框架和基础实现

### 完成的工作
1. 创建 `crates/lol_champions/src/darius/tests.rs` 测试文件
   - 测试 Q 造成伤害
   - 测试 Q 施加出血效果
   - 测试 W 重置攻击
   - 测试 E 可以施放
   - 测试 R 造成伤害

2. 修复 `Darius` 组件缺少 `Default` trait 的问题

3. 实现 W 技能额外伤害
   - W 现在使用 `ActionDamage` 造成伤害
   - 仍然触发 `CommandAttackReset` 重置攻击

4. 修复出血效果检查逻辑
   - 原来直接查询 `BuffDariusBleed` 组件
   - 改为通过 `Buffs` 组件和关系查询

### 待解决的问题
- W 减速效果未实现（当前对所有伤害施加减速，包括 Q）
- E 拉回效果未实现（需要位移系统）
- Q 内圈不叠血怒未实现（需要区分伤害来源）
- 5 层血怒触发 AD 加成未实现
- R 血怒层数增伤未实现

## 2026-04-26 - Q 技能内圈/外圈伤害实现

### 背景
用户要求实现 Darius Q 技能（大杀四方）的测试版本，其中内圈伤害和外圈伤害不一致。

### 实现内容
1. 创建 `crates/lol_champions/src/darius/q.rs` 模块
2. 实现 `cast_darius_q` 函数，支持内圈和外圈双重伤害

### 技术细节

**内圈（斧柄）**:
- 半径: 150
- 伤害: 外圈的 50%
- 不叠加血怒（Hemorrhage）

**外圈（斧刃）**:
- 半径: 150-350 (环形)
- 伤害: 100%
- 叠加血怒

**伤害数值**（5级）:
- 外圈: 150 + 0.9 AD
- 内圈: 75 + 0.45 AD

### 使用的数据结构
- `DamageShape::Circle` - 内圈伤害
- `DamageShape::Annular` - 外圈伤害（环形，从150到350）
- `ActionDamageEffect` - 两种效果组合

### 局限性
1. 目前内圈和外圈的出血效果由同一个 observer (`on_darius_damage_hit`) 处理，无法区分
2. `apply_hemorrhage_outer` 参数目前未使用，内部出血逻辑尚未实现
3. 伤害值目前是硬编码的测试值 (150/75)，应从 Spell 数据中读取

### 待解决
- [ ] 实现内圈伤害不叠加血怒的逻辑
- [ ] 从 Spell Asset 读取实际伤害数值
- [ ] 实现外圈命中的生命回复 (17% 已损失生命值)
