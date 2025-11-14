## 系统

- 移动系统
  不关心是谁下达的移动指令，只关心移动方式和移动速度，负责按帧修改 transform 组件。
- 攻击系统
  不关心与目标相隔多远，只关心攻击谁，负责攻击状态（前摇、冷却）的更新。
- 自动攻击系统
  通过查询攻击者与目标的距离，不在攻击范围则对寻路系统下达寻路移动指令，在攻击范围则下达攻击指令和移动停止指令。
- 技能系统
  在收到含有位移效果的技能时，命令移动系统沿着技能所指的方向进行移动。
- 控制器系统
  为实体上的技能数组注册快捷键，输入触发对应技能的释放，包括移动、停止。

当自动攻击指令、移动指令、位移位移指令争抢最终移动系统的控制权时，比如自动攻击向目标靠近时收到了移动指令需要停止攻击改为移动，当位移技能位移时攻击指令和移动命令都会被忽略，传统命令式的设计需要移动指令中手动停止自动攻击行为、移动命令需要检查是否处于位移状态。当越来越多的指令争抢移动控制权的时候，每个系统都需要知道其它系统的存在，那么该如何使用声明式的设计来解决这个问题呢

## 移动权争夺问题

Run 指令，位移指令

1. 命令式

当 Run 指令触发时，命令一次移动系统移动，当位移指令触发时，命令一次移动系统按路径快速位移。

问题：指令并发时，新指令会覆盖旧指令的移动，没有优先级。

2. 优先级式

移动系统保存上一次移动命令的优先级，对比当前移动命令的优先级，决定是否进行新的移动。

优点：优先级高的指令不会被优先级低的指令覆盖。
问题：先 Run 指令移动，然后位移指令移动，位移指令移动结束后，不能继续之前的 Run 指令。

3. 优先级 + 持续命令式

保存上一次移动命令的优先级，对比当前移动命令的优先级，决定是否进行新的移动。 Run 指令系统通过添加 Run 组件，Run 系统持续发出低优先级的移动命令。

优点：高优先级的移动命令结束时会立即恢复到低优先级的移动命令。
问题：需要移除 Run 组件

## 在什么条件下移除 Run 组件

在到达 Run 的目的地时被移除

如何判断是否到达目的地

1. 在 Run 系统中检测与目的地的距离，如果小于移动速度 \* dt 的距离则认为抵达目的地。

问题：移动系统中已经存在是否到达目的地的检测了，会有冗余

2. 使用移动系统的 EventMovementEnd 事件，移除 Run 组件。

问题：位移的移动结束不应该移除 Run 组件。

3. EventMovementEnd 事件加一个 type 字段，当 type 类型为 Run 时，移除 Run 组件

问题：需要维护一个包含所有类型 MovementType 的 enum，每增加一种类型的移动，就需要增加一个变体。

## 先寻路还是先决定移动

先寻路的话，寻路会耗时，耗时后决定不移动的话就浪费了，所以是先决定是否移动，再寻路

## 自动攻击系统与移动

自动攻击系统 = Run 系统 + 攻击循环系统

当处于攻击范围外时：添加 Run 组件，命令一次攻击系统停止攻击
当处于攻击范围内时：移除 Run 组件，命令一次攻击系统攻击

问题：当移除 Run 组件后，移动系统可能仍处于移动中。

1. Run 组件移除时下达移动停止指令。

优点：解决了移除 Run 组件后会继续移动的问题
问题：移除 Run 组件时，如果此时处于位移移动状态，位移会被 Run 组件的移除而中断。

2. Run 组件移除时下达移动停止指令，移动停止指令的优先级要对比上一次移动命令的优先级才决定是否停止移动。

优点：解决了高优先级的移动被低优先级的停止移动命令中断的问题

## 原版逻辑

1. 在攻击范围内

无论攻击是否就绪，不进行任何移动，攻击就绪时直接开始攻击，攻击未就绪时等待攻击就绪开始攻击。

2. 在攻击范围外

会向目标移动，进入攻击范围内后：

- 攻击就绪时

停止移动，开始攻击目标。

- 攻击未就绪时

继续移动，直到敌人的 bounding 完全处于攻击范围内停止移动，等待攻击就绪，一旦攻击就绪则开始攻击。

## 自动攻击实现

调整攻击系统，收到攻击指令时检测是否处于攻击范围，若不处于攻击范围则不开始攻击。

每帧发出攻击指令。

添加 Run 组件，每帧发起移动到目标的指令。

收到攻击开始事件时，移除 Run 组件，即停止移动。

优点：时刻发起攻击，一旦成功发起攻击，则停止移动。
问题：攻击未就绪时会一直移动向目标移动。

每帧检测是否处于攻击前摇，若处于攻击前摇则不进行攻击范围检测及后续逻辑。

每帧检测是否处于攻击范围内：

- 如果处于攻击范围内，则尝试移除 Run 组件，保证不进行移动，并且每帧发起攻击指令。
- 如果不处于攻击范围内，则添加 Run 组件，每帧发起移动到目标的指令。

优点：

- 时刻发起攻击，处于攻击范围内时无论是否攻击就绪都不会移动，处于攻击范围外时会向目标移动，移动到攻击范围内时会停止移动并尝试攻击。
- 攻击前摇时，敌人离开攻击范围，不会出现一边向敌人移动，一边进行攻击的问题。
  问题：攻击未就绪时进入攻击范围时会停止移动，而原版是攻击未就绪时仍会移动，直到敌人的 bounding 完全处于攻击范围内才会停止移动。

## 自动攻击系统与移动系统的交互

AutoAttack FixedUpdate
├── Attack FixedUpdate
└── Run FixedUpdate
└── Movement PostUpdate

Run 在 FixedUpdate 发出 CommandMovementStart 指令
处于攻击范围时，AutoAttack 在 FixedUpdate 发出停止 CommandRunStop 指令，CommandRunStop 监听器移除 Run 组件并发出 CommandMovementStop 指令
Run 和 AutoAttack 都处于 FixedUpdate 中，所以会出现 Run 先发出 CommandMovementStart 指令，AutoAttack 再发出 CommandRunStop 指令，导致 Run 移除滞后。
而当 Movement 在一帧中收到同时收到 CommandMovementStart 指令和 CommandMovementStop 指令时，由于 CommandMovementStart 需要先收集到 RequestBuffer 再 Reduce 和 Apply ，所以 CommandMovementStop 还需要清理 RequestBuffer 中的 CommandMovementStart 指令。
就算清理了 RequestBuffer ，CommandMovementStart 在 CommandMovementStop 之后执行的话，先 Stop 再 Start ，也会无法停止移动。

- Run 每帧发出 CommandMovementStart 指令有问题吗？
  Run 需要持续发出低优先级的移动指令，才能确保高优先级移动结束时立即执行低优先级的 Run

- 处于攻击范围时，AutoAttack 每帧发出停止 CommandRunStop 指令有问题吗？
  AutoAttack 每帧发出移动停止指令，是为了确保在攻击范围内时，不管攻击是否就绪都停止移动。

1. 调整 AutoAttack 和 Run 的执行顺序，确保 AutoAttack 在 Run 之前执行。

Run 是一个更基础的系统，不应该因为上层系统调整自己的执行顺序，所以只能调整 AutoAttack 的执行阶段为 FixedPreUpdate。

优点：Run 组件在未发出移动指令之前被移除。
问题：一帧内，CommandMovementStop 在 CommandMovementStart 后面出现时，依然会开始移动。

## 移动指令的问题

1. 将 Movement 开始和停止的命令统一放进 RequestBuffer ，在 Reduce 阶段检测到 RequestBuffer 存在停止命令时，只以停止移动为最终决策。

问题：在同一帧内，先收到 Stop 再收到 Start 时，会无脑 Stop

2. 收到 CommandMovementStop 时清理 RequestBuffer 中的 CommandMovementStart 指令。

优点：确保在同一帧内，先收到 Start 再收到 Stop 时，不会开始移动。

3. 将 CommandMovementStart 和 CommandMovementStop 合并为 CommandMovement 指令，取其中 priority 最高的作为最终决策。

[Start-0, Stop-0]
取 Stop-0 作为最终决策

[Start-0, Stop-1]
取 Stop-1 作为最终决策

[Start-1, Stop-0]
取 Start-1 作为最终决策

[Start-1, Stop-1]
取 Stop-1 作为最终决策

[Start-1, Stop-0, Start-2]
取 Start-2 作为最终决策

[Start-1, Stop-2, Start-2]
取 Start-2 作为最终决策

[Start-2, Stop-1, Start-1]
取 Start-2 作为最终决策

[Start-1, Stop-2, Start-1]
取 Stop-2 作为最终决策

问题：

- 没有考虑上一次的最终决策，比如上一次最终决策为 Start-1，这一次收到 Start-0，则应该忽略
- [Start-1, Stop-2, Start-1] 实际上应该取 Start-1 作为最终决策

4. 将 CommandMovementStart 和 CommandMovementStop 合并为 CommandMovement 指令，buffer 在开始的地方加入上次的决策，依次按顺序模拟执行，高的指令会覆盖低的指令，除了 start 不管多低都覆盖高的 stop。

[Start-1, Stop-2, Start-1]
取 Start-1 作为最终决策

# 伤害与生命

问题1：