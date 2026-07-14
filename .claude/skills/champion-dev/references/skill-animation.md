# 动画系统

- 配置位置：每个英雄的动画名与节点映射定义在 `assets/characters/{champion}/animations/skin{N}.ron` 中。代码里播放动画时使用的名字必须与该文件中的 key 完全一致，区分大小写。
- 资产类型：`LOLAnimationGraph`，包含 `gltf_path`、`hash_to_node`（动画名 -> `ConfigAnimationNode`）、`blend_data`。

# 节点类型（ConfigAnimationNode）

- `Clip { node_index }`：单个动画片段，最常见。如 `"Spell1": Clip(node_index: 10)`。
- `ConditionFloat { updater, conditions }`：按某个 float 参数在多个子片段间混合。典型如 `Run = ConditionFloat(updater: MoveSpeed, [(RunWalk, 0.0), (RunFast, 380.0)])`，移速 345 时 RunWalk 权重 1、RunFast 权重 0。
- `Selector { probably_nodes }`：按权重随机选一个子片段播放（多变体）。如 `Idle1`/`Attack` 有多个动作变体。
- `Sequence { hashes }`：按顺序播放一组片段。如 `Recall`。
- `Parallel { hashes }`：同时播放多个片段。
- `Parametric { pairs }`：按参数在多对片段间混合。
- `ConditionBool { updater, true_node, false_node }`：按布尔条件选 true/false 子节点。

# 核心组件

- `LOLAnimationGraphHandle`：挂在角色实体上，持有动画图资产句柄，`#[require(LOLAnimationState)]` 自动附带状态组件。
- `LOLAnimationState`：动画状态机的当前状态。字段：`current`/`last`（当前与上一个动画名）、`current_duration`、`repeat`，以及 `selector_states`/`sequence_states`（记录 Selector/Sequence 的运行时选择，决定下次播放哪个变体）。默认 `current = "Idle1"`、`repeat = true`。
- `AnimationConfigOf` / `AnimationConfig`：角色实体与骨骼实体（持有 `AnimationPlayer`、`AnimationGraphHandle`）之间的 Bevy 关系型组件，`linked_spawn` 级联。
- `AnimationTransitionOut`：旧动画淡出过渡，字段 `hash`/`weight`/`duration`/`start_time`，默认 100ms。

# 状态驱动

- `State`（`Idle`/`Running`/`Attacking`）变化触发 `on_state_change`，调用 `LOLAnimationState.update(动画名)`：
  - `Idle` -> `"Idle1"`，`Running` -> `"Run"`，`Attacking` -> `"Attack"`（`repeat=false`，`duration=攻击动画时长`）。
- `CommandAnimationPlay` 事件（技能/指令主动触发）由 `on_command_animation_play` 观察者处理，调用 `state.update(hash)`，可附带 `repeat`/`duration`。

# 渲染系统（PluginAnimation，均在 Update）

- `on_state_change`：`State` -> `LOLAnimationState`（写入当前动画名，mark Changed）。
- `on_animation_state_change`：监听 `Changed<LOLAnimationState>`，执行真正的播放：若存在旧动画则插入 `AnimationTransitionOut` 淡出、`play(current)`、按 `repeat` 设置循环。
- `update_transition_out`：每帧推进 100ms 淡出过渡，完成后 `stop` 旧片段并移除过渡组件。
- `update_condition_animation`：每帧按参数（如 `MoveSpeed`）更新 `ConditionFloat` 节点子片段的权重。
- `apply_animation_speed`：按参数（如 `AttackSpeed`）调整播放速率。

# LOLAnimationGraph 方法

- `play(player, key, weight, state)`：递归展开 `key` 的全部叶节点片段并 `player.play(...)`（从第 0 帧启动），设置权重。
- `repeat(player, key, state)`：对当前播放的片段设置循环。
- `stop(player, key, state)`：停止片段并清理 `Selector` 的选择状态。
- `set_weight` / `set_speed` / `get_weight`：调整正在播放片段的权重/速率。

# 开发要点与坑

- 动画名必须与 `skin{N}.ron` 中的 key 一致，区分大小写。
- `play()` 会从第 0 帧重启片段；`set_weight`/`set_speed` 对未在播放的片段是空操作，必须先 `play` 启动。
- `ConditionFloat` 的全部子片段必须由 `play` 一次性启动，再由 `update_condition_animation` 每帧调权；若只启动一个子片段，未启动的 `set_weight` 无效，表现为动画冻结/错误。
- `stop()`、`set_weight()` 等只读或清理 `LOLAnimationState` 内部数据（如 `selector_states`）的调用，应传入 `state.bypass_change_detection()` 而非 `&mut state`。否则 Bevy 的 `Mut::deref_mut` 会 `set_changed()`，误触发 `on_animation_state_change` 重启当前片段，造成每 100ms 重启一次的抽帧。
