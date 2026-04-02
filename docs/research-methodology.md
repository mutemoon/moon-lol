# 英雄技能研究方法论

## 数据来源

### 官方数据源

- **CommunityDragon**: `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champions/{id}.json`
  - 包含完整的技能描述、冷却、消耗、范围、效果数值
  - 字段说明：`effectAmounts` 对应技能效果，`coefficients` 对应 AD/AP 加成
  - 注意：JSON 中的描述使用游戏内部变量（如 `@Damage@`、`@BonusAD@`），需要运行时替换

### 社区讨论来源

- **Reddit**: r/leagueoflegends, r/{champion}Mains（需要解决网络封锁问题）
- **Mobalytics.gg**: 每个英雄都有 Combos 页面，包含大量连招视频和文字说明
- **LeagueTips.gg**: 提供详细的动画取消和连招指南
- **YouTube**: 技能测试视频评论区
- **Mobafire**: 攻略站（可能被 Cloudflare 封锁）
- **NGA/贴吧**: 国内社区（可能需要登录）

## 浏览器自动化经验

### agent-browser 使用技巧

1. **处理动态加载页面**: 使用 `agent-browser wait --load networkidle` 等待网络空闲
2. **处理重定向**: 某些网站会重定向（如腾讯官网），需要检查最终 URL
3. **处理封锁**:
   - Reddit 需要开发者 Token 或登录
   - Mobafire/NGA 被 Cloudflare 保护
   - 可以尝试备用域名或搜索引擎快照
4. **获取页面内容**: 使用 `agent-browser get text body` 获取文本，使用 `snapshot -i` 获取交互元素

### 有效的来源

| 来源            | URL 模式                                           | 可用性                     |
| --------------- | -------------------------------------------------- | -------------------------- |
| Mobalytics      | `https://mobalytics.gg/lol/champions/{name}/build` | 高（需点击 Combos tab）    |
| LeagueTips      | `https://leaguetips.gg/{champion}-combos...`       | 高                         |
| CommunityDragon | JSON API                                           | 高（通过 curl 或脚本获取） |
| 搜索引擎        | DuckDuckGo/Google                                  | 中（需要绕过封锁）         |

### 搜索查询模板

```
# 英文搜索
"{champion} hidden mechanics"
"{champion} animation cancel guide"
"{champion} combos league of legends"
"{champion} bug list"

# 组合搜索
"{champion}" + "mechanics" + "reddit"
"{champion}" + "连招" + "技巧"
```

## 研究步骤

### 第一阶段：官方数据提取

1. 从 CommunityDragon 获取 champion JSON（使用 champion ID）
2. 解析 `spells` 数组，按 `spellKey` (q/w/e/r) 映射技能
3. 提取 `effectAmounts` 和 `coefficients`
4. 记录所有变量占位符（如 `@FirstSlashDamage@`）

### 第二阶段：技能机制深度挖掘

1. 搜索 "champion ability mechanic reddit" / "champion bug list" / "champion hidden mechanic"
2. 查询技能协同效果（combo、synergy）
3. 验证边缘情况（target caps、minimum damage、distance calculations）
4. 查找 "does X ability trigger Y" 类型的问题

### 第三阶段：中文社区补充

1. 搜索对应英雄的技能细节讨论
2. 验证国服翻译与机制差异
3. 收集连招技巧和实战心得

## 常见技能机制分类

### 位移类

- 穿墙判定（collision radius vs terrain thickness）
- 位移过程中的无敌帧（i-frames）
- 取消窗口（cast time vs animation cancel）
- 目标选取优先级

### 增益/护盾类

- 护盾类型（physical/magic/true）
- 护盾衰减机制
- 增益持续时间
- 攻击特效触发（是否能触发命中特效、吸血等）

### 伤害类

- 伤害类型（physical/magic/true）
- 最小伤害保证
- 伤害加成系数（AD/AP）
- 范围递减（edge vs center）
- 目标数量上限

### 控制类

- 控制时长与等级关系
- 多个控制效果的叠加规则
- 韧性减免
- 免疫机制（净化、水银鞋）

## 验证方法

- 对比多个来源的一致性
- 查找 Riot 官方补丁说明
- 参考 Skill Rework 历史记录
- 实际游戏测试（如果可能）

## Champion ID 速查表

| 英雄     | ID  |
| -------- | --- |
| Riven    | 92  |
| Fiora    | 114 |
| Camille  | 164 |
| Irelia   | 39  |
| Darius   | 122 |
| Renekton | 58  |
| Aatrox   | 266 |
| Sylas    | 517 |
| Kayn     | 141 |
| Jax      | 24  |
| Gnar     | 150 |
| Sett     | 875 |
| Urgot    | 6   |
| Hecarim  | 120 |
| Garen    | 86  |
| Olaf     | 2   |
| LeeSin   | 64  |
| Pantheon | 80  |
| Volibear | 106 |
| Kled     | 240 |
| Rengar   | 107 |
| Kha'Zix  | 121 |
| Viego    | 234 |
| Vi       | 254 |
| Wukong   | 62  |

# 英雄技能文档

champions 目录包含 25 位上单/打野战士英雄的详细技能介绍、连招技巧和隐藏机制。

## 目录

| 英雄     | 文件          | 特点                          |
| -------- | ------------- | ----------------------------- |
| 锐雯     | `riven.md`    | 大量动画取消技巧，Fast Q 机制 |
| 剑姬     | `fiora.md`    | 要害系统，W 格挡时机          |
| 青钢影   | `camille.md`  | E 钩索地形系统，双形态连招    |
| 刀妹     | `irelia.md`   | 不稳状态，Q 刷新机制          |
| 诺手     | `darius.md`   | 血怒叠加系统，5 层斩杀        |
| 鳄鱼     | `renekton.md` | 怒气系统，强化技能机制        |
| 剑魔     | `aatrox.md`   | Q 边缘判定，E 取消后摇        |
| 塞拉斯   | `sylas.md`    | 大招盗取系统                  |
| 凯隐     | `kayn.md`     | 双形态系统（红凯/蓝凯）       |
| 武器大师 | `jax.md`      | Q 跳眼位移，E 闪避反击        |
| 纳尔     | `gnar.md`     | 变身机制，怒气管理系统        |
| 瑟提     | `sett.md`     | 斗心系统，E 两侧眩晕          |
| 乌加斯   | `urgot.md`    | 斩杀机制，Q 消耗配合          |
| 赫卡里姆 | `hecarim.md`  | E 冲锋穿墙，R 恐惧团控        |
| 盖伦     | `garen.md`    | Q 移除减速，E 旋转输出        |
| 奥拉夫   | `olaf.md`     | R 免疫控制，斧头回收机制      |
| 李青     | `leesin.md`   | 眼位跳，Insec 踢              |
| 潘森     | `pantheon.md` | 盾牌格挡，被动强化            |
| 沃里克   | `volibear.md` | R 越塔，W 标记系统            |
| 克烈     | `kled.md`     | 骑乘状态系统，勇气值回复机制   |
| 雷恩加尔 | `rengar.md`   | 残暴值系统，伪装潜行          |
| 卡兹克   | `khazix.md`   | 孤立判定，进化技能系统        |
| 佛耶戈   | `viego.md`    | 破败之王，被动已损失生命伤害  |
| 蔚       | `vi.md`       | Q 蓄力位移，W 护甲削减        |
| 悟空     | `wukong.md`   | W 隐形分身，E 突进击飞        |

## 文档结构

每个英雄文档包含以下部分：

### 基础信息

- 称号（中/英文）
- 类型（物理/魔法）
- 难度等级
- 角色定位
- 主要位置

### 技能介绍

- 被动技能详解
- Q/W/E/R 四个技能
  - 冷却、消耗、范围
  - 技能描述
  - 效果数值
  - 隐藏机制

### 连招技巧

- 基础连招
- 进阶连招
- 团战连招

### 隐藏机制

- 官方文档未明确说明的机制
- 动画取消技巧
- 技能协同效果
