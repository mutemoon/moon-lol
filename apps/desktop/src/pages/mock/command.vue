<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";
import { TerminalIcon, Trash2Icon, RefreshCwIcon, ChevronRightIcon } from "@lucide/vue";

// 定义预设指令
interface PresetCmd {
  name: string;
  description: string;
  cmd: string;
}

const presets: PresetCmd[] = [
  {
    name: "命令帮助",
    description: "查看 lol_cli 的完整命令行帮助说明",
    cmd: "lol_cli --help"
  },
  {
    name: "系统观测",
    description: "获取指定 ID 英雄（如 Order 阵营）的当前状态及周边小兵",
    cmd: "lol_cli observe -e 4294967185"
  },
  {
    name: "移动指令",
    description: "让英雄移动到双方小兵最初的碰撞交汇点 (2649, 12875)",
    cmd: "lol_cli action -e 4294967185 move 2649 12875"
  },
  {
    name: "普通攻击",
    description: "让英雄普通攻击指定的实体（例如附近敌方的近战兵/远程兵）",
    cmd: "lol_cli action -e 4294967185 attack 4294967186"
  },
  {
    name: "释放技能",
    description: "在坐标 (2750, 12320) 处对目标区域释放 Q 技能 (Index 0)",
    cmd: "lol_cli action -e 4294967185 skill 0 2750 12320"
  },
  {
    name: "紧急停止",
    description: "中止英雄正在进行的移动、普通攻击或施法状态，立于原地",
    cmd: "lol_cli action -e 4294967185 stop"
  }
];

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
    const isError = result.includes("错误:") || result.includes("指令执行失败") || result.includes("执行命令行时发生系统错误");
    
    historyLogs.value.unshift({
      id: Math.random().toString(36).substring(2, 9),
      timestamp,
      command: cmdStr,
      output: result,
      status: isError ? "error" : "success"
    });
  } catch (e: any) {
    const timestamp = new Date().toLocaleTimeString();
    const errorMsg = typeof e === "string" ? e : e.message || "未知执行错误";
    executionError.value = errorMsg;
    
    historyLogs.value.unshift({
      id: Math.random().toString(36).substring(2, 9),
      timestamp,
      command: cmdStr,
      output: `[系统异常] ${errorMsg}`,
      status: "error"
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
      <div
        class="from-primary/40 via-primary to-primary/80 absolute top-0 left-0 h-1 w-full bg-gradient-to-r"
      ></div>

      <div class="flex items-center gap-3">
        <div
          class="bg-primary/10 border-primary/30 flex h-10 w-10 items-center justify-center rounded-lg border"
        >
          <TerminalIcon class="text-primary size-5" />
        </div>
        <div>
          <h1 class="text-foreground text-base font-bold tracking-wide">Bash 命令行调试工具</h1>
          <p class="text-muted-foreground text-[11px]">Direct BashTool Sandbox & Command-line Bed</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <Badge variant="outline" class="border-primary/20 text-primary bg-primary/5 px-2.5 py-1">
          当前环境: sandboxed
        </Badge>
        <router-link to="/debug">
          <Button
            variant="outline"
            size="sm"
            class="border-border text-muted-foreground hover:text-foreground hover:border-primary/40 h-8 cursor-pointer rounded px-3 text-xs transition-all"
          >
            返回调试页
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
            <h2 class="text-foreground text-xs font-bold uppercase tracking-wider">内置预设指令</h2>
          </div>

          <ScrollArea class="flex-1 w-full">
            <div class="flex flex-col gap-2 p-3 pr-4">
              <div
                v-for="p in presets"
                :key="p.name"
                class="border-border bg-muted/10 hover:border-primary/40 hover:bg-primary/5 group flex flex-col gap-1.5 rounded-md border p-3 transition-colors cursor-pointer"
                @click="fillCommand(p.cmd)"
              >
                <div class="flex items-center justify-between">
                  <span class="text-foreground text-xs font-bold group-hover:text-primary transition-colors">
                    {{ p.name }}
                  </span>
                  <button
                    class="text-muted-foreground hover:text-primary text-[10px] cursor-pointer font-medium flex items-center"
                    @click.stop="executePreset(p.cmd)"
                  >
                    <span>直接运行</span>
                    <ChevronRightIcon class="size-3 ml-0.5" />
                  </button>
                </div>
                <p class="text-muted-foreground text-[10px] leading-relaxed">
                  {{ p.description }}
                </p>
                <code class="bg-muted/80 text-foreground border border-border block overflow-x-auto rounded px-2 py-1 text-[10px] font-normal font-mono select-all">
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
        <div class="bg-card border-border rounded-lg border p-4 shadow-sm flex flex-col gap-3">
          <label class="text-muted-foreground text-[10px] font-bold uppercase tracking-wide">自定义指令执行</label>
          <div class="flex items-center gap-3">
            <div class="bg-muted/40 border-border focus-within:border-primary flex flex-1 items-center rounded-md border px-3 transition-colors">
              <span class="text-muted-foreground font-mono text-xs select-none mr-1.5">$</span>
              <input
                v-model="currentCmd"
                type="text"
                placeholder="lol_cli observe -e <ENTITY_ID>"
                class="text-foreground font-mono placeholder:text-muted-foreground/30 h-8 flex-1 bg-transparent text-xs outline-none"
                @keyup.enter="runCommand"
              />
            </div>
            
            <Button
              class="relative h-8 px-4 cursor-pointer font-semibold rounded"
              :disabled="isExecuting || !currentCmd.trim()"
              @click="runCommand"
            >
              <span class="relative z-1 flex items-center gap-1.5 text-xs">
                <RefreshCwIcon v-if="isExecuting" class="h-3.5 w-3.5 animate-spin mr-1 text-primary-foreground" />
                <span>{{ isExecuting ? "正在执行..." : "运行命令" }}</span>
              </span>
            </Button>
          </div>
        </div>

        <!-- 2. Terminal Output Viewer -->
        <div
          class="bg-card border-border relative flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border shadow-lg"
        >
          <!-- Terminal Header -->
          <div
            class="border-border flex shrink-0 items-center justify-between border-b bg-muted/40 px-4 py-2.5"
          >
            <div class="flex items-center gap-2">
              <div class="flex items-center gap-1">
                <span class="h-2 w-2 rounded-full bg-red-500/80"></span>
                <span class="h-2 w-2 rounded-full bg-yellow-500/80"></span>
                <span class="h-2 w-2 rounded-full bg-green-500/80"></span>
              </div>
              <span class="text-foreground font-mono text-[10px] ml-2 tracking-wider">
                bash_output_console.log
              </span>
            </div>

            <button
              v-if="historyLogs.length > 0"
              class="text-muted-foreground hover:text-foreground text-[10px] cursor-pointer flex items-center gap-1 transition-colors"
              @click="clearConsole"
            >
              <Trash2Icon class="size-3.5" />
              <span>清空控制台</span>
            </button>
          </div>

          <!-- Console Terminal Body -->
          <div class="min-h-0 flex-1 overflow-y-auto bg-muted/10 p-5 font-mono text-[11px] text-foreground">
            <!-- Loading Indicator -->
            <div v-if="isExecuting && historyLogs.length === 0" class="flex flex-col items-center justify-center py-16 gap-3">
              <RefreshCwIcon class="h-6 w-6 animate-spin text-primary" />
              <span class="text-muted-foreground animate-pulse">正在向 Tauri 唤起 Bash 命令行测试，请稍候...</span>
            </div>

            <!-- Empty State -->
            <div
              v-else-if="historyLogs.length === 0"
              class="flex h-full flex-col items-center justify-center py-16 text-center"
            >
              <TerminalIcon class="text-muted-foreground/30 h-10 w-10 mb-3" />
              <h3 class="text-foreground text-xs font-semibold mb-1">暂无控制台运行记录</h3>
              <p class="text-muted-foreground/60 max-w-[280px] text-[10px] leading-relaxed">
                在上方输入或在左侧选择一则预设命令，点击「运行命令」即可启动对 Bash 终端的连接测试。
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
                        ? 'border-green-500/20 text-green-500 bg-green-500/5'
                        : 'border-destructive/20 text-destructive bg-destructive/5',
                      'px-1.5 py-0 text-[8px] scale-90'
                    ]"
                  >
                    {{ log.status === "success" ? "SUCCESS" : "FAILED" }}
                  </Badge>
                </div>

                <!-- Log Output Panel -->
                <div class="overflow-x-auto p-3.5 bg-muted/10">
                  <pre class="text-foreground/95 text-[10.5px] leading-relaxed font-normal whitespace-pre-wrap select-all">{{ log.output }}</pre>
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
