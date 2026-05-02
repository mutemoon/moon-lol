# 目录

## 文档

- [资源导出](docs/extract.md) - WAD 文件结构、7 阶段提取流程、输出文件格式
- [资源加载](docs/load.md) - 三种资源管理方案对比（直接读取/按需提取/完全提取）
- [动画](docs/animation.md) - LOL 动画图组件、AnimationPlayer、骨骼实体层级
- [资产系统](docs/asset.md) - GLTF 导出时预构建皮肤、运行时加载流程
- [粒子系统](docs/particle_system.md) - 发射器层级、生命管理、Transform 父子关系
- [着色器](docs/shader.md) - 不同条件下 shader 的 struct 和 uniform 处理
- [技能系统](docs/skill.md) - ECS + Observer 架构、代码驱动、冷却管理
- [状态机](docs/state.md) - 移动系统、攻击系统、自动攻击、位移优先级
- [项目概览](docs/project.md) - 编译性能对比、wasm vs windows 速度
- [介绍](docs/introduction.md) - 左手坐标系转右手、bevy 版本
- [待办](docs/todo.md) - 开发进度追踪（粒子/技能/UI/寻路等）
- [提示词](docs/prompt.md) - 导弹系统设计、dash 与伤害分离
- [技能测试](docs/skill_test.md) - 测试方法论、harness 规则、真实资源测试策略
- [技能研究](docs/skill_research.md) - 官方/社区数据获取、英雄文档结构
- [英雄](docs/champions/) - 156 个英雄的详细技能文档

# 架构设计

## 自定义结构体 vs 游戏结构体

游戏内的数据解析结构体（如 `CharacterRecord`）会随版本更新改变，因此使用**自定义结构体**（`lol_base` crate）保存关键数据，而非直接使用 `league_core::extract` 中的游戏结构体。

好处：版本稳定、数据隔离、向后兼容已导出配置。

```
游戏文件 → league_core::extract (解析) → lol_base (稳定数据) → lol_core (逻辑)
```

# 开发规范

## 多使用 LSP

## Handle 序列化

Bevy 的 `Handle<T>` 可以被序列化为路径字符串。当 RON 文件反序列化时，Bevy 会根据 Handle 的类型和路径自动加载对应的 Asset。

这意味着：

- `AnimationHandler(Handle<Animation>)` 可以直接添加到场景文件的组件中
- Bevy 反序列化时会自动将路径解析为正确的 Asset Handle

典型用法：

```rust
// 导出时：Handle 会自动序列化为 Asset 路径
spawn(AnimationHandler(asset_server.load("characters/ahri/animation.ron")));

// 导入时：只需确保 Asset 路径正确，Bevy 自动处理
```

## cargo check 检查

检查必须包括 examples 和 tests

```sh
cargo check --workspace
cargo check --examples --tests
```
