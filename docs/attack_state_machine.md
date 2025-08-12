# 攻击状态机设计

## 概述

攻击系统已经从基于时间的简单状态管理重构为更语义化的状态机。新的设计提供了清晰的状态转换和更好的可维护性。

## 状态定义

### AttackStatus 枚举

```rust
pub enum AttackStatus {
    /// 空闲状态 - 没有攻击目标
    Idle,
    /// 锁定状态 - 已锁定目标，准备攻击
    Locked { target: Entity, lock_time: f32 },
    /// 攻击中状态 - 正在执行攻击
    Attacking { target: Entity, attack_start_time: f32 },
    /// 冷却状态 - 攻击后冷却中
    Cooldown { target: Entity, cooldown_end_time: f32 },
}
```

## 状态转换

### 状态转换图

```
Idle → Locked → Attacking → Cooldown → Idle
  ↑                                    ↓
  └─────────────── 重新锁定 ←──────────┘
```

### 状态转换方法

- `lock_target(target, current_time)` - 从 Idle 转换到 Locked
- `start_attack(current_time)` - 从 Locked 转换到 Attacking
- `finish_attack(cooldown_duration, current_time)` - 从 Attacking 转换到 Cooldown
- `end_cooldown()` - 从 Cooldown 转换到 Idle

## 使用示例

### 基本用法

```rust
use moon_lol::core::{Attack, AttackState, AttackStatus};

// 创建攻击状态
let mut attack_state = AttackState::default();

// 锁定目标
attack_state.lock_target(target_entity, current_time);

// 检查状态
if attack_state.is_locked() {
    // 开始攻击
    if let Some(target) = attack_state.start_attack(current_time) {
        // 执行攻击逻辑
    }
}
```

### 状态查询

```rust
// 获取当前目标
let target = attack_state.get_target();

// 检查特定状态
if attack_state.is_idle() {
    // 可以接受新目标
}

if attack_state.is_cooldown() && attack_state.is_cooldown_finished(current_time) {
    // 冷却结束，可以重新攻击
}
```

## 优势

1. **语义化**: 状态名称清晰表达了当前状态的含义
2. **类型安全**: 使用枚举确保状态的有效性
3. **易于扩展**: 可以轻松添加新的状态和转换逻辑
4. **状态验证**: 状态转换方法确保转换的有效性
5. **调试友好**: 状态信息包含相关的上下文数据

## 迁移指南

### 旧代码

```rust
// 旧的方式
if attack_state.last_lock_time.is_none() {
    attack_state.last_lock_time = Some(time.elapsed_secs());
    attack_state.target = Some(target);
}
```

### 新代码

```rust
// 新的方式
if attack_state.is_idle() {
    attack_state.lock_target(target, time.elapsed_secs());
}
```

## 测试

运行测试以确保状态机正常工作：

```bash
cargo test attack::tests --lib
```

## 示例程序

查看 `examples/attack_state_machine.rs` 了解完整的使用示例。

