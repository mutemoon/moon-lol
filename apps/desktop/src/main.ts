import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { router } from "./router";
import { i18n } from "./i18n";
import { getBackendClient } from "./services/backend";

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.use(i18n);

getBackendClient().then(() => {
  app.mount("#app");
});
