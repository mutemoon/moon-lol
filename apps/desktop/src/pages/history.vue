<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useGameStore } from "../stores/gameStore";
import { backendClient } from "../services/backend";
import AgentChatHistory from "../components/AgentChatHistory.vue";
import { Button } from "../components/ui/button";
import { Trash2Icon, InboxIcon } from "@lucide/vue";

const route = useRoute();
const router = useRouter();

const store = useGameStore();

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

const selectedDatetime = ref<string>("");
const selectedGameAgents = ref<SavedAgentHistory[]>([]);
const selectedAgentId = ref<string>("");
const loadingDetail = ref(false);

// Watch for route query changes (sidebar clicks)
watch(
  () => route.query.datetime,
  (newVal) => {
    if (newVal) {
      selectGame(newVal as string);
    } else {
      selectedDatetime.value = "";
      selectedGameAgents.value = [];
      selectedAgentId.value = "";
    }
  },
  { immediate: true }
);

// Select a specific game session
async function selectGame(datetime: string) {
  selectedDatetime.value = datetime;
  loadingDetail.value = true;
  try {
    const details = await backendClient.getGameHistoryDetail(datetime);
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
async function deleteHistory(datetime: string) {
  if (!confirm("确定要删除这局游戏历史记录吗？此操作无法撤销。")) {
    return;
  }
  try {
    await backendClient.deleteGameHistory(datetime);
    await store.loadHistoriesList();
    if (selectedDatetime.value === datetime) {
      router.push("/history");
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
const activeAgentHistoryFormatted = computed(() => {
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

// Format datetime string nicely
function formatDatetime(str: string): string {
  return str.replace("_", " ").replace(/-/g, "/").replace(/:/g, ":");
}

onMounted(() => {
  store.loadHistoriesList();
});
</script>

<template>
  <div class="flex h-full flex-col gap-3 overflow-hidden p-4 bg-background">
    <!-- Header Summary Bar -->
    <div
      class="flex shrink-0 items-center justify-between rounded border border-border bg-card px-4 py-2.5 shadow-sm"
    >
      <div class="flex items-center gap-3">
        <span class="text-xs font-bold uppercase tracking-wider text-primary">
          对局历史档案库
        </span>
        <span class="h-3.5 w-px bg-border"></span>
        <span class="text-xs text-muted-foreground">
          当前选择：<span class="font-semibold text-foreground font-mono">{{ selectedDatetime ? formatDatetime(selectedDatetime) : '无' }}</span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <Button
          v-if="selectedDatetime"
          variant="outline"
          size="sm"
          class="h-7 text-xs hover:bg-destructive hover:text-destructive-foreground"
          @click="deleteHistory(selectedDatetime)"
        >
          <Trash2Icon class="size-3.5 mr-1" />
          <span>删除此对局</span>
        </Button>
        <Button
          variant="outline"
          size="sm"
          class="h-7 text-xs"
          @click="store.loadHistoriesList()"
        >
          刷新列表
        </Button>
      </div>
    </div>

    <!-- Main Content Workspace -->
    <div class="flex min-h-0 flex-1 gap-4 overflow-hidden rounded border border-border bg-card shadow-sm">
      <!-- If no game is selected -->
      <div
        v-if="!selectedDatetime"
        class="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center text-sm text-muted-foreground italic"
      >
        <InboxIcon class="size-8 text-muted-foreground/40 animate-pulse" />
        <span>请从左侧“最近对局”的已完成列表中选择一局游戏历史记录进行查看</span>
      </div>

      <div v-else-if="loadingDetail" class="flex flex-1 items-center justify-center text-xs text-muted-foreground italic">
        正在载入该局游戏历史档案详情...
      </div>

      <!-- Selected Game Content -->
      <div v-else class="grid h-full w-full grid-cols-1 gap-4 overflow-hidden p-4 md:grid-cols-4">
        <!-- Sub-Sidebar: Active Game Agents Selector list -->
        <div class="flex min-h-0 flex-col gap-3 overflow-y-auto border-r border-border pr-3.5 md:col-span-1">
          <span class="text-[10px] font-bold tracking-wider uppercase text-muted-foreground">对局活动代理</span>

          <div class="flex flex-col gap-1.5">
            <button
              v-for="agent in selectedGameAgents"
              :key="agent.agent_id"
              class="flex w-full cursor-pointer flex-col rounded border p-2 text-left transition-colors"
              :class="
                selectedAgentId === agent.agent_id
                  ? 'bg-primary/10 border-primary text-primary font-semibold'
                  : 'text-muted-foreground hover:text-foreground border-transparent bg-transparent hover:bg-muted/40'
              "
              @click="selectedAgentId = agent.agent_id"
            >
              <div class="flex items-center gap-1.5 text-xs font-semibold">
                <span
                  class="h-1.5 w-1.5 rounded-full"
                  :class="agent.team === 'Order' ? 'bg-blue-500' : 'bg-red-500'"
                ></span>
                <span>{{ agent.champion }}</span>
              </div>
              <div class="flex items-center justify-between text-[9px] mt-1 uppercase tracking-wide">
                <span :class="agent.team === 'Order' ? 'text-blue-500/80' : 'text-red-500/80'">
                  {{ agent.team === 'Order' ? '秩序 Order' : '混沌 Chaos' }}
                </span>
                <span class="text-muted-foreground/70 font-mono">{{ agent.history.length }} 轮对话</span>
              </div>
            </button>
          </div>

          <!-- Agent static details and Prompt settings display -->
          <div v-if="activeAgentData" class="flex flex-col gap-3 mt-2">
            <div class="flex flex-col gap-1.5">
              <span class="text-[10px] font-bold tracking-wider uppercase text-muted-foreground">系统级全局提示词</span>
              <div class="border border-border bg-muted/30 rounded p-2 max-h-28 overflow-y-auto">
                <p class="text-muted-foreground font-sans text-[10px] leading-relaxed whitespace-pre-wrap select-text">
                  {{ activeAgentData.system_prompt || '无系统级全局提示词设定。' }}
                </p>
              </div>
            </div>

            <div class="flex flex-col gap-1.5">
              <span class="text-[10px] font-bold tracking-wider uppercase text-muted-foreground">英雄级个性化提示词</span>
              <div class="border border-border bg-muted/30 rounded p-2 max-h-32 overflow-y-auto">
                <p class="text-muted-foreground font-sans text-[10px] leading-relaxed whitespace-pre-wrap select-text">
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
</template>

<style scoped>
/* Scrollbar styles inside history view */
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
