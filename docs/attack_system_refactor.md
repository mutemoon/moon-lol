# League of Legends Attack System Implementation

## Overview

This document describes the refactored attack system that implements League of Legends attack mechanics with high fidelity to the original game.

## Key Features Implemented

### 1. Attack Speed System (攻击速度)

- **Formula**: `current_attack_speed = base_attack_speed * (1 + bonus_attack_speed)`
- **Attack Interval**: `1 / current_attack_speed` seconds
- **Attack Speed Cap**: Default 2.5 attacks per second (configurable)
- **Example**: Base AS 0.625, Bonus AS 100% → 1.25 attacks/sec → 0.8s interval

### 2. Attack Timer System (攻击计时)

The attack process is divided into two phases:

#### Windup Phase (前摇)

- **Duration**: Calculated based on `WindupConfig`
- **Behavior**: Champion must stand still, attack can be cancelled (with exceptions)
- **Damage Timing**:
  - Melee: Instant damage when windup completes
  - Ranged: Projectile launched when windup completes

#### Cooldown Phase (后摇)

- **Duration**: `attack_interval - windup_time`
- **Behavior**: Champion can move (stutter-stepping/走 A)
- **Purpose**: Waiting period before next attack can begin

### 3. Windup Time Calculation (前摇时间计算)

Four different calculation methods supported:

#### Fixed Time

```rust
WindupConfig::Fixed(0.25) // 0.25 seconds fixed windup
```

#### Percentage of Total Attack Time

```rust
WindupConfig::Percent(0.25) // 25% of attack interval
```

#### Legacy Formula (Old Champions)

```rust
WindupConfig::Legacy { attack_offset: 0.1 }
// Formula: 0.3 + attack_offset
```

#### Modern Formula (New Champions)

```rust
WindupConfig::Modern {
    attack_cast_time: 0.25,
    attack_total_time: 1.0
}
// Formula: (attack_cast_time / attack_total_time) * current_attack_interval
```

### 4. Attack Reset System (攻击重置)

Implements ability-based attack resets:

- **During Windup**: Cancels current attack (usually bad)
- **During Cooldown**: Skips cooldown, immediately starts next attack (good)
- **Common Examples**: Garen Q, Fiora Q, Jax W, Darius W, Vayne Q, etc.

### 5. Uncancellable Windup System

- **Grace Period**: 2 game ticks (0.066 seconds) where windup cannot be cancelled
- **High Attack Speed**: At very high AS, entire windup may be uncancellable
- **Special Abilities**: Some enhanced attacks have fully uncancellable windups

### 6. State Machine

```
Idle → Locked → Windup → Cooldown → Idle
  ↑                ↓
  └── Cancel ←─────┘
```

## Components

### `Attack`

Main component containing attack properties:

```rust
pub struct Attack {
    pub range: f32,
    pub base_attack_speed: f32,
    pub bonus_attack_speed: f32,
    pub attack_speed_cap: f32,
    pub windup_config: WindupConfig,
    pub windup_modifier: f32,
}
```

### `AttackState`

State machine tracking current attack phase:

```rust
pub enum AttackStatus {
    Idle,
    Locked { target: Entity },
    Windup { target: Entity, can_cancel: bool },
    Cooldown { target: Entity },
}
```

### `AttackTimer`

Timing information for current attack:

```rust
pub struct AttackTimer {
    pub phase_start_time: f32,
    pub uncancellable_windup: bool,
    pub uncancellable_remaining: f32,
}
```

## Events

### Commands

- `CommandAttackLock`: Start attacking a target
- `CommandAttackReset`: Reset attack timer (for abilities)
- `CommandAttackCancel`: Cancel current attack

### Events

- `EventAttackLock`: Target locked for attack
- `EventAttackWindupStart`: Windup phase started
- `EventAttackWindupComplete`: Windup completed, damage dealt
- `EventAttackCooldownStart`: Cooldown phase started
- `EventAttackCooldownComplete`: Attack fully completed
- `EventAttackReset`: Attack was reset by ability
- `EventAttackCancel`: Attack was cancelled

## Systems

### `attack_timer_system`

Updates uncancellable grace period timers.

### `attack_state_machine_system`

Main state machine logic:

- Transitions between attack phases
- Calculates timing based on attack speed
- Handles uncancellable periods
- Triggers appropriate events

## Usage Examples

### Basic Attack Setup

```rust
commands.spawn((
    Attack {
        range: 125.0,
        base_attack_speed: 0.625,
        bonus_attack_speed: 0.5, // 50% bonus AS
        windup_config: WindupConfig::Percent(0.25),
        ..Default::default()
    },
    Target(enemy_entity),
));
```

### Attack Reset Ability

```rust
// In ability system
commands.trigger_targets(CommandAttackReset, caster_entity);
```

### Listening for Attack Events

```rust
fn on_attack_complete(
    trigger: Trigger<EventAttackWindupComplete>,
    // Deal damage, apply on-hit effects, etc.
) {
    // Handle attack completion
}
```

## Testing

The system includes comprehensive tests covering:

- Attack speed calculations and caps
- Different windup configuration methods
- State machine transitions
- Uncancellable grace periods
- Command processing

Run tests with:

```bash
cargo test attack
```

## Integration Notes

- Requires `PluginTarget` for target management
- Uses `FixedUpdate` schedule for consistent timing
- Compatible with existing command and animation systems
- Events can be used to trigger damage, animations, and effects
