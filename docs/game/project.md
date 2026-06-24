# windows

## rust 默认设置

cargo 1.90.0 (840b83a10 2025-07-30)

rustup 1.28.2 (e4f3ad6f8 2025-04-28)
info: This is the version for the rustup toolchain manager, not the rustc compiler.
info: The currently active `rustc` version is `rustc 1.90.0 (1159e78c4 2025-09-14)`

### cargo check

- 什么也不修改
  0.38s
- 修改 src 中的内容
  1.87s
- 修改 crates 中的内容
  2.61s

### cargo run

- 修改 src 中的内容
  13.27s
  13.22s
  13.34s
- 修改 crates 中的内容
  11.35s
  11.57s
  11.43s

## 开启编译优化

- 开启 rust nightly
- 开启 share-generics
- 开启 threads
- 使用 lld

### cargo check

- 什么也不修改
  0.43s
- 修改 src 中的内容
  1.98s
- 修改 crates 中的内容
  2.61s

### cargo run

- 修改 src 中的内容
  14.06s
  14.10s
  14.07s
- 修改 crates 中的内容
  11.57s
  11.69s
  11.62s

## 链接器性能对比

> 基于 Rust example 测试，仅改一行代码启用动态链接

```
WSL Ubuntu
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
默认链接器
████████████████████████████ 4.54s
默认 + 动态链接
████████████ 1.83s  (-60%)
mold
█████████████████ 2.67s  (-41%)
mold + 动态链接
██████████ 1.64s  (-64%)

Windows
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
rust-lld + line-tables-only + 动态链接*
██████████ 1.63s
默认 + line-tables-only + 动态链接*
█████████████ 1.95s
rust-lld + 动态链接*
████████████████ 2.37s
rust-lld
██████████████████████████████████████████████████████████████████████████ 9.50s
默认链接器
██████████████████████████████████████████████████████████████████████████████████████████ 13.00s
```

> \*需启用 "Enable high optimizations for dependencies"
