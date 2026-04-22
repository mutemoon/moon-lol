# 地图数据提取流程

## 架构概述

```
游戏文件 → league_core::extract (解析) → league_to_lol (转换) → lol_base (稳定配置)
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

## 数据来源

### WAD 文件

从以下位置加载游戏资源：

- `DATA/FINAL/UI.wad.client` - UI 资源
- `DATA/FINAL/UI.zh_CN.wad.client` - 中文 UI 资源
- `DATA/FINAL/Maps/Shipping/Map11.wad.client` - 地图数据
- `DATA/FINAL/Bootstrap.windows.wad.client` - 启动资源
- `DATA/FINAL/Champions/*.wad.client` - 英雄资源（动态遍历，跳过语言版本）

### Bin 文件路径

所有 bin 文件路径格式统一为：`data/characters/{name}/{name}.bin`

例如：`data/characters/fiora/fiora.bin`

## 提取流程

### 1. 初始化 Bevy 应用

配置 AssetPlugin 和 TaskPoolPlugin，准备资源加载环境。

### 2. 加载 WAD 文件

通过 LeagueLoader 加载多个 WAD 归档文件，同时动态遍历 Champions 文件夹下的英文版 `.wad.client` 文件（文件名不含下划线）。

### 3. 提取角色数据到组件

从 bin 文件加载角色数据，将数据直接写入组件：

| 组件 | 数据来源 |
|------|----------|
| `Bounding` | `pathfinding_collision_radius`, `health_bar_height` |
| `Attack` | `acquisition_range`, `base_atted_attack_speed` |
| `Health` | `health` |
| `Damage` | `damage` |
| `Armor` | `armor` |
| `Movement` | `move_speed` |
| `Skills` | `spells` 哈希列表 |
| `ExperienceDrop` | `exp_given_on_death`, `experience_radius` |
| `Name` | `m_character_name` |

#### 角色数据的两种来源

**来源一：从英雄 WAD 文件提取**

遍历 `Champions/*.wad.client` 文件，直接从对应的 bin 文件加载。适用于大多数英雄。

**来源二：从地图 Unk0xad65d8c4 提取**

从地图的 `Unk0xad65d8c4` 组件中获取 `character_record` 路径，然后加载对应的 bin 文件。部分地图特有角色从此来源提取。

两种来源使用相同的 bin 路径格式，最终都输出到 `assets/characters/{name}/config.ron`。

### 4. 提取地图可放置物

地图中的可放置物通过 `EnumMap` 枚举区分，包含以下类型：

- **Unk0x3c995caf**：小兵路径
  - 根据名称识别线路：Top（上路）、Mid（中路）、Bot（下路）
  - 包含路径线段和变换矩阵

- **Unk0xba138ae3**：兵营
  - 包含队伍归属（Team）和线路信息
  - 生成带有 Transform、Team、Lane、ConfigBarracks(Asset) 组件的实体

- **Unk0xad65d8c4**：角色
  - 包含 Transform、Team、skin 路径
  - 生成的实体带有 Transform、Team 组件和角色组件

- **MapNavGrid**：导航网格路径
  - 指向导航网格数据文件

### 5. 提取小兵路径

遍历地图可放置物品项，识别 `Unk0x3c995caf` 条目：

- **上路**：`MinionPath_Top`、`TopLaneHomeguardsPath`
- **中路**：`MinionPath_Mid`、`MidLaneHomeguardsPath`
- **下路**：`MinionPath_Bot`、`BotLaneHomeguardsPath`

将变换矩阵转换为平移向量，路径线段根据平移向量进行偏移。

### 6. 导出导航网格

从 MapNavGrid 路径加载二进制导航数据，解析 AI 网格后导出为二进制格式。

### 7. 导出地图几何

解析地图几何数据，遍历所有网格和子网格，查找对应的材质定义，导出为 GLTF 格式。

### 8. 序列化场景

将所有生成的实体（兵营、角色、小兵路径）序列化为 RON 格式保存。

## 输出文件

| 文件 | 描述 |
|------|------|
| `assets/characters/{name}/config.ron` | 角色场景（包含 Bounding、Attack、Health、Damage、Armor、Movement、Skills、Name 等组件的序列化实体） |
| `assets/maps/{map_name}_navgrid.bin` | 二进制导航网格数据 |
| `assets/maps/{map_name}_mapgeo.gltf` | GLTF 格式的地图几何 |
| `assets/maps/{map_name}_scene.ron` | 包含所有地图对象的序列化场景 |

### Character 场景文件结构

`assets/characters/{name}/config.ron` 是一个 Bevy DynamicScene RON 文件，包含以下组件：

- **Champion** - 空组件，标记该实体为英雄
- **Bounding** - 包含 `radius`（碰撞半径）和 `height`（生命条高度）
- **Attack** - 包含 `range`（攻击距离）、`base_attack_speed`（基础攻击速度）、`windup_config`（前摇时间配置）
- **Health** - 包含 `value` 和 `max`（生命值）
- **Damage** - 包含 `0`（攻击力）
- **Armor** - 包含 `0`（护甲值）
- **Movement** - 包含 `speed`（移动速度）
- **Skills** - 包含技能实体列表
- **Name** - 实体名称

可直接通过 Bevy 的场景系统反序列化加载为实体。

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

| 类型 | 存储位置 | 用途 |
|------|----------|------|
| `ConfigBarracks` | `lol_base` (Asset) | 兵营配置，可被多个兵营实例共享 |
| `Barrack` | `lol_core` (Component) | 兵营运行时状态，持有 `Handle<ConfigBarracks>` |
| `ConfigNavigationGrid` | `lol_base` (Asset) | 导航网格配置 |
| `Spell` | `lol_base` (Asset) | 技能数据配置 |
| `Bounding`, `Attack`, `Health` 等 | `lol_core` (Component) | 角色初始配置，直接序列化到 config.ron |
