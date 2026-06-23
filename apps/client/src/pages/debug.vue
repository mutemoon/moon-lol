<route lang="yaml">
meta:
  layout: desktop
</route>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useGameStore } from "@/stores/gameStore";
import { useLocale } from "@/composables/useLocale";
import GameConsoleLogs from "@/components/GameConsoleLogs.vue";
import AgentChatHistory from "@/components/AgentChatHistory.vue";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { backendClient } from "@/services/backend";
import type { UnsubscribeFn } from "@/services/backend";
import { CommandIcon, StopCircleIcon } from "@lucide/vue";

const store = useGameStore();
const { ws } = store;
const { stopGame } = store;
const { t } = useLocale();

// Access unwrapped values reactively via computed properties
const connected = computed(() => ws.connected);
const gameState = computed(() => ws.gameState);

const switchTarget = ref("Riven");

function toggleGodMode() {
  store.ws.send("god_mode", { enabled: !gameState.value.godMode });
}

function toggleCooldown() {
  store.ws.send("toggle_cooldown", { enabled: !gameState.value.cooldownDisabled });
}

function togglePause() {
  store.ws.send("toggle_pause", {});
}

function resetPosition() {
  store.ws.send("reset_position", {});
}

function switchChampion() {
  store.ws.send("switch_champion", { name: switchTarget.value });
}

const champions = ["Riven", "Fiora"];

// --- Tab & AI Agent History Monitoring States ---
const activeTab = ref("logs"); // "logs" or "ai_agents"

interface ChatMessage {
  role: string;
  content: string;
}

interface AgentHistory {
  champion: string;
  history: ChatMessage[];
}

const agentHistories = ref<Record<string, AgentHistory>>({});
const selectedHistoryAgentId = ref<string>("");

let unlistenHistory: UnsubscribeFn | null = null;

onMounted(async () => {
  unlistenHistory = await backendClient.onAgentHistoryUpdated((payload) => {
    console.log("[Debug Page] agent-history-updated", payload);
    const { agent_id, champion, history } = payload;
    const formattedHistory = (history || []).map((msg: any) => {
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

    agentHistories.value[agent_id] = { champion, history: formattedHistory };
    if (!selectedHistoryAgentId.value) {
      selectedHistoryAgentId.value = agent_id;
    }
  });
});

onUnmounted(() => {
  if (unlistenHistory) {
    unlistenHistory();
  }
});
</script>

<template>
  <div class="bg-background flex h-full flex-col gap-3 overflow-hidden p-4">
    <!-- Status Bar -->
    <div class="border-border bg-card flex shrink-0 items-center justify-between rounded border px-3.5 py-2 shadow-sm">
      <div class="flex items-center gap-3">
        <span
          class="inline-flex items-center gap-1.5 rounded border px-2 py-0.5 text-[11px] font-semibold tracking-wider uppercase transition-colors"
          :class="
            connected
              ? 'border-green-500/15 bg-green-500/10 text-green-500'
              : 'text-destructive border-destructive/15 bg-destructive/10'
          "
        >
          <span
            class="h-1.5 w-1.5 rounded-full transition-shadow"
            :class="
              connected
                ? 'bg-green-500 shadow-[0_0_6px_rgba(34,197,94,0.5)]'
                : 'bg-destructive shadow-[0_0_6px_rgba(239,68,68,0.4)]'
            "
          ></span>
          {{ connected ? t("debug.connected") : t("debug.disconnected") }}
        </span>
        <span class="bg-border h-3.5 w-px"></span>
        <span class="flex items-center gap-1.5">
          <span class="text-muted-foreground text-[11px] uppercase">{{ t("debug.currentHero") }}</span>
          <span class="text-foreground text-xs font-semibold">{{ gameState.champion || "—" }}</span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <router-link to="/mock/command">
          <Button
            variant="outline"
            size="sm"
            class="border-primary/20 text-primary hover:border-primary hover:bg-primary/10 bg-primary/5 h-7 text-xs font-medium"
          >
            <CommandIcon class="mr-1 size-3.5" />
            <span>{{ t("debug.commandBed") }}</span>
          </Button>
        </router-link>
        <Button
          variant="outline"
          size="sm"
          class="text-destructive hover:bg-destructive hover:text-destructive-foreground border-destructive/20 bg-destructive/5 h-7 text-xs font-medium"
          @click="stopGame"
        >
          <StopCircleIcon class="mr-1 size-3.5" />
          <span>{{ t("debug.stopMatch") }}</span>
        </Button>
      </div>
    </div>

    <!-- Main Workspace Layout -->
    <div class="flex min-h-0 flex-1 gap-3.5 overflow-hidden">
      <!-- LEFT COLUMN: Global Control Sidebar -->
      <div class="flex min-h-0 w-44 flex-col gap-3 overflow-hidden">
        <!-- Toggles Group -->
        <div class="border-border bg-card flex flex-col gap-1.5 rounded border p-2.5">
          <span class="text-muted-foreground text-[11px] font-semibold uppercase">{{ t("debug.switchLabel") }}</span>
          <div class="flex flex-col gap-1">
            <Button
              variant="outline"
              size="sm"
              class="border-border bg-muted/20 hover:text-foreground hover:border-primary/40 flex h-8 w-full cursor-pointer items-center justify-start gap-1.5 rounded border px-2.5 py-1 text-xs transition-colors"
              :class="{ 'text-primary border-primary bg-primary/10': gameState.godMode }"
              @click="toggleGodMode"
            >
              <span
                class="h-1.5 w-1.5 rounded-full transition-all"
                :class="gameState.godMode ? 'bg-primary' : 'bg-muted-foreground/40'"
              ></span>
              {{ t("debug.godMode") }}
            </Button>
            <Button
              variant="outline"
              size="sm"
              class="border-border bg-muted/20 hover:text-foreground hover:border-primary/40 flex h-8 w-full cursor-pointer items-center justify-start gap-1.5 rounded border px-2.5 py-1 text-xs transition-colors"
              :class="{ 'text-primary border-primary bg-primary/10': gameState.cooldownDisabled }"
              @click="toggleCooldown"
            >
              <span
                class="h-1.5 w-1.5 rounded-full transition-all"
                :class="gameState.cooldownDisabled ? 'bg-primary' : 'bg-muted-foreground/40'"
              ></span>
              {{ t("debug.cooldown") }}
            </Button>
            <Button
              variant="outline"
              size="sm"
              class="border-border bg-muted/20 hover:text-foreground hover:border-primary/40 flex h-8 w-full cursor-pointer items-center justify-start gap-1.5 rounded border px-2.5 py-1 text-xs transition-colors"
              :class="{ 'text-primary border-primary bg-primary/10': gameState.paused }"
              @click="togglePause"
            >
              <span
                class="h-1.5 w-1.5 rounded-full transition-all"
                :class="gameState.paused ? 'bg-primary' : 'bg-muted-foreground/40'"
              ></span>
              {{ gameState.paused ? t("debug.resume") : t("debug.paused") }}
            </Button>
          </div>
        </div>

        <!-- Champion Group -->
        <div class="border-border bg-card flex flex-col gap-1.5 rounded border p-2.5">
          <span class="text-muted-foreground text-[11px] font-semibold uppercase">
            {{ t("debug.championControl") }}
          </span>
          <div class="flex w-full flex-col gap-1.5">
            <Select v-model="switchTarget">
              <SelectTrigger
                class="bg-muted/40 border-border text-foreground hover:border-primary/40 focus:border-primary focus-visible:ring-primary/20 h-8 w-full cursor-pointer px-2 text-xs focus-visible:ring-1"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent class="border-border bg-popover text-foreground">
                <SelectItem v-for="c in champions" :key="c" :value="c" class="cursor-pointer text-xs">
                  {{ c }}
                </SelectItem>
              </SelectContent>
            </Select>
            <Button
              variant="outline"
              size="xs"
              class="text-muted-foreground border-border hover:text-primary hover:border-primary/40 h-8 w-full cursor-pointer rounded bg-transparent px-2.5 py-1 text-xs transition-all"
              @click="switchChampion"
            >
              {{ t("debug.switchChampion") }}
            </Button>
          </div>
        </div>

        <!-- Actions Group -->
        <div class="border-border bg-card flex flex-col gap-1.5 rounded border p-2.5">
          <span class="text-muted-foreground text-[11px] font-semibold uppercase">{{ t("debug.quickActions") }}</span>
          <Button
            variant="outline"
            size="xs"
            class="text-muted-foreground border-border hover:text-primary hover:border-primary/40 h-8 w-full cursor-pointer rounded bg-transparent px-2.5 py-1 text-xs transition-all"
            @click="resetPosition"
          >
            {{ t("debug.resetCoords") }}
          </Button>
        </div>
      </div>

      <!-- RIGHT COLUMN: Game Workspace (Tabs layout) -->
      <div class="border-border bg-card flex min-h-0 flex-1 flex-col overflow-hidden rounded border">
        <!-- Tabs Header -->
        <div class="border-border bg-muted/40 flex shrink-0 items-center justify-between border-b px-4 py-2">
          <div class="flex gap-2">
            <button
              class="rounded px-3 py-1.5 text-xs font-semibold transition-all"
              :class="
                activeTab === 'logs'
                  ? 'bg-primary/10 text-primary border-primary/30 border'
                  : 'text-muted-foreground hover:bg-muted/50 border border-transparent'
              "
              @click="activeTab = 'logs'"
            >
              {{ t("debug.consoleLogsTab") }}
            </button>
            <button
              class="flex items-center gap-1.5 rounded px-3 py-1.5 text-xs font-semibold transition-all"
              :class="
                activeTab === 'ai_agents'
                  ? 'bg-primary/10 text-primary border-primary/30 border'
                  : 'text-muted-foreground hover:bg-muted/50 border border-transparent'
              "
              @click="activeTab = 'ai_agents'"
            >
              <span
                v-if="Object.keys(agentHistories).length > 0"
                class="bg-primary h-1.5 w-1.5 animate-pulse rounded-full"
              ></span>
              {{ t("debug.aiMindTab") }}
            </button>
          </div>
          <Badge variant="outline" class="border-primary/30 text-primary text-[10px]">
            {{ activeTab === "logs" ? t("debug.runningLogsBadge") : t("debug.aiMindBadge") }}
          </Badge>
        </div>

        <!-- Tabs Content -->
        <div class="min-h-0 flex-1 overflow-hidden">
          <!-- Tab 1: Game Console Logs -->
          <div v-show="activeTab === 'logs'" class="flex h-full w-full flex-col overflow-hidden">
            <GameConsoleLogs />
          </div>

          <!-- Tab 2: AI Agent Mind Watcher -->
          <div
            v-show="activeTab === 'ai_agents'"
            class="grid h-full w-full grid-cols-1 gap-4 overflow-hidden p-4 md:grid-cols-4"
          >
            <!-- Left sidebar: Agent Tabs -->
            <div class="border-border flex min-h-0 flex-col gap-2 overflow-y-auto border-r pr-3 md:col-span-1">
              <span class="text-muted-foreground mb-1 text-[10px] font-semibold tracking-wider uppercase">
                {{ t("debug.activeAgents") }}
              </span>
              <div class="flex flex-col gap-1.5">
                <div
                  v-if="Object.keys(agentHistories).length === 0"
                  class="text-muted-foreground py-6 text-center text-xs italic"
                >
                  {{ t("debug.noActiveAgents") }}
                </div>
                <button
                  v-else
                  v-for="(historyData, agentId) in agentHistories"
                  :key="agentId"
                  class="flex w-full cursor-pointer items-center justify-between rounded border p-2 text-left font-sans text-xs transition-colors"
                  :class="
                    selectedHistoryAgentId === agentId
                      ? 'bg-primary/10 border-primary text-primary font-semibold'
                      : 'text-muted-foreground hover:text-foreground hover:bg-muted/40 border-transparent bg-transparent'
                  "
                  @click="selectedHistoryAgentId = agentId"
                >
                  <div class="flex items-center gap-1.5 truncate">
                    <span
                      class="h-1.5 w-1.5 rounded-full"
                      :class="agentId.includes('single') || agentId.includes('riven') ? 'bg-blue-500' : 'bg-red-500'"
                    ></span>
                    <span class="truncate font-medium">{{ historyData.champion }}</span>
                  </div>
                  <Badge
                    variant="outline"
                    class="border-border bg-muted/30 text-muted-foreground px-1.5 py-0 text-[9px]"
                  >
                    {{ t("debug.roundsCount", { count: historyData.history.length }) }}
                  </Badge>
                </button>
              </div>
            </div>

            <!-- Right content: Chat scrolling history -->
            <div class="flex h-full min-h-0 flex-col overflow-hidden md:col-span-3">
              <AgentChatHistory
                :history="selectedHistoryAgentId ? agentHistories[selectedHistoryAgentId]?.history : []"
                :champion="
                  selectedHistoryAgentId
                    ? agentHistories[selectedHistoryAgentId]?.champion || t('debug.defaultAgentName')
                    : t('debug.defaultAgentName')
                "
                :loading="!selectedHistoryAgentId || !agentHistories[selectedHistoryAgentId]"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Custom styled premium glassmorphism shadow */
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
