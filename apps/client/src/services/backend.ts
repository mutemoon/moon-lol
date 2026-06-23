// Frontend-Backend Interface Definitions

export interface SpawnPreset {
  name: string;
  x: number;
  z: number;
  team: string;
}

export interface AgentPreset {
  name: string;
  agent_type: string;
  prompt: string;
  preamble?: string;
  model?: string;
}

export interface HeroPreset {
  name: string;
  champion: string;
  agent_preset_name: string;
  spawn_preset_name: string;
}

export interface AiConfig {
  api_key: string;
  base_url: string;
  preamble: string;
}

export interface FrontAgentConfig {
  id?: string;
  champion: string;
  team: string;
  prompt: string;
  spawn_point: number[];
  agent_type: string;
}

// WS Protocol Types
export interface WsResponse {
  id: number;
  type: "result";
  ok: boolean;
  data?: any;
  error?: string;
}

export interface WsEvent {
  type: "event";
  event: "game_loaded" | "game_paused" | "champion_changed" | "game_close" | "entity_selected";
  data: Record<string, any>;
}

// Log Types
export interface LogRow {
  id: number;
  timestamp: number;
  level: string;
  file: string | null;
  line: number | null;
  entity_id: number | null;
  entity_name: string | null;
  category: string | null;
  message: string;
}

export interface QueryLogsResult {
  rows: LogRow[];
  total_count: number;
}

export interface GameConfig {
  mode: string;
  champion: string;
  sceneName: string | null;
}

export type UnsubscribeFn = () => void;

// Unified Backend Client Interface
export interface IBackendClient {
  // AI Config
  getAiConfig(): Promise<AiConfig>;
  setAiConfig(config: AiConfig): Promise<void>;

  // Spawn Presets
  listSpawnPresets(): Promise<SpawnPreset[]>;
  saveSpawnPreset(preset: SpawnPreset): Promise<void>;
  deleteSpawnPreset(name: string): Promise<void>;

  // Agent Presets
  listAgentPresets(): Promise<AgentPreset[]>;
  saveAgentPreset(preset: AgentPreset): Promise<void>;
  deleteAgentPreset(name: string): Promise<void>;

  // Hero Presets
  listHeroPresets(): Promise<HeroPreset[]>;
  saveHeroPreset(preset: HeroPreset): Promise<void>;
  deleteHeroPreset(name: string): Promise<void>;

  // Custom Scenarios & Win Conditions
  listCustomScenarios(): Promise<string[]>;
  loadCustomScenario(sceneName: string): Promise<FrontAgentConfig[]>;
  saveCustomScenario(sceneName: string, agents: FrontAgentConfig[]): Promise<void>;
  deleteCustomScenario(sceneName: string): Promise<void>;
  loadScenarioWinCondition(sceneName: string): Promise<any>;
  saveScenarioWinCondition(sceneName: string, condition: any): Promise<void>;

  // Game Control
  startGame(config: GameConfig): Promise<void>;
  stopGame(): Promise<void>;

  // Game Histories
  listGameHistories(): Promise<any[]>;
  getGameHistoryDetail(datetime: string): Promise<any[]>;
  deleteGameHistory(datetime: string): Promise<void>;

  // Logs Querying
  queryLogEntities(): Promise<{ entity_id: number | null; entity_name: string | null }[]>;
  queryLogCategories(): Promise<{ category: string | null }[]>;
  queryLogs(params: {
    offset: number;
    limit: number;
    levels: string[] | null;
    entityId: number | null;
    category: string | null;
    searchText: string | null;
  }): Promise<QueryLogsResult>;
  clearLogs(): Promise<void>;

  // WebSocket proxy communication (via Tauri Backend or direct)
  connectWs(): Promise<void>;
  disconnectWs(): Promise<void>;
  sendWsCmd(cmd: string, params?: Record<string, any>): Promise<any>;

  // App-level window event subscriptions
  onWsEvent(callback: (event: WsEvent) => void): Promise<UnsubscribeFn>;
  onAgentFinished(callback: (data: { minion_kills: number; gold: number }) => void): Promise<UnsubscribeFn>;
  onAgentHistoryUpdated(callback: (data: { agent_id: string; champion: string; history: any[] }) => void): Promise<UnsubscribeFn>;
}

// Detect Tauri environment
const isTauri = typeof window !== "undefined" && ((window as any).__TAURI__ !== undefined || (window as any).__TAURI_INTERNALS__ !== undefined) && (window.IS_DESKTOP !== false);

let client: IBackendClient;

export async function getBackendClient(): Promise<IBackendClient> {
  if (client) return client;

  if (isTauri) {
    const { TauriBackendClient } = await import("./tauriBackend");
    client = new TauriBackendClient();
  } else {
    const { WebBackendClient } = await import("./webBackend");
    client = new WebBackendClient();
  }
  return client;
}

// Single instance export (pre-resolved if synchronously required, fallback to lazy resolved)
export const backendClient: IBackendClient = new Proxy({} as IBackendClient, {
  get(_, prop) {
    if (!client) {
      // Synchronous access check. In most Vue files it will be called inside hooks or handlers,
      // where getBackendClient() has already been initialized on mount.
      throw new Error(`[BackendClient] Client not initialized yet. Please await getBackendClient() first.`);
    }
    return (client as any)[prop];
  }
});
