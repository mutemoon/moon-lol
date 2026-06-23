import { resolve } from "node:path";
import { fileURLToPath, URL } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";
import AutoImport from "unplugin-auto-import/vite";
import Components from "unplugin-vue-components/vite";
import Pages from "unplugin-vue-router/vite";
import Layouts from "vite-plugin-vue-layouts";

// @ts-ignore process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

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
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  optimizeDeps: {
    exclude: ["vue-element-plus-x"],
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
    proxy: {
      "/assets": {
        target: "http://localhost:1420",
        rewrite: (path) => {
          const res = path.replace(/^\/assets/, `/@fs/${resolve("../../assets").replaceAll("\\", "/")}`);
          return res;
        },
        bypass: (req, res) => {
          if (req.url?.includes(".meta") && res) {
            res.statusCode = 404;
            res.end();
            return false;
          }
        },
      },
    },
  },
});
