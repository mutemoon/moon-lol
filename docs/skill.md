# 技能系统架构设计

当前技能系统已经彻底切换为 **纯代码驱动的 ECS + Observer 架构**。

系统中不再存在以下能力：

- `SkillEffect`
- `SkillAction`
- “静态动作表”
- 基于 effect 资产的技能执行分发

所有技能都必须通过 Rust 代码响应 `EventSkillCast`，由 observer 直接编排动作、状态、窗口和冷却。

---

## 设计目标

`docs/champions` 中的英雄技能包含大量固定动作表无法正确表达的机制：

- 分段技能：锐雯 Q、剑魔 Q、鳄鱼 E、刀妹 E
- 二段窗口：盲僧 Q/W/E、卡蜜尔 E
- 形态/强化态：纳尔、凯隐、鳄鱼、潘森
- 标记与被动联动：剑姬要害、诺手血怒、沃利贝尔 W、塞拉斯偷大
- 条件冷却：命中减冷却、击杀刷新、终段才进主冷却
- 复杂目标逻辑：最近目标、命中英雄才刷新、对特定目标开放二段

这类逻辑如果继续用静态配置表达，只会把复杂性转移到隐蔽角落。  
因此现在的原则很直接：

- **技能的行为写在代码里**
- **技能的状态挂在 ECS 实体上**
- **统一施法管线只负责校验和派发**

---

## 核心原则

### 1. 技能是实体

每个技能都对应一个独立实体，通过：

- `SkillOf`
- `Skills`
- `PassiveSkillOf`

挂到英雄实体上。

技能实体本身承担状态，而不是只做索引占位。

常见状态包括：

- 技能等级
- 冷却
- 重施窗口
- 当前阶段
- 临时标记
- 形态相关信息

### 2. 施法走统一入口

统一施法链路如下：

1. 输入层触发 `CommandAction`
2. `Action::Skill` 转发为 `CommandSkillBeforeStart` 与 `CommandSkillStart`
3. `on_skill_cast` 完成通用校验
4. 校验通过后触发 `EventSkillCast`
5. 具体技能 observer 接管后续逻辑

统一入口负责的只有这些通用职责：

- 查找技能实体
- 校验技能是否已学习
- 校验资源
- 校验冷却
- 记录施法日志
- 按统一规则启动默认冷却

具体技能行为不再放在这个入口里。

### 3. observer 是唯一技能执行方式

技能施法通过校验后，会触发：

```rust
EventSkillCast {
    entity,
    skill_entity,
    index,
    point,
}
```

之后必须由 observer 处理。

observer 可以直接：

- 读取技能槽位
- 查询技能实体状态
- 查询施法者状态
- 决定表现
- 决定判定
- 推进阶段
- 启动或延后冷却

这让复杂技能的逻辑和状态推进保持在一处，可读、可测、可调试。

---

## 核心数据结构

### `Skill`

`Skill` 现在只表达代码驱动技能所需的最小元数据：

- `key_spell_object`
- `level`
- `slot`
- `cooldown_mode`

字段语义：

- `key_spell_object`：数值来源
- `level`：技能等级
- `slot`：`Q/W/E/R/Passive/Custom`
- `cooldown_mode`：默认施法后冷却，或手动冷却

不再包含：

- effect key
- 执行模式切换
- 静态动作定义

### `SkillCooldownMode`

当前有两种冷却方式：

- `AfterCast`
- `Manual`

`AfterCast` 适用于：

- 普通单段技能
- 按下即进入冷却的技能

`Manual` 适用于：

- 分段技能
- 二段技能
- 终段才进入主冷却的技能

### `SkillRecastWindow`

这是当前最基础的通用技能状态组件。

包含：

- `stage`
- `max_stage`
- `timer`

用于表示：

- 当前处于第几段
- 最多可重施到第几段
- 剩余重施时间

任何二段/三段技能都应优先考虑基于这个组件建模。

### `SkillCastLog`

系统会记录每次施法尝试，供测试和调试使用。

日志内容包括：

- 施法者
- 技能实体
- 索引
- 槽位
- 目标点
- 成功或失败原因

当前失败原因覆盖：

- 缺少技能列表
- 非法索引
- 技能实体不存在
- spell object 缺失
- 技能未学习
- 资源组件缺失
- 资源不足
- 冷却中

这保证了技能行为可以在 headless 环境里直接断言，而不是只能看视觉结果。

---

## 动作触发方式

虽然不再有 `SkillAction`，但技能行为仍然是由一组标准化原子动作拼装出来的。

当前常用原子包括：

- `play_skill_animation(...)`
- `spawn_skill_particle(...)`
- `despawn_skill_particle(...)`
- `skill_dash(...)`
- `skill_damage(...)`
- `reset_skill_attack(...)`

这些函数只是对已有 action/event 的代码封装，不是新的数据层。

它们的作用是：

- 降低 observer 中的重复样板
- 保持技能代码直接可读
- 不重新引入静态 effect 配置层

---

## 推荐实现范式

### 1. 单段技能

适合：

- 一个施法动作
- 一次位移或一次伤害
- 无复杂阶段状态

实现方式：

1. 注册 `Skill::new(slot, spell_key)`
2. 在 `EventSkillCast` 中按 `slot` 匹配
3. 直接调用动画、粒子、伤害、位移等原子函数

### 2. 分段技能

适合：

- 锐雯 Q
- 盲僧二段技能
- 鳄鱼 E

实现方式：

1. 使用 `SkillCooldownMode::Manual`
2. 在技能实体上挂 `SkillRecastWindow`
3. observer 根据 `stage` 决定当前段行为
4. 非终段只刷新窗口，不进入主冷却
5. 终段结束后手动启动冷却

### 3. 被动技能

被动技能仍然是技能实体，但通常不走主动施法路径。

它们应通过：

- `PassiveSkillOf`
- observer
- 周期系统
- 伤害/攻击/状态事件

与主动技能和角色行为联动。

---

## 典型例子：锐雯 Q

锐雯 Q 是当前纯代码驱动技能的样板。

实现结构：

- 技能实体槽位为 `Q`
- 冷却模式为 `Manual`
- observer 监听 `EventSkillCast`
- 从技能实体读取 `SkillRecastWindow`
- 根据 `stage` 分别执行 Q1/Q2/Q3
- Q1/Q2 写回下一段窗口
- Q3 清理窗口并手动启动冷却

这个实现直接对应文档里的真实语义：

- 分段状态不再通过替换 effect key 实现
- 主冷却时机由技能自己决定
- 技能逻辑与技能状态写在一处

---

## 面向英雄文档的建模建议

根据 `docs/champions`，后续技能可以按以下方式建模：

### A. 分段/重施技能

优先使用：

- `SkillRecastWindow`
- `Manual` 冷却
- `EventSkillCast` observer

代表：

- Riven Q
- Lee Sin Q/W/E
- Renekton E
- Irelia E

### B. 形态/强化态技能

优先使用：

- 英雄实体上的形态状态
- 技能实体上的局部状态
- observer 中的显式分支

代表：

- Gnar 变身
- Kayn 形态
- Renekton 怒气强化
- Pantheon 被动强化

### C. 命中刷新/击杀刷新

优先使用：

- 伤害或命中事件 observer
- 直接修改技能实体冷却或窗口

代表：

- Fiora Q 命中减冷却
- Irelia Q 刷新
- Hecarim Q 叠层减冷却

### D. 被动标记系统

优先使用：

- 被动技能实体
- 目标身上的标记组件
- 伤害事件联动

代表：

- Fiora Vital
- Darius 血怒
- Volibear W
- Sylas R 偷取大招

---

## 测试策略

技能系统必须优先保证在 tests 中可验证。

当前测试关注两层：

### 1. 核心技能单测

见：

- [tests/skill.rs](/Users/zhr/Documents/workspace/moon-lol/tests/skill.rs)

覆盖内容：

- 升级规则
- 资源门槛
- 施法日志
- observer 技能执行
- `SkillRecastWindow` 生命周期
- 手动冷却逻辑

### 2. 管线集成测试

见：

- [tests/skill_integration.rs](/Users/zhr/Documents/workspace/moon-lol/tests/skill_integration.rs)
- [tests/riven_integration.rs](/Users/zhr/Documents/workspace/moon-lol/tests/riven_integration.rs)

覆盖内容：

- 通过 `Action` 输入走完整施法管线
- headless 环境中的资源、伤害、冷却变化
- 代码驱动技能的真实阶段推进

### 可测试性标准

一个技能设计如果做不到下面几点，通常说明实现方式不够好：

- 能在 headless `App` 中直接施放
- 能直接断言技能实体状态
- 能直接断言施法失败原因
- 不依赖渲染即可验证技能结果

---

## 当前结论

现在的技能系统是彻底的代码驱动版本：

- **技能行为全部由 observer 实现**
- **技能状态全部挂在 ECS 实体上**
- **统一施法管线只做通用校验和派发**
- **不再依赖任何静态 effect 配置**

这套设计才能覆盖 `docs/champions` 中那些真正复杂的英雄机制，并在 tests 中保持足够高的可测试性。
