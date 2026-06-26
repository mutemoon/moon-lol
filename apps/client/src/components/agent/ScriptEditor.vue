<script setup lang="ts">
// Script Agent 脚本编辑器：Monaco + 宿主 API 类型提示 + 模板库 + 编译/热重载状态 + 调试日志面板。
//
// 设计意图（产品文档 §3 Script Agent）：
//   - 用 Monaco 替换普通 Textarea，提供 JS 语法高亮与智能补全；
//   - 注入宿主 API 的 `.d.ts`，让 observe()/action()/log()/wait_ticks() 获得精确类型；
//   - 提供常用脚本模板一键填充；
//   - 实时展示编译状态（来自 Monaco 静态诊断）、上次热重载时间与报错日志；
//   - 提供独立日志流与断点/单步/变量观测的操控入口（运行时接入后启用）。
import { ref, computed, onMounted, onBeforeUnmount, shallowRef } from "vue";
import "@/lib/monaco";
import { VueMonacoEditor, type MonacoEditor } from "@guolao/vue-monaco-editor";
import type { editor } from "monaco-editor";
import { useLocale } from "@/composables/useLocale";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  SCRIPT_API_DTS,
  SCRIPT_TEMPLATES,
  DEFAULT_SCRIPT,
} from "@/services/scriptAgentTemplates";
import {
  RotateCwIcon,
  StepForwardIcon,
  CircleDotIcon,
  EyeIcon,
  CheckCircle2Icon,
  AlertTriangleIcon,
} from "@lucide/vue";

const props = defineProps<{
  modelValue: string;
  /** 运行时（ScriptDriver）是否已接入当前对局；未接入时断点/单步等仅作入口展示。 */
  runtimeAttached?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  /** 用户请求把当前脚本热重载到运行中的对局。 */
  (e: "reload", source: string): void;
  /** 单步执行 / 断点切换 等调试控制入口（运行时接入后处理）。 */
  (e: "step"): void;
  (e: "toggle-breakpoint", line: number): void;
}>();

const { t } = useLocale();

const editorRef = shallowRef<editor.IStandaloneCodeEditor | null>(null);
const markers = ref<editor.IMarker[]>([]);
const lastReloadAt = ref<number | null>(null);
const selectedTemplate = ref<string>("");

interface LogLine {
  at: number;
  level: "info" | "warn" | "error";
  text: string;
}
const logs = ref<LogLine[]>([]);
const logScroll = ref<HTMLElement | null>(null);

function pushLog(level: LogLine["level"], text: string) {
  logs.value.push({ at: Date.now(), level, text });
  if (logs.value.length > 200) logs.value.splice(0, logs.value.length - 200);
  requestAnimationFrame(() => {
    if (logScroll.value) logScroll.value.scrollTop = logScroll.value.scrollHeight;
  });
}

// 编译状态：直接取 Monaco 的静态诊断（JS 语法/类型错误）。
const errorCount = computed(
  () => markers.value.filter((m) => m.severity === 8 /* Error */).length,
);
const compileOk = computed(() => errorCount.value === 0);

const isDark = ref(true);
let themeObserver: MutationObserver | null = null;

function syncTheme() {
  isDark.value = document.documentElement.classList.contains("dark");
}

function handleBeforeMount(monaco: MonacoEditor) {
  // 注入宿主 API 类型库，并放宽 JS 默认诊断（脚本是运行期注入的全局函数）。
  const js = monaco.languages.typescript.javascriptDefaults;
  js.setCompilerOptions({
    target: monaco.languages.typescript.ScriptTarget.ES2020,
    allowNonTsExtensions: true,
    lib: ["es2020"],
    noLib: false,
  });
  js.addExtraLib(SCRIPT_API_DTS, "ts:moonlol-agent-api.d.ts");
}

function handleMount(ed: editor.IStandaloneCodeEditor) {
  editorRef.value = ed;
  pushLog("info", t("scriptEditor.logReady"));
}

function handleValidate(ms: editor.IMarker[]) {
  markers.value = ms;
  const errs = ms.filter((m) => m.severity === 8);
  if (errs.length) {
    pushLog("error", t("scriptEditor.logCompileError", { count: errs.length }));
  }
}

function handleChange(value: string | undefined) {
  emit("update:modelValue", value ?? "");
}

function applyTemplate(id: string) {
  const tpl = SCRIPT_TEMPLATES.find((x) => x.id === id);
  if (!tpl) return;
  const current = props.modelValue.trim();
  const isDefault = current === "" || current === DEFAULT_SCRIPT.trim();
  if (!isDefault && !window.confirm(t("scriptEditor.confirmOverwrite"))) {
    selectedTemplate.value = "";
    return;
  }
  emit("update:modelValue", tpl.code);
  pushLog("info", t("scriptEditor.logTemplateLoaded", { name: t("scriptEditor.templates." + tpl.labelKey) }));
  // 重置选择器，方便再次选择同一模板。
  selectedTemplate.value = "";
}

function requestReload() {
  if (!compileOk.value) {
    pushLog("warn", t("scriptEditor.logReloadBlocked"));
    return;
  }
  lastReloadAt.value = Date.now();
  emit("reload", props.modelValue);
  pushLog("info", t("scriptEditor.logReloaded"));
}

function clearLogs() {
  logs.value = [];
}

function fmtTime(ts: number): string {
  return new Date(ts).toLocaleTimeString();
}

const lastReloadLabel = computed(() =>
  lastReloadAt.value ? fmtTime(lastReloadAt.value) : t("scriptEditor.never"),
);

onMounted(() => {
  syncTheme();
  themeObserver = new MutationObserver(syncTheme);
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["class"],
  });
});

onBeforeUnmount(() => {
  themeObserver?.disconnect();
});
</script>

<template>
  <div class="flex flex-col gap-2">
    <!-- 工具栏：模板库 + 编译状态 + 热重载 -->
    <div class="flex flex-wrap items-center justify-between gap-2">
      <div class="flex items-center gap-2">
        <Select v-model="selectedTemplate" @update:model-value="(v) => applyTemplate(v as string)">
          <SelectTrigger class="h-8 w-44 text-xs" data-testid="script-template-select">
            <SelectValue :placeholder="t('scriptEditor.templatePlaceholder')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="tpl in SCRIPT_TEMPLATES" :key="tpl.id" :value="tpl.id">
              {{ t("scriptEditor.templates." + tpl.labelKey) }}
            </SelectItem>
          </SelectContent>
        </Select>

        <Badge
          :variant="compileOk ? 'secondary' : 'destructive'"
          class="gap-1 font-mono text-[10px]"
          data-testid="script-compile-status"
        >
          <CheckCircle2Icon v-if="compileOk" class="size-3" />
          <AlertTriangleIcon v-else class="size-3" />
          {{ compileOk ? t("scriptEditor.compileOk") : t("scriptEditor.compileError", { count: errorCount }) }}
        </Badge>
        <span class="text-muted-foreground text-[11px] tabular-nums">
          {{ t("scriptEditor.lastReload") }}: {{ lastReloadLabel }}
        </span>
      </div>

      <Button
        size="sm"
        variant="outline"
        class="h-8"
        :disabled="!compileOk"
        @click="requestReload"
        data-testid="script-reload-btn"
      >
        <RotateCwIcon class="size-3.5" />
        {{ t("scriptEditor.reloadBtn") }}
      </Button>
    </div>

    <!-- 编辑器 -->
    <div class="overflow-hidden rounded-md border" data-testid="script-monaco">
      <VueMonacoEditor
        :value="modelValue"
        language="javascript"
        :theme="isDark ? 'vs-dark' : 'vs'"
        height="320px"
        path="moonlol-agent-script.js"
        :options="{
          fontSize: 12,
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          tabSize: 2,
          automaticLayout: true,
          lineNumbers: 'on',
          fixedOverflowWidgets: true,
        }"
        @before-mount="handleBeforeMount"
        @mount="handleMount"
        @change="handleChange"
        @validate="handleValidate"
      />
    </div>

    <!-- 调试面板：日志流 + 操控入口 -->
    <div class="rounded-md border">
      <div class="flex items-center justify-between border-b px-3 py-1.5">
        <span class="text-[11px] font-medium tracking-wider uppercase">
          {{ t("scriptEditor.debugTitle") }}
        </span>
        <div class="flex items-center gap-1">
          <Button
            size="sm"
            variant="ghost"
            class="h-7 px-2 text-xs"
            :disabled="!runtimeAttached"
            :title="runtimeAttached ? '' : t('scriptEditor.needRuntime')"
            @click="emit('step')"
            data-testid="script-step-btn"
          >
            <StepForwardIcon class="size-3.5" />
            {{ t("scriptEditor.step") }}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            class="h-7 px-2 text-xs"
            :disabled="!runtimeAttached"
            :title="runtimeAttached ? '' : t('scriptEditor.needRuntime')"
            @click="emit('toggle-breakpoint', editorRef?.getPosition()?.lineNumber ?? 1)"
            data-testid="script-breakpoint-btn"
          >
            <CircleDotIcon class="size-3.5" />
            {{ t("scriptEditor.breakpoint") }}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            class="h-7 px-2 text-xs"
            :disabled="!runtimeAttached"
            :title="runtimeAttached ? '' : t('scriptEditor.needRuntime')"
            data-testid="script-watch-btn"
          >
            <EyeIcon class="size-3.5" />
            {{ t("scriptEditor.watch") }}
          </Button>
          <Button size="sm" variant="ghost" class="h-7 px-2 text-xs" @click="clearLogs">
            {{ t("scriptEditor.clearLogs") }}
          </Button>
        </div>
      </div>
      <div
        ref="logScroll"
        class="h-28 overflow-y-auto px-3 py-2 font-mono text-[11px] leading-relaxed"
        data-testid="script-log-stream"
      >
        <p v-if="logs.length === 0" class="text-muted-foreground">
          {{ t("scriptEditor.noLogs") }}
        </p>
        <div
          v-for="(l, i) in logs"
          :key="i"
          class="flex gap-2"
          :class="{
            'text-destructive': l.level === 'error',
            'text-amber-500': l.level === 'warn',
          }"
        >
          <span class="text-muted-foreground tabular-nums">{{ fmtTime(l.at) }}</span>
          <span>{{ l.text }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
