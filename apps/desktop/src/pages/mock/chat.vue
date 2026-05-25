<script setup lang="ts">
import { ref, computed } from "vue";
import AgentChatHistory from "../../components/AgentChatHistory.vue";
import mockData from "../../assets/mock.json";
import { Button } from "../../components/ui/button";
import { Badge } from "../../components/ui/badge";

// 从 mock.json 载入数据
const initialPayload = mockData.payload;

// 响应式历史数据，方便用户进行交互调试（如重置或动态追加数据观察渲染）
const currentHistory = ref<any[]>([...initialPayload.history]);
const currentChampion = ref(initialPayload.champion);
const currentAgentId = ref(initialPayload.agent_id);

// 调试交互状态
const newUserText = ref("");
const newAssistantText = ref("");
const simulateLoading = ref(false);

// 统计信息
const totalTurns = computed(() => {
  return Math.floor(currentHistory.value.length / 2);
});

// 重置数据
function resetData() {
  simulateLoading.value = true;
  setTimeout(() => {
    currentHistory.value = [...initialPayload.history];
    simulateLoading.value = false;
  }, 400);
}

// 模拟添加一条用户消息
function addUserMessage() {
  const text = newUserText.value.trim();
  if (!text) return;
  currentHistory.value.push({
    role: "user",
    content: [
      {
        type: "text",
        text: text,
      },
    ],
  });
  newUserText.value = "";
}

// 模拟添加一条AI普通发言
function addAssistantMessage() {
  const text = newAssistantText.value.trim();
  if (!text) return;
  currentHistory.value.push({
    role: "assistant",
    content: [
      {
        text: text,
      },
    ],
  });
  newAssistantText.value = "";
}

// 模拟添加一条包含 Thought + Tool Call 的复杂AI消息
function addComplexAssistantMessage() {
  currentHistory.value.push({
    role: "assistant",
    content: [
      {
        id: null,
        content: [
          {
            type: "text",
            content: {
              text: "我已经分析了最新的敌方小兵坐标。目前锐雯在(2700, 12300)，距离敌方近战兵很近。我需要使用第三段Q技能（折翼之舞）进行补刀，以赚取21金币，并利用走位拉开距离防范敌方的反击。",
              signature: "debug-mock-signature-12345",
            },
          },
        ],
      },
      {
        text: "正在执行补刀操作。三段折翼之舞，斩！",
      },
      {
        id: "call_mock_tool_q_strike",
        function: {
          name: "bash",
          arguments: {
            cmd: "lol_cli action skill 0 2750 12320",
          },
        },
      },
    ],
  });
}

// 模拟添加对应的 Tool Result 消息
function addToolResultMessage() {
  currentHistory.value.push({
    role: "user",
    content: [
      {
        type: "toolresult",
        id: "call_mock_tool_q_strike",
        content: [
          {
            type: "text",
            text: '{\n  "status": "success",\n  "damage_dealt": 84.5,\n  "gold_gained": 21,\n  "current_position": [2712, 12308]\n}',
          },
        ],
      },
    ],
  });
}
</script>

<template>
  <div class="bg-bg-deep flex h-full flex-col gap-4 overflow-hidden p-6 font-sans">
    <!-- Header -->
    <div
      class="bg-bg-surface border-border-subtle relative flex shrink-0 items-center justify-between overflow-hidden rounded-lg border px-5 py-3 shadow-[0_4px_12px_rgba(0,0,0,0.3)]"
    >
      <div
        class="from-gold-muted via-gold-default to-gold-bright absolute top-0 left-0 h-1 w-full bg-gradient-to-r"
      ></div>

      <div class="flex items-center gap-3">
        <div
          class="bg-gold-dimmer/10 border-gold-dimmer/30 flex h-10 w-10 items-center justify-center rounded-lg border"
        >
          <svg class="text-gold-bright h-6 w-6 animate-pulse" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
            />
          </svg>
        </div>
        <div>
          <h1 class="text-text-bright text-base font-bold tracking-wide">对话渲染测试平台</h1>
          <p class="text-text-muted text-[11px]">Chat Rendering Mock Bench & Interactive Debugger</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <Badge variant="outline" class="border-gold-dimmer/20 text-gold-bright bg-gold-dimmer/5 px-2.5 py-1">
          数据源: mock.json
        </Badge>
        <router-link to="/debug">
          <Button
            variant="outline"
            size="sm"
            class="border-border-subtle text-text-muted hover:text-text-bright hover:border-gold-muted h-8 cursor-pointer rounded px-3 text-xs transition-all"
          >
            返回调试页 (Back to Debug)
          </Button>
        </router-link>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="flex min-h-0 flex-1 gap-5 overflow-hidden">
      <!-- LEFT PANEL: Mock Controls -->
      <div class="flex min-h-0 w-80 flex-col gap-4 overflow-y-auto">
        <!-- 1. Mock Agent Info -->
        <div class="bg-bg-surface border-border-subtle flex flex-col gap-3 rounded-lg border p-4 shadow-sm">
          <span
            class="text-text-muted border-border-subtle/30 border-b pb-1.5 text-[10px] font-bold tracking-wider uppercase"
          >
            当前渲染代理 (Mock Agent)
          </span>

          <div class="flex flex-col gap-2">
            <div class="border-border-subtle/30 flex items-center justify-between rounded border bg-[#0d0b0f] p-2.5">
              <div class="flex items-center gap-2">
                <span class="h-2 w-2 rounded-full bg-blue-400"></span>
                <span class="text-text-bright text-xs font-bold">{{ currentChampion }}</span>
              </div>
              <span class="text-text-muted font-mono text-[10px]">{{ currentAgentId }}</span>
            </div>

            <div class="text-text-muted mt-1 grid grid-cols-2 gap-2 text-[10px]">
              <div class="border-border-subtle/10 rounded border bg-[#0c0a0e]/40 p-2">
                <span class="text-text-muted/60 mb-0.5 block">决策周期</span>
                <span class="text-gold-bright font-mono text-xs font-bold">{{ totalTurns }} 轮</span>
              </div>
              <div class="border-border-subtle/10 rounded border bg-[#0c0a0e]/40 p-2">
                <span class="text-text-muted/60 mb-0.5 block">消息总数</span>
                <span class="text-text-bright font-mono text-xs font-bold">{{ currentHistory.length }} 条</span>
              </div>
            </div>

            <Button
              variant="outline"
              size="xs"
              class="hover:bg-gold-dimmer/10 hover:text-gold-bright hover:border-gold-dimmer/50 mt-2 h-7 w-full cursor-pointer text-[11px] font-medium transition-all"
              @click="resetData"
            >
              🔄 重置 Mock 初始数据
            </Button>
          </div>
        </div>

        <!-- 2. Simulation Debugger Actions -->
        <div class="bg-bg-surface border-border-subtle flex flex-1 flex-col gap-4.5 rounded-lg border p-4 shadow-sm">
          <span
            class="text-text-muted border-border-subtle/30 border-b pb-1.5 text-[10px] font-bold tracking-wider uppercase"
          >
            渲染模拟控制器 (Mock Injector)
          </span>

          <!-- 预设的复杂指令 -->
          <div class="flex flex-col gap-2">
            <span class="text-text-muted text-[10px] font-semibold">复杂行为模拟 (Sequential Simulation)</span>
            <Button
              size="xs"
              class="h-8 w-full cursor-pointer justify-start rounded border-0 bg-blue-600 pl-3 text-xs font-semibold text-white transition-all hover:bg-blue-500"
              @click="addComplexAssistantMessage"
            >
              🧠 1. 模拟 AI 复杂决策 (Thought+Tool)
            </Button>
            <Button
              size="xs"
              class="h-8 w-full cursor-pointer justify-start rounded border-0 bg-green-600 pl-3 text-xs font-semibold text-white transition-all hover:bg-green-500"
              @click="addToolResultMessage"
            >
              ⚙️ 2. 模拟返回工具执行结果
            </Button>
          </div>

          <div class="bg-border-subtle/25 h-px"></div>

          <!-- 自定义消息插入 -->
          <div class="flex flex-col gap-3">
            <span class="text-text-muted text-[10px] font-semibold">手动追加普通消息 (Append Custom)</span>

            <!-- 用户环境消息 -->
            <div class="flex flex-col gap-1.5">
              <input
                v-model="newUserText"
                placeholder="输入环境观测/用户消息..."
                class="bg-bg-deep border-border-subtle focus:border-gold-default text-text-bright placeholder:text-text-muted/40 h-8 rounded px-2 text-xs transition-all focus:outline-none"
                @keyup.enter="addUserMessage"
              />
              <Button
                variant="outline"
                size="xs"
                class="text-text-muted hover:text-text-bright hover:border-gold-muted h-7 w-full cursor-pointer text-[10px]"
                @click="addUserMessage"
              >
                ➕ 插入用户/环境指令
              </Button>
            </div>

            <!-- AI 回复消息 -->
            <div class="mt-1 flex flex-col gap-1.5">
              <input
                v-model="newAssistantText"
                placeholder="输入 AI 决策文本发言..."
                class="bg-bg-deep border-border-subtle focus:border-gold-default text-text-bright placeholder:text-text-muted/40 h-8 rounded px-2 text-xs transition-all focus:outline-none"
                @keyup.enter="addAssistantMessage"
              />
              <Button
                variant="outline"
                size="xs"
                class="text-text-muted hover:text-text-bright hover:border-gold-muted h-7 w-full cursor-pointer text-[10px]"
                @click="addAssistantMessage"
              >
                ➕ 插入 AI 普通发言
              </Button>
            </div>
          </div>
        </div>
      </div>

      <!-- RIGHT PANEL: Chat History Renderer -->
      <div
        class="bg-bg-surface border-border-subtle relative flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border shadow-lg"
      >
        <!-- Renderer Header -->
        <div
          class="border-border-subtle flex shrink-0 items-center justify-between border-b bg-[#0c0a0e]/40 px-4.5 py-3.5"
        >
          <div class="flex items-center gap-2">
            <span class="bg-green h-2 w-2 animate-pulse rounded-full"></span>
            <span class="text-text-bright text-xs font-bold tracking-wide">
              组件渲染实时观察面板 (Component Renderer)
            </span>
          </div>
          <Badge
            variant="outline"
            class="border-gold-dimmer/20 text-gold-bright bg-gold-dimmer/5 px-2 py-0.5 text-[9px]"
          >
            AgentChatHistory.vue
          </Badge>
        </div>

        <!-- Render Component -->
        <div class="min-h-0 flex-1 overflow-hidden p-5">
          <AgentChatHistory
            :history="currentHistory"
            :champion="currentChampion"
            :loading="simulateLoading"
            placeholder-text="正在等待重新加载 Mock 对话历史流..."
          />
        </div>
      </div>
    </div>
  </div>
</template>
