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

