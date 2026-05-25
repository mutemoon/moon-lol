<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { storeToRefs } from "pinia";
import { useGameStore } from "../stores/gameStore";
import { Button } from "../components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../components/ui/select";
import { Checkbox } from "../components/ui/checkbox";
import { Input } from "../components/ui/input";
import { Textarea } from "../components/ui/textarea";
import { Card, CardHeader, CardTitle, CardContent } from "../components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "../components/ui/dialog";
import { ScrollArea } from "../components/ui/scroll-area";
import { Badge } from "../components/ui/badge";
import { invoke } from "@tauri-apps/api/core";

const store = useGameStore();
const { mode, isStarting, launchError: error, champions } = storeToRefs(store);
const { ws } = store;
const { startGame, stopGame } = store;

// --- AI Agent Mode Configuration States ---
interface AgentConfigItem {
  champion: string;
  prompt: string;
  spawnPoint: [number, number];
}

const blueAgents = ref<AgentConfigItem[]>(
  (() => {
    const saved = localStorage.getItem("moon_lol_last_blue_agents");
    if (!saved) {
      return [
        {
          champion: "Riven",
          prompt: "你是一个专业的Riven玩家，在对线期专注于精细走位、合理补兵，在取得装备优势后寻找破绽斩杀对手。",
          spawnPoint: [1981.0, 11441.0],
        },
      ];
    }
    try {
      return JSON.parse(saved);
    } catch {
      return [
        {
          champion: "Riven",
          prompt: "你是一个专业的Riven玩家，在对线期专注于精细走位、合理补兵，在取得装备优势后寻找破绽斩杀对手。",
          spawnPoint: [1981.0, 11441.0],
        },
      ];
    }
  })(),
);

const redAgents = ref<AgentConfigItem[]>(
  (() => {
    const saved = localStorage.getItem("moon_lol_last_red_agents");
    if (!saved) {
      return [
        {
          champion: "Fiora",
          prompt: "你是一个极具压迫感的Fiora玩家，观察对手弱点进行侧翼打击，利用Q和E的高机动性及真实伤害消耗斩杀对手。",
          spawnPoint: [3318.0, 12875.0],
        },
      ];
    }
    try {
      return JSON.parse(saved);
    } catch {
      return [
        {
          champion: "Fiora",
          prompt: "你是一个极具压迫感的Fiora玩家，观察对手弱点进行侧翼打击，利用Q和E的高机动性及真实伤害消耗斩杀对手。",
          spawnPoint: [3318.0, 12875.0],
        },
      ];
    }
  })(),
);

// Scenario Management States
const currentSceneName = ref(localStorage.getItem("moon_lol_last_scene_name") || "default_scenario");
const scenariosList = ref<string[]>([]);
const selectedScenario = ref(localStorage.getItem("moon_lol_last_selected_scenario") || "");

watch(
  [currentSceneName, blueAgents, redAgents, selectedScenario],
  () => {
    localStorage.setItem("moon_lol_last_scene_name", currentSceneName.value);
    localStorage.setItem("moon_lol_last_blue_agents", JSON.stringify(blueAgents.value));
    localStorage.setItem("moon_lol_last_red_agents", JSON.stringify(redAgents.value));
    localStorage.setItem("moon_lol_last_selected_scenario", selectedScenario.value);
  },
  { deep: true },
);

// Map Coordinates Modal
const mapTargetAgent = ref<AgentConfigItem | null>(null);
const isMapModalOpen = ref(false);

const agentMarkerX = computed(() => {
  if (!mapTargetAgent.value) return 0;
  return (mapTargetAgent.value.spawnPoint[0] / 15000) * 500;
});

const agentMarkerY = computed(() => {
  if (!mapTargetAgent.value) return 0;
  return (1 - mapTargetAgent.value.spawnPoint[1] / 15000) * 500;
});

// Load saved historical scenarios list
async function loadScenariosList() {
  try {
    scenariosList.value = await invoke<string[]>("list_custom_scenarios");
  } catch (e) {
    console.error("加载历史配置场景列表失败", e);
  }
}

// Load a specific historical scenario into config card
async function handleLoadScenario(name: string) {
  if (!name) return;
  try {
    const agents = await invoke<any[]>("load_custom_scenario", { sceneName: name });

    // Split loaded agents by team
    blueAgents.value = agents
      .filter((a) => a.team === "Order")
      .map((a) => ({
        champion: a.champion,
        prompt: a.prompt,
        spawnPoint: a.spawn_point,
      }));

    redAgents.value = agents
      .filter((a) => a.team === "Chaos")
      .map((a) => ({
        champion: a.champion,
        prompt: a.prompt,
        spawnPoint: a.spawn_point,
      }));

    currentSceneName.value = name;
    selectedScenario.value = name;
    error.value = "";
  } catch (e: any) {
    error.value = e.message || typeof e === "string" ? e : "无法加载指定的配置";
  }
}

// Reset configuration editor to start a new scenario
function handleNewScenario() {
  currentSceneName.value = "new_scenario";
  selectedScenario.value = "";
  blueAgents.value = [
    {
      champion: "Riven",
      prompt: "你是一个专业的Riven玩家，在对线期专注于精细走位、合理补兵，在取得装备优势后寻找破绽斩杀对手。",
      spawnPoint: [1981.0, 11441.0],
    },
  ];
  redAgents.value = [
    {
      champion: "Fiora",
      prompt: "你是一个极具压迫感的Fiora玩家，观察对手弱点进行侧翼打击，利用Q和E的高机动性及真实伤害消耗斩杀对手。",
      spawnPoint: [3318.0, 12875.0],
    },
  ];
  error.value = "";
}

// Standalone save configuration (Without starting the game)
async function handleSaveConfig() {
  error.value = "";
  const name = currentSceneName.value.trim();
  if (!name) {
    error.value = "请输入要保存的配置命名！";
    return;
  }

  const agents = [
    ...blueAgents.value.map((a) => ({
      champion: a.champion,
      team: "Order",
      prompt: a.prompt,
      spawn_point: a.spawnPoint,
    })),
    ...redAgents.value.map((a) => ({
      champion: a.champion,
      team: "Chaos",
      prompt: a.prompt,
      spawn_point: a.spawnPoint,
    })),
  ];

  try {
    await invoke("save_custom_scenario", { sceneName: name, agents });
    await loadScenariosList();
    selectedScenario.value = name;
    error.value = `配置 "${name}" 保存成功！`;
  } catch (e: any) {
    error.value = e.message || typeof e === "string" ? e : "保存配置失败";
  }
}

// Delete configuration
async function handleDeleteScenario(sceneName: string) {
  error.value = "";
  try {
    await invoke("delete_custom_scenario", { sceneName });
    await loadScenariosList();
    if (selectedScenario.value === sceneName) {
      handleNewScenario();
    }
    error.value = `配置 "${sceneName}" 已成功删除！`;
  } catch (e: any) {
    error.value = e.message || typeof e === "string" ? e : "删除配置失败";
  }
}

onMounted(() => {
  loadScenariosList();
});

function openMapModal(agent: AgentConfigItem) {
  mapTargetAgent.value = agent;
  isMapModalOpen.value = true;
}

function closeMapModal() {
  isMapModalOpen.value = false;
  mapTargetAgent.value = null;
}

function handleMapClick(event: MouseEvent) {
  const rect = (event.currentTarget as SVGElement).getBoundingClientRect();
  const clickX = (event.clientX - rect.left) / rect.width;
  const clickY = (event.clientY - rect.top) / rect.height;
  if (mapTargetAgent.value) {
    mapTargetAgent.value.spawnPoint = [Math.round(clickX * 15000), Math.round((1 - clickY) * 15000)];
  }
}

function addBlueAgent() {
  blueAgents.value.push({
    champion: "Riven",
    prompt: "你是一个专业的Riven玩家，在对线期专注于精细走位、合理补兵，在取得装备优势后寻找破绽斩杀对手。",
    spawnPoint: [1500.0, 2000.0],
  });
}

function deleteBlueAgent(index: number) {
  blueAgents.value.splice(index, 1);
}

function addRedAgent() {
  redAgents.value.push({
    champion: "Fiora",
    prompt: "你是一个极具压迫感的Fiora玩家，观察对手弱点进行侧翼打击，利用Q和E的高机动性及真实伤害消耗斩杀对手。",
    spawnPoint: [13500.0, 13000.0],
  });
}

function deleteRedAgent(index: number) {
  redAgents.value.splice(index, 1);
}

// Custom Launch handler for multi-agent game
async function handleLaunchAgentMode() {
  isStarting.value = true;
  error.value = "";

  const name = currentSceneName.value.trim() || `custom_agents_${Date.now()}`;
  const agents = [
    ...blueAgents.value.map((a) => ({
      champion: a.champion,
      team: "Order",
      prompt: a.prompt,
      spawn_point: a.spawnPoint,
    })),
    ...redAgents.value.map((a) => ({
      champion: a.champion,
      team: "Chaos",
      prompt: a.prompt,
      spawn_point: a.spawnPoint,
    })),
  ];

  if (agents.length === 0) {
    error.value = "请至少添加一个 AI 代理！";
    isStarting.value = false;
    return;
  }

  try {
    // 1. Save custom JSON config and RON scene config
    await invoke("save_custom_scenario", { sceneName: name, agents });

    // 2. Launch game with custom scene name
    await startGame(name);
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message || "无法拉起自定义 AI 对战";
    isStarting.value = false;
  }
}
</script>

<template>
  <div class="flex h-full w-full flex-col px-6 py-6">
    <div class="mx-auto flex w-full max-w-6xl flex-1 flex-col gap-6 transition-all duration-300">
      <!-- Connect overlay (Shows globally when Tauri is spawning connection) -->
      <div
        v-if="ws.connecting"
        class="fixed inset-0 z-50 flex flex-col items-center justify-center gap-4 bg-[#09070a]/90 backdrop-blur-md"
      >
        <!-- Spinner -->
        <div class="border-border-subtle border-t-gold-default h-10 w-10 animate-spin rounded-full border-2"></div>
        <p class="text-gold-bright text-sm font-medium tracking-wide">Connecting to Bevy game session...</p>
        <p class="text-text-muted max-w-xs text-center text-xs" :class="{ 'text-gold-dimmer': ws.connectTimeout }">
          {{
            ws.connectTimeout
              ? "Still connecting… game process may be compiling or loading resources."
              : "Establishing WebSocket pipeline..."
          }}
        </p>
        <Button
          variant="outline"
          size="sm"
          class="text-text-muted border-border-subtle hover:text-text-default hover:border-gold-muted mt-2 h-9 cursor-pointer bg-transparent px-8 text-xs transition-all"
          @click="stopGame"
        >
          Cancel & Terminate
        </Button>
      </div>

      <!-- Elite Multi-Agent Split Layout Dashboard -->
      <div class="grid w-full flex-1 grid-cols-1 items-start gap-6 md:grid-cols-4">
        <!-- 左侧：已保存的配置列表 Sidebar (col-span-1) -->
        <Card
          class="bg-bg-surface border-border-subtle flex min-h-[480px] flex-col gap-4 overflow-hidden rounded-[0.625rem] p-0 py-0 shadow-lg ring-0! md:col-span-1"
          style="background: linear-gradient(180deg, rgba(17, 14, 20, 0.6) 0%, rgba(26, 21, 30, 0.8) 100%)"
        >
          <CardHeader
            class="border-border-subtle flex flex-row items-center justify-between gap-0 space-y-0 border-b p-4 pb-2"
          >
            <CardTitle class="text-gold-bright text-xs font-bold tracking-wider uppercase">已存配置</CardTitle>
            <Button
              variant="ghost"
              size="xs"
              class="text-gold-dimmer hover:text-gold-bright hover:bg-gold-dimmer/10 h-6 cursor-pointer px-2 text-[10px]"
              @click="handleNewScenario"
            >
              + 新建
            </Button>
          </CardHeader>

          <!-- 列表内容 -->
          <CardContent class="flex flex-1 flex-col overflow-hidden p-4 pt-0">
            <ScrollArea class="h-[400px] w-full">
              <div class="flex flex-col gap-1.5 pr-3">
                <div v-if="scenariosList.length === 0" class="text-text-muted py-6 text-center text-xs italic">
                  暂无已存的配置
                </div>
                <button
                  v-else
                  v-for="s in scenariosList"
                  :key="s"
                  class="group/item flex w-full cursor-pointer items-center justify-between rounded-md border px-3 py-2.5 text-left font-mono text-xs transition-all duration-200"
                  :class="
                    selectedScenario === s
                      ? 'bg-gold-dimmer/15 border-gold-dimmer text-gold-bright font-semibold'
                      : 'text-text-muted hover:text-text-bright border-transparent bg-transparent hover:bg-white/[0.03]'
                  "
                  @click="handleLoadScenario(s)"
                >
                  <span class="mr-2 truncate">{{ s }}</span>
                  <div class="flex shrink-0 items-center gap-1.5">
                    <!-- Delete Config Trash Button -->
                    <button
                      class="text-text-muted hover:text-red cursor-pointer rounded p-0.5 opacity-0 transition-all group-hover/item:opacity-100 hover:bg-red-500/20"
                      title="删除配置"
                      @click.stop="handleDeleteScenario(s)"
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="13"
                        height="13"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      >
                        <path d="M3 6h18" />
                        <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
                        <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                      </svg>
                    </button>
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="12"
                      height="12"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      class="text-gold-default shrink-0"
                    >
                      <path d="m9 18 6-6-6-6" />
                    </svg>
                  </div>
                </button>
              </div>
            </ScrollArea>
          </CardContent>
        </Card>

        <!-- 右侧：配置编辑 Panel (col-span-3) -->
        <div class="flex flex-col gap-5 md:col-span-3">
          <!-- Scenario Naming & Actions Panel -->
          <Card
            class="bg-bg-surface border-border-subtle flex w-full flex-col items-center justify-between gap-4 rounded-[0.625rem] p-4 py-4 shadow-lg ring-0! sm:flex-row"
            style="background: linear-gradient(180deg, rgba(17, 14, 20, 0.6) 0%, rgba(26, 21, 30, 0.8) 100%)"
          >
            <!-- Scenario Naming input -->
            <div class="flex w-full items-center gap-3 sm:w-[40%]">
              <label class="text-text-muted shrink-0 text-[11px] font-bold tracking-wider uppercase">配置名称:</label>
              <Input
                v-model="currentSceneName"
                type="text"
                class="border-gold-dimmer text-text-bright focus-visible:border-gold-default focus-visible:ring-gold-default/20 placeholder:text-text-muted/50 h-9 flex-1 rounded-md border bg-[#0f1115] px-3 font-mono text-xs transition-colors"
                placeholder="请输入配置名称..."
              />
            </div>

            <!-- AI Mode Toggle Switch -->
            <div
              class="border-border-subtle/50 flex h-9 shrink-0 items-center gap-2.5 rounded-md border bg-[#0f1115] px-3.5"
            >
              <Checkbox
                id="aiModeToggle"
                :model-value="mode === 'agent'"
                @update:model-value="(val) => (mode = val ? 'agent' : 'sandbox')"
              />
              <label
                for="aiModeToggle"
                class="cursor-pointer text-xs font-semibold select-none"
                :class="mode === 'agent' ? 'text-gold-bright' : 'text-text-muted'"
              >
                AI 模式
              </label>
            </div>

            <!-- Save & Launch actions -->
            <div class="flex w-full shrink-0 items-center justify-end gap-3 sm:w-auto">
              <Button
                variant="outline"
                class="border-gold-dimmer text-gold-bright hover:bg-gold-dimmer/15 h-9 cursor-pointer px-4 text-xs font-medium transition-all"
                @click="handleSaveConfig"
              >
                保存
              </Button>

              <Button
                class="text-gold-bright bg-bg-surface hover:text-gold-glow hover:shadow-glow-gold group/btn relative h-9 cursor-pointer overflow-hidden rounded-[0.375rem] border border-transparent px-6 text-xs font-semibold tracking-[0.05em] uppercase shadow-sm transition-all active:scale-[0.97] disabled:cursor-not-allowed disabled:opacity-50"
                :disabled="isStarting"
                @click="handleLaunchAgentMode"
              >
                <div
                  class="pointer-events-none absolute inset-0 rounded-[0.375rem] border border-transparent [mask-composite:exclude] [-webkit-mask-composite:xor] [background:linear-gradient(135deg,var(--color-gold-dimmer),var(--color-gold-default),var(--color-gold-bright))_border-box] [mask:linear-gradient(#fff_0_0)_content-box,linear-gradient(#fff_0_0)]"
                ></div>
                <span
                  class="pointer-events-none absolute top-0 left-[-50%] h-full w-1/2 -skew-x-[20deg] bg-gradient-to-r from-transparent via-[rgba(232,201,122,0.15)] to-transparent transition-none group-hover/btn:left-[150%] group-hover/btn:transition-[left] group-hover/btn:duration-700 group-hover/btn:ease-[cubic-bezier(0.16,1,0.3,1)]"
                  aria-hidden="true"
                ></span>
                <span class="relative z-1">
                  {{ isStarting ? "启动中…" : mode === "agent" ? "开始 AI 模拟对战" : "开始沙盒模拟对战" }}
                </span>
              </Button>
            </div>
          </Card>

          <!-- Status / Error Notification -->
          <Transition
            enter-active-class="transition-opacity duration-200 ease-out"
            leave-active-class="transition-opacity duration-200 ease-out"
            enter-from-class="opacity-0"
            leave-to-class="opacity-0"
          >
            <div v-if="error" class="border-border-subtle/30 rounded border bg-[#110e14]/50 py-2.5 text-center">
              <p class="text-xs font-semibold" :class="error.includes('成功') ? 'text-green-500' : 'text-red'">
                {{ error }}
              </p>
            </div>
          </Transition>

          <!-- Dual Column Agent configuration grids -->
          <div class="grid w-full grid-cols-1 gap-6 md:grid-cols-2">
            <!-- Left Column: Blue Team (Order) -->
            <Card
              class="bg-bg-surface relative flex w-full flex-col gap-4 overflow-hidden rounded-[0.625rem] border border-blue-500/20 p-0 py-0 shadow-[0_4px_12px_rgba(0,0,0,0.5)] ring-0!"
              style="background: linear-gradient(180deg, rgba(30, 41, 59, 0.2) 0%, rgba(15, 23, 42, 0.4) 100%)"
            >
              <CardHeader
                class="flex flex-row items-center justify-between space-y-0 border-b border-blue-500/20 p-6 pb-2"
              >
                <div class="flex items-center gap-2">
                  <Badge
                    variant="outline"
                    class="rounded-sm border-blue-500/25 bg-blue-500/10 px-2 py-0.5 text-[10px] font-bold tracking-wider text-blue-400 uppercase"
                  >
                    Order
                  </Badge>
                  <CardTitle class="text-xs font-semibold tracking-wider text-blue-400 uppercase">秩序阵营</CardTitle>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  class="h-7 cursor-pointer border-blue-500/30 text-[11px] text-blue-300 hover:bg-blue-500/10"
                  @click="addBlueAgent"
                >
                  + 添加英雄
                </Button>
              </CardHeader>

              <CardContent class="flex flex-1 flex-col overflow-hidden p-6 pt-0">
                <ScrollArea class="h-[420px] w-full">
                  <div class="flex flex-col gap-4 pr-3">
                    <div v-if="blueAgents.length === 0" class="text-text-muted py-10 text-center text-xs italic">
                      当前阵营无英雄，请点击右上角添加。
                    </div>

                    <div
                      v-else
                      v-for="(agent, index) in blueAgents"
                      :key="index"
                      class="bg-bg-deep group relative rounded-md border border-blue-500/10 p-4 text-left"
                    >
                      <!-- Delete Button -->
                      <button
                        class="text-text-muted hover:text-red absolute top-2 right-2 cursor-pointer transition-colors"
                        title="删除代理"
                        @click="deleteBlueAgent(index)"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="14"
                          height="14"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        >
                          <path d="M3 6h18" />
                          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
                          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                          <line x1="10" x2="10" y1="11" y2="17" />
                          <line x1="14" x2="14" y1="11" y2="17" />
                        </svg>
                      </button>

                      <div class="mb-3 grid grid-cols-2 gap-3">
                        <div>
                          <label class="text-text-muted mb-1 block text-[10px] font-semibold uppercase">选择英雄</label>
                          <Select
                            :model-value="agent.champion"
                            @update:model-value="(val) => (agent.champion = val as string)"
                          >
                            <SelectTrigger
                              class="text-text-bright h-8 cursor-pointer border-blue-500/25 bg-[#0f1115] text-xs"
                            >
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent class="border-border-subtle text-text-bright bg-[#110e14]">
                              <SelectItem v-for="c in champions" :key="c" :value="c" class="cursor-pointer text-xs">
                                {{ c }}
                              </SelectItem>
                            </SelectContent>
                          </Select>
                        </div>

                        <div>
                          <label class="text-text-muted mb-1 block text-[10px] font-semibold uppercase">
                            出生坐标 (X, Z)
                          </label>
                          <button
                            class="text-gold-bright hover:border-gold-muted flex h-8 w-full cursor-pointer items-center justify-between rounded-md border border-blue-500/25 bg-[#0f1115] px-2 text-xs transition-all"
                            @click="openMapModal(agent)"
                          >
                            <span>{{ agent.spawnPoint[0] }}, {{ agent.spawnPoint[1] }}</span>
                            <svg
                              xmlns="http://www.w3.org/2000/svg"
                              width="12"
                              height="12"
                              viewBox="0 0 24 24"
                              fill="none"
                              stroke="currentColor"
                              stroke-width="2"
                              stroke-linecap="round"
                              stroke-linejoin="round"
                              class="text-blue-400"
                            >
                              <path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z" />
                              <circle cx="12" cy="10" r="3" />
                            </svg>
                          </button>
                        </div>
                      </div>

                      <div v-if="mode === 'agent'">
                        <label class="text-text-muted mb-1 block text-[10px] font-semibold uppercase">
                          Agent 系统提示词 (System Prompt)
                        </label>
                        <Textarea
                          v-model="agent.prompt"
                          rows="3"
                          class="text-text-bright min-h-16 w-full resize-none rounded-md border border-blue-500/10 bg-[#0f1115] p-2 font-sans text-xs leading-relaxed focus-visible:border-blue-500/40 focus-visible:ring-blue-500/20"
                          placeholder="输入该 AI 的系统微调 Prompts..."
                        />
                      </div>
                    </div>
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>

            <!-- Right Column: Red Team (Chaos) -->
            <Card
              class="bg-bg-surface relative flex w-full flex-col gap-4 overflow-hidden rounded-[0.625rem] border border-red-500/20 p-0 py-0 shadow-[0_4px_12px_rgba(0,0,0,0.5)] ring-0!"
              style="background: linear-gradient(180deg, rgba(69, 26, 26, 0.15) 0%, rgba(15, 23, 42, 0.4) 100%)"
            >
              <CardHeader
                class="flex flex-row items-center justify-between space-y-0 border-b border-red-500/20 p-6 pb-2"
              >
                <div class="flex items-center gap-2">
                  <Badge
                    variant="outline"
                    class="rounded-sm border-red-500/25 bg-red-500/10 px-2 py-0.5 text-[10px] font-bold tracking-wider text-red-400 uppercase"
                  >
                    Chaos
                  </Badge>
                  <CardTitle class="text-xs font-semibold tracking-wider text-red-400 uppercase">混沌阵营</CardTitle>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  class="h-7 cursor-pointer border-red-500/30 text-[11px] text-red-300 hover:bg-red-500/10"
                  @click="addRedAgent"
                >
                  + 添加英雄
                </Button>
              </CardHeader>

              <CardContent class="flex flex-1 flex-col overflow-hidden p-6 pt-0">
                <ScrollArea class="h-[420px] w-full">
                  <div class="flex flex-col gap-4 pr-3">
                    <div v-if="redAgents.length === 0" class="text-text-muted py-10 text-center text-xs italic">
                      当前阵营无英雄，请点击右上角添加。
                    </div>

                    <div
                      v-else
                      v-for="(agent, index) in redAgents"
                      :key="index"
                      class="bg-bg-deep group relative rounded-md border border-red-500/10 p-4 text-left"
                    >
                      <!-- Delete Button -->
                      <button
                        class="text-text-muted hover:text-red absolute top-2 right-2 cursor-pointer transition-colors"
                        title="删除代理"
                        @click="deleteRedAgent(index)"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="14"
                          height="14"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        >
                          <path d="M3 6h18" />
                          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
                          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                          <line x1="10" x2="10" y1="11" y2="17" />
                          <line x1="14" x2="14" y1="11" y2="17" />
                        </svg>
                      </button>

                      <div class="mb-3 grid grid-cols-2 gap-3">
                        <div>
                          <label class="text-text-muted mb-1 block text-[10px] font-semibold uppercase">选择英雄</label>
                          <Select
                            :model-value="agent.champion"
                            @update:model-value="(val) => (agent.champion = val as string)"
                          >
                            <SelectTrigger
                              class="text-text-bright h-8 cursor-pointer border-red-500/25 bg-[#0f1115] text-xs"
                            >
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent class="border-border-subtle text-text-bright bg-[#110e14]">
                              <SelectItem v-for="c in champions" :key="c" :value="c" class="cursor-pointer text-xs">
                                {{ c }}
                              </SelectItem>
                            </SelectContent>
                          </Select>
                        </div>

                        <div>
                          <label class="text-text-muted mb-1 block text-[10px] font-semibold uppercase">
                            出生坐标 (X, Z)
                          </label>
                          <button
                            class="text-gold-bright hover:border-gold-muted flex h-8 w-full cursor-pointer items-center justify-between rounded-md border border-red-500/25 bg-[#0f1115] px-2 text-xs transition-all"
                            @click="openMapModal(agent)"
                          >
                            <span>{{ agent.spawnPoint[0] }}, {{ agent.spawnPoint[1] }}</span>
                            <svg
                              xmlns="http://www.w3.org/2000/svg"
                              width="12"
                              height="12"
                              viewBox="0 0 24 24"
                              fill="none"
                              stroke="currentColor"
                              stroke-width="2"
                              stroke-linecap="round"
                              stroke-linejoin="round"
                              class="text-red-400"
                            >
                              <path d="M20 10c0 6-8 12-8 12s-8-6-8-12a8 8 0 0 1 16 0Z" />
                              <circle cx="12" cy="10" r="3" />
                            </svg>
                          </button>
                        </div>
                      </div>

                      <div v-if="mode === 'agent'">
                        <label class="text-text-muted mb-1 block text-[10px] font-semibold uppercase">
                          Agent 系统提示词 (System Prompt) {{ mode }}
                        </label>
                        <Textarea
                          v-model="agent.prompt"
                          rows="3"
                          class="text-text-bright min-h-16 w-full resize-none rounded-md border border-red-500/10 bg-[#0f1115] p-2 font-sans text-xs leading-relaxed focus-visible:border-red-500/40 focus-visible:ring-red-500/20"
                          placeholder="输入该 AI 的系统微调 Prompts..."
                        />
                      </div>
                    </div>
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  </div>

  <!-- Cybernetic Summoner's Rift Coordinates Map Picker Modal -->
  <Dialog
    :open="isMapModalOpen"
    @update:open="
      (val) => {
        if (!val) closeMapModal();
      }
    "
  >
    <DialogContent
      class="border-gold-dimmer text-text-bright [&>button]:text-text-muted [&>button]:hover:text-text-bright flex w-full max-w-lg flex-col gap-4 rounded-[0.625rem] border border-solid bg-[#110e14] p-6 shadow-2xl"
      style="box-shadow: 0 0 50px rgba(212, 175, 92, 0.08)"
    >
      <DialogHeader class="border-border-subtle space-y-1 border-b pb-2 text-left">
        <DialogTitle class="text-gold-bright text-sm font-semibold tracking-wide">
          召集成群：点击地图标记出生点
        </DialogTitle>
        <DialogDescription class="text-text-muted text-[11px] leading-relaxed">
          在 Summoner's Rift 模拟图上进行点击，系统会自动转换成游戏中的绝对地图尺寸坐标（15000 x
          15000）。左下角为秩序基地，右上角为混沌基地。
        </DialogDescription>
      </DialogHeader>

      <!-- Dynamic SVG Map -->
      <div
        class="border-border-subtle/50 relative mx-auto aspect-square w-full max-w-[380px] overflow-hidden rounded-md border"
      >
        <svg
          viewBox="0 0 500 500"
          class="relative h-full w-full cursor-crosshair bg-[#0d0a11] shadow-inner select-none"
          @click="handleMapClick"
        >
          <!-- Cyber grid patterns -->
          <defs>
            <pattern id="mapGrid" width="25" height="25" patternUnits="userSpaceOnUse">
              <path d="M 25 0 L 0 0 0 25" fill="none" stroke="rgba(212, 175, 92, 0.03)" stroke-width="0.75" />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#mapGrid)" />

          <!-- Lanes (Cyber paths) -->
          <!-- Top lane -->
          <path
            d="M 60 440 L 60 60 L 440 60"
            fill="none"
            stroke="rgba(212, 175, 92, 0.12)"
            stroke-width="6"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M 60 440 L 60 60 L 440 60"
            fill="none"
            stroke="#e8c97a"
            stroke-width="1.5"
            stroke-dasharray="6,8"
            stroke-linecap="round"
            stroke-linejoin="round"
          />

          <!-- Mid lane -->
          <line
            x1="60"
            y1="440"
            x2="440"
            y2="60"
            stroke="rgba(212, 175, 92, 0.12)"
            stroke-width="6"
            stroke-linecap="round"
          />
          <line
            x1="60"
            y1="440"
            x2="440"
            y2="60"
            stroke="#e8c97a"
            stroke-width="1.5"
            stroke-dasharray="6,8"
            stroke-linecap="round"
          />

          <!-- Bot lane -->
          <path
            d="M 60 440 L 440 440 L 440 60"
            fill="none"
            stroke="rgba(212, 175, 92, 0.12)"
            stroke-width="6"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            d="M 60 440 L 440 440 L 440 60"
            fill="none"
            stroke="#e8c97a"
            stroke-width="1.5"
            stroke-dasharray="6,8"
            stroke-linecap="round"
            stroke-linejoin="round"
          />

          <!-- Blue base area -->
          <circle
            cx="60"
            cy="440"
            r="28"
            fill="rgba(59, 130, 246, 0.03)"
            stroke="rgba(59, 130, 246, 0.2)"
            stroke-width="1"
            stroke-dasharray="3,3"
          />
          <rect
            x="48"
            y="428"
            width="24"
            height="24"
            fill="rgba(59, 130, 246, 0.12)"
            stroke="#3b82f6"
            stroke-width="1.5"
          />
          <text
            x="60"
            y="415"
            text-anchor="middle"
            fill="#3b82f6"
            font-size="8"
            font-family="sans-serif"
            font-weight="bold"
            letter-spacing="1"
          >
            BLUE
          </text>

          <!-- Red base area -->
          <circle
            cx="440"
            cy="60"
            r="28"
            fill="rgba(239, 68, 68, 0.03)"
            stroke="rgba(239, 68, 68, 0.2)"
            stroke-width="1"
            stroke-dasharray="3,3"
          />
          <rect
            x="428"
            y="48"
            width="24"
            height="24"
            fill="rgba(239, 68, 68, 0.12)"
            stroke="#ef4444"
            stroke-width="1.5"
          />
          <text
            x="440"
            y="92"
            text-anchor="middle"
            fill="#ef4444"
            font-size="8"
            font-family="sans-serif"
            font-weight="bold"
            letter-spacing="1"
          >
            RED
          </text>

          <!-- Hover/Click Ping Ring and Target Dot -->
          <g v-if="mapTargetAgent">
            <circle
              :cx="agentMarkerX"
              :cy="agentMarkerY"
              r="16"
              fill="none"
              stroke="#e8c97a"
              stroke-width="1"
              class="animate-ping"
              style="transform-origin: center; animation-duration: 2s"
            ></circle>
            <circle
              :cx="agentMarkerX"
              :cy="agentMarkerY"
              r="8"
              fill="rgba(232, 201, 122, 0.2)"
              stroke="#e8c97a"
              stroke-width="1.5"
            ></circle>
            <circle :cx="agentMarkerX" :cy="agentMarkerY" r="2.5" fill="#ffffff"></circle>
          </g>
        </svg>
      </div>

      <DialogFooter class="border-border-subtle flex items-center gap-4 border-t pt-2 text-xs sm:justify-between">
        <span class="text-text-muted">
          当前坐标：
          <span class="text-gold-bright font-mono font-semibold">
            {{ mapTargetAgent ? `${mapTargetAgent.spawnPoint[0]}, ${mapTargetAgent.spawnPoint[1]}` : "无" }}
          </span>
        </span>
        <Button
          size="sm"
          class="text-gold-bright border-gold-dimmer hover:bg-gold-dimmer/20 cursor-pointer border bg-transparent px-6"
          @click="closeMapModal"
        >
          确认坐标
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<style scoped>
/* Scrollbar styles inside cards */
::-webkit-scrollbar {
  width: 4px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: rgba(212, 175, 92, 0.15);
  border-radius: 2px;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(212, 175, 92, 0.3);
}
</style>
