import "./init";
import { createApp } from "vue";
import { createPinia } from "pinia";
import Persist from "pinia-plugin-persistedstate";
import App from "./App.vue";
import { router } from "./router";
import { i18n } from "./i18n";
import { initServices } from "./services/provider";
import { useAuthStore } from "./stores/authStore";

const app = createApp(App);

const pinia = createPinia();
pinia.use(Persist);

app.use(pinia);
app.use(router);
app.use(i18n);

initServices().then(async () => {
  const authStore = useAuthStore();
  await authStore.init();
  app.mount("#app");
});
