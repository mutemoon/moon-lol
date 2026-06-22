---
name: shadcn-component
description: 安装、覆盖重装 shadcn-vue 组件（apps/desktop 前端），及三个常见坑的排查。
---

> 本技能是 apps/desktop 前端踩坑后的标准做法。装新组件或重装时先读这里，避免重复踩雷。

# 标准命令

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

# 三个坑

### 坑 1：pnpm store 版本不匹配（已根治）

报错：`[ERR_PNPM_UNEXPECTED_STORE] ... currently linked from the store at v10 ... pnpm now wants to use the store at v11`

根因：corepack 用 pnpm 11，node_modules 链接到 pnpm 10 的 v10 store。
修法：见 `pnpm-corepack` 技能，把 corepack 锁定到 pnpm 10.33.2，两端 store 统一。

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

# 覆盖式重装的注意事项

`--overwrite` 会把组件还原成 shadcn **官方默认版本**。如果项目对某个组件做过自定义（改样式、加 props），覆盖后这些改动会丢失。

- 覆盖前先 `git status` 确认哪些组件文件有未提交改动，按需 `git stash` 或备份。
- 覆盖后**必须** `pnpm build`（vue-tsc + vite）验证业务代码与新组件 API 兼容——shadcn 偶尔会改组件的 props/emit 签名。

# 当前已装组件（14 个）

`badge` `button` `card` `checkbox` `command` `dialog` `dropdown-menu` `input` `input-group` `popover` `scroll-area` `select` `table` `textarea`

新装组件可能带依赖组件（如 `command` 带 `input-group`、`dialog` 等），CLI 会一并写入。

# 不在 shadcn-vue 里、需手动补的

- **图标**：统一用 `@lucide/vue`（`<XxxIcon class="size-4" />`），组件已自带依赖。
- **reka-ui 原语**：`Popover`/`Select`/`Dialog` 等都封装自 reka-ui（已装）。若需 reka-ui 未封装的原语（如无对应 shadcn 组件），可直接 `import { PopoverRoot } from 'reka-ui'`。
- **cmdk-vue**：仅 `command` 组件依赖，装 `command` 时 CLI 会自动带上。
