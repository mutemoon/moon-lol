# 操作历史

代码修改后，需要更新操作历史，保持信息密度，内容精简

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

---

# shadcn-vue 组件安装经验

> 本节是踩坑后的标准做法。装新组件或重装时先读这里，避免重复踩雷。

## 标准命令

```bash
# 安装单个组件（依赖不存在时 CLI 会自动 pnpm add）
pnpm dlx shadcn-vue@latest add <component>

# 安装多个
pnpm dlx shadcn-vue@latest add command popover

# 覆盖式重装（已有文件直接覆盖，跳过交互）
pnpm dlx shadcn-vue@latest add <component...> --overwrite --yes

# 装全部可用组件
pnpm dlx shadcn-vue@latest add --all
```

常用标志：
- `-y, --yes`：跳过确认提示
- `-o, --overwrite`：覆盖已存在文件（**默认不覆盖**，遇到同名会停下问 y/N）
- `--dry-run`：⚠️ **shadcn-vue 尚未实现**，会直接报错

## 根因：corepack 与 shell pnpm 版本不一致

CLI 内部用 `corepack pnpm` 跑 `pnpm add`。corepack 默认用 `lastKnownGood.json` 里的 pnpm（曾为 11.4.0），而本机 shell 的 `pnpm`（fnm 装的）是 10.33.2。两者各自有 store（v10 vs v11），不通用 → 触发下面三个坑。

**根治方案：把 corepack 也锁定到 pnpm 10.33.2**（两层都要，已配置）：

1. **项目级**（最高优先级，随仓库走）：仓库根 `package.json` 加
   ```json
   "packageManager": "pnpm@10.33.2+sha512.a90faf6feeab71ad6c6e57f94e0fe1a12f5dcc22cd754db40ae9593eb6a3e0b6b12e3540218bb37ae083404b1f2ce6db2a4121e979829b4aff94b99f49da1cf8"
   ```
   corepack 检测到此字段后会**强制**使用该版本，覆盖 lastKnownGood。仓库内任何 `corepack pnpm` 调用都用 10.33.2。
   - 拿带 sha512 的完整串：`cd /tmp && mkdir pm-probe && cd pm-probe && echo '{"name":"x"}' > package.json && corepack use pnpm@10.33.2 && grep packageManager package.json`

2. **全局级**（影响仓库外的目录）：改 `~/.cache/node/corepack/lastKnownGood.json` 的 `pnpm` 值为 `10.33.2+sha512....`。
   - ⚠️ 此文件**只在 Node/corepack 自身被安装/升级时才更新**（不会每次运行回写），所以手改是稳定的，直到下次升级 Node。
   - corepack 0.34 **没有 `--activate/--global` 命令**（`corepack use` 只做项目级，`--global` 参数会报 `Unknown Syntax Error`）。

验证：仓库内外 `corepack pnpm --version` 都应返回 10.33.2。

> 历史教训：早期临时解法是「`rm -rf node_modules && corepack pnpm install` 把 store 迁到 v11 去凑 corepack 的版本」——那是反向迁就，不如直接把 corepack 锁回 10。

---

## 三个坑（根治后大多不再出现）

根治 corepack 版本后，以下坑基本消失；但 `allowBuilds` 那个与 corepack 无关，仍需注意。

### 坑 1：pnpm store 版本不匹配（已根治）

报错：`[ERR_PNPM_UNEXPECTED_STORE] ... currently linked from the store at v10 ... pnpm now wants to use the store at v11`

根因：corepack 用 pnpm 11，node_modules 链接到 pnpm 10 的 v10 store。
修法：见上方「根治方案」，把 corepack 锁定到 pnpm 10.33.2，两端 store 统一。

### 坑 2：`pnpm-workspace.yaml` 的 `allowBuilds` 是占位符

根目录 `pnpm-workspace.yaml` 里可能写成：
```yaml
allowBuilds:
  esbuild: set this to true or false   # ← 占位符，未填值
```
这会让 pnpm 在非 TTY（即 CLI 子进程）下抛 `[ERR_PNPM_IGNORED_BUILDS]` 并**以非零状态退出**，导致 CLI 在「装依赖」这一步中止，**连组件文件都不写**。

修法：把占位符填成 `true`（esbuild / protobufjs / sharp / vue-demi 都是安全的原生构建脚本）：
```yaml
allowBuilds:
  esbuild: true
  protobufjs: true
  sharp: true
  vue-demi: true
```

### 坑 3：`--overwrite` 未指定时，每个同名文件都问 y/N

装 `command` 这种会带出一堆依赖组件（button、dialog、input…），如果项目里已有同名文件，CLI 会逐个停下问「Would you like to overwrite? (y/N)」。在非交互环境下会卡死。

修法：
- **要覆盖**：加 `--overwrite --yes`
- **不覆盖**：`yes n |` 管道喂「否」（CLI 会跳过已有文件，只写新文件）

## 覆盖式重装的注意事项

`--overwrite` 会把组件还原成 shadcn **官方默认版本**。如果项目对某个组件做过自定义（改样式、加 props），覆盖后这些改动会丢失。

- 覆盖前先 `git status` 确认哪些组件文件有未提交改动，按需 `git stash` 或备份。
- 覆盖后**必须** `npm run build`（vue-tsc + vite）验证业务代码与新组件 API 兼容——shadcn 偶尔会改组件的 props/emit 签名。

## 切换 style（风格迁移）

shadcn-vue 的 `style` 决定**组件代码本身**（不只是 CSS token）。`components.json` 里的 `style` 字段可选 `reka-vega`/`reka-nova`/`reka-maia`/`reka-lyra`/`reka-mira`/`reka-luma`/`reka-sera`。换风格 = 改 `components.json` 的 `style` + 覆盖重装全部组件。

> 关键认知：**不同 style 的 CSS 颜色 token 几乎一致**（都是 neutral 灰阶基色），真正差异在组件代码——圆角类名（`rounded-4xl` vs `rounded-md`）、focus ring 强度（`ring-ring/30` vs `/50`）、variant 细节、间距。以及 `--radius` 基准值不同。

### 标准切换流程（本项目已从 vega → luma）

1. 改 `components.json`：`"style": "reka-luma"`（目标风格名）。
2. **确认 node_modules 的 store 与 corepack 对齐**（见上方「根治方案」）。锁定 pnpm 10 后，node_modules 必须在 v10 store，否则 CLI 报 store 不匹配。
3. 覆盖重装全部组件：
   ```bash
   pnpm dlx shadcn-vue@latest add badge button card checkbox command dialog dropdown-menu input input-group popover scroll-area select table textarea --overwrite --yes
   ```
4. 手动改 `src/style.css` 的 `--radius` 为目标风格值（CLI 不会覆盖已存在的 `:root` 变量）。各风格值：vega `0.625rem`、luma `0.45rem`、其余见 shadcn 文档。
5. **保留业务自定义 token**：本项目的 `--background-alt`/`--surface-hover`/`--foreground-subtle`/`--font-mono`/`--shadow-*`/`--color-red` 等不在 shadcn 默认里，迁移时务必保留，不能被覆盖。
6. `npm run build` 验证。

### preset code 是什么

`shadcn-vue init --preset <code>` 里的 code（如 `a2vfHGK`）是一个**预设配置包**，解出来就是一组 `components.json` 参数（style/font/baseColor/menuColor 等）。它本身不含样式文件——风格由它指定的 `style` 决定，靠后续 `add` 组件注入 CSS。所以"看一个 preset 长什么样"= 生成项目后读它的 `components.json` 的 `style` 字段，再 add 几个组件看 CSS 注入结果。

## 当前已装组件（14 个）

`badge` `button` `card` `checkbox` `command` `dialog` `dropdown-menu` `input` `input-group` `popover` `scroll-area` `select` `table` `textarea`

新装组件可能带依赖组件（如 `command` 带 `input-group`、`dialog` 等），CLI 会一并写入。

## 不在 shadcn-vue 里、需手动补的

- **图标**：统一用 `@lucide/vue`（`<XxxIcon class="size-4" />`），组件已自带依赖。
- **reka-ui 原语**：`Popover`/`Select`/`Dialog` 等都封装自 reka-ui（已装）。若需 reka-ui 未封装的原语（如无对应 shadcn 组件），可直接 `import { PopoverRoot } from 'reka-ui'`。
- **cmdk-vue**：仅 `command` 组件依赖，装 `command` 时 CLI 会自动带上。
