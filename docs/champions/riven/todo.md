# Riven 待办

## 高优先级

- [x] Q 第三段击退 (knockback) 效果
- [x] W 眩晕 (CrowdControl) 效果
- [x] R 被动 buff（额外 AD、攻击距离、技能范围增加）
- [x] R 二段 Wind Slash（锥形范围伤害，基于已损失生命值增伤）
- [x] E 护盾值从配置/spell 数据读取，而非硬编码 100.0

## 中优先级

- [ ] Wind Slash 伤害改为 per-target calculation（当前使用平均 HP，需在 missile 系统增加 on-hit 伤害公式回调）
- [ ] 被动充能层数限制（最多 3 层），平 A 消耗一层
- [ ] Q 重置普攻计时器
- [ ] 技能升级影响冷却时间
- [ ] 测试覆盖：三段 Q 伤害数值验证

## 低优先级

- [ ] Fast Q 动画取消机制
- [ ] E+R 动画取消
- [ ] Q 第三段穿墙检测
