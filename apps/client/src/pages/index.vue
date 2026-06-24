<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { storeToRefs } from "pinia";
import { useGameStore } from "@/stores/gameStore";
import { useLocale } from "@/composables/useLocale";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { backendClient } from "@/services/backend";
import { PlayIcon, SaveIcon } from "@lucide/vue";
import TeamSlots from "@/components/TeamSlots.vue";
import {
  type Slot,
  emptySlot,
  toBackend,
  fromBackend,
} from "@/composables/useSlotConfig";

// 编排页：只做"从选手预设里选 + 独立的出生点选择"，不做内联深度编辑。
// 每个槽位 = 一个选手预设（英雄+AI配置） + 一个出生点预设。
// 红蓝两阵营结构完全一致，差异只在 accent 配色，复用 TeamSlots 组件。

const store = useGameStore();
const {
  mode,
  isStarting,
  launchError: error,
  selectedScenario,
  heroPresets,
  spawnPresets,
} = storeToRefs(store);
const { ws } = store;
const { startGame, stopGame, loadScenariosList } = store;
const { t, locale } = useLocale();

const blueSlots = ref<Slot[]>([emptySlot()]);
const redSlots = ref<Slot[]>([emptySlot()]);

const currentSceneName = ref("default_scenario");

// 阵营配色：唯一区分红蓝的地方。
const TEAMS = computed(() => ({
  blue: {
    team: {
      label: t("spawnPresets.teamOrder"),
      dot: "bg-blue-500",
      titleText: "text-foreground",
      panel: "bg-card",
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
      label: t("spawnPresets.teamChaos"),
      dot: "bg-red-500",
      titleText: "text-foreground",
      panel: "bg-card",
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
}));

// 各阵营槽位操作的统一封装
function makeHandlers(slotsRef: typeof blueSlots) {
  return {
    add: () => slotsRef.value.push(emptySlot()),
    remove: (idx: number) => slotsRef.value.splice(idx, 1),
  };
}
const blueHandlers = makeHandlers(blueSlots);
const redHandlers = makeHandlers(redSlots);

const validSlots = computed(
  () => blueSlots.value.filter((s) => s.champion).length + redSlots.value.filter((s) => s.champion).length,
);

// --- 场景存取 ---
async function handleLoadScenario(name: string) {
  if (!name) return;
  try {
    const agents = await backendClient.loadCustomScenario(name);
    // 确保预设列表已加载，反向匹配才有意义
    await Promise.all([store.loadHeroPresets(), store.loadSpawnPresets()]);
    blueSlots.value = fromBackend(
      agents.filter((a) => a.team === "Order"),
      heroPresets.value,
      spawnPresets.value,
    );
    redSlots.value = fromBackend(
      agents.filter((a) => a.team === "Chaos"),
      heroPresets.value,
      spawnPresets.value,
    );
    currentSceneName.value = name;
    error.value = "";
    await store.loadWinCondition(name);
  } catch (e: any) {
    error.value = e.message || (typeof e === "string" ? e : t("launcher.loadFailed"));
  }
}

function handleNewScenario() {
  currentSceneName.value =
    locale.value === "zh" ? t("launcher.newScenarioDefaultZh") : t("launcher.newScenarioDefaultEn");
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
    ...toBackend("Order", blueSlots.value, heroPresets.value, spawnPresets.value),
    ...toBackend("Chaos", redSlots.value, heroPresets.value, spawnPresets.value),
  ];
}

async function handleSaveConfig() {
  error.value = "";
  const name = currentSceneName.value.trim();
  if (!name) {
    error.value = t("launcher.errorSceneNameRequired");
    return;
  }
  if (validSlots.value === 0) {
    error.value = t("launcher.errorHeroPresetRequired");
    return;
  }
  try {
    await backendClient.saveCustomScenario(name, buildAgents());
    await store.saveWinCondition(name, store.winCondition);
    await loadScenariosList();
    selectedScenario.value = name;
    error.value = t("launcher.saveSuccess", { name });
  } catch (e: any) {
    error.value = e.message || (typeof e === "string" ? e : t("launcher.saveFailed"));
  }
}

async function handleLaunch() {
  isStarting.value = true;
  error.value = "";
  if (validSlots.value === 0) {
    error.value = t("launcher.errorLaunchHeroPresetRequired");
    isStarting.value = false;
    return;
  }
  const name = currentSceneName.value.trim() || `custom_agents_${Date.now()}`;
  try {
    await backendClient.saveCustomScenario(name, buildAgents());
    await startGame(name);
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message || t("launcher.errorLaunchFailed");
    isStarting.value = false;
  }
}

onMounted(() => {
  loadScenariosList();
  store.loadHeroPresets();
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
      <p class="text-foreground text-sm font-medium tracking-wide">{{ t("launcher.connecting") }}</p>
      <Button
        variant="outline"
        size="sm"
        class="hover:bg-destructive hover:text-destructive-foreground mt-2 h-9 px-8 text-xs"
        @click="stopGame"
      >
        {{ t("launcher.cancelLaunch") }}
      </Button>
    </div>

    <!-- 双阵营并排卡片 -->
    <div class="grid min-h-0 flex-1 grid-cols-1 gap-3 md:grid-cols-2">
      <TeamSlots
        :slots="blueSlots"
        :hero-presets="heroPresets"
        :spawn-presets="spawnPresets"
        :team="TEAMS.blue.team"
        :accent="TEAMS.blue.accent"
        @add="blueHandlers.add"
        @remove="blueHandlers.remove"
      />
      <TeamSlots
        :slots="redSlots"
        :hero-presets="heroPresets"
        :spawn-presets="spawnPresets"
        :team="TEAMS.red.team"
        :accent="TEAMS.red.accent"
        @add="redHandlers.add"
        @remove="redHandlers.remove"
      />
    </div>

    <!-- 合并控制底栏 -->
    <footer
      class="border-border bg-card mt-3 flex shrink-0 flex-wrap items-center justify-between gap-3 rounded-lg border px-4 py-2.5"
    >
      <!-- 左侧控制：对局模式 -->
      <div class="flex flex-wrap items-center gap-4">
        <div class="flex items-center gap-2">
          <span class="text-muted-foreground text-[10px] font-semibold tracking-wider uppercase">
            {{ t("launcher.modeLabel") }}
          </span>
          <div class="bg-muted flex rounded-md p-0.5">
            <button
              class="rounded px-2.5 py-0.5 text-[11px] font-semibold transition-colors"
              :class="mode === 'agent' ? 'bg-card text-foreground' : 'text-muted-foreground'"
              @click="mode = 'agent'"
            >
              {{ t("launcher.modeAgent") }}
            </button>
            <button
              class="rounded px-2.5 py-0.5 text-[11px] font-semibold transition-colors"
              :class="mode === 'sandbox' ? 'bg-card text-foreground' : 'text-muted-foreground'"
              @click="mode = 'sandbox'"
            >
              {{ t("launcher.modeSandbox") }}
            </button>
          </div>
        </div>
      </div>

      <!-- 右侧控制：错误信息、场景配置、保存、启动对局 -->
      <div class="flex flex-wrap items-center gap-3">
        <div v-if="error" class="text-destructive mr-2 text-[11px] font-medium">{{ error }}</div>

        <div class="flex items-center gap-2">
          <span class="text-muted-foreground text-[10px] font-semibold tracking-wider uppercase">
            {{ t("launcher.scenarioLabel") }}
          </span>
          <Input
            v-model="currentSceneName"
            type="text"
            class="border-border bg-muted/50 text-foreground h-7 w-36 rounded px-2 font-mono text-[11px]"
            :placeholder="t('launcher.scenarioPlaceholder')"
          />
          <Button variant="outline" size="xs" class="h-7 gap-1 text-[11px]" @click="handleSaveConfig">
            <SaveIcon class="size-3" />
            {{ t("launcher.saveBtn") }}
          </Button>
        </div>

        <div class="bg-border/60 h-4 w-px" />

        <Button
          size="sm"
          class="bg-primary text-primary-foreground hover:bg-primary/90 h-8 gap-1.5 px-6 font-semibold shadow"
          :disabled="isStarting || validSlots === 0"
          @click="handleLaunch"
        >
          <PlayIcon class="size-3.5" />
          {{ t("launcher.launchBtn") }}
        </Button>
      </div>
    </footer>
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
