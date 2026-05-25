<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";

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
  <div class="bg-bg-deep flex h-full flex-col gap-4 overflow-hidden p-6 font-sans">
    <!-- Header -->
    <div
      class="bg-bg-surface border-border-subtle relative flex shrink-0 items-center justify-between overflow-hidden rounded-lg border px-5 py-3 shadow-[0_4px_12px_rgba(0,0,0,0.3)]"
    >
      <!-- 金色发光边框效果 -->
      <div
        class="from-gold-muted via-gold-default to-gold-bright absolute top-0 left-0 h-[3px] w-full bg-gradient-to-r"
      ></div>

      <div class="flex items-center gap-3">
        <div
          class="bg-gold-dimmer/10 border-gold-dimmer/30 flex h-10 w-10 items-center justify-center rounded-lg border shadow-[0_0_15px_rgba(212,175,92,0.1)]"
        >
          <!-- 终端 Icon -->
          <svg class="text-gold-bright h-5.5 w-5.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
            />
          </svg>
        </div>
        <div>
          <h1 class="text-text-bright text-base font-bold tracking-wide">BashTool 命令行测试床</h1>
          <p class="text-text-muted text-[11px]">Command Mock Bench & Execution Sandbox</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <router-link to="/debug">
          <Button
            variant="outline"
            size="sm"
            class="border-border-subtle text-text-muted hover:text-text-bright hover:border-gold-muted h-8 cursor-pointer rounded px-3.5 text-xs transition-all duration-300"
          >
            返回调试页 (Back to Debug)
          </Button>
        </router-link>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="flex min-h-0 flex-1 gap-5 overflow-hidden">
      <!-- LEFT PANEL: Preset Library -->
      <div class="flex min-h-0 w-80 shrink-0 flex-col gap-4 overflow-y-auto">
        <!-- 核心安全策略卡片 -->
        <div class="bg-bg-surface border-border-subtle relative flex flex-col gap-2.5 rounded-lg border p-4 shadow-sm">
          <span
            class="text-text-muted border-border-subtle/30 border-b pb-1.5 text-[10px] font-bold tracking-wider uppercase"
          >
            ⚠️ 安全准则 (Security Policy)
          </span>
          <p class="text-text-bright text-[11px] leading-relaxed">
            此命令行沙盒直接对接后端的 <code class="text-gold-bright bg-black/40 font-mono px-1 py-0.5 rounded">BashTool</code>。由于安全策略限制，目前只允许执行以
            <code class="text-gold-bright font-bold">lol_cli</code> 开头的指令。
          </p>
        </div>

        <!-- 预设快速测试 -->
        <div class="bg-bg-surface border-border-subtle flex flex-1 flex-col gap-3 rounded-lg border p-4 shadow-sm">
          <span
            class="text-text-muted border-border-subtle/30 border-b pb-1.5 text-[10px] font-bold tracking-wider uppercase"
          >
            💡 预设测试指令 (Presets)
          </span>

          <div class="flex flex-1 flex-col gap-3 overflow-y-auto pr-1">
            <div
              v-for="preset in presets"
              :key="preset.name"
              class="border-border-subtle/30 bg-bg-deep/50 hover:border-gold-dimmer/40 hover:bg-gold-dimmer/5 group flex flex-col gap-1.5 rounded-md border p-3 transition-all duration-300"
            >
              <div class="flex items-center justify-between">
                <span class="text-text-bright text-[11px] font-semibold tracking-wide">{{ preset.name }}</span>
                <div class="flex items-center gap-1.5 opacity-60 transition-opacity group-hover:opacity-100">
                  <button
                    class="text-text-muted hover:text-gold-bright text-[10px] cursor-pointer font-medium"
                    title="填入输入框"
                    @click="fillCommand(preset.cmd)"
                  >
                    填入
                  </button>
                  <span class="text-text-muted/40 text-[9px]">•</span>
                  <button
                    class="text-gold-bright hover:text-gold-glow text-[10px] cursor-pointer font-semibold"
                    title="一键直接测试"
                    @click="executePreset(preset.cmd)"
                  >
                    执行
                  </button>
                </div>
              </div>
              <p class="text-text-muted text-[10px] leading-relaxed">{{ preset.description }}</p>
              <code
                class="bg-black/50 text-gold-bright/90 border-border-subtle/10 block overflow-x-auto rounded border px-2 py-1 text-[10px] font-normal leading-normal select-all font-mono"
              >
                {{ preset.cmd }}
              </code>
            </div>
          </div>
        </div>
      </div>

      <!-- RIGHT PANEL: Command Input & Terminal Console Output -->
      <div class="flex min-h-0 flex-1 flex-col gap-4">
        <!-- 1. Command Input Panel -->
        <div class="bg-bg-surface border-border-subtle flex flex-col gap-3 rounded-lg border p-4 shadow-sm">
          <span
            class="text-text-muted border-border-subtle/30 border-b pb-1.5 text-[10px] font-bold tracking-wider uppercase"
          >
            💻 命令控制台输入 (Console Input)
          </span>

          <div class="flex gap-3">
            <div class="bg-bg-deep border-border-subtle focus-within:border-gold-default flex flex-1 items-center rounded-md border px-3 transition-all">
              <span class="text-gold-dimmer font-mono text-xs mr-2 select-none">$</span>
              <input
                v-model="currentCmd"
                placeholder="在此输入 lol_cli 命令..."
                class="text-text-bright placeholder:text-text-muted/30 h-10 w-full bg-transparent font-mono text-xs focus:outline-none"
                :disabled="isExecuting"
                @keyup.enter="runCommand"
              />
            </div>

            <Button
              size="sm"
              class="text-gold-bright bg-bg-surface hover:text-gold-glow hover:shadow-glow-gold relative cursor-pointer overflow-hidden rounded-[4px] border border-transparent px-6 font-semibold transition-all active:scale-[0.98] h-10 shrink-0"
              :disabled="isExecuting || !currentCmd.trim()"
              @click="runCommand"
            >
              <!-- 边框金色渐变 -->
              <div
                class="pointer-events-none absolute inset-0 rounded-[4px] border border-transparent [mask-composite:exclude] [-webkit-mask-composite:xor] [background:linear-gradient(135deg,var(--color-gold-dimmer),var(--color-gold-default),var(--color-gold-bright))_border-box] [mask:linear-gradient(#fff_0_0)_content-box,linear-gradient(#fff_0_0)]"
              ></div>
              <span class="relative z-1 flex items-center gap-1.5 text-xs">
                <svg v-if="isExecuting" class="h-3 w-3 animate-spin text-gold-bright" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                {{ isExecuting ? "正在执行..." : "运行命令" }}
              </span>
            </Button>
          </div>
        </div>

        <!-- 2. Terminal Output Viewer -->
        <div
          class="bg-bg-surface border-border-subtle relative flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border shadow-lg"
        >
          <!-- Terminal Header (模仿黑客终端) -->
          <div
            class="border-border-subtle flex shrink-0 items-center justify-between border-b bg-[#0d0b0f] px-4 py-2.5"
          >
            <div class="flex items-center gap-2">
              <div class="flex items-center gap-1.5">
                <span class="h-2.5 w-2.5 rounded-full bg-red-500/80"></span>
                <span class="h-2.5 w-2.5 rounded-full bg-yellow-500/80"></span>
                <span class="h-2.5 w-2.5 rounded-full bg-green-500/80"></span>
              </div>
              <span class="text-text-bright font-mono text-[10px] ml-2 tracking-wider">
                bash_output_console.log
              </span>
            </div>

            <button
              v-if="historyLogs.length > 0"
              class="text-text-muted hover:text-text-bright text-[10px] cursor-pointer flex items-center gap-1 transition-colors"
              @click="clearConsole"
            >
              🧹 清空控制台
            </button>
          </div>

          <!-- Console Terminal Body -->
          <div class="min-h-0 flex-1 overflow-y-auto bg-[#070509] p-5 font-mono text-[11px] text-zinc-300">
            <!-- Loading Indicator -->
            <div v-if="isExecuting && historyLogs.length === 0" class="flex flex-col items-center justify-center py-16 gap-3">
              <svg class="h-6 w-6 animate-spin text-gold-bright" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              <span class="text-text-muted animate-pulse">正在向 Tauri 唤起 Bash 命令行测试，请稍候...</span>
            </div>

            <!-- Empty State -->
            <div
              v-else-if="historyLogs.length === 0"
              class="flex h-full flex-col items-center justify-center py-16 text-center"
            >
              <svg class="text-text-muted/20 h-10 w-10 mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="1.5"
                  d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z"
                />
              </svg>
              <h3 class="text-text-bright text-xs font-semibold mb-1">暂无控制台运行记录</h3>
              <p class="text-text-muted/60 max-w-[280px] text-[10px] leading-relaxed">
                在上方输入或在左侧选择一则预设命令，点击「运行命令」即可启动对 Bash 终端的连接测试。
              </p>
            </div>

            <!-- Execution Logs List -->
            <div v-else class="flex flex-col gap-4">
              <div
                v-for="log in historyLogs"
                :key="log.id"
                class="border-border-subtle/15 bg-black/40 overflow-hidden rounded-md border"
              >
                <!-- Log Header -->
                <div class="bg-black/60 border-border-subtle/10 flex items-center justify-between border-b px-3.5 py-2">
                  <div class="flex items-center gap-2">
                    <span class="text-text-muted text-[10px] select-none">[{{ log.timestamp }}]</span>
                    <span class="text-gold-bright font-bold select-all">$ {{ log.command }}</span>
                  </div>
                  
                  <Badge
                    variant="outline"
                    :class="[
                      log.status === 'success'
                        ? 'border-green/20 text-green bg-green/5'
                        : 'border-red/20 text-red bg-red/5',
                      'px-1.5 py-0 text-[8px] scale-90'
                    ]"
                  >
                    {{ log.status === "success" ? "SUCCESS" : "FAILED" }}
                  </Badge>
                </div>

                <!-- Log Output Panel -->
                <div class="overflow-x-auto p-3.5 bg-black/25">
                  <pre class="text-text-bright/95 text-[10.5px] leading-relaxed font-normal whitespace-pre-wrap select-all">{{ log.output }}</pre>
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
.bg-bg-deep {
  background-color: #070509;
}
.bg-bg-surface {
  background-color: #120e16;
}
.border-border-subtle {
  border-color: rgba(212, 175, 92, 0.12);
}
.text-text-bright {
  color: #f1ede4;
}
.text-text-muted {
  color: #a39e93;
}
.text-gold-bright {
  color: #e8c97a;
}
.text-gold-glow {
  color: #ffebbc;
  text-shadow: 0 0 10px rgba(232, 201, 122, 0.4);
}
.text-gold-dimmer {
  color: #d4af5c;
}
.text-green {
  color: #4ade80;
}
.text-red {
  color: #f87171;
}
</style>
