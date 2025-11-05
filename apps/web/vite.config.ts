import path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";
import AutoImport from "unplugin-auto-import/vite";
import Components from "unplugin-vue-components/vite";
import Pages from "unplugin-vue-router/vite";
import Layouts from "vite-plugin-vue-layouts";

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
    AutoImport({
      imports: ["vue", "vue-router"],
      dirs: ["./src/composables"],
      dts: "./src/types/auto-imports.d.ts",
      vueTemplate: true,
    }),
    Components({
      dts: "./src/types/components.d.ts",
    }),
    Pages({
      dts: "./src/types/typed-router.d.ts",
    }),
    Layouts(),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
