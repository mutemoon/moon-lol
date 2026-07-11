# Riven 反馈

## 2026-07-11 - Q1 释放后应显示可释放 Q2 而非直接显示 CD

**需求**: 锐雯的 Q1 释放后，虽然已经进入 cd 但应该显示仍然可释放 q2，而不是直接显示 cd。

**处理**: 见 [history.md](./history.md) 2026-07-11 条目。根因在 UI 显示层（`lol_render/src/ui/skill.rs` 忽略 `SkillRecastWindow`），冷却机制本身（AfterCast，从 Q1 起算）符合设计、未改动。已通过新增 `is_skill_ready` 共享判定修复，并补充测试。
