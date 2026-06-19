<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { storeToRefs } from "pinia";
import { useGameStore } from "../stores/gameStore";
import { Button } from "../components/ui/button";
import { Badge } from "../components/ui/badge";
import { Input } from "../components/ui/input";
import { invoke } from "@tauri-apps/api/core";
import { PlayIcon, SaveIcon, SparklesIcon } from "@lucide/vue";
import TeamSlots from "../components/TeamSlots.vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "../components/ui/dialog";
import {
  type Slot,
  emptySlot,
  toBackend,
  fromBackend,
  uniquePresetName,
  rebindSlot,
} from "../composables/useSlotConfig";

// 编排页（产品文档 §3.0 / §3.1.1）：只做"从预设里选 + 临时覆盖"，不做内联深度编辑。
// 每个槽位 = 一个英雄预设（内含英雄 + Agent 预设 + 出生点预设）。
// 拖入后 Agent / 出生点选择器始终展开，初始显示继承值；改任一项即解绑（dirty），
// 槽位提供「编辑」(跳管理页原地编辑) 与「存为新预设」(把临时配置固化成新预设)。
// 红蓝两阵营结构完全一致，差异只在 accent 配色，复用 TeamSlots 组件。

const store = useGameStore();
const {
  mode,
  isStarting,
  launchError: error,
  selectedScenario,
  heroPresets,
  agentPresets,
  spawnPresets,
} = storeToRefs(store);
const { ws } = store;
const { startGame, stopGame, loadScenariosList } = store;

const blueSlots = ref<Slot[]>([emptySlot()]);
const redSlots = ref<Slot[]>([emptySlot()]);

const currentSceneName = ref("default_scenario");

// 阵营配色：唯一区分红蓝的地方。
const TEAMS = {
  blue: {
    team: {
      label: "Order / 秩序",
      dot: "bg-muted-foreground/60",
      titleText: "text-foreground",
      panel: "border-blue-500 bg-card",
      divider: "border-border",
      countBadge: "border-border text-muted-foreground",
      addButton: "text-muted-foreground hover:bg-muted hover:text-foreground",
    },
    accent: {
      border: "border-border",
      edit: "text-muted-foreground hover:bg-muted hover:text-foreground",
      indexText: "text-muted-foreground",
      inheritBadge: "border-border text-muted-foreground",
    },
  },
  red: {
    team: {
      label: "Chaos / 混沌",
      dot: "bg-muted-foreground/60",
      titleText: "text-foreground",
      panel: "border-red-500 bg-card",
      divider: "border-border",
      countBadge: "border-border text-muted-foreground",
      addButton: "text-muted-foreground hover:bg-muted hover:text-foreground",
    },
    accent: {
      border: "border-border",
      edit: "text-muted-foreground hover:bg-muted hover:text-foreground",
      indexText: "text-muted-foreground",
      inheritBadge: "border-border text-muted-foreground",
    },
  },
} as const;


// 各阵营槽位操作的统一封装（避免模板里写两遍）
function makeHandlers(slotsRef: typeof blueSlots) {
  return {
    add: () => slotsRef.value.push(emptySlot()),
    remove: (idx: number) => slotsRef.value.splice(idx, 1),
  };
}
const blueHandlers = makeHandlers(blueSlots);
const redHandlers = makeHandlers(redSlots);

const totalSlots = computed(() => blueSlots.value.length + redSlots.value.length);
const validSlots = computed(
  () => blueSlots.value.filter((s) => s.champion).length + redSlots.value.filter((s) => s.champion).length,
);

// --- 场景存取 ---
async function handleLoadScenario(name: string) {
  if (!name) return;
  try {
    const agents = await invoke<any[]>("load_custom_scenario", { sceneName: name });
    // 确保预设列表已加载，反向匹配才有意义
    await Promise.all([store.loadHeroPresets(), store.loadAgentPresets(), store.loadSpawnPresets()]);
    blueSlots.value = fromBackend(
      agents.filter((a) => a.team === "Order"),
      heroPresets.value,
      agentPresets.value,
      spawnPresets.value,
    );
    redSlots.value = fromBackend(
      agents.filter((a) => a.team === "Chaos"),
      heroPresets.value,
      agentPresets.value,
      spawnPresets.value,
    );
    currentSceneName.value = name;
    error.value = "";
    await store.loadWinCondition(name);
  } catch (e: any) {
    error.value = e.message || (typeof e === "string" ? e : "无法加载指定的配置");
  }
}

function handleNewScenario() {
  currentSceneName.value = "new_scenario";
  blueSlots.value = [emptySlot()];
  redSlots.value = [emptySlot()];
  store.winCondition = null;
  error.value = "";
}

watch(
  () => selectedScenario.value,
  (newVal) => {
    if (newVal) handleLoadScenario(newVal);
    else handleNewScenario();
  },
  { immediate: true },
);

function buildAgents() {
  return [
    ...toBackend("Order", blueSlots.value, agentPresets.value, spawnPresets.value),
    ...toBackend("Chaos", redSlots.value, agentPresets.value, spawnPresets.value),
  ];
}

async function handleSaveConfig() {
  error.value = "";
  const name = currentSceneName.value.trim();
  if (!name) {
    error.value = "请输入场景命名！";
    return;
  }
  if (validSlots.value === 0) {
    error.value = "每个槽位需选择一个英雄预设";
    return;
  }
  try {
    await invoke("save_custom_scenario", { sceneName: name, agents: buildAgents() });
    await store.saveWinCondition(name, store.winCondition);
    await loadScenariosList();
    selectedScenario.value = name;
    error.value = `配置 "${name}" 保存成功！`;
  } catch (e: any) {
    error.value = e.message || (typeof e === "string" ? e : "保存配置失败");
  }
}

async function handleLaunch() {
  isStarting.value = true;
  error.value = "";
  if (validSlots.value === 0) {
    error.value = "请至少为一个槽位选择英雄预设！";
    isStarting.value = false;
    return;
  }
  const name = currentSceneName.value.trim() || `custom_agents_${Date.now()}`;
  try {
    await invoke("save_custom_scenario", { sceneName: name, agents: buildAgents() });
    await startGame(name);
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message || "无法拉起自定义 AI 对战";
    isStarting.value = false;
  }
}

// 「存为新预设」Dialog
const showSaveAsDialog = ref(false);
const saveAsTarget = ref<Slot | null>(null);
const saveAsName = ref("");
const saveAsError = ref("");

function openSaveAs(slot: Slot) {
  saveAsTarget.value = slot;
  // 预填命名：原预设名优先；否则 英雄 · Agent类型
  const base =
    slot.heroPresetName ||
    `${slot.champion || "英雄"} · ${
      agentPresets.value.find((p) => p.name === slot.agentPresetName)?.agent_type.toUpperCase() ?? "LLM"
    }`;
  saveAsName.value = uniquePresetName(`${base} · 副本`, heroPresets.value);
  saveAsError.value = "";
  showSaveAsDialog.value = true;
}

async function confirmSaveAs() {
  if (!saveAsTarget.value) return;
  saveAsError.value = "";
  const name = saveAsName.value.trim();
  if (!name) {
    saveAsError.value = "请填写预设名称";
    return;
  }
  const slot = saveAsTarget.value;
  try {
    await store.saveHeroPreset({
      name,
      champion: slot.champion || "Riven",
      agent_preset_name: slot.agentPresetName,
      spawn_preset_name: slot.spawnPresetName,
    });
    // 回填：槽位重新绑定到新预设，恢复"继承"态
    rebindSlot(slot, name);
    showSaveAsDialog.value = false;
    saveAsTarget.value = null;
  } catch (e: any) {
    saveAsError.value = e.message || (typeof e === "string" ? e : "保存失败");
  }
}

onMounted(() => {
  loadScenariosList();
  store.loadHeroPresets();
  store.loadAgentPresets();
  store.loadSpawnPresets();
});
</script>

<template>
  <div class="bg-background flex h-full w-full flex-col overflow-hidden p-4">
    <!-- 连接遮罩 -->
    <div
      v-if="ws.connecting"
      class="bg-background/90 fixed inset-0 z-50 flex flex-col items-center justify-center gap-4 backdrop-blur-md"
    >
      <div class="border-border border-t-primary h-10 w-10 animate-spin rounded-full border-2"></div>
      <p class="text-foreground text-sm font-medium tracking-wide">正在连接 Bevy 游戏进程...</p>
      <Button
        variant="outline"
        size="sm"
        class="hover:bg-destructive hover:text-destructive-foreground mt-2 h-9 px-8 text-xs"
        @click="stopGame"
      >
        取消并终止进程
      </Button>
    </div>

    <!-- 顶部紧凑 Header：场景命名 + 保存 -->
    <header
      class="border-border bg-card flex shrink-0 flex-wrap items-center justify-between gap-3 rounded-lg border px-4 py-2.5 shadow-sm"
    >
      <div class="flex items-center gap-2.5">
        <div class="from-primary to-primary/60 flex size-8 items-center justify-center rounded-lg bg-gradient-to-tr">
          <SparklesIcon class="text-primary-foreground size-4" />
        </div>
        <div class="flex items-baseline gap-2">
          <h1 class="text-foreground text-sm font-bold tracking-tight">对局编排</h1>
          <Badge variant="secondary" class="text-[10px]">{{ validSlots }}/{{ totalSlots }} 槽位</Badge>
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <span class="text-muted-foreground text-[10px] font-semibold tracking-wider uppercase">场景</span>
        <Input
          v-model="currentSceneName"
          type="text"
          class="border-border bg-muted/50 text-foreground h-7 w-44 rounded px-2 font-mono text-[11px]"
          placeholder="scenario name"
        />
        <Button variant="outline" size="xs" class="h-7 gap-1 text-[11px]" @click="handleSaveConfig">
          <SaveIcon class="size-3" />
          保存
        </Button>
      </div>
    </header>

    <!-- 双阵营并排卡片（结构一致，配色由 TEAMS 注入） -->
    <div class="mt-3 grid min-h-0 flex-1 grid-cols-1 gap-3 md:grid-cols-2">
      <TeamSlots
        :slots="blueSlots"
        :hero-presets="heroPresets"
        :agent-presets="agentPresets"
        :spawn-presets="spawnPresets"
        :team="TEAMS.blue.team"
        :accent="TEAMS.blue.accent"
        @add="blueHandlers.add"
        @remove="blueHandlers.remove"
        @save-as="openSaveAs"
      />
      <TeamSlots
        :slots="redSlots"
        :hero-presets="heroPresets"
        :agent-presets="agentPresets"
        :spawn-presets="spawnPresets"
        :team="TEAMS.red.team"
        :accent="TEAMS.red.accent"
        @add="redHandlers.add"
        @remove="redHandlers.remove"
        @save-as="openSaveAs"
      />
    </div>

    <!-- 底栏：模式 + 启动 -->
    <footer
      class="border-border bg-card mt-3 flex shrink-0 items-center justify-between gap-3 rounded-lg border px-4 py-2.5 shadow-sm"
    >
      <div class="flex flex-wrap items-center gap-3">
        <span class="text-muted-foreground text-[10px] font-semibold tracking-wider uppercase">模式</span>
        <div class="bg-muted flex rounded-md p-0.5">
          <button
            class="rounded px-2.5 py-0.5 text-[11px] font-semibold transition-colors"
            :class="mode === 'agent' ? 'bg-card text-foreground shadow-sm' : 'text-muted-foreground'"
            @click="mode = 'agent'"
          >
            AI 决策
          </button>
          <button
            class="rounded px-2.5 py-0.5 text-[11px] font-semibold transition-colors"
            :class="mode === 'sandbox' ? 'bg-card text-foreground shadow-sm' : 'text-muted-foreground'"
            @click="mode = 'sandbox'"
          >
            沙盒
          </button>
        </div>
        <div v-if="error" class="text-destructive text-[11px] font-medium">{{ error }}</div>
      </div>
      <Button
        size="sm"
        class="bg-primary text-primary-foreground hover:bg-primary/90 h-8 gap-1.5 px-6 font-semibold shadow"
        :disabled="isStarting || validSlots === 0"
        @click="handleLaunch"
      >
        <PlayIcon class="size-3.5" />
        启动对局
      </Button>
    </footer>

    <!-- 「存为新预设」Dialog -->
    <Dialog :open="showSaveAsDialog" @update:open="(v) => (showSaveAsDialog = v)">
      <DialogContent class="border-border bg-card text-foreground max-w-sm p-6">
        <DialogHeader>
          <DialogTitle class="text-foreground text-sm">将临时配置存为新英雄预设</DialogTitle>
          <DialogDescription class="text-muted-foreground text-[11px]">
            当前槽位已与原预设解绑。保存后槽位会重新绑定到这个新预设，原预设不会被修改。
          </DialogDescription>
        </DialogHeader>
        <div class="flex flex-col gap-2 py-1">
          <label class="text-muted-foreground text-[10px] font-semibold tracking-wider uppercase">新预设名称</label>
          <Input
            v-model="saveAsName"
            placeholder="如：锐雯 · 激进压制 · 副本"
            class="border-border bg-muted/40 h-9 text-sm"
          />
          <div v-if="saveAsTarget" class="text-muted-foreground mt-1 font-mono text-[10px]">
            {{ saveAsTarget.champion }} · {{ saveAsTarget.agentPresetName || "（无大脑）" }} ·
            {{ saveAsTarget.spawnPresetName || "（默认出生点）" }}
          </div>
          <div v-if="saveAsError" class="text-destructive text-[11px] font-medium">{{ saveAsError }}</div>
        </div>
        <DialogFooter class="gap-2">
          <Button variant="outline" size="sm" @click="showSaveAsDialog = false">取消</Button>
          <Button size="sm" class="gap-1.5" @click="confirmSaveAs">
            <SaveIcon class="size-3.5" />
            保存为新预设
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>

<style scoped>
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
