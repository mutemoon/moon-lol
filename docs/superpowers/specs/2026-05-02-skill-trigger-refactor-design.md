# Skill Trigger Refactor — Design

## Summary

Eliminate thin wrapper helpers in `lol_core::skill::helpers` and unify all skill action dispatching to use `commands.trigger()` directly. Upgrade `ActionDash` to an `EntityEvent` with observer, symmetric to `ActionDamage`.

## Motivation

Six helper functions exist in `lol_core::skill::helpers`:

| Function | What it actually does |
|---|---|
| `play_skill_animation` | `commands.trigger(CommandAnimationPlay {...})` |
| `spawn_skill_particle` | `commands.trigger(CommandSkinParticleSpawn {...})` |
| `despawn_skill_particle` | `commands.trigger(CommandSkinParticleDespawn {...})` |
| `reset_skill_attack` | `commands.trigger(CommandAttackReset {...})` |
| `skill_damage` | `commands.trigger(ActionDamage {...})` (already EntityEvent) |
| `skill_dash` | Direction math + insert component + `commands.trigger(CommandMovement)` |

Five of six are pure wrappers — the champion code gains nothing from calling them instead of calling `commands.trigger()` directly. The sixth (`skill_dash`) embeds movement computation that belongs in an observer, not a free function.

## Design

### Remove five thin wrappers

Delete from `helpers.rs`:
- `play_skill_animation`
- `spawn_skill_particle`
- `despawn_skill_particle`
- `reset_skill_attack`
- `skill_damage`

Retain in `helpers.rs`:
- `get_skill_value`
- `calculate_part`

### Upgrade `ActionDash` to EntityEvent

`ActionDash` currently is a plain `struct`. It becomes:

```rust
#[derive(Debug, Clone, EntityEvent)]
pub struct ActionDash {
    pub entity: Entity,
    pub skill: Handle<Spell>,
    pub move_type: DashMoveType,
    pub damage: Option<DashDamage>,
    pub speed: f32,
    pub point: Vec2,   // NEW: target point from player input
}
```

A new observer `on_action_dash` handles the old `skill_dash` logic:
1. Read `Transform` from entity
2. Compute direction and distance to `point`
3. Resolve destination based on `DashMoveType` (Fixed vs Pointer)
4. If damage configured, insert `DashDamageComponent`
5. Trigger `CommandMovement`

Champion code changes from:
```rust
skill_dash(commands, &q_transform, entity, point, &ActionDash { ... });
```
To:
```rust
commands.trigger(ActionDash { entity, point, ... });
```

The `q_transform` query parameter disappears from champion cast functions since the observer handles it internally — same pattern as `ActionDamage`.

### Remove `skill_dash` from helpers

The `skill_dash` function in `helpers.rs` is deleted. Its logic moves to `on_action_dash` in `action/dash.rs`.

### Update `skill/mod.rs`

Remove re-exports of deleted helpers. Keep `get_skill_value`.

## Champion-side changes

Before:
```rust
fn cast_hero_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash);
    skill_dash(commands, q_transform, entity, point, &ActionDash { ... });
}
```

After:
```rust
fn cast_hero_q(
    commands: &mut Commands,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay { entity, hash: "spell1".to_string(), repeat: false, duration: None });
    commands.trigger(CommandSkinParticleSpawn { entity, hash });
    commands.trigger(ActionDash { entity, skill: skill_spell, point, ... });
}
```

## Affected files

- `crates/lol_core/src/skill/helpers.rs` — delete 5 functions
- `crates/lol_core/src/skill/mod.rs` — delete 5 re-exports
- `crates/lol_core/src/action/dash.rs` — `ActionDash` → EntityEvent, add `on_action_dash` observer
- 120 champion `mod.rs` files — mechanical replacement of helper calls with direct `commands.trigger()`
- `crates/lol_core/src/skill/tests.rs` — update if they reference deleted helpers
- `crates/lol_core/src/skill/integration_tests.rs` — same
- `crates/lol_champions/src/riven/tests.rs` — update imports

## Risk

Low risk — purely mechanical refactoring. The control flow is preserved exactly; only the call site changes. `cargo check --examples --tests` validates correctness.
