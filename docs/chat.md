我认为没必要加一个 machine_state，因为这是可以从是否存在上一次锁定时间推断出来的，然后为什么要记录上一次锁定时间而不是记录未来发起攻击的时间，因为如果在锁定后突然攻速变快了，那么就不好改未来发起攻击的时间了，另外记录时间的话，我未来可能会加速游戏时钟，所以要使用 Time<Fixed> 对吗

需要更多测试，例如
两次攻击命令 CommandCommandAttack 的 target 不同或者相同时的处理
取消攻击命令（通常来自移动）在攻击生效前 2 帧之前可以被取消和之后不能被取消
发起攻击后，攻击生效前发生攻速变化，攻击生效时间会不会提前
多想一些单元测试，测试中要使用 CommandCommandAttack 作为攻击命令，因为在 Command 系统中我组合了 Target 系统和 Attack 系统的原子 Command

不要使用直接修改状态的 trick 通过测试

你理解错了，不是发起攻击后的两帧不可取消，而是攻击生效前的两帧是不可取消的，而且 AttackTimer 可以直接去掉了，需要保存的信息直接放在 AttackState 中，不要在 Windup 中存放 can_cancel

我已经决定不使用 rvo 进行寻路和避障了，而是采用 A* 算法直接规划路径，将这部分代码放在 navigation.rs，movement.rs 只负责按路径移动，而且 movement 去掉 MovementDestination，只保留 MovementPath，CommandMovementMoveTo 时只需要传一个 length 为 1 的 path，游戏的导航网格的定义在 config.rs 中，网格有预制的启发式值可以给 A* 算法用
