import { ref, watch } from "vue";
import { defineStore } from "pinia";
import { useWsClient } from "../composables/useWsClient";
import { createLogContext } from "../composables/useLogPoller";
import { router } from "../router";
import { backendClient } from "../services/backend";
import type { SpawnPreset, AgentPreset, HeroPreset } from "../services/backend";

export type { SpawnPreset, AgentPreset, HeroPreset };

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

export const BUILTIN_AGENT_PRESETS: AgentPreset[] = [
  {
    name: "激进压制 (LLM)",
    agent_type: "llm",
    prompt: "你是一个激进、好斗的玩家。在对线中主动寻找敌方的破绽，频繁消耗并伺机斩杀对手。",
  },
  {
    name: "稳健发育 (LLM)",
    agent_type: "llm",
    prompt: "你是一个冷静、稳健的玩家。优先保证补刀与发育，在取得装备或等级优势后再寻找击杀机会。",
  },
  {
    name: "游走支援 (LLM)",
    agent_type: "llm",
    prompt: "你是一个重视全局的玩家。在对线之余寻找游走支援队友的机会，用人数优势建立胜势。",
  },
];

export const BUILTIN_HERO_PRESETS: HeroPreset[] = [
  {
    name: "锐雯 · 激进压制",
    champion: "Riven",
    agent_preset_name: "激进压制 (LLM)",
    spawn_preset_name: "上路一塔前 (秩序方)",
  },
  {
    name: "菲奥娜 · 稳健发育",
    champion: "Fiora",
    agent_preset_name: "稳健发育 (LLM)",
    spawn_preset_name: "上路一塔前 (混沌方)",
  },
];

export const useGameStore = defineStore("game", () => {
  const champion = ref(localStorage.getItem("moon_lol_last_champion") || "Riven");
  const mode = ref(localStorage.getItem("moon_lol_last_mode") || "sandbox");
  const launchError = ref("");
  const isStarting = ref(false);
  const showStatsModal = ref(false);
  const statsResult = ref({ minionKills: 0, gold: 0.0 });

  const ws = useWsClient();
  const log = createLogContext();

  const champions = ["Riven", "Fiora"];

  const scenariosList = ref<string[]>([]);
  const histories = ref<any[]>([]);
  const selectedScenario = ref("");
  const selectedHistoryDatetime = ref("");

  // 出生点预设：纯文件持久化，无运行时副作用，立即可用。
  const spawnPresets = ref<SpawnPreset[]>([]);

  async function loadSpawnPresets() {
    try {
      const presets = await backendClient.listSpawnPresets();
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
    await backendClient.saveSpawnPreset(preset);
    await loadSpawnPresets();
  }

  async function deleteSpawnPreset(name: string) {
    await backendClient.deleteSpawnPreset(name);
    await loadSpawnPresets();
  }

  // Agent 预设：可复用的 Agent 资产（英雄 + 类型 + Prompt + 模型/前言）。
  // 在编排页通过下拉框选择；深度配置在 Agent 预设管理页完成。
  // 编排页保存场景时，由前端把所选预设展开为具体 champion/prompt/agent_type 写入场景。
  const agentPresets = ref<AgentPreset[]>([]);

  async function loadAgentPresets() {
    try {
      const presets = await backendClient.listAgentPresets();
      const merged = [...presets];
      for (const b of BUILTIN_AGENT_PRESETS) {
        if (!merged.some((p) => p.name === b.name)) {
          merged.push(b);
        }
      }
      agentPresets.value = merged;
    } catch (e) {
      console.error("加载 Agent 预设失败", e);
      agentPresets.value = [...BUILTIN_AGENT_PRESETS];
    }
  }

  async function saveAgentPreset(preset: AgentPreset) {
    await backendClient.saveAgentPreset(preset);
    await loadAgentPresets();
  }

  async function deleteAgentPreset(name: string) {
    await backendClient.deleteAgentPreset(name);
    await loadAgentPresets();
  }

  // 英雄预设：编排页槽位的唯一选择单元。
  // 内部绑定英雄 + Agent 预设（大脑）+ 出生点预设（坐标）。
  const heroPresets = ref<HeroPreset[]>([]);

  async function loadHeroPresets() {
    try {
      const presets = await backendClient.listHeroPresets();
      const merged = [...presets];
      for (const b of BUILTIN_HERO_PRESETS) {
        if (!merged.some((p) => p.name === b.name)) {
          merged.push(b);
        }
      }
      heroPresets.value = merged;
    } catch (e) {
      console.error("加载英雄预设失败", e);
      heroPresets.value = [...BUILTIN_HERO_PRESETS];
    }
  }

  async function saveHeroPreset(preset: HeroPreset) {
    await backendClient.saveHeroPreset(preset);
    await loadHeroPresets();
  }

  async function deleteHeroPreset(name: string) {
    await backendClient.deleteHeroPreset(name);
    await loadHeroPresets();
  }

  // 胜利条件：按场景独立持久化。评估逻辑尚未接入运行时（当前对局仍以 120s 计时结束），
  // 此处仅负责存取结构，便于用户配置与跨会话保留。
  const winCondition = ref<any>(null);

  async function loadWinCondition(sceneName: string) {
    try {
      winCondition.value = await backendClient.loadScenarioWinCondition(sceneName);
    } catch (e) {
      console.error("加载胜利条件失败", e);
      winCondition.value = null;
    }
  }

  async function saveWinCondition(sceneName: string, condition: any) {
    if (!sceneName.trim()) return;
    await backendClient.saveScenarioWinCondition(sceneName, condition);
  }

  async function loadScenariosList() {
    try {
      scenariosList.value = await backendClient.listCustomScenarios();
    } catch (e) {
      console.error("加载自定义场景列表失败", e);
    }
  }

  async function loadHistoriesList() {
    try {
      histories.value = await backendClient.listGameHistories();
    } catch (e) {
      console.error("加载游戏历史记录失败", e);
    }
  }

  // Sync log filter when ws selected entity changes
  watch(
    () => ws.selectedEntityId.value,
    (entityId) => {
      if (entityId !== null) {
        log.setEntityFilter(entityId);
      }
    },
  );

  watch(champion, (val) => {
    localStorage.setItem("moon_lol_last_champion", val);
  });

  watch(mode, (val) => {
    localStorage.setItem("moon_lol_last_mode", val);
  });

  async function startGame(sceneName?: string) {
    launchError.value = "";
    isStarting.value = true;

    try {
      await backendClient.startGame({
        mode: mode.value,
        champion: champion.value,
        sceneName: sceneName || null,
      });
    } catch (e: any) {
      launchError.value = typeof e === "string" ? e : e.message || "Unknown error";
      isStarting.value = false;
      return;
    }

    ws.connect().then(() => {
      log.start();
      isStarting.value = false;
      router.push("/debug");
    });
  }

  function stopGame() {
    ws.disconnect();
    log.stop();
    backendClient.stopGame().catch(() => {});
    router.push("/");
  }

  return {
    champion,
    mode,
    launchError,
    isStarting,
    showStatsModal,
    statsResult,
    ws,
    log,
    champions,
    scenariosList,
    histories,
    selectedScenario,
    selectedHistoryDatetime,
    spawnPresets,
    loadSpawnPresets,
    saveSpawnPreset,
    deleteSpawnPreset,
    agentPresets,
    loadAgentPresets,
    saveAgentPreset,
    deleteAgentPreset,
    heroPresets,
    loadHeroPresets,
    saveHeroPreset,
    deleteHeroPreset,
    winCondition,
    loadWinCondition,
    saveWinCondition,
    loadScenariosList,
    loadHistoriesList,
    startGame,
    stopGame,
  };
});
