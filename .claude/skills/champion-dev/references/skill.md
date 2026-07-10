# 技能系统架构设计

- 架构类型：基于 `ECS` 与 `Observer` 的解耦技能架构。
- 核心设计原则：回归代码主导逻辑，施法校验由统一管线处理，技能的具体动作与状态变迁完全由代码观察者系统控制。
- 行为与状态：技能的运行逻辑以系统形式写在代码中，运行时状态挂载在 `ECS` 实体上。

# 英雄技能开发流程与核心组件

## 统一管线
- 管线定位：通用施法前置校验与扣减资源，校验通过后派发事件。详见 [skill-pipe.md](./skill-pipe.md)。

## 观察者与联动
- 英雄观察者：通过监听施法事件分发技能逻辑。
- 联动观察者：监听伤害造成与普攻结束等衍生事件。

## 原子动作与状态
- 原子动作：播放动画、冲刺位移、范围伤害场、发射飞弹等。详见 [skill-action.md](./skill-action.md)。
- 状态系统：提供护盾、眩晕、减速等通用效果。

# 英雄标准目录结构

- 路径规则：英雄目录位于 `crates/lol_champions/src/<hero>/`。
- `mod.rs`：注册插件与核心施法监听器。
- 技能文件：`q.rs`、`w.rs`、`e.rs`、`r.rs` 存放对应主动技能逻辑。
- `passive.rs`：存放被动技能逻辑。
- `buffs.rs`：声明英雄特有的状态组件。
- `tests.rs`：集成测试与测试配置。

# 数值读取机制

- 计算公式值：通过 `get_skill_value` 动态解析属性加成公式。详见 [skill-value.md](./skill-value.md)。
- 配置原始值：通过 `get_skill_data_value` 读取特定等级原始数组。

# 典型技能参考文档

- 单段技能：[skill-single-stage.md](./skill-single-stage.md)
- 分段重施技能：[skill-multi-stage.md](./skill-multi-stage.md)
- 被动联动技能：[skill-passive.md](./skill-passive.md)
- 飞弹与投射物：[skill-missile.md](./skill-missile.md)
- 冷却控制机制：[skill-cooldown.md](./skill-cooldown.md)
- 测试与调试实践：[skill-test.md](./skill-test.md)
