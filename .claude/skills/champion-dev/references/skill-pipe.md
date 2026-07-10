# 统一施法管线

- 管线定位：位于 `lol_core::skill::observers::on_skill_cast`，仅负责公共施法前置校验与事件分发，不处理具体技能行为。
- 执行链路：输入动作触发后派发命令，通过管线校验后派发施法成功事件，最终由英雄观察者处理具体逻辑。

# 核心命令与事件

- `CommandSkillStart`：施法指令，包含施法者实体、槽位索引与目标点。
- `EventSkillCast`：施法成功事件，由校验通过后的管线触发，包含施法者实体、技能实体、索引与目标点。

# 调试与断言数据

- `SkillCastFailureReason`：施法失败原因分类，包含缺少技能组件、无效索引、技能未学习、资源不足、冷却中、被阻塞、施法者死亡等。
- `SkillCastRecord`：记录单次施法的最终状态和结果。
- `SkillCastLog`：存储在世界中的施法日志历史资源，测试中常用于检索以进行施法断言。

# 校验网关与过滤顺序

- 死亡检查：施法者已死亡则拦截，返回 `SkillCastFailureReason::CasterDead`。
- 阻塞检查：如果施法者身上包含 `CastBlock` 标记组件则拦截，返回 `SkillCastFailureReason::Blocked`。
- 实体检索：如果无法根据索引在施法者身上检索到技能实体，返回 `SkillCastFailureReason::MissingSkillEntity`。
- 重施判定：如果技能实体挂载了未过期的 `SkillRecastWindow`，跳过冷却时间校验。
- 冷却校验：若不满足重施判定，且技能实体的 `cooldown.timer` 未结束，拦截并返回 `SkillCastFailureReason::CoolingDown`。
- 学习校验：如果技能等级为 0 则拦截，返回 `SkillCastFailureReason::NotLearned`。
- 资源扣除：校验通过后，从施法者的 `AbilityResource` 中扣减 `Spell` 中对应当前等级的资源消耗。如果资源不足则拦截，返回 `SkillCastFailureReason::InsufficientAbilityResource`。
- 派发事件：校验全部通过，派发 `EventSkillCast` 成功事件。
- 进入冷却：如果技能为 `SkillCooldownMode::AfterCast` 且非重施窗口，由管线自动启动冷却计时器。
