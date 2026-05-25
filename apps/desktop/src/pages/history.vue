<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import AgentChatHistory from "../components/AgentChatHistory.vue";
import { Button } from "../components/ui/button";
import { Card, CardHeader, CardTitle, CardContent } from "../components/ui/card";
import { ScrollArea } from "../components/ui/scroll-area";
import { Badge } from "../components/ui/badge";

interface AgentSummary {
  agent_id: string;
  champion: string;
  team: string;
}

interface GameHistorySummary {
  datetime: string;
  duration: number;
  agents: AgentSummary[];
}

interface ChatMessage {
  role: string;
  content: string;
}

interface SavedAgentHistory {
  agent_id: string;
  champion: string;
  team: string;
  prompt: string;
  system_prompt?: string;
  history: any[];
  game_duration: number;
  datetime: string;
}

const histories = ref<GameHistorySummary[]>([]);
const selectedDatetime = ref<string>("");
const selectedGameAgents = ref<SavedAgentHistory[]>([]);
const selectedAgentId = ref<string>("");
const loadingList = ref(true);
const loadingDetail = ref(false);

// Load all histories
async function loadHistories() {
  loadingList.value = true;
  try {
    histories.value = await invoke<GameHistorySummary[]>("list_game_histories");
    if (histories.value.length > 0 && !selectedDatetime.value) {
      selectGame(histories.value[0].datetime);
    }
  } catch (e) {
    console.error("加载历史记录失败", e);
  } finally {
    loadingList.value = false;
  }
}

// Select a specific game session
async function selectGame(datetime: string) {
  selectedDatetime.value = datetime;
  loadingDetail.value = true;
  try {
    const details = await invoke<SavedAgentHistory[]>("get_game_history_detail", { datetime });
    selectedGameAgents.value = details;
    if (details.length > 0) {
      selectedAgentId.value = details[0].agent_id;
    } else {
      selectedAgentId.value = "";
    }
  } catch (e) {
    console.error("加载对局详情失败", e);
  } finally {
    loadingDetail.value = false;
  }
}

// Delete a history entry
async function deleteHistory(datetime: string, event: MouseEvent) {
  event.stopPropagation();
  if (!confirm("确定要删除这局游戏历史记录吗？此操作无法撤销。")) {
    return;
  }
  try {
    await invoke("delete_game_history", { datetime });
    histories.value = histories.value.filter((h) => h.datetime !== datetime);
    if (selectedDatetime.value === datetime) {
      selectedDatetime.value = "";
      selectedGameAgents.value = [];
      selectedAgentId.value = "";
      if (histories.value.length > 0) {
        selectGame(histories.value[0].datetime);
      }
    }
  } catch (e) {
    console.error("删除历史记录失败", e);
  }
}

// Get details of active agent
const activeAgentData = computed(() => {
  return selectedGameAgents.value.find((a) => a.agent_id === selectedAgentId.value) || null;
});

// Format game history Rig messages into standard format
const activeAgentHistoryFormatted = computed<ChatMessage[]>(() => {
  if (!activeAgentData.value) return [];
  return (activeAgentData.value.history || []).map((msg: any) => {
    if (msg.User) {
      return { role: "user", content: msg.User.content };
    }
    if (msg.Assistant) {
      return { role: "assistant", content: msg.Assistant.content };
    }
    return {
      role: msg.role || "user",
      content: msg.content || "",
    };
  });
});

// Format duration from seconds to MM:SS
function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}

// Format datetime string nicely
function formatDatetime(str: string): string {
  // str format is "YYYY-MM-DD_HH-MM-SS"
  return str.replace("_", " ").replace(/-/g, "/").replace(/:/g, ":");
}

onMounted(() => {
  loadHistories();
});
</script>

<template>
  <div class="bg-bg-deep flex h-full flex-col gap-3 overflow-hidden p-4">
    <!-- Header Summary Bar -->
    <div
      class="bg-bg-surface border-border-subtle flex shrink-0 items-center justify-between rounded border px-4 py-2.5 shadow-[0_1px_2px_rgba(0,0,0,0.4)]"
    >
      <div class="flex items-center gap-3">
        <span class="font-sans text-gold-bright text-xs font-bold tracking-wider uppercase">
          对局历史档案库 (Game History Vault)
        </span>
        <span class="bg-border-subtle h-3.5 w-px"></span>
        <span class="text-text-muted text-[11px]">
          共存有 <span class="text-gold-default font-semibold">{{ histories.length }}</span> 局模拟记录
        </span>
      </div>
      <Button
        variant="outline"
        size="xs"
        class="border-gold-dimmer text-gold-bright hover:bg-gold-dimmer/10 h-7 cursor-pointer text-xs"
        @click="loadHistories"
      >
        刷新列表
      </Button>
    </div>

    <!-- Main Content Workspace -->
    <div class="flex min-h-0 flex-1 gap-4 overflow-hidden">
      <!-- LEFT SECTION: Historical Session Cards -->
      <Card
        class="bg-bg-surface border-border-subtle flex w-80 flex-col overflow-hidden rounded border p-0 shadow-lg"
        style="background: linear-gradient(180deg, rgba(17, 14, 20, 0.6) 0%, rgba(26, 21, 30, 0.8) 100%)"
      >
        <CardHeader class="border-border-subtle border-b p-4 py-3">
          <CardTitle class="text-text-muted text-[11px] font-bold tracking-wider uppercase">历史对局列表</CardTitle>
        </CardHeader>
        <CardContent class="flex-1 overflow-hidden p-3 pt-2">
          <div v-if="loadingList" class="text-text-muted py-12 text-center text-xs italic">
            加载历史记录中...
          </div>
          <div v-else-if="histories.length === 0" class="text-text-muted py-12 text-center text-xs italic">
            暂无已保存的游戏历史记录
          </div>
          <ScrollArea v-else class="h-full w-full">
            <div class="flex flex-col gap-2.5 pr-3">
              <div
                v-for="item in histories"
                :key="item.datetime"
                class="border-border-subtle hover:border-gold-dimmer group relative flex cursor-pointer flex-col gap-2 rounded border bg-[#0d0b0f] p-3 transition-all duration-200"
                :class="{
                  'border-gold-default/60 bg-[rgba(185,145,71,0.05)] shadow-md ring-1 ring-gold-muted/20': selectedDatetime === item.datetime
                }"
                @click="selectGame(item.datetime)"
              >
                <!-- Datetime and Trash action -->
                <div class="flex items-center justify-between">
                  <span class="text-text-bright font-mono text-[11px] font-semibold">
                    {{ formatDatetime(item.datetime) }}
                  </span>
                  <button
                    class="text-text-muted hover:text-red cursor-pointer rounded p-0.5 opacity-40 transition-all group-hover:opacity-100 hover:bg-red-500/20"
                    title="删除此记录"
                    @click="deleteHistory(item.datetime, $event)"
                  >
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
                    >
                      <path d="M3 6h18" />
                      <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
                      <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                    </svg>
                  </button>
                </div>

                <!-- Info row: duration and agents counts -->
                <div class="flex items-center justify-between text-[11px]">
                  <div class="flex items-center gap-1">
                    <span class="text-text-muted">时长:</span>
                    <span class="text-gold-bright font-mono font-medium">{{ formatDuration(item.duration) }}</span>
                  </div>
                  <Badge variant="outline" class="border-border-subtle bg-bg-deep text-text-muted py-0 text-[10px]">
                    {{ item.agents.length }} 个 Agent
                  </Badge>
                </div>

                <!-- Champions badges involved -->
                <div class="flex flex-wrap gap-1.5 mt-0.5">
                  <span
                    v-for="agent in item.agents"
                    :key="agent.agent_id"
                    class="rounded px-1.5 py-0.5 text-[10px] font-semibold font-mono tracking-wide"
                    :class="agent.team === 'Order' ? 'bg-blue-900/40 text-blue-300 border border-blue-800/30' : 'bg-red-950/40 text-red-300 border border-red-900/30'"
                  >
                    {{ agent.champion }}
                  </span>
                </div>
              </div>
            </div>
          </ScrollArea>
        </CardContent>
      </Card>

      <!-- RIGHT SECTION: Dialog Drilling Detail Panel -->
      <div class="bg-bg-surface border-border-subtle flex min-h-0 flex-1 flex-col overflow-hidden rounded border shadow-lg">
        <!-- If no game is selected -->
        <div
          v-if="!selectedDatetime"
          class="text-text-muted flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center text-sm italic"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="32"
            height="32"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            class="text-text-muted/40 animate-pulse"
          >
            <rect width="18" height="18" x="3" y="3" rx="2" />
            <path d="M21 9H3" />
            <path d="M21 15H3" />
            <path d="M12 3v18" />
          </svg>
          <span>请从左侧选择一局游戏历史记录进行查看</span>
        </div>

        <div v-else-if="loadingDetail" class="text-text-muted flex flex-1 items-center justify-center text-xs italic">
          加载对局历史详情中...
        </div>

        <!-- Selected Game Content -->
        <div v-else class="grid h-full w-full grid-cols-1 gap-4 overflow-hidden p-4 md:grid-cols-4">
          <!-- Sub-Sidebar: Active Game Agents Selector list -->
          <div class="border-border-subtle/40 flex min-h-0 flex-col gap-3 overflow-y-auto border-r pr-3.5 md:col-span-1">
            <span class="text-text-muted text-[10px] font-bold tracking-wider uppercase">对局活动代理</span>

            <div class="flex flex-col gap-2">
              <button
                v-for="agent in selectedGameAgents"
                :key="agent.agent_id"
                class="flex w-full cursor-pointer flex-col rounded border p-2 text-left transition-all duration-200"
                :class="
                  selectedAgentId === agent.agent_id
                    ? 'bg-gold-dimmer/10 border-gold-dimmer text-gold-bright'
                    : 'text-text-muted hover:text-text-bright border-transparent bg-transparent hover:bg-white/[0.02]'
                "
                @click="selectedAgentId = agent.agent_id"
              >
                <div class="flex items-center gap-1.5 font-sans text-xs font-semibold">
                  <span
                    class="h-1.5 w-1.5 rounded-full"
                    :class="agent.team === 'Order' ? 'bg-blue-400 shadow-[0_0_4px_rgba(96,165,250,0.5)]' : 'bg-red-400 shadow-[0_0_4px_rgba(248,113,113,0.5)]'"
                  ></span>
                  <span>{{ agent.champion }}</span>
                </div>
                <div class="flex items-center justify-between text-[9px] mt-1 uppercase tracking-wide">
                  <span :class="agent.team === 'Order' ? 'text-blue-400/80' : 'text-red-400/80'">
                    {{ agent.team === 'Order' ? '秩序 Order' : '混沌 Chaos' }}
                  </span>
                  <span class="text-text-muted/70 font-mono">{{ agent.history.length }} 轮对话</span>
                </div>
              </button>
            </div>

            <!-- Agent static details and Prompt settings display -->
            <div v-if="activeAgentData" class="flex flex-col gap-3 mt-2">
              <div class="flex flex-col gap-1.5">
                <span class="text-text-muted text-[10px] font-bold tracking-wider uppercase">系统级全局提示词</span>
                <div class="border-border-subtle bg-bg-deep rounded border p-2 max-h-28 overflow-y-auto">
                  <p class="text-text-muted font-sans text-[10px] leading-relaxed whitespace-pre-wrap select-text">
                    {{ activeAgentData.system_prompt || '无系统级全局提示词设定。' }}
                  </p>
                </div>
              </div>

              <div class="flex flex-col gap-1.5">
                <span class="text-text-muted text-[10px] font-bold tracking-wider uppercase">英雄级个性化提示词</span>
                <div class="border-border-subtle bg-bg-deep rounded border p-2 max-h-32 overflow-y-auto">
                  <p class="text-text-muted font-sans text-[10px] leading-relaxed whitespace-pre-wrap select-text">
                    {{ activeAgentData.prompt || '无系统提示词设定。' }}
                  </p>
                </div>
              </div>
            </div>
          </div>

          <!-- Dialog chat list panel -->
          <div class="flex h-full min-h-0 flex-col overflow-hidden md:col-span-3">
            <AgentChatHistory
              v-if="activeAgentData"
              :history="activeAgentHistoryFormatted"
              :champion="activeAgentData.champion"
              :loading="false"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Custom styled premium glassmorphism shadow */
.shadow-glow-gold {
  box-shadow: 0 0 15px rgba(185, 145, 71, 0.15);
}
</style>
