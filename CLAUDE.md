# 目录

## 资源导出 `docs/extract.md`

## 资源加载 `docs/load.md`

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
cargo check --examples --tests
```
