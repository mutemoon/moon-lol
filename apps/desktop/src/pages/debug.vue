<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useGameStore } from "../stores/gameStore";
import GameConsoleLogs from "../components/GameConsoleLogs.vue";
import AgentChatHistory from "../components/AgentChatHistory.vue";
import { Button } from "../components/ui/button";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "../components/ui/select";
import { Badge } from "../components/ui/badge";
import { listen } from "@tauri-apps/api/event";

const store = useGameStore();
const { ws } = store;
const { stopGame } = store;

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

let unlistenHistory: (() => void) | null = null;

onMounted(async () => {
  unlistenHistory = await listen<any>("agent-history-updated", (event) => {
    console.log("[Debug Page] agent-history-updated", event);
    const { agent_id, champion, history } = event.payload;
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
  <div class="bg-bg-deep flex h-full flex-col gap-3 overflow-hidden p-4">
    <!-- Status Bar -->
    <div
      class="bg-bg-surface border-border-subtle flex shrink-0 items-center justify-between rounded border px-3.5 py-2 shadow-[0_1px_2px_rgba(0,0,0,0.4)]"
    >
      <div class="flex items-center gap-3">
        <span
          class="inline-flex items-center gap-1.5 rounded border px-2 py-0.5 text-[11px] font-semibold tracking-wider uppercase transition-colors"
          :class="connected ? 'text-green border-green/15 bg-green/8' : 'text-red border-red/15 bg-red/8'"
        >
          <span
            class="h-1.5 w-1.5 rounded-full transition-shadow"
            :class="
              connected
                ? 'bg-green shadow-[0_0_6px_rgba(74,158,90,0.6)]'
                : 'bg-red shadow-[0_0_6px_rgba(200,74,74,0.4)]'
            "
          ></span>
          {{ connected ? "Connected" : "Disconnected" }}
        </span>
        <span class="bg-border-subtle h-3.5 w-px"></span>
        <span class="flex items-center gap-1.5">
          <span class="text-text-muted text-[11px] uppercase">Champion</span>
          <span class="text-text-bright text-xs font-semibold">{{ gameState.champion || "—" }}</span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <router-link to="/mock/command">
          <Button
            variant="outline"
            size="sm"
            class="border-gold-dimmer/20 text-gold-bright hover:border-gold-default hover:text-gold-glow bg-gold-dimmer/5 h-7 cursor-pointer rounded px-3 py-1 text-xs font-medium transition-all duration-200"
          >
            💻 命令行测试床
          </Button>
        </router-link>
        <Button
          variant="outline"
          size="sm"
          class="text-red hover:text-red hover:bg-red/12 hover:border-red/45 border-red/25 bg-red/4 h-7 cursor-pointer rounded px-3 py-1 text-xs font-medium transition-all duration-200"
          @click="stopGame"
        >
          Stop Game
        </Button>
      </div>
    </div>

    <!-- Main Workspace Layout -->
    <div class="flex min-h-0 flex-1 gap-3.5 overflow-hidden">
      <!-- LEFT COLUMN: Global Control Sidebar -->
      <div class="flex min-h-0 w-44 flex-col gap-3 overflow-hidden">
        <!-- Toggles Group -->
        <div class="bg-bg-surface border-border-subtle flex flex-col gap-1.5 rounded border p-2.5">
          <span class="text-text-muted text-[11px] font-semibold uppercase">Toggles</span>
          <div class="flex flex-col gap-1">
            <Button
              variant="outline"
              size="sm"
              class="text-text-muted bg-bg-deep border-border-subtle hover:text-text-default hover:border-gold-muted flex h-8 w-full cursor-pointer items-center justify-start gap-1.5 rounded px-2.5 py-1 text-xs whitespace-nowrap transition-all duration-200"
              :class="{ 'text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.06)]': gameState.godMode }"
              @click="toggleGodMode"
            >
              <span
                class="h-1.5 w-1.5 rounded-full transition-all"
                :class="
                  gameState.godMode ? 'bg-gold-default shadow-[0_0_6px_rgba(185,145,71,0.5)]' : 'bg-border-default'
                "
              ></span>
              God Mode
            </Button>
            <Button
              variant="outline"
              size="sm"
              class="text-text-muted bg-bg-deep border-border-subtle hover:text-text-default hover:border-gold-muted flex h-8 w-full cursor-pointer items-center justify-start gap-1.5 rounded px-2.5 py-1 text-xs whitespace-nowrap transition-all duration-200"
              :class="{ 'text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.06)]': gameState.cooldownDisabled }"
              @click="toggleCooldown"
            >
              <span
                class="h-1.5 w-1.5 rounded-full transition-all"
                :class="
                  gameState.cooldownDisabled
                    ? 'bg-gold-default shadow-[0_0_6px_rgba(185,145,71,0.5)]'
                    : 'bg-border-default'
                "
              ></span>
              No Cooldown
            </Button>
            <Button
              variant="outline"
              size="sm"
              class="text-text-muted bg-bg-deep border-border-subtle hover:text-text-default hover:border-gold-muted flex h-8 w-full cursor-pointer items-center justify-start gap-1.5 rounded px-2.5 py-1 text-xs whitespace-nowrap transition-all duration-200"
              :class="{ 'text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.06)]': gameState.paused }"
              @click="togglePause"
            >
              <span
                class="h-1.5 w-1.5 rounded-full transition-all"
                :class="
                  gameState.paused ? 'bg-gold-default shadow-[0_0_6px_rgba(185,145,71,0.5)]' : 'bg-border-default'
                "
              ></span>
              {{ gameState.paused ? "Resume" : "Pause" }}
            </Button>
          </div>
        </div>

        <!-- Champion Group -->
        <div class="bg-bg-surface border-border-subtle flex flex-col gap-1.5 rounded border p-2.5">
          <span class="text-text-muted text-[11px] font-semibold uppercase">Champion</span>
          <div class="flex w-full flex-col gap-1.5">
            <Select v-model="switchTarget">
              <SelectTrigger
                class="bg-bg-deep border-gold-dimmer text-text-bright hover:border-gold-muted focus:border-gold-default focus-visible:ring-gold-default/30 h-8 w-full cursor-pointer px-2 text-xs focus-visible:ring-1"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent class="border-border-subtle text-text-bright border bg-[#110e14]">
                <SelectGroup>
                  <SelectItem
                    v-for="c in champions"
                    :key="c"
                    :value="c"
                    class="cursor-pointer text-xs hover:bg-white/[0.04]"
                  >
                    {{ c }}
                  </SelectItem>
                </SelectGroup>
              </SelectContent>
            </Select>
            <Button
              variant="outline"
              size="xs"
              class="text-text-muted border-border-subtle hover:text-gold-bright hover:border-gold-muted h-8 w-full cursor-pointer rounded bg-transparent px-2.5 py-1 text-xs transition-all duration-200"
              @click="switchChampion"
            >
              Switch Champion
            </Button>
          </div>
        </div>

        <!-- Actions Group -->
        <div class="bg-bg-surface border-border-subtle flex flex-col gap-1.5 rounded border p-2.5">
          <span class="text-text-muted text-[11px] font-semibold uppercase">Actions</span>
          <Button
            variant="outline"
            size="xs"
            class="text-text-muted border-border-subtle hover:text-gold-bright hover:border-gold-muted h-8 w-full cursor-pointer rounded bg-transparent px-2.5 py-1 text-xs transition-all duration-200"
            @click="resetPosition"
          >
            Reset Position
          </Button>
        </div>
      </div>

      <!-- RIGHT COLUMN: Game Workspace (Tabs layout) -->
      <div class="bg-bg-surface border-border-subtle flex min-h-0 flex-1 flex-col overflow-hidden rounded border">
        <!-- Tabs Header -->
        <div class="border-border-subtle flex shrink-0 items-center justify-between border-b bg-[#0c0a0e]/40 px-4 py-2">
          <div class="flex gap-2">
            <button
              class="hover:text-text-default rounded px-3 py-1.5 text-xs font-semibold transition-all duration-200"
              :class="
                activeTab === 'logs'
                  ? 'bg-gold-dimmer/15 text-gold-bright border-gold-dimmer/30 border'
                  : 'text-text-muted border border-transparent hover:bg-white/[0.02]'
              "
              @click="activeTab = 'logs'"
            >
              控制台日志 (Console Logs)
            </button>
            <button
              class="hover:text-text-default flex items-center gap-1.5 rounded px-3 py-1.5 text-xs font-semibold transition-all duration-200"
              :class="
                activeTab === 'ai_agents'
                  ? 'bg-gold-dimmer/15 text-gold-bright border-gold-dimmer/30 border'
                  : 'text-text-muted border border-transparent hover:bg-white/[0.02]'
              "
              @click="activeTab = 'ai_agents'"
            >
              <span
                v-if="Object.keys(agentHistories).length > 0"
                class="bg-gold-bright h-1.5 w-1.5 animate-pulse rounded-full"
              ></span>
              AI 决策思维 (AI Mind Watcher)
            </button>
          </div>
          <Badge variant="outline" class="border-gold-dimmer/30 text-gold-bright text-[10px]">
            {{ activeTab === "logs" ? "系统日志" : "实时决策流" }}
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
            <div
              class="border-border-subtle/40 flex min-h-0 flex-col gap-2 overflow-y-auto border-r pr-3 md:col-span-1"
            >
              <span class="text-text-muted mb-1 text-[10px] font-semibold tracking-wider uppercase">活动代理</span>
              <div class="flex flex-col gap-1.5">
                <div
                  v-if="Object.keys(agentHistories).length === 0"
                  class="text-text-muted py-6 text-center text-xs italic"
                >
                  暂无活动代理决策数据
                </div>
                <button
                  v-else
                  v-for="(historyData, agentId) in agentHistories"
                  :key="agentId"
                  class="flex w-full cursor-pointer items-center justify-between rounded border p-2 text-left font-sans text-xs transition-all duration-200"
                  :class="
                    selectedHistoryAgentId === agentId
                      ? 'bg-gold-dimmer/10 border-gold-dimmer text-gold-bright font-semibold'
                      : 'text-text-muted hover:text-text-bright border-transparent bg-transparent hover:bg-white/[0.02]'
                  "
                  @click="selectedHistoryAgentId = agentId"
                >
                  <div class="flex items-center gap-1.5 truncate">
                    <span
                      class="h-1.5 w-1.5 rounded-full"
                      :class="agentId.includes('single') || agentId.includes('riven') ? 'bg-blue-400' : 'bg-red-400'"
                    ></span>
                    <span class="truncate font-medium">{{ historyData.champion }}</span>
                  </div>
                  <Badge
                    variant="outline"
                    class="border-border-subtle bg-bg-deep text-text-muted px-1.5 py-0 text-[9px]"
                  >
                    {{ historyData.history.length }} 轮
                  </Badge>
                </button>
              </div>
            </div>

            <!-- Right content: Chat scrolling history -->
            <div class="flex h-full min-h-0 flex-col overflow-hidden md:col-span-3">
              <AgentChatHistory
                :history="selectedHistoryAgentId && agentHistories[selectedHistoryAgentId] ? agentHistories[selectedHistoryAgentId].history : []"
                :champion="selectedHistoryAgentId && agentHistories[selectedHistoryAgentId] ? agentHistories[selectedHistoryAgentId].champion : 'AI Agent'"
                :loading="!selectedHistoryAgentId || !agentHistories[selectedHistoryAgentId]"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
