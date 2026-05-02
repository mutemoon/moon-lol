# Move Skill Tests to Champion Modules

## Summary

Move skill tests from root `tests/` directory into their respective crates:
hero tests → `lol_champions` hero modules, generic skill system tests → `lol_core::skill`.

## Motivation

- Hero skill tests logically belong with the hero implementation, not in the root test directory
- Colocating tests with implementation makes it obvious which tests exist for which champion
- Generic skill system tests belong in `lol_core`, the crate that owns the skill pipeline

## File Mapping

| Original | Destination |
|----------|-------------|
| `tests/skill.rs` | `crates/lol_core/src/skill/tests.rs` |
| `tests/skill_integration.rs` | `crates/lol_core/src/skill/integration_tests.rs` |
| `tests/riven.rs` | Split into `crates/lol_champions/src/riven/tests.rs` (logic) + `render_tests.rs` (render) |
| `tests/fiora_render_test.rs` | `crates/lol_champions/src/fiora/render_tests.rs` |

## Hero Module Structure (example: Riven)

```
crates/lol_champions/src/riven/
├── mod.rs            # unchanged, add #[cfg(test)] mod declarations
├── passive.rs        # unchanged
├── q.rs              # unchanged
├── tests.rs          # NEW: #[cfg(test)] logic tests
└── render_tests.rs   # NEW: #[cfg(test)] render/video tests
```

`mod.rs` gains two lines at the end:
```rust
#[cfg(test)]
mod tests;
#[cfg(test)]
mod render_tests;
```

`tests.rs` contains `#[test]` functions covering Q1/Q2/Q3 recast, cooldown timing, mana gating, E shield, dash distance, etc. All use headless Bevy + manual time stepping, no render dependency.

`render_tests.rs` contains one `#[test]` per skill behavior, each producing an independent video file. Depends on `lol_render::test_render::PluginSkillTestRender`.

## Shared Test Infrastructure

### Logic test harness → `lol_core::skill::test_utils`

Extract common patterns from current `tests/skill.rs` and `tests/skill_integration.rs`:

- `test_app()` — creates headless Bevy App with PluginSkill + PluginCooldown + PluginDamage + PluginLife + PluginMovement + PluginAction, configured with `TimeUpdateStrategy::ManualDuration`
- `spawn_caster(app, team, position)` — spawns caster entity with Team, Transform, Health, Skills, SkillPoints, AbilityResource
- `spawn_target(app, team, position)` — spawns target entity
- `advance_frames(app, n)` — advances N Update frames
- `test_spell(key)` — constructs a minimal Spell handle for test use

### Render test harness → `lol_render::test_render`

Already exists and is sufficiently generic. No structural changes needed.

## Dependency Changes

### `lol_champions/Cargo.toml` — new `[dev-dependencies]`

```toml
[dev-dependencies]
crossbeam-channel.workspace = true
image.workspace = true
serde_json.workspace = true
ron.workspace = true
rand.workspace = true
```

### `lol_core/Cargo.toml`

Verify existing dependencies cover test needs (lol_base should already be a regular dependency). Add dev-deps if needed.

### Root `Cargo.toml`

Remove dev-dependencies that only existed for skill tests (if no other tests use them). Keep deps that `tests/attack.rs`, `tests/minion.rs`, etc. still need.

## PluginCore Replacement

Current hero tests reference `moon_lol::PluginCore`. Since `lol_champions` cannot depend on the root crate (would be circular), each render test replaces `PluginCore` with individual plugin registrations needed for that champion's test scenario.

## Non-Moving Files

These files stay in `tests/` and are out of scope:

- `tests/attack.rs` — attack system tests, not skill-related
- `tests/minion.rs` — minion tests
- `tests/test_render_smoke.rs` — render smoke test
