---
name: client-dev-with-browser
description: Using browser automation (playwright-cli) to verify, debug, and review frontend client UI and functionality.
---

# 浏览器辅助前端开发技能

本技能指导如何利用浏览器自动化工具 `playwright-cli` 对 `apps/client` 模块进行开发、UI 验证、Bug 排查及设计确认。

## 1. 适用场景

- **UI 布局与设计验证**：对 Vue 组件、Shadcn UI 样式微调后，快速在真实浏览器中预览并截图确认。
- **端到端流程测试**：测试用户登录、注册、密码重置、房间创建、大厅刷新、槽位管理等复杂前后端交互流程。
- **用户界面设计反馈**：利用交互式标注工具（`playwright-cli show --annotate`）展示新界面，获取用户的修改建议与确认。
- **自动化 Bug 重现**：编写并运行简易的浏览器操作链，复现特定的前端异常或控制台报错。

## 2. 本地开发与服务端口

在开始验证前，请先检查前端和后端服务的对应端口或进程是否已在运行。**如果相关端口已启动/已被占用，请勿重复执行启动命令。**

- **前端开发服务器 (Vite)**
  - 检查端口：`1420`
  - 启动命令（若未启动）：在 `apps/client` 下执行 `pnpm run dev`
  - 本地地址：`http://localhost:1420`
- **后端 Web 服务 (Axum)**
  - 检查端口：`8000`
  - 启动命令（若未启动）：执行 `pnpm run start:server`
  - 本地地址：`http://localhost:8000`

> [!NOTE]
> 前端项目在开发环境中会自动尝试连接 `VITE_BASE_URL` 所指的后端地址。如果遇到 CORS 跨域问题，请确保后端 Axum 的 CORS 配置（如 `main.rs`）中已允许 `http://localhost:1420` 来源。

## 3. 使用 playwright-cli 进行浏览器自动化

本项目的浏览器自动化测试、UI 截图及交互，统一使用 `playwright-cli` 工具完成。
关于 `playwright-cli` 的详细命令（如 `open`, `goto`, `click`, `fill`, `snapshot`, `requests`, `show --annotate` 等）和用法规范，**请直接加载并阅读 playwright-cli 技能**。

## 4. Web 与 Desktop 模式的路由差异

由于 `apps/client` 是一套代码兼容 Tauri 桌面版和 Web 网页版，部分路由在纯浏览器环境下会有保护拦截（参见 `apps/client/src/router/index.ts` 中的导航守卫）：

- **桌面独占视图**：`/`（本地对局启动器）、`/debug`（日志调试终端）、`/history`（本地对局历史）、`/rl-training`（强化学习训练）。
- **浏览器自动重定向**：如果使用浏览器打开上述桌面独占路由，路由守卫会自动将其重定向至 `/rooms`（房间大厅）。
- **测试焦点**：在浏览器模式下，应重点测试 **`/rooms` (房间)**、**`/rank` (Rank竞技)**、**`/leaderboard` (排行榜)**、**`/community` (社区)**、**`/billing` (精粹与订阅)** 等在线云端服务功能。
