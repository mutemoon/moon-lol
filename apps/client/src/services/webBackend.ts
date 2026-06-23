import type {
  IBackendClient,
  SpawnPreset,
  AgentPreset,
  HeroPreset,
  AiConfig,
  FrontAgentConfig,
  QueryLogsResult,
  GameConfig,
  WsEvent,
  UnsubscribeFn
} from "./backend";

export class WebBackendClient implements IBackendClient {
  private ws: WebSocket | null = null;
  private wsCallbacks: ((event: WsEvent) => void)[] = [];
  private agentFinishedCallbacks: ((data: { minion_kills: number; gold: number }) => void)[] = [];
  private agentHistoryUpdatedCallbacks: ((data: { agent_id: string; champion: string; history: any[] }) => void)[] = [];

  // Helper for localStorage
  private getStorageItem<T>(key: string, defaultValue: T): T {
    const item = localStorage.getItem(`moon_lol_${key}`);
    return item ? JSON.parse(item) : defaultValue;
  }

  private setStorageItem<T>(key: string, value: T): void {
    localStorage.setItem(`moon_lol_${key}`, JSON.stringify(value));
  }

  // AI Config
  async getAiConfig(): Promise<AiConfig> {
    return this.getStorageItem<AiConfig>("ai_config", {
      api_key: "",
      base_url: "",
      preamble: ""
    });
  }

  async setAiConfig(config: AiConfig): Promise<void> {
    this.setStorageItem("ai_config", config);
  }

  // Spawn Presets
  async listSpawnPresets(): Promise<SpawnPreset[]> {
    return this.getStorageItem<SpawnPreset[]>("spawn_presets", []);
  }

  async saveSpawnPreset(preset: SpawnPreset): Promise<void> {
    const presets = await this.listSpawnPresets();
    const existingIdx = presets.findIndex(p => p.name === preset.name);
    if (existingIdx !== -1) {
      presets[existingIdx] = preset;
    } else {
      presets.push(preset);
    }
    this.setStorageItem("spawn_presets", presets);
  }

  async deleteSpawnPreset(name: string): Promise<void> {
    const presets = await this.listSpawnPresets();
    this.setStorageItem("spawn_presets", presets.filter(p => p.name !== name));
  }

  // Agent Presets
  async listAgentPresets(): Promise<AgentPreset[]> {
    return this.getStorageItem<AgentPreset[]>("agent_presets", []);
  }

  async saveAgentPreset(preset: AgentPreset): Promise<void> {
    const presets = await this.listAgentPresets();
    const existingIdx = presets.findIndex(p => p.name === preset.name);
    if (existingIdx !== -1) {
      presets[existingIdx] = preset;
    } else {
      presets.push(preset);
    }
    this.setStorageItem("agent_presets", presets);
  }

  async deleteAgentPreset(name: string): Promise<void> {
    const presets = await this.listAgentPresets();
    this.setStorageItem("agent_presets", presets.filter(p => p.name !== name));
  }

  // Hero Presets
  async listHeroPresets(): Promise<HeroPreset[]> {
    return this.getStorageItem<HeroPreset[]>("hero_presets", []);
  }

  async saveHeroPreset(preset: HeroPreset): Promise<void> {
    const presets = await this.listHeroPresets();
    const existingIdx = presets.findIndex(p => p.name === preset.name);
    if (existingIdx !== -1) {
      presets[existingIdx] = preset;
    } else {
      presets.push(preset);
    }
    this.setStorageItem("hero_presets", presets);
  }

  async deleteHeroPreset(name: string): Promise<void> {
    const presets = await this.listHeroPresets();
    this.setStorageItem("hero_presets", presets.filter(p => p.name !== name));
  }

  // Custom Scenarios & Win Conditions
  async listCustomScenarios(): Promise<string[]> {
    return this.getStorageItem<string[]>("custom_scenarios_list", []);
  }

  async loadCustomScenario(sceneName: string): Promise<FrontAgentConfig[]> {
    return this.getStorageItem<FrontAgentConfig[]>(`scenario_${sceneName}`, []);
  }

  async saveCustomScenario(sceneName: string, agents: FrontAgentConfig[]): Promise<void> {
    const list = await this.listCustomScenarios();
    if (!list.includes(sceneName)) {
      list.push(sceneName);
      this.setStorageItem("custom_scenarios_list", list);
    }
    this.setStorageItem(`scenario_${sceneName}`, agents);
  }

  async deleteCustomScenario(sceneName: string): Promise<void> {
    const list = await this.listCustomScenarios();
    this.setStorageItem("custom_scenarios_list", list.filter(s => s !== sceneName));
    localStorage.removeItem(`moon_lol_scenario_${sceneName}`);
    localStorage.removeItem(`moon_lol_win_condition_${sceneName}`);
  }

  async loadScenarioWinCondition(sceneName: string): Promise<any> {
    return this.getStorageItem(`win_condition_${sceneName}`, null);
  }

  async saveScenarioWinCondition(sceneName: string, condition: any): Promise<void> {
    this.setStorageItem(`win_condition_${sceneName}`, condition);
  }

  // Game Control
  async startGame(config: GameConfig): Promise<void> {
    console.log("[WebBackend] start_game invoked:", config);
    // In web mode, we assume the game server is already running standalone or launched by some other means.
    // We just simulate success and expect standard WS connect next.
    return Promise.resolve();
  }

  async stopGame(): Promise<void> {
    console.log("[WebBackend] stop_game invoked");
    return Promise.resolve();
  }

  // Game Histories
  async listGameHistories(): Promise<any[]> {
    return this.getStorageItem<any[]>("game_histories", []);
  }

  async getGameHistoryDetail(datetime: string): Promise<any[]> {
    return this.getStorageItem<any[]>(`history_detail_${datetime}`, []);
  }

  async deleteGameHistory(datetime: string): Promise<void> {
    const histories = await this.listGameHistories();
    this.setStorageItem("game_histories", histories.filter(h => h.datetime !== datetime));
    localStorage.removeItem(`moon_lol_history_detail_${datetime}`);
  }

  // Logs Querying
  async queryLogEntities(): Promise<{ entity_id: number | null; entity_name: string | null }[]> {
    return this.getStorageItem("log_entities", []);
  }

  async queryLogCategories(): Promise<{ category: string | null }[]> {
    return this.getStorageItem("log_categories", []);
  }

  async queryLogs(params: {
    offset: number;
    limit: number;
    levels: string[] | null;
    entityId: number | null;
    category: string | null;
    searchText: string | null;
  }): Promise<QueryLogsResult> {
    // Basic local mock querying
    const allLogs = this.getStorageItem<any[]>("logs", []);
    const filtered = allLogs.filter(log => {
      if (params.levels && !params.levels.includes(log.level)) return false;
      if (params.entityId !== null && log.entity_id !== params.entityId) return false;
      if (params.category !== null && log.category !== params.category) return false;
      if (params.searchText && !log.message.includes(params.searchText)) return false;
      return true;
    });

    const offset = params.offset === -1 ? Math.max(0, filtered.length - params.limit) : params.offset;
    const rows = filtered.slice(offset, offset + params.limit);
    return {
      rows,
      total_count: filtered.length
    };
  }

  async clearLogs(): Promise<void> {
    this.setStorageItem("logs", []);
  }

  // WebSocket proxy communication
  async connectWs(): Promise<void> {
    return new Promise((resolve, reject) => {
      // Connect directly to local Bevy server running on default WS port
      const url = "ws://127.0.0.1:9001";
      this.ws = new WebSocket(url);

      this.ws.onopen = () => {
        console.log("[WebBackend] Connected to Bevy WS Server at", url);
        resolve();
      };

      this.ws.onerror = (err) => {
        console.error("[WebBackend] WebSocket connection error:", err);
        reject(err);
      };

      this.ws.onclose = () => {
        console.log("[WebBackend] WebSocket closed");
        this.triggerEvent({ type: "event", event: "game_close", data: {} });
      };

      this.ws.onmessage = (msg) => {
        try {
          const data = JSON.parse(msg.data);
          if (data.type === "event") {
            this.triggerEvent(data as WsEvent);
          } else if (data.type === "result") {
            // Handled via promises if we had request-response tracking,
            // or just broadcast it.
            this.triggerEvent({
              type: "event",
              event: "game_loaded", // simple default mappings
              data: data.data || {}
            });
          }
        } catch (e) {
          console.error("[WebBackend] Failed to parse WebSocket message:", e);
        }
      };
    });
  }

  async disconnectWs(): Promise<void> {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  async sendWsCmd(cmd: string, params: Record<string, any> = {}): Promise<any> {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error("WebSocket not connected");
    }

    const payload = {
      id: Date.now(),
      type: "cmd",
      cmd,
      params
    };

    this.ws.send(JSON.stringify(payload));
    return Promise.resolve(); // Async send, mock response wrapper
  }

  // Event Trigger helpers
  private triggerEvent(event: WsEvent) {
    this.wsCallbacks.forEach(cb => cb(event));
  }

  // Event Listeners
  async onWsEvent(callback: (event: WsEvent) => void): Promise<UnsubscribeFn> {
    this.wsCallbacks.push(callback);
    return () => {
      this.wsCallbacks = this.wsCallbacks.filter(cb => cb !== callback);
    };
  }

  async onAgentFinished(callback: (data: { minion_kills: number; gold: number }) => void): Promise<UnsubscribeFn> {
    this.agentFinishedCallbacks.push(callback);
    return () => {
      this.agentFinishedCallbacks = this.agentFinishedCallbacks.filter(cb => cb !== callback);
    };
  }

  async onAgentHistoryUpdated(callback: (data: { agent_id: string; champion: string; history: any[] }) => void): Promise<UnsubscribeFn> {
    this.agentHistoryUpdatedCallbacks.push(callback);
    return () => {
      this.agentHistoryUpdatedCallbacks = this.agentHistoryUpdatedCallbacks.filter(cb => cb !== callback);
    };
  }
}
