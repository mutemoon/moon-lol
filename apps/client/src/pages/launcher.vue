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
import { services } from "@/services/provider";
import { PlayIcon, SaveIcon } from "@lucide/vue";
import TeamSlots from "@/components/TeamSlots.vue";
import SlotCard from "@/components/SlotCard.vue";
import {
  type Slot,
  emptySlot,
  toBackend,
  fromBackend,
} from "@/composables/useSlotConfig";

const store = useGameStore();
const {
  mode,
  isStarting,
  launchError: error,
  selectedScenario,
  heroPresets,
  spawnPresets,
  scenariosList,
} = storeToRefs(store);
const { startGame, loadScenariosList } = store;
const { t, locale } = useLocale();

function handleSelectScenarioInLauncher(s: string) {
  selectedScenario.value = s;
}

const blueSlots = ref<Slot[]>([emptySlot()]);
const redSlots = ref<Slot[]>([emptySlot()]);

const currentSceneName = ref("default_scenario");

const blueLabel = computed(() => t("spawnPresets.teamOrder"));
const redLabel = computed(() => t("spawnPresets.teamChaos"));

const ACCENT = {
  border: "border-border",
  edit: "text-muted-foreground hover:bg-muted hover:text-foreground",
  indexText: "text-muted-foreground",
  inheritBadge: "border-border text-muted-foreground",
};

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
    const agents = await services.loadCustomScenario(name);
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
    await services.saveCustomScenario(name, buildAgents());
    await store.saveWinCondition(name, store.winCondition);
    await loadScenariosList();
    selectedScenario.value = name;
    localStorage.setItem("moon_lol_last_scenario", name);
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
    const agents = buildAgents();
    await services.saveCustomScenario(name, agents);
    await store.saveWinCondition(name, store.winCondition);
    await loadScenariosList();
    await startGame(name, agents);
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message || t("launcher.errorLaunchFailed");
    isStarting.value = false;
  }
}

onMounted(async () => {
  await loadScenariosList();
  await store.loadHeroPresets();
  await store.loadSpawnPresets();

  if (!selectedScenario.value && store.scenariosList.length > 0) {
    selectedScenario.value = store.scenariosList[0]!;
  }
});
</script>

<template>
  <div class="bg-background flex h-full w-full flex-col overflow-hidden p-4">
    <!-- 连接遮罩 -->
    <div
      v-if="isStarting"
      class="bg-background/90 fixed inset-0 z-50 flex flex-col items-center justify-center gap-4 backdrop-blur-md"
    >
      <div class="border-border border-t-primary h-10 w-10 animate-spin rounded-full border-2"></div>
      <p class="text-foreground text-sm font-medium tracking-wide">正在初始化本地仿真对局进程…</p>
    </div>

    <!-- 合并控制顶栏 -->
    <header
      class="border-border bg-card mb-3 flex shrink-0 flex-col gap-3 rounded-lg border p-4 shadow-sm"
    >
      <!-- Top Row: Mode selector, Scenario name & save, Launch button -->
      <div class="flex flex-wrap items-center justify-between gap-3">
        <!-- Left: Game Mode Selector -->
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

        <!-- Right: Error, Scenario Name & Actions -->
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
      </div>

      <!-- Bottom Row: Quick Scenario Templates Buttons List -->
      <div v-if="scenariosList.length > 0" class="flex items-center flex-wrap gap-3 border-t border-border/50 pt-2.5">
        <span class="text-muted-foreground text-[10px] font-semibold tracking-wider uppercase shrink-0">
          快速载入场景模板:
        </span>
        <div class="flex flex-wrap gap-1.5 flex-1">
          <button
            v-for="s in scenariosList"
            :key="s"
            class="h-6 rounded px-2.5 text-[10.5px] font-medium border border-border bg-muted/30 text-foreground-subtle transition-all hover:bg-primary/10 hover:text-primary hover:border-primary/30 cursor-pointer"
            :class="{ 'border-primary text-primary bg-primary/5': currentSceneName === s }"
            @click="handleSelectScenarioInLauncher(s)"
          >
            {{ s }}
          </button>
        </div>
      </div>
    </header>

    <!-- 双阵营并排卡片 -->
    <div class="grid min-h-0 flex-1 grid-cols-1 gap-3 md:grid-cols-2">
      <TeamSlots
        :count="blueSlots.length"
        :label="blueLabel"
        color="blue"
        @add="blueHandlers.add"
      >
        <SlotCard
          v-for="(slot, idx) in blueSlots"
          :key="slot.id"
          :slot="slot"
          :index="idx"
          :hero-presets="heroPresets"
          :spawn-presets="spawnPresets"
          :accent="ACCENT"
          @remove="blueHandlers.remove(idx)"
        />
      </TeamSlots>
      <TeamSlots
        :count="redSlots.length"
        :label="redLabel"
        color="red"
        @add="redHandlers.add"
      >
        <SlotCard
          v-for="(slot, idx) in redSlots"
          :key="slot.id"
          :slot="slot"
          :index="idx"
          :hero-presets="heroPresets"
          :spawn-presets="spawnPresets"
          :accent="ACCENT"
          @remove="redHandlers.remove(idx)"
        />
      </TeamSlots>
    </div>


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
