---
name: frontend-op-history
description: apps/desktop 前端代码修改后，按规范更新操作历史记录。
---

# 规则

apps/desktop 前端代码修改后，**需要更新操作历史**，保持信息密度，内容精简。

追加格式：
- 编号续接上一条（当前最后一条为 10）。
- 每条一句话概括做了什么 + 关键命令/坑。
- 新增条目追加到下方列表末尾，不改动已有条目（除非是更正）。

# 操作历史

1. 安装 table 组件

```
pnpm dlx shadcn-vue@latest add table
```

2. 安装 `@tanstack/vue-table`，并使用 shadcn-vue CLI 安装 `button`、`badge`、`dropdown-menu`、`select` 和 `checkbox` 支撑组件。

3. 重构 `GameConsoleLogs.vue`，利用 `@tanstack/vue-table` 与 Shadcn Table 替换原有的列表，支持多维度过滤、自选列开启/隐藏配置（Customize Columns）、完美保留物理服务端 SQL 分页。

4. 修复了由于替换偏差导致的组件语法与截断问题，完善了事件参数类型标注（`MouseEvent`）及下拉选择框更新回调中的类型安全转换（`String(val)`），彻底消除了编译与 TypeScript 检查报错，实现 0 Error 完美通过构建。

5. 重构表格列定义及模板，完成全站汉化本地化（Localization）；并将原生表情和陈旧动作符号全部升级为精致的 Lucide 矢量图标，通过 `vue-tsc` 严格类型检测并顺利编译通过。

6. 安装 `command` 与 `popover` 组件（`pnpm dlx shadcn-vue@latest add command popover`，CLI 自动带入 `cmdk-vue`、`input-group`）。安装前需将 `pnpm-workspace.yaml` 的 `allowBuilds` 占位值填为 `true`（esbuild/protobufjs/sharp/vue-demi），否则 CLI 在非 TTY 下因 `[ERR_PNPM_IGNORED_BUILDS]` 中止；并通过 `corepack pnpm install` 将 node_modules 迁移到 v11 store 以匹配 CLI 调用的 corepack pnpm 11。

7. 用 `Popover` + `Command` 组合（combobox 模式）重写 `PresetSelect.vue`：支持搜索过滤、勾选回显、末尾内置「＋ 新建预设」入口 emit `new`。供编排页 `index.vue` 的双阵营槽位选择 Agent 预设 / 出生点预设。

8. 覆盖式重装全部 14 个 UI 组件（`npx shadcn-vue@latest add <组件> --overwrite --yes`），将组件文件还原为 shadcn 官方默认版本。

9. 根治 corepack 与 shell pnpm 版本不一致（11.4.0 vs 10.33.2、v10/v11 store 打架）：仓库根 `package.json` 加 `packageManager: pnpm@10.33.2+sha512...`（项目级，最高优先级），并改 `~/.cache/node/corepack/lastKnownGood.json`（全局级，影响仓库外目录）。验证仓库内外 `corepack pnpm --version` 均为 10.33.2。

10. 风格迁移 reka-vega → reka-luma（preset `a2vfHGK`）：改 `components.json` 的 `style` 为 `reka-luma`，覆盖重装全部 14 个组件（`--overwrite --yes`），`style.css` 的 `--radius` 从 `0.625rem` 改为 `0.45rem`。注意：node_modules 此前被迁到 v11 store（凑旧 corepack pnpm 11），锁定 pnpm 10 后需 `rm -rf node_modules && pnpm install` 迁回 v10 store，否则 shadcn CLI 又报 store 不匹配。
