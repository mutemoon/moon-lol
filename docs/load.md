# 资源加载流程

## 概述

运行时加载分为两条独立链路：

1. **游戏场景加载** — 从 `games/{game_setting}.ron` 加载英雄角色出生点
2. **地图加载** — 从 `maps/{map_name}/scene.ron` 加载地图对象（防御塔、野怪、兵营、草丛等）

两条链路最终都将 `DynamicWorld` 反序列化为实体组件，写入 ECS World。**地图场景中的防御塔、野怪、兵营等同样是有 `ConfigCharacterRecord` + `ConfigSkin` 的角色实体，共享英雄的加载逻辑。**

```
App Startup
├── PluginGame → startup_load_game_scenes → DynamicWorldRoot("games/riven.ron")
├── PluginMap  → startup_load_map_geometry → DynamicWorldRoot("maps/sr/scene.ron")
└── PluginRender → setup → WorldAssetRoot("maps/sr/mapgeo.glb")

FixedUpdate
├── try_load_config_characters (lol_core)
│      读取 ConfigCharacterRecord → 反序列化 config.ron → 写入 Health/Attack/Skills 等
├── try_load_config_skin_characters (lol_render)
│      读取 ConfigSkin → 反序列化 skin.ron → 写入 WorldAssetRoot + Skin + HealthBar
└── glTF 加载完成后 → Bevy 自动处理 Mesh/Material/SkinnedMesh/AnimationPlayer
```

## 1. 游戏场景加载

### 1.1 入口：games/{game_setting}.ron

游戏场景文件（如 `assets/games/riven.ron`）定义了一局游戏中所有角色出生点：

```ron
(
  resources: {},
  entities: {
    4294967262: (
      components: {
        "lol_base::character::ConfigCharacterRecord": (
          character_record: Path("characters/riven/config.ron"),
        ),
        "lol_base::character::ConfigSkin": (
          skin: Path("characters/riven/skins/skin0.ron"),
        ),
        "lol_base::team::Team": (team: Blue),
        "lol_base::transform::Transform": (scale: [1.0, 1.0, 1.0], ...),
        "lol_core::character::Champion": (),
        "lol_base::life::HealthBar": (bar_type: 12),
      },
    ),
  },
)
```

### 1.2 加载系统：PluginGame

**文件：** [lol_core/src/game.rs](file:///d:/Users/admin/workspace/moon-lol-minimax/crates/lol_core/src/game.rs)

```rust
pub struct PluginGame {
    pub scenes: Vec<String>,  // e.g., vec!["games/riven.ron"]
}

fn startup_load_game_scenes(
    mut commands: Commands,
    res_asset_server: Res<AssetServer>,
    scenes: Res<GameScenes>,
) {
    for scene_path in scenes.0.iter() {
        commands.spawn(DynamicWorldRoot(res_asset_server.load(scene_path)));
    }
}
```

`DynamicWorldRoot` 是 Bevy 场景系统的根组件，挂载后 Bevy 会自动将 `.ron` 反序列化为 `DynamicWorld` Asset。

### 1.3 ConfigCharacterRecord 消费

**文件：** [lol_core/src/character.rs](file:///d:/Users/admin/workspace/moon-lol-minimax/crates/lol_core/src/character.rs)

```rust
fn try_load_config_characters(
    mut commands: Commands,
    character_record_query: Query<(Entity, &ConfigCharacterRecord)>,
    dynamic_worlds: Res<Assets<DynamicWorld>>,
) {
    for (entity, config) in &character_record_query {
        if dynamic_worlds.get(&config.character_record).is_none() {
            return;  // Asset 尚未加载完成，等待下一帧
        }
        // 取出 handle，queue 中使用
        let handle = config.character_record.clone();
        commands.queue(move |world: &mut World| {
            world.resource_scope(|world, dynamic_worlds: Mut<Assets<DynamicWorld>>| {
                let dw = dynamic_worlds.get(&handle).unwrap();
                let mut map = EntityMap::default();
                // 将 DynamicWorld 中所有实体写入目标 World
                // 第一个实体会映射到角色实体
                dw.write_to_world(world, &mut map);
            });
        });
        commands.entity(entity).remove::<ConfigCharacterRecord>();
    }
}
```

### 1.4 ConfigSkin 消费

**文件：** [lol_render/src/skin/skin.rs](file:///d:/Users/admin/workspace/moon-lol-minimax/crates/lol_render/src/skin/skin.rs)

```rust
fn try_load_config_skin_characters(
    mut commands: Commands,
    skin_query: Query<(Entity, &ConfigSkin)>,
    dynamic_worlds: Res<Assets<DynamicWorld>>,
) {
    for (entity, config) in &skin_query {
        if dynamic_worlds.get(&config.skin).is_none() {
            return;
        }
        let handle = config.skin.clone();
        commands.queue(move |world: &mut World| {
            world.resource_scope(|world, dynamic_worlds: Mut<Assets<DynamicWorld>>| {
                let dw = dynamic_worlds.get(&handle).unwrap();
                let mut map = EntityMap::default();
                map.insert(entity, entity);  // 关键：原始第一个实体映射到角色实体
                dw.write_to_world(world, &mut map);
            });
        });
        commands.entity(entity)
            .remove::<ConfigSkin>()
            .observe(migrate_animation_graph_handle);  // 动画迁移
    }
}
```

## 2. 角色配置结构

### 2.1 Character config.ron

**文件：** `assets/characters/{name}/config.ron`

包含角色的逻辑组件（Health、Attack、Skills 等），由 `ConfigCharacterRecord` 引用时触发写入：

```ron
(
  resources: {},
  entities: {
    4294967262: (  // 根实体（glb mesh 挂载点）
      components: {
        "bevy_world_serialization::components::WorldAssetRoot": (
          Path("characters/riven/skins/skin0.glb#Scene0"),
        ),
        "lol_base::character::Skin": (scale: 1.5),
        "lol_base::life::HealthBar": (bar_type: 12),
      },
    ),
    4294967271: (  // 角色逻辑实体
      components: {
        "lol_core::character::Character": (),
        "lol_core::life::Health": (value: 630.0, max: 630.0),
        "lol_core::attack::Attack": (range: 600.0, ...),
        "lol_core::movement::Movement": (speed: 340.0),
        "lol_core::skill::Skills": ([Q, W, E, R, Passive]),
        ...
      },
    ),
  },
)
```

### 2.2 Skin.ron

**文件：** `assets/characters/{name}/skins/{skinN}.ron`

包含皮肤的渲染组件（WorldAssetRoot、Skin、HealthBar）：

```ron
(
  resources: {},
  entities: {
    4294967262: (
      components: {
        "bevy_world_serialization::components::WorldAssetRoot": (
          Path("characters/zyra/skins/skin0.glb#Scene0"),
        ),
        "lol_base::character::Skin": (scale: 1.0),
        "lol_base::life::HealthBar": (bar_type: 12),
        "lol_base::transform::Visibility": (),
      },
    ),
  },
)
```

### 2.3 CharacterSkin 组件

当前实现使用两个独立组件：

| 组件 | 文件 | 用途 |
|------|------|------|
| `ConfigCharacterRecord` | `lol_base/src/character.rs` | 持有 `Handle<DynamicWorld>` 指向 `config.ron` |
| `ConfigSkin` | `lol_base/src/character.rs` | 持有 `Handle<DynamicWorld>` 指向 `skin.ron` |

在 `load.md` 重构完成后，将合并为单一 `CharacterSkin` 组件，详见 [load-refactor](docs/load.md)（重构皮肤渲染：从运行时加载到 GLTF 导出）。

## 3. 地图加载

### 3.1 地图场景加载

**文件：** [lol_core/src/map.rs](file:///d:/Users/admin/workspace/moon-lol-minimax/crates/lol_core/src/map.rs)

```rust
fn startup_load_map_geometry(
    mut commands: Commands,
    res_map_paths: Res<MapPaths>,
    res_asset_server: Res<AssetServer>,
) {
    commands.spawn(DynamicWorldRoot(
        res_asset_server.load(res_map_paths.scene_ron()),
    ));
}
```

`MapPaths::scene_ron()` 返回 `"maps/{map_name}/scene.ron"`。

### 3.2 地图场景内容

`maps/{map_name}/scene.ron` 包含地图上所有可放置对象：

| 对象类型 | 组件 | 说明 |
|----------|------|------|
| 防御塔 | `ConfigCharacterRecord` + `ConfigSkin` + `Team` | 归属蓝/红方 |
| 水晶/兵营 | `ConfigCharacterRecord` + `ConfigSkin` + `Team` | 归属蓝/红方 |
| 野怪营地 | `ConfigCharacterRecord` + `ConfigSkin` + `MinionPath` | 包含刷新路径 |
| 草丛/障碍物 | 仅 `Transform` + `Visibility` | 无角色逻辑 |

**地图场景结构示例：**

```ron
(
  resources: {},
  entities: {
    // 蓝方防御塔
    100001: (
      components: {
        "lol_base::character::ConfigCharacterRecord": (
          character_record: Path("characters/turret_tower_ascension/config.ron"),
        ),
        "lol_base::character::ConfigSkin": (
          skin: Path("characters/turret_tower_ascension/skins/skin0.ron"),
        ),
        "lol_base::team::Team": (team: Blue),
        "lol_base::transform::Transform": (...),
      },
    ),
    // 红方野怪
    200002: (
      components: {
        "lol_base::character::ConfigCharacterRecord": (
          character_record: Path("characters/wormboss/config.ron"),
        ),
        "lol_base::character::ConfigSkin": (
          skin: Path("characters/wormboss/skins/skin0.ron"),
        ),
        "lol_base::team::Team": (team: Neutral),
        "lol_core::map::MinionPath": (waypoints: [...]),
      },
    ),
    // 草丛（无角色逻辑）
    300003: (
      components: {
        "lol_base::transform::Transform": (...),
        "lol_base::transform::Visibility": (),
      },
    ),
  },
)
```

### 3.3 地图场景的角色加载

**地图场景中的实体同样是角色实体**，共享 `try_load_config_characters` 和 `try_load_config_skin_characters` 系统：

- 防御塔有 `config.ron`（包含 `Health`、攻击力等）和 `skin.ron`（包含 glb 引用）
- 野怪有 `config.ron`（包含 `Health`、金币/经验值等）和 `skin.ron`
- 只有 `Transform` + `Visibility` 的实体不触发角色加载

### 3.4 地图几何加载

**文件：** [lol_render/src/map.rs](file:///d:/Users/admin/workspace/moon-lol-minimax/crates/lol_render/src/map.rs)

```rust
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    res_map_paths: Res<MapPaths>,
) {
    let handle = asset_server
        .load_builder()
        .with_settings(|s: &mut GltfLoaderSettings| {
            s.validate = false;
            s.load_materials = RenderAssetUsages::RENDER_WORLD;
        })
        .load(GltfAssetLabel::Scene(0).from_asset(res_map_paths.mapgeo_glb()));

    commands.spawn(WorldAssetRoot(handle));
}
```

`MapPaths::mapgeo_glb()` 返回 `"maps/{map_name}/mapgeo.glb"`。

## 4. 组件类型说明

### lol_base vs lol_core

| 层 | 类型 | 用途 |
|----|------|------|
| `lol_base` | `ConfigCharacterRecord`, `ConfigSkin`, `Skin`, `HealthBar`, `Team` | 配置数据，直接序列化到场景文件 |
| `lol_core` | `Character`, `Health`, `Attack`, `Movement`, `Skills` | 运行时逻辑组件，通过 `config.ron` 写入 |

配置数据（`lol_base`）与逻辑数据（`lol_core`）分离的好处：
- `lol_base` 类型可直接序列化到 RON 文件
- `lol_core` 类型可自由增删，不影响场景文件兼容性

### Component vs Asset

| 类型 | 存储位置 | 加载方式 |
|------|----------|----------|
| `DynamicWorld` (Asset) | `lol_base` | 通过 `Handle<DynamicWorld>` 引用 |
| `Gltf` (Asset) | Bevy 内置 | 通过 `Handle<Gltf>` 引用 |
| 组件 (Component) | ECS World | 通过 `DynamicWorld.write_to_world` 写入 |

## 5. glTF 加载后自动处理

当 `WorldAssetRoot` 持有的 `Handle<Gltf>` 加载完成后，Bevy 会自动：

1. 解析 GLTF Scene 中的节点层级
2. 为每个 Mesh 节点生成 `Mesh3d` + `MeshMaterial3d` 组件
3. 为带骨骼的 Mesh 生成 `SkinnedMesh` + `InverseBindMatrices` 组件
4. 为动画片段生成 `AnimationPlayer` 组件

`lol_render` 中无需额外系统处理这些原生组件，动画迁移通过 Observer 模式实现（见 [lol_render/src/skin/skin.rs](file:///d:/Users/admin/workspace/moon-lol-minimax/crates/lol_render/src/skin/skin.rs) 的 `migrate_animation_graph_handle`）。

## 6. 完整时序

```
App::run()
└── PluginCore::build()
│      └── PluginGame::build()
│             └── GameScenes { scenes: ["games/riven.ron"] }
│
└── PluginRender::build()
       └── PluginSkin::build()
              └── try_load_config_skin_characters (触发条件: ConfigSkin Added)
       └── PluginRenderMap::build()
              └── setup → spawn WorldAssetRoot(mapgeo.glb)

【Startup】
│
├─ startup_load_game_scenes
│      └─ spawn DynamicWorldRoot("games/riven.ron")
│
├─ startup_load_map_geometry
│      └─ spawn DynamicWorldRoot("maps/sr/scene.ron")
│
└─ setup
       └─ spawn WorldAssetRoot("maps/sr/mapgeo.glb")

【FixedUpdate】(等 Asset 加载完成)

**来自 games/{game_setting}.ron 的角色**（如英雄）
├─ try_load_config_characters
│      └─ DynamicWorld(config.ron) → write_to_world → Health/Attack/Skills...
└─ try_load_config_skin_characters
       └─ DynamicWorld(skin.ron) → write_to_world → WorldAssetRoot/Skin/HealthBar

**来自 maps/{map_name}/scene.ron 的角色**（如防御塔、野怪、兵营）
└─ 复用同一套系统，流程同上

**glTF Ready (Bevy 自动)**
└─ Mesh3d + MeshMaterial3d + SkinnedMesh + AnimationPlayer
```
