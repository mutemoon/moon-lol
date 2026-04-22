# 架构设计

## 自定义结构体 vs 游戏结构体

游戏内的数据解析结构体（如 `CharacterRecord`）会随版本更新改变，因此使用**自定义结构体**（`lol_base` crate）保存关键数据，而非直接使用 `league_core::extract` 中的游戏结构体。

好处：版本稳定、数据隔离、向后兼容已导出配置。

```
游戏文件 → league_core::extract (解析) → lol_base (稳定数据) → lol_core (逻辑)
```

# 开发规范

## 多使用 LSP
