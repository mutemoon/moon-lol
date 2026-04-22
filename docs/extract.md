# 地图数据提取流程

## 概述

地图提取过程用于读取英雄联盟游戏文件，提取地图几何、导航网格、小兵路径、物体放置数据以及英雄角色数据，并转换为项目可用的各种格式。

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

### 3. 提取英雄 CharacterRecord

CharacterRecord 是英雄的核心数据结构，包含以下信息：

| 字段 | 用途 |
|------|------|
| pathfinding_collision_radius | 碰撞半径 |
| health_bar_height | 生命条高度 |
| m_character_name | 角色名称 |
| spells | 技能哈希列表 |
| m_character_passive_spell | 被动技能哈希 |
| acquisition_range | 攻击距离 |
| selection_radius | 选择半径 |
| selection_height | 选择高度 |

#### CharacterRecord 的两种来源

**来源一：从英雄 WAD 文件提取**

遍历 `Champions/*.wad.client` 文件，直接从对应的 bin 文件加载。适用于大多数英雄。

**来源二：从地图 Unk0xad65d8c4 提取**

从地图的 `Unk0xad65d8c4` 组件中获取 `character_record` 路径，然后加载对应的 bin 文件。部分地图特有角色从此来源提取。

两种来源使用相同的 bin 路径格式，最终都会输出到 `assets/characters/{name}/config.ron`。

> 注意：导出的 config.ron 只包含基本的组件配置。CharacterRecord 的完整数据（技能、属性等）在运行时通过 `character_record` 路径动态加载。

### 4. 提取地图可放置物

地图中的可放置物通过 `EnumMap` 枚举区分，包含以下类型：

- **Unk0x3c995caf**：小兵路径
  - 根据名称识别线路：Top（上路）、Mid（中路）、Bot（下路）
  - 包含路径线段和变换矩阵

- **Unk0xba138ae3**：兵营
  - 包含队伍归属（Team）和线路信息
  - 生成带有 Transform、Team、Lane、ConfigBarracks 组件的实体

- **Unk0xad65d8c4**：角色
  - 包含 Transform、Team、character_record 路径和 skin 路径
  - 生成的实体带有 Transform、Team、ConfigCharacter 组件

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
| `assets/characters/{name}/config.ron` | 英雄角色场景（包含 Champion、Bounding、Attack、Health、Damage、Armor、Movement、Name 组件的序列化实体） |
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

- `league_core::extract` - 数据结构
- `league_file::grid::AiMeshNGrid` - 导航网格解析
- `league_file::mapgeo::LeagueMapGeo` - 地图几何解析
- `league_loader` - WAD 文件加载
- `lol_base::barrack::ConfigBarracks` - 兵营配置
- `lol_base::character::ConfigCharacter` - 角色配置（用于地图对象）
- `lol_core::attack::Attack` - 攻击组件
- `lol_core::base::bounding::Bounding` - 碰撞边界
- `lol_core::damage::Damage` - 伤害组件
- `lol_core::damage::Armor` - 护甲组件
- `lol_core::entities::champion::Champion` - 英雄标记组件
- `lol_core::lane::Lane` - 路线枚举
- `lol_core::life::Health` - 生命值组件
- `lol_core::map::MinionPath` - 小兵路径存储
- `lol_core::movement::Movement` - 移动组件
- `lol_core::team::Team` - 队伍组件（用于地图对象）
