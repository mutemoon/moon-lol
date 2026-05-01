# 地图数据提取流程

## WAD 文件结构

游戏资源存储在 `.wad.client` 文件中，采用 hash 索引机制：

```
.wad.client 文件 → 文件表 + 二进制内容
```

### 文件表条目

每个文件在文件表中记录：

| 字段 | 类型 | 说明 |
|------|------|------|
| 路径 hash | u64 | 文件路径的 FNV-1a hash 值 |
| offset | u64 | 文件内容在二进制数据中的偏移 |
| size | u32 | 文件大小 |
| 其它 | - | 额外元数据 |

### Hash 算法

路径到 hash 的转换使用 **FNV-1a** 算法，源码来自 [LeagueToolkit](https://github.com/Keengs/LibraryForLeagueToolkit)：

```
示例: "data/maps/mapgeometry/map11/bloom.mapgeo"
  → hash: 0xe8b4704f422901d9
```

已知路径即可算出 hash，再通过 hash 查找 offset 和 size，最后从二进制数据中读取文件内容。

### 文件类型

WAD 中的文件主要包括：

| 类型 | 后缀 | 说明 |
|------|------|------|
| 模型文件 | `.skn`, `.skl`, `.glb` | 网格、骨骼、GLTF 模型 |
| 贴图文件 | `.tex` | 纹理数据 |
| 动画文件 | `.anm` | 骨骼动画 |
| 配置文件 | `.bin` | 游戏数据配置 |

## 架构概述

```
游戏文件 (wad.client) → league_core::extract (解析) → league_to_lol (转换) → lol_base (稳定配置)
                                                      ↓
                                              lol_core (运行时逻辑)
                                                      ↓
                                              lol_render (渲染)
```

### 层级说明

- **league_core** - 游戏数据提取层，包含原始数据结构（随版本变化）
- **league_to_lol** - 转换层，将 league_core 类型转换为 lol_base 类型
- **lol_base** - 稳定配置层，包含不随版本变化的配置数据（Asset）
- **lol_core** - 运行时逻辑层，包含组件、系统、状态管理
- **lol_render** - 渲染层

### 设计原则

- `lol_base` 和 `lol_core` **不依赖** `league_core`，实现数据层解耦
- 游戏数据解析在 `league_core`，转换为稳定格式在 `league_to_lol`
- `ConfigXxx` 类型放在 `lol_base` 作为 Bevy Asset
- 角色初始配置直接存储在组件中，通过场景序列化保存

## 提取模块结构

```
league_to_lol::extract
├── mod.rs              # 模块入口，导出所有公共函数
├── utils.rs            # 文件写入工具
├── champion.rs         # Champion 提取逻辑
├── skin.rs            # 皮肤提取逻辑
├── spell.rs           # 技能提取逻辑
└── map.rs             # 地图提取逻辑和阶段函数
```

## 提取流程（7 阶段）

### Phase 1: 创建 Loader

`extract_phase_1_create_loader(game_path: &str) -> LeagueLoader`

扫描 Game 路径下的 Champions 文件夹，收集所有英文版 WAD 文件（不含下划线），创建 LeagueLoader。

### Phase 2: 提取所有英雄

`extract_phase_2_champions(loader: &LeagueLoader)`

并行提取所有英雄角色数据：

- 遍历 `Champions/*.wad.client` 文件
- 使用 rayon 并行处理
- 每个英雄导出 `config.ron`、`skins/{skinN}.glb`、`skins/{skinN}.ron`、`spells/*.ron`

#### 技能数据提取

从 `CharacterRecord.spells` 获取技能哈希列表，通过 `prop_group.get_data::<SpellObject>(hash)` 解析每个技能，转换为 `DataSpell` 后导出到 `spells/{object_name}.ron`。

角色实体通过 `Skills(Vec<Entity>)` 组件关联技能实体，每个技能实体包含：

- **SkillOf** - 关联到角色实体
- **Skill** - 技能配置（spell Handle、level、slot、cooldown_mode）
- **CoolDown** - 冷却持续时间配置
- **CoolDownState** - 运行时冷却状态（timer）

#### 资源数据提取

从 `CharacterRecord.primary_ability_resource` 提取资源数据到 `AbilityResource` 组件：

- **ar_type** - 资源类型（`AbilityResourceSlotInfo.ar_type`）
  - 0: Mana
  - 1: Energy
  - 7: Turret
  - 8: Camp
- **value/max** - 当前/最大资源值（初始为 base 值）
- **base** - 基础资源值（从 `AbilityResourceSlotInfo.unk_0x3a509002.base_value` 获取）
- **per_level** - 每级成长值（从 `AbilityResourceSlotInfo.unk_0x452033bb.base_value` 获取）
- **base_static_regen** - 基础资源回复（从 `AbilityResourceSlotInfo.unk_0x6216bf7b.base_value` 获取）
- **regen_per_level** - 每级回复成长（从 `AbilityResourceSlotInfo.unk_0x726ee5cd.base_value` 获取）

lol_core 中的 `AbilityResource` 组件结构：

```rust
pub struct AbilityResource {
    pub ar_type: AbilityResourceType,  // Mana, Energy, Turret, Camp
    pub value: f32,
    pub max: f32,
    pub base: f32,
    pub per_level: f32,
    pub base_static_regen: f32,
    pub regen_per_level: f32,
}
```

### Phase 3: 提取地图块数据

`extract_phase_3_map_chunks(world: &mut World, loader: &LeagueLoader, map_name: &MapName) -> HashMap<String, ChampionRecordData>`

遍历地图可放置物，提取：

- **小兵路径** (Unk0x3c995caf)：兵线路径点
- **兵营** (Unk0xba138ae3)：生成兵营实体
- **角色** (Unk0xad65d8c4)：收集角色记录数据

返回 `map_character_records` 供 Phase 6 使用。

### Phase 4: 提取导航网格

`extract_phase_4_nav_grid(world: &mut World, loader: &LeagueLoader, map_name: &MapName)`

从 MapNavGrid 路径加载导航网格数据，导出为二进制文件并插入 World 资源。

### Phase 5: 导出地图几何

`extract_phase_5_map_geo(loader: &LeagueLoader, map_name: &MapName)`

解析地图几何数据，遍历所有网格和子网格，导出为 GLTF 格式。

### Phase 6: 从地图提取角色记录

`extract_phase_6_map_character_records(loader: &LeagueLoader, map_character_records: &HashMap<String, ChampionRecordData>)`

并行提取地图中收集的角色记录数据（用于地图特有角色）。

### Phase 7: 序列化 World

`extract_phase_7_serialize_world(world: &mut World, map_name: &MapName)`

将所有生成的实体序列化为 RON 格式保存。

## 一键提取

```rust
use league_to_lol::extract::extract_all;

fn main() {
    let game_path = r"D:\WeGameApps\英雄联盟\Game";
    let hashes_dir = "assets/hashes";
    extract_all(game_path, hashes_dir);
}
```

`extract_all` 函数自动完成所有 7 个阶段的提取工作。

## 独立阶段调用

也可以单独调用某个阶段：

```rust
use league_to_lol::extract::{
    extract_phase_1_create_loader,
    extract_phase_2_champions,
    extract_phase_3_map_chunks,
    extract_phase_4_nav_grid,
    extract_phase_5_map_geo,
    extract_phase_6_map_character_records,
    extract_phase_7_serialize_world,
};

let loader = extract_phase_1_create_loader(game_path);
extract_phase_2_champions(&loader);
// ... 调用其他阶段
```

## 日志输出

```
[1/7] Phase 1: 扫描 WAD 文件并创建 Loader...
[2/7] Phase 2: 提取所有英雄...
[SUMMARY] 英雄提取完成: 成功 173 个, 跳过 0 个
[3/7] Phase 3: 提取地图块数据...
[4/7] Phase 4: 提取导航网格...
⏱️ 导出耗时统计: ...
[5/7] Phase 5: 导出地图几何到 GLTF...
[6/7] Phase 6: 从地图中提取 26 个角色记录...
[SUMMARY] 地图角色记录提取完成: 成功 26 个
[7/7] Phase 7: 序列化 World 到文件...
```

## 输出文件

| 文件                                    | 描述                                                                                                |
| --------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `assets/characters/{name}/config.ron`   | 角色场景（包含 Bounding、Attack、Health、Damage、Armor、Movement、Skills、Name 等组件的序列化实体） |
| `assets/characters/{name}/skins/{skinN}.glb` | 皮肤 GLB 文件（网格 + 材质 + 贴图）                                                               |
| `assets/characters/{name}/skins/{skinN}.ron` | 皮肤场景（包含 Skin、HealthBar、Visibility 组件）                                               |
| `assets/characters/{name}/spells/*.ron` | 技能数据 Asset（包含 Spell、DataSpell、calculations、effect_amounts 等）                            |
| `assets/maps/{map_name}/navgrid.bin`    | 二进制导航网格数据                                                                                  |
| `assets/maps/{map_name}/mapgeo.glb`     | GLTF 格式的地图几何                                                                                 |
| `assets/maps/{map_name}/scene.ron`      | 包含所有地图对象的序列化场景                                                                        |
| `assets/maps/{map_name}/barracks/{id}.ron` | 兵营配置                                                                                    |

### Character 场景文件结构

`assets/characters/{name}/config.ron` 是一个 Bevy DynamicScene RON 文件，包含以下组件：

- **Champion** - 空组件，标记该实体为英雄
- **Bounding** - 包含 `radius`（碰撞半径）和 `height`（生命条高度）
- **Attack** - 包含 `range`（攻击距离）、`base_attack_speed`（基础攻击速度）、`windup_config`（前摇时间配置）
- **Health** - 包含 `value` 和 `max`（生命值）
- **Damage** - 包含 `0`（攻击力）
- **Armor** - 包含 `0`（护甲值）
- **Movement** - 包含 `speed`（移动速度）
- **AbilityResource** - 包含 `ar_type`（资源类型：Mana/Energy/Turret/Camp）、`value`、`max`、`base`、`per_level`、`base_static_regen`、`regen_per_level`
- **Skills** - 包含技能实体列表（`Vec<Entity>`）
- **Name** - 实体名称

技能实体独立存在，通过 `SkillOf` relationship 关联到角色实体。每个技能实体包含：

- **SkillOf** - 关联到角色实体（`SkillOf(Entity)`）
- **Skill** - 技能配置（spell Handle 指向 spells/\*.ron、level、slot、cooldown_mode）
- **CoolDown** - 冷却持续时间配置
- **CoolDownState** - 运行时冷却状态（timer，不序列化）

可直接通过 Bevy 的场景系统反序列化加载为实体。

### 皮肤数据提取

皮肤数据从 `SkinCharacterDataProperties` 中提取，包含以下信息：

| 信息     | 字段                                        |
| -------- | ------------------------------------------- |
| 皮肤缩放 | `skin_mesh_properties.skin_scale`           |
| 血条类型 | `health_bar_data.unit_health_bar_style`     |
| 网格路径 | `skin_mesh_properties.simple_skin` (`.skn`) |
| 贴图路径 | `skin_mesh_properties.texture` (`.tex`)     |

#### 皮肤 bin 路径

皮肤 bin 文件路径格式：`data/characters/{name}/skins/{skinname}.bin`

- **Champion 类型**：默认使用 `skin0.bin`（如 `data/characters/aatrox/skins/skin0.bin`）
- **Map 类型**：从 `Unk0xad65d8c4.character.skin` 路径转换
  - 例如：`Characters/Aatrox/Skins/Skin0` → `data/characters/aatrox/skins/skin0.bin`

#### 皮肤 GLB 文件结构

`assets/characters/{name}/skins/{skinN}.glb` 包含：

- **Mesh**: 从 `.skn` 解析的网格数据（Position、Normal、UV、JOINTS_0、WEIGHTS_0）
- **Material**: PBR 材质（metallic=0, roughness=1, alpha_mask=0.3）
- **Texture**: 从 `.tex` 解码的 PNG 贴图
- **Skeleton**: 从 `.skl` 解析的骨骼节点树，使用 `local_transform` 作为 rest pose
- **Skin**: 蒙皮信息，`joints` 和 `inverseBindMatrices` 按 `influences` 顺序排列
- **Animation**: 从 `.anm` 文件解析的动画数据，通道通过 `hash_joint` 匹配到正确的骨骼节点。**动画名称**通过 `clip_data_map` 的 key（u32 hash）查找 hash 对照表获取可读名称，导出时 `hash_to_node` 使用 string 类型作为 key。

#### 皮肤场景文件结构

`assets/characters/{name}/skins/{skinN}.ron` 是一个 Bevy DynamicScene RON 文件，包含以下组件：

- **Skin** - 包含 `scale`（皮肤缩放）
- **HealthBar** - 包含 `bar_type`（血条类型）
- **Visibility** - 默认为 `Visible`

## 关键方法

### PropGroup::get_by_class

通过 class hash 查找数据，而非 entry hash。利用 Bevy 反射系统获取类型的短路径，再转换为哈希值：

```rust
let type_name = T::short_type_path();
let type_hash = type_name_to_hash(type_name);
```

例如 `CharacterRecord` 类型的 type_name 为 `"league_core::extract::CharacterRecord"`，通过 `type_name_to_hash` 得到 class hash。

## 依赖项

### league_core (游戏数据提取)

- `league_core::extract` - 数据结构
- `league_file::grid::AiMeshNGrid` - 导航网格解析
- `league_file::mapgeo::LeagueMapGeo` - 地图几何解析
- `league_loader` - WAD 文件加载

### league_to_lol (转换层)

- `barrack_to_barracks` - 将 `league_core::extract::ConfigBarracks` 转换为 `lol_base::barrack::ConfigBarracks`
- `navgrid_to_navgrid` - 将 `league_file::grid::AiMeshNGrid` 转换为 `lol_base::grid::ConfigNavigationGrid`

### lol_base (稳定配置 Asset)

- `lol_base::barrack::ConfigBarracks` - 兵营配置 (Asset)
- `lol_base::grid::ConfigNavigationGrid` - 导航网格配置 (Asset)
- `lol_base::grid::GridFlagsVisionPathing` - 视野路径标志
- `lol_base::grid::GridFlagsRiverRegion` - 河流区域标志
- `lol_base::grid::GridFlagsJungleQuadrant` - 野区象限标志
- `lol_base::grid::GridFlagsMainRegion` - 主要区域枚举
- `lol_base::grid::GridFlagsNearestLane` - 最近线路枚举
- `lol_base::grid::GridFlagsPOI` - 兴趣点枚举
- `lol_base::grid::GridFlagsRing` - 环形区域枚举
- `lol_base::grid::GridFlagsSRX` - SRX 标志
- `lol_base::spell::Spell` - 技能数据 (Asset)
- `lol_base::spell_calc::CalculationType` - 技能计算类型
- `lol_base::spell_calc::CalculationPart` - 技能计算部件

### lol_core (运行时逻辑)

- `lol_core::attack::Attack` - 攻击组件
- `lol_core::base::bounding::Bounding` - 碰撞边界组件
- `lol_core::base::level::ExperienceDrop` - 经验掉落组件 (exp_given_on_death, experience_radius)
- `lol_core::base::level::Level` - 等级组件
- `lol_core::damage::Damage` - 伤害组件
- `lol_core::damage::Armor` - 护甲组件
- `lol_core::entities::barrack::Barrack` - 兵营运行时组件 (持有 Handle<ConfigBarracks>)
- `lol_core::entities::champion::Champion` - 英雄标记组件
- `lol_core::lane::Lane` - 路线枚举
- `lol_core::life::Health` - 生命值组件
- `lol_core::map::MinionPath` - 小兵路径存储
- `lol_core::movement::Movement` - 移动组件
- `lol_core::skill::Skills` - 技能列表组件
- `lol_core::team::Team` - 队伍组件

### Asset vs Component 设计

| 类型                              | 存储位置               | 用途                                          |
| --------------------------------- | ---------------------- | --------------------------------------------- |
| `ConfigBarracks`                  | `lol_base` (Asset)     | 兵营配置，可被多个兵营实例共享                |
| `Barrack`                         | `lol_core` (Component) | 兵营运行时状态，持有 `Handle<ConfigBarracks>` |
| `ConfigNavigationGrid`            | `lol_base` (Asset)     | 导航网格配置                                  |
| `Spell`                           | `lol_base` (Asset)     | 技能数据配置                                  |
| `ConfigAnimation`                 | `lol_base` (Asset)     | 动画图配置                                    |
| `AnimationHandler`                | `lol_core` (Component) | 动画处理器，持有 `Handle<ConfigAnimation>`    |
| `Bounding`, `Attack`, `Health` 等 | `lol_core` (Component) | 角色初始配置，直接序列化到 config.ron         |

### Asset Loader 注册

新增 Asset 类型后，需在 `lol_render/loaders` 中注册对应的 AssetLoader：

- `lol_base::animation::ConfigAnimation` → `lol_render::loaders::animation::AnimationLoader`

## 粒子特效提取

VfxPrimitiveMesh.mMesh.mSimpleMeshName 是 scb 文件路径，可以转为 Mesh
