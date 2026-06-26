// Monaco 本地化加载与 Web Worker 环境。
//
// `monaco-editor` 默认通过 CDN 加载语言 Worker；在 Tauri 桌面端与离线场景下不可用，
// 这里改为用 Vite 的 `?worker` 把 Worker 打进本地产物，并把 `@guolao/vue-monaco-editor`
// 的 loader 指向本地安装的 monaco 实例，避免任何网络请求。
//
// 任意使用 Monaco 的组件在挂载前 `import "@/lib/monaco"` 即可（副作用模块）。
import { loader } from "@guolao/vue-monaco-editor";
import * as monaco from "monaco-editor";
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";

// Monaco 通过全局 `self.MonacoEnvironment.getWorker` 决定如何拉起语言服务 Worker。
self.MonacoEnvironment = {
  getWorker(_workerId: string, label: string) {
    if (label === "json") return new jsonWorker();
    if (label === "typescript" || label === "javascript") return new tsWorker();
    return new editorWorker();
  },
};

// 让 loader 使用本地 monaco，而非默认的 CDN 下载。
loader.config({ monaco });

export { monaco };
