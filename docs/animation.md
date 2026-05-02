# 动画系统

## 组件总览

### lol_base 自定义组件

| 组件 | 位置 | 说明 |
|------|------|------|
| `LOLAnimationGraphHandle(pub Handle<LOLAnimationGraph>)` | 角色实体 | 手持 LOL 动画图资源句柄 |
| `LOLAnimationState` | 角色实体 | 当前播放的动画名、重复、持续时长等 |
| `AnimationConfig(pub Entity)` | 角色实体 | Bevy 自动维护，指向骨骼实体 |
| `AnimationConfigOf(pub Entity)` | 骨骼实体 | 指向所属角色实体，[relationship] 维护双向关系 |

### Bevy 原生组件

| 组件 | 位置 | 说明 |
|------|------|------|
| `AnimationPlayer` | 骨骼实体 | Bevy 动画播放器，GLTF 加载时自动创建 |
| `AnimationGraphHandle` | 骨骼实体 | 指向 Bevy AnimationGraph 资源 |
| `AnimationGraph` (Asset) | — | Bevy 动画图，存放剪辑节点等 |
| `AnimationTransitionOut` | 骨骼实体 | 过渡动画状态，运行时临时插入 |

### 自定义 Asset

| Asset | 说明 |
|-------|------|
| `LOLAnimationGraph` | LOL 动画图，包含 `hash_to_node` 动画名→节点映射、`blend_data` 混合数据、`gltf_path` |

## 实体层级

```
角色实体 (LOLAnimationGraphHandle, LOLAnimationState, AnimationConfig)
  └── GLTF World Root (Name)
        └── 骨骼实体 (AnimationPlayer, AnimationGraphHandle, AnimationConfigOf)
              └── 子骨骼层级...
```

- 角色实体持有**动画配置**（LOL 动画图句柄 + 动画状态）
- 骨骼实体持有**动画播放**（Bevy AnimationPlayer + AnimationGraphHandle）
- `AnimationConfigOf` / `AnimationConfig` 双向关系连接两者

## 导出流程

### 1. 动画数据提取 (league_to_lol)

1. 读取 League `.anm` 文件 → `AnimationFile::parse()`
2. 读取动画图定义（hash→节点、混合数据）
3. 构建 `LOLAnimationGraph`：`gltf_path` 指向皮肤 GLB，`hash_to_node` 映射动画名到节点类型（Clip/Sequence/Selector/Parallel/ConditionFloat/ConditionBool）
4. 序列化为 `assets/characters/{name}/animations/skin0.ron`

### 2. 皮肤场景导出

`assets/characters/{name}/skins/skin0.ron` 中，角色实体包含：
```ron
"lol_base::animation::LOLAnimationGraphHandle": (Path("characters/ahri/animations/skin0.ron")),
"bevy_animation::graph::AnimationGraphHandle": (Path("characters/ahri/animations/skin0.ron#animation_graph")),
```

`AnimationGraphHandle` 通过 RON 的 `#animation_graph` label 引用加载器构建的 Bevy AnimationGraph。

## 加载流程

### 1. Asset Loader 加载动画 RON

`LoaderConfigAnimationLoader` (crates/lol_render/src/loaders/animation.rs)：

1. 反序列化 RON → `LOLAnimationGraph`
2. 遍历 `hash_to_node`，将每个 Clip 节点替换为 Bevy `AnimationGraph` 节点：
   - 通过 `"{gltf_path}#Animation{index}"` 加载 `AnimationClip` Handle
   - `animation_graph.add_clip(handle, 1.0, root)`
3. 以 label `"animation_graph"` 导出构建好的 Bevy `AnimationGraph`
4. 返回 `LOLAnimationGraph`

### 2. 皮肤场景加载

`try_load_config_skin_characters` 将 skin.ron 中的组件写入角色实体：
- `LOLAnimationGraphHandle` → 角色实体
- `AnimationGraphHandle` → 角色实体（**临时**）
- `WorldAssetRoot` → 角色实体（触发 GLTF 加载）

### 3. AnimationGraphHandle 迁移

GLTF 加载完成后，`migrate_animation_graph_handle` 系统（在 Update 中运行）：

```
q1: (Entity, &AnimationGraphHandle), Without<AnimationPlayer>  // 角色实体
q2: &Children                                                  // 层级遍历
q3: &AnimationPlayer                                           // 骨骼实体判定

对每个角色实体：
  iter_descendants → 找到第一个有 AnimationPlayer 的后代骨骼实体
  → 从角色实体 remove AnimationGraphHandle
  → 插入到骨骼实体 + AnimationConfigOf(角色实体)
  → Bevy 自动在角色实体添加 AnimationConfig(骨骼实体)
```

天然幂等：移除后不再匹配。

## 运行时系统

所有系统注册在 `PluginAnimation`（crates/lol_render/src/animation.rs）。

### on_state_change

**触发**：角色实体的 `State` 组件变更

| State | → 动画 | 参数 |
|-------|--------|------|
| Idle | Idle1 | repeat=true |
| Running | Run | repeat=true |
| Attacking | Attack | repeat=false, duration=attack.animation_duration() |

**查询**：`(Entity, &State, &mut LOLAnimationState), Changed<State>`

### on_animation_state_change

**触发**：角色实体的 `LOLAnimationState` 变更

**逻辑**：
1. 从角色实体的 `AnimationConfig` 获取骨骼实体
2. 如果有正在进行的过渡动画 → stop
3. 如果 `last != current` → 对 last 插入 `AnimationTransitionOut`（100ms 渐变）
4. 播放 current 动画，按需设置 repeat

**查询**：
- 角色侧：`(&LOLAnimationGraphHandle, &mut LOLAnimationState, &AnimationConfig), Changed<LOLAnimationState>`
- 骨骼侧：`(Entity, &mut AnimationPlayer, &AnimationGraphHandle)` — 通过 `animation_config.deref()` 获取

### update_transition_out

**功能**：逐帧更新过渡动画权重

**逻辑**：
1. 计算 elapsed / duration 进度
2. `weight = transition_out.weight * (1.0 - progress)`
3. 超过 duration → stop + 移除 `AnimationTransitionOut`

**查询**：
- 骨骼侧：`(Entity, &mut AnimationPlayer, &AnimationGraphHandle, &AnimationConfigOf, &AnimationTransitionOut)`
- 角色侧：`(&LOLAnimationGraphHandle, &mut LOLAnimationState)` — 通过 `anim_config_of.0` 获取

### update_condition_animation

**功能**：处理条件动画（如 Run 根据 MoveSpeed 在不同剪辑间混合权重）

**逻辑**：
1. 查询角色实体，找当前动画是 `ConditionFloat` 节点的
2. 评估条件（MoveSpeed 与阈值比较）→ 计算各子动画权重
3. 在骨骼实体的 `AnimationPlayer` 上应用权重

**查询**：
- 角色侧：`(Entity, &LOLAnimationGraphHandle, &LOLAnimationState)` — 采集条件数据
- 骨骼侧：`(&mut AnimationPlayer, &AnimationConfigOf)` — 通过 `cf.0 == entity` 查找并应用

### apply_animation_speed

**功能**：根据 `current_duration` 覆盖调整动画播放速度

**逻辑**：
1. 读取 `LOLAnimationState.current_duration`（目标时长）
2. 从 Bevy `AnimationGraph` 获取实际 `AnimationClip` 的 duration
3. `speed = clip_duration / target_duration`
4. 应用到 AnimationPlayer

**查询**：
- 骨骼侧：`(&mut AnimationPlayer, &AnimationGraphHandle, &AnimationConfigOf)`
- 角色侧：`(&LOLAnimationGraphHandle, &LOLAnimationState)` — 通过 `anim_config_of.0` 获取

## 跨实体查询模式

动画系统采用两种查询方向：

```
角色 → 骨骼（on_animation_state_change）:
  角色实体.AnimationConfig.deref() → bone_entity → 骨骼侧查询 AnimationPlayer

骨骼 → 角色（update_transition_out, apply_animation_speed）:
  骨骼实体.AnimationConfigOf.0 → char_entity → 角色侧查询 LOLAnimationState
```
