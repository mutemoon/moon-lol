# AnimationGraphHandle 运行时迁移

## 问题

Skin RON 加载后 `AnimationGraphHandle` 在角色根实体上，但 GLTF 加载后 `AnimationPlayer` 在子孙骨骼实体上。Bevy 要求 `AnimationGraphHandle` 与 `AnimationPlayer` 在同一实体，否则动画不播放。

## 实体层级

```
角色实体 (LOLAnimationGraphHandle, AnimationState)
  └── GLTF 根实体 (Name)
        └── 骨骼实体 (AnimationPlayer)  ← 需要 AnimationGraphHandle
```

## 设计

### 新增组件 (`lol_base::animation`)

```rust
/// 骨骼实体上有 AnimationPlayer 的实体，指向角色实体
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = AnimationConfig)]
pub struct AnimationConfigOf(pub Entity);

/// Bevy 自动维护在角色实体上
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = AnimationConfigOf)]
pub struct AnimationConfig(pub Entity);
```

插入 `AnimationConfigOf` 时 Bevy 自动维护双向关系。

### 新增迁移 System (`lol_render::skin`)

- 注册在 `Update`
- q1: `(Entity, &AnimationGraphHandle), Without<AnimationPlayer>` — 角色实体
- q2: `(Entity, &AnimationPlayer)` — 骨骼实体
- 对每个角色实体，`q2.iter_descendants(root_entity).next()` 找唯一骨骼实体
- 移除角色实体的 `AnimationGraphHandle`，插入到骨骼实体 + `AnimationConfigOf(root_entity)`
- 天然幂等：移除后不再匹配

### 修改动画系统 (`lol_render::animation`)

4 个 system 改为在骨骼实体上查询 `(AnimationPlayer, AnimationGraphHandle)`，通过 `AnimationConfigOf.0` 回溯角色实体获取 `LOLAnimationGraphHandle` + `AnimationState`：

| System | 新增查询 |
|--------|---------|
| `on_animation_state_change` | `AnimationConfigOf.0` → `LOLAnimationGraphHandle`, `AnimationState` |
| `update_transition_out` | 同上 + `AnimationTransitionOut` 移到骨骼实体 |
| `update_condition_animation` | 查询骨骼实体，`AnimationConfigOf.0` 回溯 |
| `apply_animation_speed` | 同上 |

`AnimationTransitionOut` 原来插入在角色实体上，现改为插入在骨骼实体上（与 `AnimationPlayer` 同实体），简化查询。

## 验证

```bash
cargo check --examples --tests
```
