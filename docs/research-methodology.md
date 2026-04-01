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

| 来源 | URL 模式 | 可用性 |
|------|----------|--------|
| Mobalytics | `https://mobalytics.gg/lol/champions/{name}/build` | 高（需点击 Combos tab） |
| LeagueTips | `https://leaguetips.gg/{champion}-combos...` | 高 |
| CommunityDragon | JSON API | 高（通过 curl 或脚本获取） |
| 搜索引擎 | DuckDuckGo/Google | 中（需要绕过封锁） |

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

| 英雄 | ID |
|------|-----|
| Riven | 92 |
| Fiora | 114 |
| Camille | 164 |
| Irelia | 39 |
| Darius | 122 |
| Renekton | 58 |
| Aatrox | 266 |
| Sylas | 517 |
| Kayn | 141 |
| Jax | 24 |
| Gnar | 150 |
| Sett | 875 |
| Urgot | 6 |
| Hecarim | 120 |
| Garen | 86 |
| Olaf | 2 |
| LeeSin | 64 |
| Pantheon | 80 |
| Volibear | 106 |
