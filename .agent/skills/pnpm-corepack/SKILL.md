---
name: pnpm-corepack
description: 排查 corepack 与 pnpm 版本不一致、ERR_PNPM_UNEXPECTED_STORE、v10/v11 store 打架。
---

# 根因：corepack 与 shell pnpm 版本不一致

shadcn-vue CLI 内部用 `corepack pnpm` 跑 `pnpm add`。corepack 默认用 `lastKnownGood.json` 里的 pnpm（曾为 11.4.0），而本机 shell 的 `pnpm`（fnm 装的）是 10.33.2。两者各自有 store（v10 vs v11），不通用 → 触发 store 不匹配报错。

# 根治方案：把 corepack 也锁定到 pnpm 10.33.2（两层都要，已配置）

## 1. 项目级（最高优先级，随仓库走）

仓库根 `package.json` 加 `packageManager` 字段（**当前已配置**）：
```json
"packageManager": "pnpm@10.33.2+sha512.a90faf6feeab71ad6c6e57f94e0fe1a12f5dcc22cd754db40ae9593eb6a3e0b6b12e3540218bb37ae083404b1f2ce6db2a4121e979829b4aff94b99f49da1cf8"
```
corepack 检测到此字段后会**强制**使用该版本，覆盖 lastKnownGood。仓库内任何 `corepack pnpm` 调用都用 10.33.2。

> ⚠️ 该字段在**仓库根** `package.json`，不在 `apps/desktop/package.json`。

拿带 sha512 的完整串：
```bash
cd /tmp && mkdir pm-probe && cd pm-probe && echo '{"name":"x"}' > package.json && corepack use pnpm@10.33.2 && grep packageManager package.json
```

## 2. 全局级（影响仓库外的目录）

改 `~/.cache/node/corepack/lastKnownGood.json` 的 `pnpm` 值为 `10.33.2+sha512....`。
- ⚠️ 此文件**只在 Node/corepack 自身被安装/升级时才更新**（不会每次运行回写），所以手改是稳定的，直到下次升级 Node。
- corepack 0.34 **没有 `--activate/--global` 命令**（`corepack use` 只做项目级，`--global` 参数会报 `Unknown Syntax Error`）。

# 验证

仓库内外 `corepack pnpm --version` 都应返回 10.33.2。

# ERR_PNPM_UNEXPECTED_STORE 修复

报错：`[ERR_PNPM_UNEXPECTED_STORE] ... currently linked from the store at v10 ... pnpm now wants to use the store at v11`

根因：corepack 用 pnpm 11，node_modules 链接到 pnpm 10 的 v10 store。
修法：见上方「根治方案」，把 corepack 锁定到 pnpm 10.33.2，两端 store 统一。

> **历史教训**：早期临时解法是「`rm -rf node_modules && corepack pnpm install` 把 store 迁到 v11 去凑 corepack 的版本」——那是反向迁就，不如直接把 corepack 锁回 10。
