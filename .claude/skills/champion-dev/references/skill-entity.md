# 技能实体与数据结构

- 独立实体设计：技能本身是独立的 `ECS` 实体，而不是简单的索引。运行时状态直接挂载在技能实体本身。
- 英雄与技能关联：通过 Bevy 关系型组件模式关联。

# 核心关系组件

- `SkillOf`：挂载在技能实体上，指向所属的英雄实体。
- `Skills`：挂载在英雄实体上，持有包含所有主动技能实体 ID 的列表。
- `PassiveSkillOf`：挂载在被动技能实体上，指向所属的英雄实体。
- `PassiveSkill`：挂载在英雄实体上，持有被动技能实体 ID 的组件。
- 级联清理：由于使用了 `linked_spawn`，英雄实体销毁时关联的技能实体会自动销毁。

# 主动技能核心运行时组件

- `Skill`：存放元数据。
- `Skill.spell`：关联的技能配置资产句柄。
- `Skill.level`：当前等级，0 表示未学习。
- `Skill.slot`：槽位，可选 `SkillSlot::Q`、`SkillSlot::W`、`SkillSlot::E`、`SkillSlot::R`、`SkillSlot::Passive` 或 `SkillSlot::Custom(u8)`。
- `Skill.cooldown_mode`：冷却模式，可选 `SkillCooldownMode::AfterCast` 或 `SkillCooldownMode::Manual`。

# 冷却与连招组件

- `CoolDown`：挂载在技能实体上，其 `cooldown.timer` 控制冷却倒计时，`cooldown.duration` 记录基础冷却总时长。
- `SkillRecastWindow`：在多段技能流转中临时挂载于技能实体，其 `recast.stage` 记录下一段数，`recast.max_stage` 记录上限段数，`recast.timer` 控制重施窗口的超时倒计时。

# 辅助状态组件

- `SkillPoints`：挂载在英雄实体上，记录可用的技能加点数。
- `LOLSpells`：挂载在英雄实体上，通过哈希图映射技能名到对应的 `Spell` 资产句柄。
