# 英雄技能研究方法论

## 信息获取策略

本研究方法论采用**优先级递减**的信息获取策略：

| 优先级 | 来源 | 用途 |
|--------|------|------|
| **1st** | LoL Wiki | 技能描述、数值、机制的**主要来源** |
| 2nd | CommunityDragon | JSON 原始数据、用于交叉验证 |
| 3rd | 社区讨论 | 隐藏机制、连招技巧、边缘案例 |

---

## 第一优先级：LoL Wiki

**URL**: `https://wiki.leagueoflegends.com/en-us/{champion}`

### 访问方式

**必须使用 `/browser-use` 技能**进行浏览，该网站有反爬机制。

```bash
browser-use open "https://wiki.leagueoflegends.com/en-us/riven"
```

### 获取的信息

- 技能完整描述（变量已替换，如 `@Damage@` → 实际数值）
- 技能数值（伤害、冷却、范围、消耗）
- 技能协同效果
- 控制优先级
- 范围指示器
- 动画时间（施法时间、冷却等）

### 页面结构

典型页面包含：
1. **Champion Header** - 基础属性、角色定位
2. **Abilities Section** - 被动 + Q/W/E/R 详细数据
3. **Base Statistics** - HP、AD、Armor 等成长数值

### 验证要点

- Wiki 描述已包含游戏内实际数值
- 注意 "based on level" 和 "based on target's missing health" 等动态描述
- 确认技能是 "Physical" / "Magic" / "True" 伤害类型

---

## 第二优先级：CommunityDragon

**URL**: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champions/{id}.json`

### Champion ID 速查表

| 英雄 | ID | 英雄 | ID | 英雄 | ID |
|------|-----|------|-----|------|-----|
| Riven | 92 | Fiora | 114 | Camille | 164 |
| Irelia | 39 | Darius | 122 | Renekton | 58 |
| Aatrox | 266 | Sylas | 517 | Kayn | 141 |
| Jax | 24 | Gnar | 150 | Sett | 875 |
| Urgot | 6 | Hecarim | 120 | Garen | 86 |
| Olaf | 2 | LeeSin | 64 | Pantheon | 80 |
| Volibear | 106 | Kled | 240 | Rengar | 107 |
| Kha'Zix | 121 | Viego | 234 | Vi | 254 |
| Wukong | 62 | | | | |

### 获取的数据

- `effectAmounts` - 技能效果数值数组
- `coefficients` - AD/AP 加成系数
- 技能冷却、消耗、范围原始数据

### 变量占位符

CommunityDragon JSON 使用游戏内部变量（如 `@Damage@`、`@BonusAD@`），需要结合 Wiki 描述进行替换解读。

---

## 第三优先级：社区讨论

### 来源

| 来源 | URL 模式 | 用途 |
|------|----------|------|
| Reddit | r/leagueoflegends, r/{champion}Mains | 隐藏机制、技能 bug |
| Mobalytics | `mobalytics.gg/lol/champions/{name}/build` | 连招视频、Combos |
| LeagueTips | `leaguetips.gg/{champion}-combos...` | 动画取消指南 |
| YouTube | 技能测试视频评论区 | 实战技巧 |
| NGA/贴吧 | 国内社区 | 中文攻略 |

### 搜索查询模板

```
# 英文搜索
"{champion} hidden mechanics"
"{champion} animation cancel guide"
"{champion} combos league of legends"
"{champion} bug list"

# 直接获取 Wiki
site:wiki.leagueoflegends.com "{champion} abilities"

# 组合搜索
"{champion}" + "mechanics" + "reddit"
"{champion}" + "连招" + "技巧"
```

---

## 研究步骤

### 第一阶段：Wiki 数据提取

1. 使用 `/browser-use` 访问 LoL Wiki 页面
2. 提取所有技能数据（被动、Q、W、E、R）
3. 记录完整数值：伤害、冷却、消耗、范围、持续时间
4. 标注伤害类型（Physical/Magic/True）

### 第二阶段：CommunityDragon 交叉验证

1. 根据 Champion ID 获取 JSON 数据
2. 对比 `effectAmounts` 与 Wiki 数值是否一致
3. 提取 `coefficients` 确认 AD/AP 加成
4. 记录任何差异并以 Wiki 为准

### 第三阶段：社区补充

1. 搜索 "champion ability mechanic reddit"
2. 查询技能协同效果（combo、synergy）
3. 验证边缘情况（target caps、minimum damage）
4. 查找动画取消技巧

---

## 验证方法

- **一致性检验**：Wiki 与 CommunityDragon 数值是否匹配
- **补丁历史**：Riot 官方补丁说明
- **Skill Rework**：技能重做历史记录
- **实测验证**：游戏内测试（如果可能）

---

## 文档结构

每个英雄文档应包含：

### 基础信息
- 称号（中/英文）
- 类型（物理/魔法）
- 难度等级
- 角色定位
- 主要位置
- 资源类型
- 攻击距离

### 技能介绍
- 被动技能详解
- Q/W/E/R 四个技能
  - 冷却、消耗、范围
  - 技能描述
  - 效果数值（表格形式）
  - 详细机制说明

### 连招技巧
- 基础连招
- 进阶连招
- 动画取消机制
- 团战进场

### 隐藏机制
- 官方文档未明确说明的机制
- 动画取消技巧
- 技能协同效果

### 常见问题
- FAQ 格式
