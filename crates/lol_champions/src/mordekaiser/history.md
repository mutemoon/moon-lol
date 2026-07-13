# 莫德凯撒开发历史

## 2026-07-13 — 基础框架搭建

### 背景

`crates/lol_champions/src/lib.rs` 注释中明确铁男 `mordekaiser` 尚未实现，资产目录 `assets/characters/mordekaiser/` 与 `config.ron`、各技能 `.ron`、动画 `skin0.ron` 均已就绪，但缺少源码。本次任务为「收集莫德凯撒英雄的文档，以及创建英雄的基础代码框架」，目标是产出可编译的英雄骨架，后续技能按 TDD 流程逐步实现。

### 决策与过程

1. **文档先行**：以 `assets/characters/mordekaiser/spells/*.ron` 的 `dataValues` / `cooldownTime` 为数值权威来源，结合 LoL Wiki 机制描述，编写 `wiki.md`。所有数值均标注其 dataValue 名称，便于实现时与 `get_skill_value` / `get_skill_data_value` 对应。

2. **框架结构对齐既有英雄**：参照 `riven/` 的目录与 `mod.rs` 观察者模式，建立 `mordekaiser/` 目录：
   - `mod.rs`：注册 `PluginMordekaiser`、`Mordekaiser` 组件、统一施法观察者 `on_mordekaiser_skill_cast`
   - `passive.rs` / `q.rs` / `w.rs` / `e.rs` / `r.rs`：各技能 stub 函数，仅播放动画 + 日志占位
   - `buffs.rs`：声明被动层数、W 护盾、R 窃取等特有状态组件
   - `tests.rs`：测试 harness 配置 + 冒烟测试

3. **基础编译而非完整实现**：本阶段不实现任何伤害、位移、护盾逻辑，仅在 `lib.rs` 注册模块并保证 `cargo check -p lol_champions` 通过。各技能 stub 保留 TODO 注释，指明后续 TDD 实现要点。

### 局限性

- 当前 stub 仅播放施法动画，无任何实际技能效果
- 被动层数、W 储存护盾、R 死亡领域等复杂状态机尚未实现
- 测试仅覆盖英雄构造冒烟，技能行为测试待 TDD 阶段补充

### 资产对应

- 配置：`assets/characters/mordekaiser/config.ron`（实体 ID 4294967271 为 Mordekaiser 本体）
- 技能槽位：Q=4294967270、W=4294967269、E=4294967268、R=4294967267、Passive=4294967266
- 动画：`skin0.ron` 提供 `Spell1`(Q) / `Spell2`(W) / `Spell3`(E) / `Spell4`(R) / `Passive` 等剪辑
