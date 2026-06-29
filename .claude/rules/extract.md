---
paths:
  - "crates/league_to_lol/src/extract/**/*.rs"
---

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
