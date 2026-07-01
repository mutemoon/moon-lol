import type { RouteRecordRaw } from "vue-router";

import { setupLayouts } from "virtual:generated-layouts";
import { createRouter, createWebHistory, createWebHashHistory } from "vue-router";
import { handleHotUpdate, routes } from "vue-router/auto-routes";
import { isDesktop } from "@/lib/utils";

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
    to.path === "/launcher" ||
    to.path.startsWith("/debug") ||
    to.path === "/games" ||
    to.path === "/history" ||
    to.path === "/rl-training" ||
    to.path === "/logs-archive";
  if (isLocalOnly) {
    next("/rooms");
    return;
  }
  next();
});

if (import.meta.hot) {
  handleHotUpdate(router);
}
