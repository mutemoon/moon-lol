# 人类玩家反馈的问题

从上往下逐个解决，解决完成后标记为完成，用一句话概括代码的实现。没有问题就啥也别干，但是不要取消循环

## 问题列表

- [x] 补兵后没有跳出金币和经验的数字

抽象了通用 `FloatingNumber` 系统，新增 `EventGoldGain`/`EventExperienceGain` 事件，在 gold.rs 和 character.rs 中触发，floating_number.rs 中监听并创建飘字

- [x] 对于 SelfPlayer 击杀的小兵呢，跳出金币的数字应该在小兵死亡的位置，而不是英雄的位置跳出来，对于其它的英雄则不显示，只显示自己收获的金币

`EventGoldGain` 新增 `world_position` 记录死亡位置，`on_event_gold_gain` 使用死者位置并限制仅 `SelfPlayer` 显示

- [x] 经验的文字颜色不明显，改为白色

`on_event_experience_gain` 颜色从 GREEN 改为 WHITE

- [x] 小兵对英雄造成的伤害时，英雄身上不应该跳数字

`on_event_damage_create` 增加 `source` 检查，只有英雄对英雄的伤害才显示数字

- [x] 尸体不应该作为攻击指令的目标

`on_command_attack_start` 增加 `q_death.get(target)` 检查，目标有 `Death` 组件时跳过攻击

- [x] 我错了，小兵对英雄造成的伤害时，英雄身上应该跳数字，但是数字的大小要跟伤害大小有关

恢复 source 检查（小兵伤害也显示），font_size = clamp(14 + damage * 0.05, 14, 36) 自适应

- [x] 跳出来的金币的文字大1.5倍

金币字号从 22px 改为 33px