<route lang="yaml">
meta:
  layout: desktop
</route>

<script setup lang="ts">
import { ref } from "vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useLocale } from "@/composables/useLocale";
import AgentChatHistory from "@/components/AgentChatHistory.vue";

const { t } = useLocale();

const currentChampion = ref("Riven");
const currentAgentId = ref("mock-agent-12345");
const totalTurns = ref(2);
const simulateLoading = ref(false);

const newUserText = ref("");
const newAssistantText = ref("");

interface ChatMessage {
  role: string;
  content: string;
}

const initialHistory: ChatMessage[] = [
  {
    role: "user",
    content: `【系统观测数据】
英雄状态: { hp: 550, max_hp: 600, attack_power: 64, level: 1 }
附近敌方小兵:
  - { id: 101, hp: 45, max_hp: 300, position: [2650, 12870] }
  - { id: 102, hp: 120, max_hp: 300, position: [2680, 12890] }`,
  },
  {
    role: "assistant",
    content: `思考过程：
1. 观察到附近有两个敌方小兵。
2. 小兵 101 当前生命值为 45，低于我的基础攻击力 64。
3. 符合补刀规则 (hp <= attack_power)。
4. 我应该立即对其发起普攻进行补刀，获取金币。

决策动作：
lol_cli action -e mock-agent-12345 attack 101`,
  },
];

const currentHistory = ref<ChatMessage[]>([...initialHistory]);

function resetData() {
  currentHistory.value = [...initialHistory];
  totalTurns.value = 2;
}

function addUserMessage() {
  const text = newUserText.value.trim();
  if (!text) return;
  currentHistory.value.push({
    role: "user",
    content: text,
  });
  newUserText.value = "";
}

function addAssistantMessage() {
  const text = newAssistantText.value.trim();
  if (!text) return;
  currentHistory.value.push({
    role: "assistant",
    content: text,
  });
  newAssistantText.value = "";
  totalTurns.value++;
}

function addComplexAssistantMessage() {
  const complexText = `思考过程：
1. 观测返回：自身 HP=180 (低于 30% 安全线)。
2. 敌方小兵 103 处于攻击距离，但安全优先，必须立即后撤。
3. 撤退目标点选择己方防御塔防区 (1500, 2000)。

决策动作：
lol_cli action -e mock-agent-12345 move 1500 2000`;

  currentHistory.value.push({
    role: "assistant",
    content: complexText,
  });
  totalTurns.value++;
}

function addToolResultMessage() {
  const toolResult = `【执行工具 BashTool 结果】
stdout:
{
  "status": "success",
  "damage_dealt": 84.5,
  "gold_gained": 21,
  "current_position": [2712, 12308]
}
stderr: ""`;

  currentHistory.value.push({
    role: "user",
    content: toolResult,
  });
}
</script>

<template>
  <div class="bg-background flex h-full w-full flex-col gap-4 overflow-hidden p-6 font-sans">
    <!-- Header -->
    <div
      class="bg-card border-border relative flex shrink-0 items-center justify-between overflow-hidden rounded-lg border px-5 py-3 shadow-sm"
    >
      <div class="from-primary/40 via-primary to-primary/80 absolute top-0 left-0 h-1 w-full bg-gradient-to-r"></div>

      <div class="flex items-center gap-3">
        <div class="bg-primary/10 border-primary/30 flex h-10 w-10 items-center justify-center rounded-lg border">
          <svg class="text-primary h-6 w-6 animate-pulse" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
            />
          </svg>
        </div>
        <div>
          <h1 class="text-foreground text-base font-bold tracking-wide">{{ t("mock.chat.title") }}</h1>
          <p class="text-muted-foreground text-[11px]">{{ t("mock.chat.subtitle") }}</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <Badge variant="outline" class="border-primary/20 text-primary bg-primary/5 px-2.5 py-1">
          {{ t("mock.chat.dataSource") }}: mock.json
        </Badge>
        <router-link to="/debug">
          <Button
            variant="outline"
            size="sm"
            class="border-border text-muted-foreground hover:text-foreground hover:border-primary/40 h-8 cursor-pointer rounded px-3 text-xs transition-all"
          >
            {{ t("mock.chat.backBtn") }}
          </Button>
        </router-link>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="flex min-h-0 flex-1 gap-5 overflow-hidden">
      <!-- LEFT PANEL: Mock Controls -->
      <div class="flex min-h-0 w-80 flex-col gap-4 overflow-y-auto">
        <!-- 1. Mock Agent Info -->
        <div class="bg-card border-border flex flex-col gap-3 rounded-lg border p-4 shadow-sm">
          <span
            class="text-muted-foreground border-border border-b pb-1.5 text-[10px] font-bold tracking-wider uppercase"
          >
            {{ t("mock.chat.currentAgent") }}
          </span>

          <div class="flex flex-col gap-2">
            <div class="border-border bg-muted/40 flex items-center justify-between rounded border p-2.5">
              <div class="flex items-center gap-2">
                <span class="h-2 w-2 rounded-full bg-blue-500"></span>
                <span class="text-foreground text-xs font-bold">{{ currentChampion }}</span>
              </div>
              <span class="text-muted-foreground font-mono text-[10px]">{{ currentAgentId }}</span>
            </div>

            <div class="text-muted-foreground mt-1 grid grid-cols-2 gap-2 text-[10px]">
              <div class="border-border bg-muted/30 rounded border p-2">
                <span class="text-muted-foreground mb-0.5 block">{{ t("mock.chat.decisionCycle") }}</span>
                <span class="text-primary font-mono text-xs font-bold">
                  {{ t("mock.chat.turnsCount", { count: totalTurns }) }}
                </span>
              </div>
              <div class="border-border bg-muted/30 rounded border p-2">
                <span class="text-muted-foreground mb-0.5 block">{{ t("mock.chat.messagesCount") }}</span>
                <span class="text-foreground font-mono text-xs font-bold">
                  {{ t("mock.chat.messagesCount", { count: currentHistory.length }) }}
                </span>
              </div>
            </div>

            <Button
              variant="outline"
              size="xs"
              class="hover:bg-primary/10 hover:text-primary hover:border-primary/50 mt-2 h-7 w-full cursor-pointer text-[11px] font-medium transition-all"
              @click="resetData"
            >
              {{ t("mock.chat.resetBtn") }}
            </Button>
          </div>
        </div>

        <!-- 2. Simulation Debugger Actions -->
        <div class="bg-card border-border flex flex-1 flex-col gap-4.5 rounded-lg border p-4 shadow-sm">
          <span
            class="text-muted-foreground border-border border-b pb-1.5 text-[10px] font-bold tracking-wider uppercase"
          >
            {{ t("mock.chat.injectorTitle") }}
          </span>

          <!-- Preset Complex Actions -->
          <div class="flex flex-col gap-2">
            <span class="text-muted-foreground text-[10px] font-semibold">{{ t("mock.chat.complexSim") }}</span>
            <Button
              size="xs"
              class="h-8 w-full cursor-pointer justify-start rounded border-0 bg-blue-600 pl-3 text-xs font-semibold text-white transition-all hover:bg-blue-500"
              @click="addComplexAssistantMessage"
            >
              {{ t("mock.chat.simDecisionBtn") }}
            </Button>
            <Button
              size="xs"
              class="h-8 w-full cursor-pointer justify-start rounded border-0 bg-green-600 pl-3 text-xs font-semibold text-white transition-all hover:bg-green-500"
              @click="addToolResultMessage"
            >
              {{ t("mock.chat.simResultBtn") }}
            </Button>
          </div>

          <div class="bg-border h-px"></div>

          <!-- Custom Messages Insertion -->
          <div class="flex flex-col gap-3">
            <span class="text-muted-foreground text-[10px] font-semibold">{{ t("mock.chat.manualAdd") }}</span>

            <!-- User environment message -->
            <div class="flex flex-col gap-1.5">
              <input
                v-model="newUserText"
                :placeholder="t('mock.chat.inputUserPlaceholder')"
                class="bg-muted/40 border-border focus:border-primary text-foreground placeholder:text-muted-foreground/40 h-8 rounded px-2 text-xs transition-all focus:outline-none"
                @keyup.enter="addUserMessage"
              />
              <Button
                variant="outline"
                size="xs"
                class="text-muted-foreground hover:text-foreground hover:border-primary/40 h-7 w-full cursor-pointer text-[10px]"
                @click="addUserMessage"
              >
                {{ t("mock.chat.insertUserBtn") }}
              </Button>
            </div>

            <!-- AI response message -->
            <div class="mt-1 flex flex-col gap-1.5">
              <input
                v-model="newAssistantText"
                :placeholder="t('mock.chat.inputAiPlaceholder')"
                class="bg-muted/40 border-border focus:border-primary text-foreground placeholder:text-muted-foreground/40 h-8 rounded px-2 text-xs transition-all focus:outline-none"
                @keyup.enter="addAssistantMessage"
              />
              <Button
                variant="outline"
                size="xs"
                class="text-muted-foreground hover:text-foreground hover:border-primary/40 h-7 w-full cursor-pointer text-[10px]"
                @click="addAssistantMessage"
              >
                {{ t("mock.chat.insertAiBtn") }}
              </Button>
            </div>
          </div>
        </div>
      </div>

      <!-- RIGHT PANEL: Chat History Renderer -->
      <div
        class="bg-card border-border relative flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border shadow-lg"
      >
        <!-- Renderer Header -->
        <div class="border-border bg-muted/30 flex shrink-0 items-center justify-between border-b px-4.5 py-3.5">
          <div class="flex items-center gap-2">
            <span class="h-2 w-2 animate-pulse rounded-full bg-green-500"></span>
            <span class="text-foreground text-xs font-bold tracking-wide">
              {{ t("mock.chat.observerPanel") }}
            </span>
          </div>
          <Badge variant="outline" class="border-primary/20 text-primary bg-primary/5 px-2 py-0.5 text-[9px]">
            AgentChatHistory.vue
          </Badge>
        </div>

        <!-- Render Component -->
        <div class="min-h-0 flex-1 overflow-hidden p-5">
          <AgentChatHistory
            :history="currentHistory"
            :champion="currentChampion"
            :loading="simulateLoading"
            :placeholder-text="t('mock.chat.loadingPlaceholder')"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Scrollbar styles inside mock view */
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
