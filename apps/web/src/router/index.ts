import { setupLayouts } from "virtual:generated-layouts";
import { createRouter, createWebHistory } from "vue-router";
import { handleHotUpdate, routes } from "vue-router/auto-routes";

export const router = createRouter({
  history: createWebHistory(),
  // @ts-ignore
  routes: setupLayouts(routes),
});

if (import.meta.hot) {
  handleHotUpdate(router);
}
