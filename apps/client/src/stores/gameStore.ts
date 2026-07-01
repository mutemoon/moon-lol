import { ref, watch } from "vue";
import { defineStore } from "pinia";
import { router } from "../router";
import { services } from "../services/provider";
import { isDesktop } from "@/lib/utils";
import type { SpawnPreset, HeroPreset, RunningGame, FrontAgentConfig } from "../services/types";
import { useProviders } from "./providersStore";

export type { SpawnPreset, HeroPreset };

export const BUILTIN_SPAWN_PRESETS: SpawnPreset[] = [
  {
    name: "上路一塔前 (秩序方)",
    x: 1981,
    z: 11441,
    team: "Order",
  },
  {
    name: "上路一塔前 (混沌方)",
    x: 3318,
    z: 12875,
    team: "Chaos",
  },
];

export const useGameStore = defineStore("game", () => {
  const champion = ref(localStorage.getItem("moon_lol_last_champion") || "Riven");
  const mode = ref(localStorage.getItem("moon_lol_last_mode") || "sandbox");
  const launchError = ref("");
  const isStarting = ref(false);
  const showStatsModal = ref(false);
  const statsResult = ref({ minionKills: 0, gold: 0.0 });

  const runningGames = ref<RunningGame[]>([]);
  const currentGameId = ref<string | null>(null);
  const currentGamePort = ref<number | null>(null);

  const champions = ref(["Riven", "Fiora"]);

  const scenariosList = ref<string[]>([]);
  const histories = ref<any[]>([]);
  const selectedScenario = ref("");
  const selectedHistoryDatetime = ref("");

  // 出生点预设：纯文件持久化，无运行时副作用，立即可用。
  const spawnPresets = ref<SpawnPreset[]>([]);

  async function loadSpawnPresets() {
    try {
      const presets = await services.listSpawnPresets();
      const merged = [...presets];
      for (const b of BUILTIN_SPAWN_PRESETS) {
        if (!merged.some((p) => p.name === b.name)) {
          merged.push(b);
        }
      }
      spawnPresets.value = merged;
    } catch (e) {
      console.error("加载出生点预设失败", e);
      spawnPresets.value = [...BUILTIN_SPAWN_PRESETS];
    }
  }

  async function saveSpawnPreset(preset: SpawnPreset) {
    await services.saveSpawnPreset(preset);
    await loadSpawnPresets();
  }

  async function deleteSpawnPreset(name: string) {
    await services.deleteSpawnPreset(name);
    await loadSpawnPresets();
  }

  // 选手预设（我的选手）：直接由英雄名、配置类型与具体的 Prompt/策略组成。
  const heroPresets = ref<HeroPreset[]>([]);

  async function loadHeroPresets() {
    try {
      heroPresets.value = await services.listHeroPresets();
    } catch (e) {
      console.error("加载选手预设失败", e);
      heroPresets.value = [];
    }
  }

  async function saveHeroPreset(preset: HeroPreset) {
    await services.saveHeroPreset(preset);
    await loadHeroPresets();
  }

  async function deleteHeroPreset(name: string) {
    await services.deleteHeroPreset(name);
    await loadHeroPresets();
  }

  // 胜利条件：按场景独立持久化。评估逻辑尚未接入运行时（当前对局仍以 120s 计时结束），
  // 此处仅负责存取结构，便于用户配置与跨会话保留。
  const winCondition = ref<any>(null);

  async function loadWinCondition(sceneName: string) {
    try {
      winCondition.value = await services.loadScenarioWinCondition(sceneName);
    } catch (e) {
      console.error("加载胜利条件失败", e);
      winCondition.value = null;
    }
  }

  async function saveWinCondition(sceneName: string, condition: any) {
    if (!sceneName.trim()) return;
    await services.saveScenarioWinCondition(sceneName, condition);
  }

  async function loadScenariosList() {
    try {
      scenariosList.value = await services.listCustomScenarios();
    } catch (e) {
      console.error("加载自定义场景列表失败", e);
    }
  }

  async function loadHistoriesList() {
    try {
      histories.value = await services.listGameHistories();
    } catch (e) {
      console.error("加载游戏历史记录失败", e);
    }
  }

  async function refreshRunningGames() {
    try {
      runningGames.value = await services.local.listRunningGames();
    } catch (e) {
      console.error("加载运行中对局失败", e);
    }
  }

  // Desktop automatic polling of running games
  if (isDesktop) {
    setInterval(refreshRunningGames, 3000);
  }

  watch(champion, (val) => {
    localStorage.setItem("moon_lol_last_champion", val);
  });

  watch(mode, (val) => {
    localStorage.setItem("moon_lol_last_mode", val);
  });

  async function startGame(sceneName?: string, customAgents?: FrontAgentConfig[]) {
    launchError.value = "";
    isStarting.value = true;

    try {
      const providersStore = useProviders();
      await providersStore.load();

      let agents = customAgents;
      if (!agents && sceneName) {
        agents = await services.loadCustomScenario(sceneName);
      }

      const res = await services.local.startGame({
        mode: mode.value,
        champion: champion.value,
        sceneName: sceneName || null,
        agents: agents || [],
        providers: providersStore.providers,
      });
      currentGameId.value = res.id;
      currentGamePort.value = res.port;
      isStarting.value = false;
      await refreshRunningGames();
      router.push(`/debug/${res.id}`);
    } catch (e: any) {
      launchError.value = typeof e === "string" ? e : e.message || "Unknown error";
      isStarting.value = false;
    }
  }

  async function stopGame(id: string) {
    try {
      await services.local.stopGame(id);
      if (currentGameId.value === id) {
        currentGameId.value = null;
        currentGamePort.value = null;
      }
      await refreshRunningGames();
    } catch (e: any) {
      console.error("停止游戏失败", e);
    }
  }

  return {
    champion,
    mode,
    launchError,
    isStarting,
    showStatsModal,
    statsResult,
    runningGames,
    currentGameId,
    currentGamePort,
    champions,
    scenariosList,
    histories,
    selectedScenario,
    selectedHistoryDatetime,
    spawnPresets,
    loadSpawnPresets,
    saveSpawnPreset,
    deleteSpawnPreset,
    heroPresets,
    loadHeroPresets,
    saveHeroPreset,
    deleteHeroPreset,
    winCondition,
    loadWinCondition,
    saveWinCondition,
    loadScenariosList,
    loadHistoriesList,
    refreshRunningGames,
    startGame,
    stopGame,
  };
});
