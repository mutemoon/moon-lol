---
name: shadcn-style
description: 切换 apps/desktop 的 shadcn-vue 风格（reka-vega/nova/maia/lyra/mira/luma/sera 互转）。
---

# 关键认知

shadcn-vue 的 `style` 决定**组件代码本身**（不只是 CSS token）。`components.json` 里的 `style` 字段可选 `reka-vega`/`reka-nova`/`reka-maia`/`reka-lyra`/`reka-mira`/`reka-luma`/`reka-sera`。换风格 = 改 `components.json` 的 `style` + 覆盖重装全部组件。

> **不同 style 的 CSS 颜色 token 几乎一致**（都是 neutral 灰阶基色），真正差异在组件代码——圆角类名（`rounded-4xl` vs `rounded-md`）、focus ring 强度（`ring-ring/30` vs `/50`）、variant 细节、间距。以及 `--radius` 基准值不同。

本项目当前风格：`reka-luma`（从 `reka-vega` 迁移而来）。

# 标准切换流程

1. 改 `components.json`：`"style": "reka-luma"`（目标风格名）。
2. **确认 node_modules 的 store 与 corepack 对齐**（见 `pnpm-corepack` 技能）。锁定 pnpm 10 后，node_modules 必须在 v10 store，否则 CLI 报 store 不匹配。
3. 覆盖重装全部组件：
   ```bash
   pnpm dlx shadcn-vue@latest add badge button card checkbox command dialog dropdown-menu input input-group popover scroll-area select table textarea --overwrite --yes
   ```
4. 手动改 `apps/desktop/src/style.css` 的 `--radius` 为目标风格值（CLI 不会覆盖已存在的 `:root` 变量）。各风格值：vega `0.625rem`、luma `0.45rem`、其余见 shadcn 文档。
   - ⚠️ **本项目当前实际 `--radius` 是 `0.3rem`**，与 luma 文档标准值 `0.45rem` 不符。切换风格时需自行决定取文档标准值还是保留当前自定义值。
5. **保留业务自定义 token**：本项目的 `--background-alt`/`--surface-hover`/`--foreground-subtle`/`--font-mono`/`--shadow-*`/`--color-red` 等不在 shadcn 默认里，迁移时务必保留，不能被覆盖。
6. `pnpm build` 验证。

# preset code 是什么

`shadcn-vue init --preset <code>` 里的 code（如 `a2vfHGK`）是一个**预设配置包**，解出来就是一组 `components.json` 参数（style/font/baseColor/menuColor 等）。它本身不含样式文件——风格由它指定的 `style` 决定，靠后续 `add` 组件注入 CSS。所以"看一个 preset 长什么样"= 生成项目后读它的 `components.json` 的 `style` 字段，再 add 几个组件看 CSS 注入结果。
