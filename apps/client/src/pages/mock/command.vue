<route lang="yaml">
meta:
  layout: desktop
</route>

<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useLocale } from "@/composables/useLocale";
import { TerminalIcon, Trash2Icon, RefreshCwIcon, ChevronRightIcon } from "@lucide/vue";

const { t } = useLocale();

// 定义预设指令
interface PresetCmd {
  key: string;
  cmd: string;
}

const presets = computed<PresetCmd[]>(() => [
  {
    key: "help",
    cmd: "lol_cli --help",
  },
  {
    key: "observe",
    cmd: "lol_cli observe -e 4294967185",
  },
  {
    key: "move",
    cmd: "lol_cli action -e 4294967185 move 2649 12875",
  },
  {
    key: "attack",
    cmd: "lol_cli action -e 4294967185 attack 4294967186",
  },
  {
    key: "skill",
    cmd: "lol_cli action -e 4294967185 skill 0 2750 12320",
  },
  {
    key: "stop",
    cmd: "lol_cli action -e 4294967185 stop",
  },
]);

// 输入与执行状态
const currentCmd = ref("lol_cli observe -e 4294967185");
const isExecuting = ref(false);
const executionError = ref("");

// 终端历史记录
interface LogEntry {
  id: string;
  timestamp: string;
  command: string;
  output: string;
  status: "success" | "error";
}

const historyLogs = ref<LogEntry[]>([]);

// 填入命令
function fillCommand(cmd: string) {
  currentCmd.value = cmd;
}

// 快速一键执行预设命令
function executePreset(cmd: string) {
  currentCmd.value = cmd;
  runCommand();
}

// 执行命令的主入口
async function runCommand() {
  const cmdStr = currentCmd.value.trim();
  if (!cmdStr) return;

  isExecuting.value = true;
  executionError.value = "";

  try {
    const timestamp = new Date().toLocaleTimeString();
    const result = await invoke<string>("run_bash_tool", { cmd: cmdStr });

    // 如果返回的内容包含"错误:"或"指令执行失败"
    const isError =
      result.includes("错误:") || result.includes("指令执行失败") || result.includes("执行命令行时发生系统错误");

    historyLogs.value.unshift({
      id: Math.random().toString(36).substring(2, 9),
      timestamp,
      command: cmdStr,
      output: result,
      status: isError ? "error" : "success",
    });
  } catch (e: any) {
    const timestamp = new Date().toLocaleTimeString();
    const errorMsg = typeof e === "string" ? e : e.message || t("settings.model.unknownError");
    executionError.value = errorMsg;

    historyLogs.value.unshift({
      id: Math.random().toString(36).substring(2, 9),
      timestamp,
      command: cmdStr,
      output: `[${t("mock.command.systemException")}] ${errorMsg}`,
      status: "error",
    });
  } finally {
    isExecuting.value = false;
  }
}

// 清空控制台
function clearConsole() {
  historyLogs.value = [];
}
</script>

<template>
  <div class="bg-background flex h-full flex-col gap-4 overflow-hidden p-6 font-sans">
    <!-- Header -->
    <div
      class="bg-card border-border relative flex shrink-0 items-center justify-between overflow-hidden rounded-lg border px-5 py-3 shadow-sm"
    >
      <!-- Gradient Border Line -->
      <div class="from-primary/40 via-primary to-primary/80 absolute top-0 left-0 h-1 w-full bg-gradient-to-r"></div>

      <div class="flex items-center gap-3">
        <div class="bg-primary/10 border-primary/30 flex h-10 w-10 items-center justify-center rounded-lg border">
          <TerminalIcon class="text-primary size-5" />
        </div>
        <div>
          <h1 class="text-foreground text-base font-bold tracking-wide">{{ t("mock.command.title") }}</h1>
          <p class="text-muted-foreground text-[11px]">{{ t("mock.command.subtitle") }}</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <Badge variant="outline" class="border-primary/20 text-primary bg-primary/5 px-2.5 py-1">
          {{ t("mock.command.currentEnv") }}: sandboxed
        </Badge>
        <router-link to="/debug">
          <Button
            variant="outline"
            size="sm"
            class="border-border text-muted-foreground hover:text-foreground hover:border-primary/40 h-8 cursor-pointer rounded px-3 text-xs transition-all"
          >
            {{ t("mock.command.backBtn") }}
          </Button>
        </router-link>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="flex min-h-0 flex-1 gap-5 overflow-hidden">
      <!-- LEFT PANEL: Preset Commands list -->
      <div class="flex min-h-0 w-80 flex-col gap-4">
        <div class="bg-card border-border flex flex-1 flex-col overflow-hidden rounded-lg border shadow-sm">
          <div class="border-border border-b px-4 py-3.5">
            <h2 class="text-foreground text-xs font-bold tracking-wider uppercase">
              {{ t("mock.command.presetTitle") }}
            </h2>
          </div>

          <ScrollArea class="w-full flex-1">
            <div class="flex flex-col gap-2 p-3 pr-4">
              <div
                v-for="p in presets"
                :key="p.key"
                class="border-border bg-muted/10 hover:border-primary/40 hover:bg-primary/5 group flex cursor-pointer flex-col gap-1.5 rounded-md border p-3 transition-colors"
                @click="fillCommand(p.cmd)"
              >
                <div class="flex items-center justify-between">
                  <span class="text-foreground group-hover:text-primary text-xs font-bold transition-colors">
                    {{ t("mock.command.presets." + p.key + ".name") }}
                  </span>
                  <button
                    class="text-muted-foreground hover:text-primary flex cursor-pointer items-center text-[10px] font-medium"
                    @click.stop="executePreset(p.cmd)"
                  >
                    <span>{{ t("mock.command.runBtn") }}</span>
                    <ChevronRightIcon class="ml-0.5 size-3" />
                  </button>
                </div>
                <p class="text-muted-foreground text-[10px] leading-relaxed">
                  {{ t("mock.command.presets." + p.key + ".description") }}
                </p>
                <code
                  class="bg-muted/80 text-foreground border-border block overflow-x-auto rounded border px-2 py-1 font-mono text-[10px] font-normal select-all"
                >
                  {{ p.cmd }}
                </code>
              </div>
            </div>
          </ScrollArea>
        </div>
      </div>

      <!-- RIGHT PANEL: Command Box & Terminal Console -->
      <div class="flex min-h-0 flex-1 flex-col gap-4">
        <!-- 1. Custom Command input card -->
        <div class="bg-card border-border flex flex-col gap-3 rounded-lg border p-4 shadow-sm">
          <label class="text-muted-foreground text-[10px] font-bold tracking-wide uppercase">
            {{ t("mock.command.customTitle") }}
          </label>
          <div class="flex items-center gap-3">
            <div
              class="bg-muted/40 border-border focus-within:border-primary flex flex-1 items-center rounded-md border px-3 transition-colors"
            >
              <span class="text-muted-foreground mr-1.5 font-mono text-xs select-none">$</span>
              <input
                v-model="currentCmd"
                type="text"
                placeholder="lol_cli observe -e <ENTITY_ID>"
                class="text-foreground placeholder:text-muted-foreground/30 h-8 flex-1 bg-transparent font-mono text-xs outline-none"
                @keyup.enter="runCommand"
              />
            </div>

            <Button
              class="relative h-8 cursor-pointer rounded px-4 font-semibold"
              :disabled="isExecuting || !currentCmd.trim()"
              @click="runCommand"
            >
              <span class="relative z-1 flex items-center gap-1.5 text-xs">
                <RefreshCwIcon v-if="isExecuting" class="text-primary-foreground mr-1 h-3.5 w-3.5 animate-spin" />
                <span>{{ isExecuting ? t("mock.command.executing") : t("mock.command.runCommandBtn") }}</span>
              </span>
            </Button>
          </div>
        </div>

        <!-- 2. Terminal Output Viewer -->
        <div
          class="bg-card border-border relative flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border shadow-lg"
        >
          <!-- Terminal Header -->
          <div class="border-border bg-muted/40 flex shrink-0 items-center justify-between border-b px-4 py-2.5">
            <div class="flex items-center gap-2">
              <div class="flex items-center gap-1">
                <span class="h-2 w-2 rounded-full bg-red-500/80"></span>
                <span class="h-2 w-2 rounded-full bg-yellow-500/80"></span>
                <span class="h-2 w-2 rounded-full bg-green-500/80"></span>
              </div>
              <span class="text-foreground ml-2 font-mono text-[10px] tracking-wider">bash_output_console.log</span>
            </div>

            <button
              v-if="historyLogs.length > 0"
              class="text-muted-foreground hover:text-foreground flex cursor-pointer items-center gap-1 text-[10px] transition-colors"
              @click="clearConsole"
            >
              <Trash2Icon class="size-3.5" />
              <span>{{ t("mock.command.clearConsoleBtn") }}</span>
            </button>
          </div>

          <!-- Console Terminal Body -->
          <div class="bg-muted/10 text-foreground min-h-0 flex-1 overflow-y-auto p-5 font-mono text-[11px]">
            <!-- Loading Indicator -->
            <div
              v-if="isExecuting && historyLogs.length === 0"
              class="flex flex-col items-center justify-center gap-3 py-16"
            >
              <RefreshCwIcon class="text-primary h-6 w-6 animate-spin" />
              <span class="text-muted-foreground animate-pulse">{{ t("mock.command.executingHint") }}</span>
            </div>

            <!-- Empty State -->
            <div
              v-else-if="historyLogs.length === 0"
              class="flex h-full flex-col items-center justify-center py-16 text-center"
            >
              <TerminalIcon class="text-muted-foreground/30 mb-3 h-10 w-10" />
              <h3 class="text-foreground mb-1 text-xs font-semibold">{{ t("mock.command.noRecord") }}</h3>
              <p class="text-muted-foreground/60 max-w-[280px] text-[10px] leading-relaxed">
                {{ t("mock.command.runHint") }}
              </p>
            </div>

            <!-- Execution Logs List -->
            <div v-else class="flex flex-col gap-4">
              <div
                v-for="log in historyLogs"
                :key="log.id"
                class="border-border bg-card/60 overflow-hidden rounded-md border"
              >
                <!-- Log Header -->
                <div class="bg-muted/40 border-border flex items-center justify-between border-b px-3.5 py-2">
                  <div class="flex items-center gap-2">
                    <span class="text-muted-foreground text-[10px] select-none">[{{ log.timestamp }}]</span>
                    <span class="text-primary font-bold select-all">$ {{ log.command }}</span>
                  </div>

                  <Badge
                    variant="outline"
                    :class="[
                      log.status === 'success'
                        ? 'border-green-500/20 bg-green-500/5 text-green-500'
                        : 'border-destructive/20 text-destructive bg-destructive/5',
                      'scale-90 px-1.5 py-0 text-[8px]',
                    ]"
                  >
                    {{ log.status === "success" ? "SUCCESS" : "FAILED" }}
                  </Badge>
                </div>

                <!-- Log Output Panel -->
                <div class="bg-muted/10 overflow-x-auto p-3.5">
                  <pre
                    class="text-foreground/95 text-[10.5px] leading-relaxed font-normal whitespace-pre-wrap select-all"
                    >{{ log.output }}</pre
                  >
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Scrollbar styles inside command view */
::-webkit-scrollbar {
  width: 4px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--muted-foreground);
}
</style>
