# 目录

## 文档

- [游戏内容文档](docs/game/) - 包含 Bevy 游戏相关的设计与实现（如资产、动画、着色器、粒子、技能系统等）以及 156 个英雄的技能文档 (champions)。
- [产品业务文档](docs/product/) - 包含产品功能、计费、对局、平台及智能体 (agent) 等相关设计和任务规划。

# 高频操作

## 使用 LSP 工具

跳转到定义，查找引用这类操作无需 READ 文件，而是直接使用 LSP 工具

## cargo check 检查

检查必须包括所有目标

```sh
cargo check --all-targets
```
