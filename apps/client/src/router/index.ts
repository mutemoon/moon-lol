import type { RouteRecordRaw } from "vue-router";

import { setupLayouts } from "virtual:generated-layouts";
import { createRouter, createWebHistory, createWebHashHistory } from "vue-router";
import { handleHotUpdate, routes } from "vue-router/auto-routes";

const isDesktop =
  typeof window !== "undefined" &&
  (window.IS_DESKTOP ?? ((window as any).__TAURI__ !== undefined || (window as any).__TAURI_INTERNALS__ !== undefined));

export const router = createRouter({
  history: isDesktop ? createWebHashHistory() : createWebHistory(),
  routes: setupLayouts(routes as RouteRecordRaw[]),
});

router.beforeEach((to, _from, next) => {
  if (isDesktop) {
    next();
    return;
  }
  // 本地对局 + 大日志调试相关仅桌面模式可见。
  // /rl-training 也仅桌面端可用（产品文档 §2.3）。
  const isLocalOnly =
    to.path === "/" ||
    to.path === "/debug" ||
    to.path === "/history" ||
    to.path === "/rl-training";
  if (isLocalOnly) {
    next("/rooms");
    return;
  }
  next();
});

if (import.meta.hot) {
  handleHotUpdate(router);
}
