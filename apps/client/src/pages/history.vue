<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useGameStore } from "@/stores/gameStore";
import { useLocale } from "@/composables/useLocale";
import { backendClient } from "@/services/backend";
import AgentChatHistory from "@/components/AgentChatHistory.vue";
import { Button } from "@/components/ui/button";
import { Trash2Icon, InboxIcon } from "@lucide/vue";

const route = useRoute();
const router = useRouter();

const store = useGameStore();
const { t } = useLocale();

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
  { immediate: true },
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
  if (!confirm(t("history.deleteConfirm"))) {
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
  <div class="bg-background flex h-full flex-col gap-3 overflow-hidden p-4">
    <!-- Header Summary Bar -->
    <div class="border-border bg-card flex shrink-0 items-center justify-between rounded border px-4 py-2.5 shadow-sm">
      <div class="flex items-center gap-3">
        <span class="text-primary text-xs font-bold tracking-wider uppercase">
          {{ t("history.title") }}
        </span>
        <span class="bg-border h-3.5 w-px"></span>
        <span class="text-muted-foreground text-xs">
          {{ t("history.currentSelection") }}
          <span class="text-foreground font-mono font-semibold">
            {{ selectedDatetime ? formatDatetime(selectedDatetime) : t("history.none") }}
          </span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <Button
          v-if="selectedDatetime"
          variant="outline"
          size="sm"
          class="hover:bg-destructive hover:text-destructive-foreground h-7 text-xs"
          @click="deleteHistory(selectedDatetime)"
        >
          <Trash2Icon class="mr-1 size-3.5" />
          <span>{{ t("history.deleteBtn") }}</span>
        </Button>
        <Button variant="outline" size="sm" class="h-7 text-xs" @click="store.loadHistoriesList()">
          {{ t("history.refreshBtn") }}
        </Button>
      </div>
    </div>

    <!-- Main Content Workspace -->
    <div class="border-border bg-card flex min-h-0 flex-1 gap-4 overflow-hidden rounded border shadow-sm">
      <!-- If no game is selected -->
      <div
        v-if="!selectedDatetime"
        class="text-muted-foreground flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center text-sm italic"
      >
        <InboxIcon class="text-muted-foreground/40 size-8 animate-pulse" />
        <span>{{ t("history.noSelectionHint") }}</span>
      </div>

      <div
        v-else-if="loadingDetail"
        class="text-muted-foreground flex flex-1 items-center justify-center text-xs italic"
      >
        {{ t("history.loadingDetail") }}
      </div>

      <!-- Selected Game Content -->
      <div v-else class="grid h-full w-full grid-cols-1 gap-4 overflow-hidden p-4 md:grid-cols-4">
        <!-- Sub-Sidebar: Active Game Agents Selector list -->
        <div class="border-border flex min-h-0 flex-col gap-3 overflow-y-auto border-r pr-3.5 md:col-span-1">
          <span class="text-muted-foreground text-[10px] font-bold tracking-wider uppercase">
            {{ t("history.activeAgents") }}
          </span>

          <div class="flex flex-col gap-1.5">
            <button
              v-for="agent in selectedGameAgents"
              :key="agent.agent_id"
              class="flex w-full cursor-pointer flex-col rounded border p-2 text-left transition-colors"
              :class="
                selectedAgentId === agent.agent_id
                  ? 'bg-primary/10 border-primary text-primary font-semibold'
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted/40 border-transparent bg-transparent'
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
              <div class="mt-1 flex items-center justify-between text-[9px] tracking-wide uppercase">
                <span :class="agent.team === 'Order' ? 'text-blue-500/80' : 'text-red-500/80'">
                  {{ agent.team === "Order" ? t("history.teamOrder") : t("history.teamChaos") }}
                </span>
                <span class="text-muted-foreground/70 font-mono">
                  {{ t("history.roundsCount", { count: agent.history.length }) }}
                </span>
              </div>
            </button>
          </div>

          <!-- Agent static details and Prompt settings display -->
          <div v-if="activeAgentData" class="mt-2 flex flex-col gap-3">
            <div class="flex flex-col gap-1.5">
              <span class="text-muted-foreground text-[10px] font-bold tracking-wider uppercase">
                {{ t("history.globalPrompt") }}
              </span>
              <div class="border-border bg-muted/30 max-h-28 overflow-y-auto rounded border p-2">
                <p class="text-muted-foreground font-sans text-[10px] leading-relaxed whitespace-pre-wrap select-text">
                  {{ activeAgentData.system_prompt || t("history.noGlobalPrompt") }}
                </p>
              </div>
            </div>

            <div class="flex flex-col gap-1.5">
              <span class="text-muted-foreground text-[10px] font-bold tracking-wider uppercase">
                {{ t("history.heroPrompt") }}
              </span>
              <div class="border-border bg-muted/30 max-h-32 overflow-y-auto rounded border p-2">
                <p class="text-muted-foreground font-sans text-[10px] leading-relaxed whitespace-pre-wrap select-text">
                  {{ activeAgentData.prompt || t("history.noHeroPrompt") }}
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
