---
name: movement-state
description: lol_core 的移动控制权争夺、自动攻击系统、CommandMovement 仲裁管线的设计与实现。
---

> 本技能描述 `crates/lol_core` 中移动/Run/AutoAttack 系统的**当前实现**，以及这些设计当初为何这样取舍。调试移动控制权问题、Run 组件不按预期移除、自动攻击一边移动一边攻击等问题时先读这里。

# 系统概览

| 系统 | 职责 | 实现位置 |
|---|---|---|
| 移动系统 | 不关心谁下达移动指令，只关心移动方式和速度，按帧改 transform | `crates/lol_core/src/movement.rs`（`update_path_movement`） |
| 攻击系统 | 不关心与目标距离，只关心攻击谁，负责前摇/冷却状态 | `crates/lol_core/src/attack.rs` |
| 自动攻击系统 | 查询与目标距离，范围外→Run 寻路，范围内→攻击+停移动 | `crates/lol_core/src/attack_auto.rs` |
| 技能系统 | 含位移效果的技能命令移动系统沿方向移动 | `crates/lol_champions/*`（见 `champion-dev` 技能） |
| Run 系统 | 持有 `Run` 组件期间，每帧发出低优先级移动命令 | `crates/lol_core/src/run.rs` |

# 核心难题：移动控制权争夺

当自动攻击、移动、位移技能争抢移动系统的控制权时（例：自动攻击正靠近目标时收到玩家移动指令；位移技能期间应忽略攻击和移动命令），命令式设计要求每个系统知晓其它系统的存在。本项目用**声明式仲裁管线**解决。

# 仲裁管线（ArbitrationPipeline）

通用框架在 `crates/lol_core/src/base/pipeline.rs`，移动用 `ArbitrationPipelinePlugin::<CommandMovement, MovementPipeline>` 装载（`movement.rs:28`）。复用于旋转（`CommandRotate`）等其它通道。

## 数据流

```
CommandMovement（Event，触发到实体）
  → accumulate_requests 观察者把事件塞进 RequestBuffer<CommandMovement>
  → [Reduce 阶段] reduce_movement_by_priority：以 LastDecision 为起点，逐条比较，产出 FinalDecision
  → [Apply 阶段] apply_final_movement_decision：执行 Start（寻路/设路径）或 Stop（清路径）
  → [Cleanup 阶段] FinalDecision → LastDecision，RequestBuffer 清空
```

全部在 `FixedPostUpdate`，按 `Modify → Reduce → Apply → Cleanup` 链式排序（`pipeline.rs:52-55`）。

## CommandMovement

```rust
// movement.rs:88-103
pub struct CommandMovement {
    pub entity: Entity,
    pub priority: i32,              // 越高越优先；Run 用 0（最低）
    pub action: MovementAction,
}
pub enum MovementAction {
    Start { way: MovementWay, speed: Option<f32>, source: MovementSource },
    Stop,
}
```

`MovementSource`（`movement.rs:51-61`）是一个**开放的来源枚举**：`Run` / `Dash` / `Knockback` / `Missile` / `Player` / `AI` / `Skill(String)` / `Pathfind`。用于在 `EventMovementEnd` 上过滤来源（见下）。

## reduce_movement_by_priority 裁决规则（movement.rs:311-364）

以实体的 `LastDecision<CommandMovement>` 作为起点（解决「不考虑上次决策」的问题），然后逐条比较缓冲区里的命令：

| 当前决策 | 收到命令 | 结果 |
|---|---|---|
| 任意 | `Start`（对手是 `Stop`） | **Start 总是覆盖 Stop**（不管优先级） |
| `Start{p1}` | `Start{p2}` | 取 `p2 >= p1` 则 p2，否则保持 |
| `Start{p1}` | `Stop{p2}` | 取 `p2 >= p1` 则 Stop，否则保持 |
| `Stop{p1}` | `Start{p2}` | Start 总是覆盖 Stop → 取 Start |
| `Stop{p1}` | `Stop{p2}` | 取 `p2 >= p1` 则 p2，否则保持 |

「Start 不管多低都覆盖 Stop」是为了避免「先 Stop 再 Start」在同一帧内被无脑停掉。

真值表（`p` = priority）：

| 缓冲区序列 | 最终决策 |
|---|---|
| `[Start-0, Stop-0]` | Stop-0 |
| `[Start-0, Stop-1]` | Stop-1 |
| `[Start-1, Stop-0]` | Start-1 |
| `[Start-1, Stop-1]` | Stop-1 |
| `[Start-1, Stop-0, Start-2]` | Start-2 |
| `[Start-1, Stop-2, Start-2]` | Start-2 |
| `[Start-2, Stop-1, Start-1]` | Start-2 |
| `[Start-1, Stop-2, Start-1]` | Start-1（靠「Start 覆盖 Stop」+ 种子 LastDecision） |

# Run 系统（run.rs）

- `CommandRunStart { target: RunTarget }` → 观察者插入 `Run { target }` 组件。
- `CommandRunStop` → 观察者移除 `Run` 组件，并触发 `CommandMovement { priority: 0, Stop }`。
- `Run` 在 `FixedUpdate` 每帧发出 `CommandMovement { priority: 0, Start{Pathfind(..), source: Run} }`。
  - **必须每帧重发低优先级移动**：高优先级移动（位移）结束后，靠下一帧 Run 的 Start 立即恢复移动。
- `EventMovementEnd { source }`：仅当 `source == MovementSource::Run` 时移除 `Run` 组件（位移/Dash 结束不会误删 Run）。同时移除 `LastDecision<CommandMovement>`（`movement.rs:521-525`）。

# 自动攻击系统（attack_auto.rs）

注意代码里叫 `AttackAuto`（不是 `AutoAttack`）。

- `CommandAttackAutoStart { target }` → 观察者跑一次 `process_attack_logic` 并插入 `AttackAuto { target }` 组件。
- `update_attack_auto` 跑在 **`FixedPreUpdate`**（关键，见下「执行顺序」），且仅在 `Changed<Transform>` 时重跑；攻击前摇（`AttackStatus::Windup`）期间跳过。
- `process_attack_logic`（attack_auto.rs:131-169）按平方距离判定：
  - 距离 > `range + radius + target_radius`：触发 `CommandRunStart{Target}`（向目标 Run）+ `CommandAttackStop`。
  - 否则：触发 `CommandRunStop` + `CommandAttackStart{target}`。

> ⚠️ 当前实现是**简单的距离判定**（含双方 bounding radius），**未实现**原版 LOL 的「攻击未就绪时继续移动直到目标 bounding 完全处于攻击范围内才停」的 overshoot 规则（见下方「设计史/未实现」）。

# 执行顺序：为何 AttackAuto 在 FixedPreUpdate

```
FixedPreUpdate:  update_attack_auto   （移除/添加 Run）
FixedUpdate:     run::fixed_update     （Run 每帧发 CommandMovement Start）
                ... attack system ...
FixedPostUpdate: MovementPipeline      （RequestBuffer 收集 → Reduce → Apply → Cleanup）
```

若 `AttackAuto` 也在 `FixedUpdate`，会和 `Run` 同帧，出现「Run 先发 Start、AttackAuto 再发 Stop」的顺序耦合，导致 Run 移除滞后、同一帧 Start/Stop 乱序。把 `AttackAuto` 提到 `FixedPreUpdate` 后，移除 Run 发生在 Run 发出移动指令之前。

同帧内「先 Start 再 Stop」的剩余歧义由仲裁管线的 Reduce 规则兜底（见上真值表）。

# 调试指引

- 移动相关日志分类：`EnumLogCategory::Movement` / `Run` / `AttackAuto`（搜 `CommandLog` + 这些分类）。
- 怀疑控制权丢失：检查发出 `CommandMovement` 时是否带了正确的 `priority` 和 `source`；Run 固定 priority 0。
- 怀疑 Run 不被移除：确认移动结束事件的 `source` 是否为 `MovementSource::Run`（其它来源不会移除 Run）。
- 怀疑仲裁结果异常：`reduce_movement_by_priority` 的起点是 `LastDecision`，注意上一帧的决策会影响本帧。

# 设计史（为何选这些方案）

> 这些是 `docs/state.md` 推理过程的结论，已被代码采纳。保留供调试时理解「为什么」。

- **为何合并 `CommandMovementStart`/`Stop` 为 `CommandMovement{action, priority}`**：分离式无法在同一帧内正确处理「先 Start 再 Stop」或「先 Stop 再 Start」；合并后用 priority 统一裁决。
- **为何 Reduce 以 `LastDecision` 为种子**：仅看本帧缓冲区会丢失「上一帧最终决策」上下文，导致 `[Start-1, Stop-2, Start-1]` 错误地取 Stop-2；种子修正后取 Start-1。
- **为何 `EventMovementEnd` 带 `source` 而非 `MovementType` 枚举**：早期方案担心每加一种移动就要加一个 enum 变体；实际改用开放的 `MovementSource`（含 `Skill(String)` 等动态变体），过滤逻辑只在 `source == Run` 时移除 Run。
- **为何先决定再寻路**：寻路耗时，若决定不移动则寻路被浪费。`apply_final_movement_decision` 先裁决出 Start，才在 Apply 阶段触发寻路，且有 `need_replan` 检查（目标未变 + 路径无阻挡时跳过重算）。

## 未实现 / 仅设计

- **原版「攻击未就绪时移动到 bounding 完全入射程才停」的 overshoot 规则**：`process_attack_logic` 当前只做距离判定，没有「攻击就绪 vs 未就绪」分支的差异化停止点。
- **`docs/state.md` 末尾「# 伤害与生命」节**：原文档在此处截断（仅一行 `问题1：`，无内容），未落地。
