## 模块迁移计划

src/core 拆分到 crates 中 lol_core lol_core_render

# 指令

- 你正在迭代执行此计划，每次迭代只完成一个可验证的小步骤，完成后立即停止，自动进入下一次迭代

- 如果碰到 crate 间循环依赖的问题，直接修改 docs/MIGRATION_PLAN.md 重新计划，然后立即停止，自动进入下一次迭代

- 不要偷懒，留一堆 TODO、FUTURE、FIXME 在代码里，直接干完，不要跳过

- 不要询问任何问题，直接开干

- 禁止使用 pub use (Re-export)

- 彻底迁移，不要存在重复代码

- 如果迁移完成，请反复确认指令所提到的内容：禁止使用 pub use (Re-export)、彻底迁移逻辑，如果存在这类情况，需要修复

- 思考过程用中文
