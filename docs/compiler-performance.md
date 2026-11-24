# 编译性能实测报告

环境：i7-13700KF (16C/24T) / 64 GB / NTFS 4K / Windows MSVC / rust-lld
工具链：nightly 1.97.0 (2026-04-24)
测试日期：2026-11-23

---

# 第一部分：`debug = false` 依赖去调试信息（已采纳 ✅）

## 改动

```toml
[profile.dev.package."*"]
# 为了开启 bevy 的动态链接
opt-level = 3
# 依赖不需要单步调试，省掉 O3 下最臃肿的那部分调试信息
debug = false
```

## 实测结果

| 指标 | 改动前 | 改动后 | 变化 |
|---|---|---|---|
| **增量编译**（改 `lol_share`） | 168.9 s | **58.9 s** | **快 2.87 倍** |
| 全量编译（热缓存） | 406.8 s | 371.0 s | 快 8.8% |
| `target` 体积 | 18.73 GB | **16.30 GB** | 小 13% |
| 其中 PDB | 9.23 GB | **7.56 GB** | 小 18% |
| 运行性能 | — | — | 无影响（`opt-level` 未变） |

## 结论

**增量编译快 2.87 倍**是这个改动最大的价值。原因：`[profile.dev.package."*"] opt-level = 3`
让所有依赖都在 O3 编译，而 O3 优化后的代码要生成大量复杂的 `DW_AT_location` 映射，
这部分调试信息在每次**链接**时都要被重新拼接。去掉后链接时间大幅下降。

代价：无法对依赖内部做单步调试和查看变量（行号堆栈仍然正常，自己的代码
`debug = "line-tables-only"` 不受影响）。日常开发几乎不会去单步 bevy/tokio 内部。

## 若想更激进

```toml
[profile.dev]
debug = false    # 自己的代码也不要调试信息，再快一档
```

但这会让断点和变量查看完全失效，不建议。

---

# 第二部分：Cranelift 后端（不建议 ❌）

## 结论速览

| 维度 | LLVM | Cranelift | 差异 |
|---|---|---|---|
| 全量编译（463 crate 子集） | 265.8 s | 145.4 s | **快 45%** |
| 增量编译（改 lol_share 及下游） | 31.1 s | 22.6 s | **快 27%** |
| 产物体积（463 crate 子集） | 4.77 GB | 4.18 GB | **小 12%** |
| 运行性能 dev (opt-level 0) | 1288 ms | 1187 ms | 快 8% |
| 运行性能 opt-level 3 | 305 ms | 1258 ms | **慢 4.1 倍** |
| 全工作区可构建 | ✅ | ❌ 链接失败 | 见下 |

## 阻塞问题：`aws-lc-sys` 链接失败

```
rust-lld: error: undefined symbol: aws_lc_0_41_0_EVP_AEAD_CTX_seal
rust-lld: error: undefined symbol: aws_lc_0_41_0_ECDSA_SIG_get0_r
...（共 40 个符号）
error: could not compile `client` (bin "client")
error: could not compile `lol_agent_runtime` (lib test)
error: could not compile `lol_game_process_manager` (lib test)
```

`aws-lc-sys`（C 库，经 `rustls` ← `reqwest` / `rig` 引入）产出了
`aws_lc_0_41_0_crypto.lib`（20.3 MB），但 Cranelift 编译出的 rlib **丢失了 native
link 搜索路径**，静态库路径没有出现在 rust-lld 命令行里。同样配置下 LLVM 完全正常。

影响链路：

| 引入者 | 受影响 crate |
|---|---|
| `rig` / `rig-core` | `lol_agent_runtime`、`lol_game_process_manager` |
| `reqwest` | `apps/client` |

## 运行性能：核心风险

基准：浮点密集 + 分支 + 随机索引的游戏逻辑热循环，200 万元素 × 30 轮。

| 配置 | 耗时 |
|---|---|
| LLVM release (`opt-level = 3`) | 305 ms |
| LLVM dev (`opt-level = 0`) | 1288 ms |
| Cranelift dev (`opt-level = 0`) | 1187 ms |
| **Cranelift `opt-level = 3`** | **1258 ms** |

1. `opt-level = 0` 时 Cranelift 比 LLVM 快 8%（LLVM 在 O0 插入了大量调试用栈操作）。
2. `opt-level = 3` 时 Cranelift 慢 4.1 倍（其优化能力远弱于 LLVM）。
3. **本项目的 `opt-level = 3` 配置会让 Cranelift 彻底失效**：

```toml
[profile.dev.package."*"]      # ← 所有依赖
opt-level = 3
[profile.dev.package.lol_env]  # ← 仿真核心
opt-level = 3
[profile.dev.package.lol_core]
opt-level = 3
[profile.dev.package.lol_rl]
opt-level = 3
```

`lol_env` / `lol_core` / `lol_rl` 是强化学习和游戏仿真的性能核心，恰恰是
Cranelift 最不擅长的部分，也正是当初为性能才调高 opt-level 的部分。

## 为什么不建议

1. **当前根本构建不出来**（aws-lc 链接失败）；
2. 即便绕过，核心竞争力（`lol_env` / `lol_rl` 仿真速度）高度依赖 O3，
   而 Cranelift 在这一档慢 4 倍；
3. `-Z codegen-backend` 是**全局 profile 设置**，无法按 package 粒度指定，
   所以"只给非热路径开 Cranelift"这条路走不通；
4. 靠 `debug = false` 已经拿到了比 Cranelift 更大的增量收益（2.87 倍 vs 1.27 倍），
   且零运行性能代价。

---

# 附：测试过程中的注意事项

- `cargo check` **不经过 codegen 后端**，不能用来测 Cranelift 编译速度。
- Cranelift 与 LLVM 不能共用 `CARGO_TARGET_DIR`，切换会触发全量重编。
- 首次构建（磁盘冷缓存）耗时会显著偏高，对比测试需重跑取稳定值。
  `target/debug` 已膨胀到 241 GB / 28437 个 deps 文件，冷启动压力明显。
- 测试期间遇到一次 rustc `STATUS_ACCESS_VIOLATION` 崩溃（`bevy_world_serialization`），
  重试后正常，与本次配置改动无关。
