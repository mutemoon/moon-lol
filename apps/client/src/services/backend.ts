// ── backend.ts (兼容层) ──
// 重新导出新架构的类型和服务，保持现有消费者的 import 路径不变
// 最终目标是消费者直接 import services/provider，此文件作为过渡

import { initServices, services } from "./provider";

// ── 类型重导出（保持现有 import 兼容） ──
export type {
  AiConfig,
  FrontAgentConfig,
  GameConfig,
  HeroPreset,
  LogRow,
  QueryLogsResult,
  SpawnPreset,
  UnsubscribeFn,
  WsEvent,
  WsResponse,
} from "./types";

// ── 兼容接口：混合了本地和云端方法的 facade ──
// 现有消费者通过 backendClient.xxx() 调用，这个 Proxy 自动路由到 local 或 cloud

export interface IBackendClient {
  // AI Config
  getAiConfig(): Promise<any>;
  setAiConfig(config: any): Promise<void>;

  // Spawn Presets
  listSpawnPresets(): Promise<any[]>;
  saveSpawnPreset(preset: any): Promise<void>;
  deleteSpawnPreset(name: string): Promise<void>;

  // Hero Presets
  listHeroPresets(): Promise<any[]>;
  saveHeroPreset(preset: any): Promise<void>;
  deleteHeroPreset(name: string): Promise<void>;

  // Custom Scenarios & Win Conditions
  listCustomScenarios(): Promise<string[]>;
  loadCustomScenario(sceneName: string): Promise<any[]>;
  saveCustomScenario(sceneName: string, agents: any[]): Promise<void>;
  deleteCustomScenario(sceneName: string): Promise<void>;
  loadScenarioWinCondition(sceneName: string): Promise<any>;
  saveScenarioWinCondition(sceneName: string, condition: any): Promise<void>;

  // Game Control
  startGame(config: any): Promise<void>;
  stopGame(): Promise<void>;

  // Game Histories
  listGameHistories(): Promise<any[]>;
  getGameHistoryDetail(datetime: string): Promise<any[]>;
  deleteGameHistory(datetime: string): Promise<void>;

  // Logs Querying
  queryLogEntities(): Promise<any[]>;
  queryLogCategories(): Promise<any[]>;
  queryLogs(params: any): Promise<any>;
  clearLogs(): Promise<void>;

  // WebSocket
  connectWs(): Promise<void>;
  disconnectWs(): Promise<void>;
  sendWsCmd(cmd: string, params?: Record<string, any>): Promise<any>;

  // Events
  onWsEvent(callback: (event: any) => void): Promise<() => void>;
  onAgentFinished(callback: (data: any) => void): Promise<() => void>;
  onAgentHistoryUpdated(callback: (data: any) => void): Promise<() => void>;
}

// ── backendClient 兼容代理 ──
// 所有本地独占功能路由到 services.local，预设 CRUD 根据在线状态路由

export const backendClient: IBackendClient = new Proxy({} as IBackendClient, {
  get(_, prop: string | symbol) {
    // Promise thenable 检测、Symbol 属性等透传
    if (typeof prop === "symbol" || prop === "then" || prop === "toJSON") {
      return undefined;
    }

    // 本地独占功能 → services.local
    const localOnlyMethods = [
      "startGame",
      "stopGame",
      "connectWs",
      "disconnectWs",
      "sendWsCmd",
      "queryLogEntities",
      "queryLogCategories",
      "queryLogs",
      "clearLogs",
      "onWsEvent",
      "onAgentFinished",
      "onAgentHistoryUpdated",
    ];
    if (localOnlyMethods.includes(prop)) {
      console.log(prop);

      return (...args: any[]) => (services.local as any)[prop](...args);
    }

    // 使用旧 Tauri invoke 的预设/场景 CRUD → 通过 local 读写（保持离线行为）
    // 现阶段保持和原来一样的本地行为，登录后的云端迁移在后续 PR 处理
    const legacyLocalMethods: Record<string, string> = {
      getAiConfig: "getAiConfig",
      setAiConfig: "setAiConfig",
      listSpawnPresets: "listSpawnPresets",
      saveSpawnPreset: "saveSpawnPreset",
      deleteSpawnPreset: "deleteSpawnPreset",
      listHeroPresets: "listHeroPresets",
      saveHeroPreset: "saveHeroPreset",
      deleteHeroPreset: "deleteHeroPreset",
      listCustomScenarios: "listCustomScenarios",
      loadCustomScenario: "loadCustomScenario",
      saveCustomScenario: "saveCustomScenario",
      deleteCustomScenario: "deleteCustomScenario",
      loadScenarioWinCondition: "loadScenarioWinCondition",
      saveScenarioWinCondition: "saveScenarioWinCondition",
      listGameHistories: "listGameHistories",
      getGameHistoryDetail: "getGameHistoryDetail",
      deleteGameHistory: "deleteGameHistory",
    };

    if (prop in legacyLocalMethods) {
      // Desktop 环境：通过 Tauri invoke（保持原有行为）
      if (services.isDesktop) {
        return async (...args: any[]) => {
          const { invoke } = await import("@tauri-apps/api/core");
          const cmdMap: Record<string, string> = {
            getAiConfig: "get_ai_config",
            setAiConfig: "set_ai_config",
            listSpawnPresets: "list_spawn_presets",
            saveSpawnPreset: "save_spawn_preset",
            deleteSpawnPreset: "delete_spawn_preset",
            listHeroPresets: "list_hero_presets",
            saveHeroPreset: "save_hero_preset",
            deleteHeroPreset: "delete_hero_preset",
            listCustomScenarios: "list_custom_scenarios",
            loadCustomScenario: "load_custom_scenario",
            saveCustomScenario: "save_custom_scenario",
            deleteCustomScenario: "delete_custom_scenario",
            loadScenarioWinCondition: "load_scenario_win_condition",
            saveScenarioWinCondition: "save_scenario_win_condition",
            listGameHistories: "list_game_histories",
            getGameHistoryDetail: "get_game_history_detail",
            deleteGameHistory: "delete_game_history",
          };
          const tauriCmd = cmdMap[prop];
          if (!tauriCmd) {
            throw new Error(`Tauri command mapping not found for method ${String(prop)}`);
          }
          // 参数映射
          const paramMap: Record<string, (a: any[]) => any> = {
            set_ai_config: (a) => ({ config: a[0] }),
            save_spawn_preset: (a) => ({ preset: a[0] }),
            delete_spawn_preset: (a) => ({ name: a[0] }),
            save_hero_preset: (a) => ({ preset: a[0] }),
            delete_hero_preset: (a) => ({ name: a[0] }),
            load_custom_scenario: (a) => ({ sceneName: a[0] }),
            save_custom_scenario: (a) => ({ sceneName: a[0], agents: a[1] }),
            delete_custom_scenario: (a) => ({ sceneName: a[0] }),
            load_scenario_win_condition: (a) => ({ sceneName: a[0] }),
            save_scenario_win_condition: (a) => ({ sceneName: a[0], condition: a[1] }),
            get_game_history_detail: (a) => ({ datetime: a[0] }),
            delete_game_history: (a) => ({ datetime: a[0] }),
          };
          const params = paramMap[tauriCmd]?.(args);
          return params ? invoke(tauriCmd, params) : invoke(tauriCmd);
        };
      }
      // Web 环境：走云端（保持原有 WebBackendClient 行为）
      // 后续可以在这里做在线/离线自动切换
    }

    throw new Error(`[backendClient] Method "${prop}" not implemented in compatibility layer`);
  },
});

// ── 旧的初始化函数兼容 ──
export async function getBackendClient(): Promise<IBackendClient> {
  await initServices();
  return backendClient;
}
