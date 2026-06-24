import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  IBackendClient,
  SpawnPreset,
  HeroPreset,
  AiConfig,
  FrontAgentConfig,
  QueryLogsResult,
  GameConfig,
  WsEvent,
  UnsubscribeFn
} from "./backend";

export class TauriBackendClient implements IBackendClient {
  // AI Config
  async getAiConfig(): Promise<AiConfig> {
    return invoke<AiConfig>("get_ai_config");
  }

  async setAiConfig(config: AiConfig): Promise<void> {
    return invoke("set_ai_config", { config });
  }

  // Spawn Presets
  async listSpawnPresets(): Promise<SpawnPreset[]> {
    return invoke<SpawnPreset[]>("list_spawn_presets");
  }

  async saveSpawnPreset(preset: SpawnPreset): Promise<void> {
    return invoke("save_spawn_preset", { preset });
  }

  async deleteSpawnPreset(name: string): Promise<void> {
    return invoke("delete_spawn_preset", { name });
  }



  // Hero Presets
  async listHeroPresets(): Promise<HeroPreset[]> {
    return invoke<HeroPreset[]>("list_hero_presets");
  }

  async saveHeroPreset(preset: HeroPreset): Promise<void> {
    return invoke("save_hero_preset", { preset });
  }

  async deleteHeroPreset(name: string): Promise<void> {
    return invoke("delete_hero_preset", { name });
  }

  // Custom Scenarios & Win Conditions
  async listCustomScenarios(): Promise<string[]> {
    return invoke<string[]>("list_custom_scenarios");
  }

  async loadCustomScenario(sceneName: string): Promise<FrontAgentConfig[]> {
    return invoke<FrontAgentConfig[]>("load_custom_scenario", { sceneName });
  }

  async saveCustomScenario(sceneName: string, agents: FrontAgentConfig[]): Promise<void> {
    return invoke("save_custom_scenario", { sceneName, agents });
  }

  async deleteCustomScenario(sceneName: string): Promise<void> {
    return invoke("delete_custom_scenario", { sceneName });
  }

  async loadScenarioWinCondition(sceneName: string): Promise<any> {
    return invoke<any>("load_scenario_win_condition", { sceneName });
  }

  async saveScenarioWinCondition(sceneName: string, condition: any): Promise<void> {
    return invoke("save_scenario_win_condition", { sceneName, condition });
  }

  // Game Control
  async startGame(config: GameConfig): Promise<void> {
    return invoke("start_game", { config });
  }

  async stopGame(): Promise<void> {
    return invoke("stop_game");
  }

  // Game Histories
  async listGameHistories(): Promise<any[]> {
    return invoke<any[]>("list_game_histories");
  }

  async getGameHistoryDetail(datetime: string): Promise<any[]> {
    return invoke<any[]>("get_game_history_detail", { datetime });
  }

  async deleteGameHistory(datetime: string): Promise<void> {
    return invoke("delete_game_history", { datetime });
  }

  // Logs Querying
  async queryLogEntities(): Promise<{ entity_id: number | null; entity_name: string | null }[]> {
    return invoke<{ entity_id: number | null; entity_name: string | null }[]>("query_log_entities");
  }

  async queryLogCategories(): Promise<{ category: string | null }[]> {
    return invoke<{ category: string | null }[]>("query_log_categories");
  }

  async queryLogs(params: {
    offset: number;
    limit: number;
    levels: string[] | null;
    entityId: number | null;
    category: string | null;
    searchText: string | null;
  }): Promise<QueryLogsResult> {
    return invoke<QueryLogsResult>("query_logs", params);
  }

  async clearLogs(): Promise<void> {
    return invoke("clear_logs");
  }

  // WebSocket proxy communication
  async connectWs(): Promise<void> {
    return invoke("connect_ws");
  }

  async disconnectWs(): Promise<void> {
    return invoke("disconnect_ws");
  }

  async sendWsCmd(cmd: string, params: Record<string, any> = {}): Promise<any> {
    return invoke("send_ws_cmd", { cmd, params });
  }

  // Event Listeners
  async onWsEvent(callback: (event: WsEvent) => void): Promise<UnsubscribeFn> {
    const unlisten = await listen<WsEvent>("ws-event", (event) => {
      callback(event.payload);
    });
    return unlisten;
  }

  async onAgentFinished(callback: (data: { minion_kills: number; gold: number }) => void): Promise<UnsubscribeFn> {
    const unlisten = await listen<{ minion_kills: number; gold: number }>("agent-finished", (event) => {
      callback(event.payload);
    });
    return unlisten;
  }

  async onAgentHistoryUpdated(callback: (data: { agent_id: string; champion: string; history: any[] }) => void): Promise<UnsubscribeFn> {
    const unlisten = await listen<{ agent_id: string; champion: string; history: any[] }>("agent-history-updated", (event) => {
      callback(event.payload);
    });
    return unlisten;
  }
}
