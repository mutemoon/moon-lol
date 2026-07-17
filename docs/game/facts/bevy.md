## 调度顺序

- First
- PreUpdate
- StateTransition
- RunFixedMainLoop（在此内部可能运行 0 次或多次 Fixed 相关的 Schedule）
  - FixedFirst
  - FixedPreUpdate
  - FixedUpdate
  - FixedPostUpdate
  - FixedLast
- Update
- SpawnScene
- PostUpdate
- Last

## 输入

- 键盘输入的更新发生在 PreUpdate 阶段

## 游戏加速

### 调整虚拟时钟速度

- 适合普通游戏加速
- 通过 `Time<Virtual>::set_relative_speed(10.0)` 实现
- 优点：超高速时，只在 `RunFixedMainLoop` 中大量循环，纯粹的 `FixedUpdate`，`Update` 运行次数极少
- 问题：不能手动 step

### 固定 `Update` 时间更新的步长，手动 `update()`

- 适合测试、强化学习这类需要精确 step 的场景
- 通过 `TimeUpdateStrategy::FixedTimesteps(1)` + `app.update()` 实现
- 优点：可以精确 step
- 问题：`app.update()` 使得 `FixedUpdate` 和 `Update` 都执行了 1 次，与调整虚拟时钟速度相比，需要考虑 `Update` 的性能损耗

## 强化学习

- 榨干 CPU 性能的配置：`SingleThreadedExecutor` + `TimeUpdateStrategy::FixedTimesteps(1)` + `app.update()` + 多线程
- 原因：bevy 本身是多线程的，但是改为单线程后，结合多线程的 bevy 实例，来减少 bevy 内多线程上下文切换的性能损耗，从而榨干 CPU
