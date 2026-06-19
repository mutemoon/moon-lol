import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { router } from "./router";
import { getBackendClient } from "./services/backend";

const app = createApp(App);
app.use(createPinia());
app.use(router);

getBackendClient().then(() => {
  app.mount("#app");
});
