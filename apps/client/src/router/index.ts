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

if (import.meta.hot) {
  handleHotUpdate(router);
}
