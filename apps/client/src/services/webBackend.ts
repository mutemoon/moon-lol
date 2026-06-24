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

const BASE_URL = import.meta.env.VITE_BASE_URL || "http://localhost:3000";

export class WebBackendClient implements IBackendClient {
  private ws: WebSocket | null = null;
  private wsCallbacks: ((event: WsEvent) => void)[] = [];
  private agentFinishedCallbacks: ((data: { minion_kills: number; gold: number }) => void)[] = [];
  private agentHistoryUpdatedCallbacks: ((data: { agent_id: string; champion: string; history: any[] }) => void)[] = [];

  // Name-to-UUID maps for cached resolution
  private spawnPresetIds = new Map<string, string>();
  private agentPresetIds = new Map<string, string>();
  private heroPresetIds = new Map<string, string>();
  private scenarioIds = new Map<string, string>();

  // Token cache
  private token: string | null = null;

  constructor() {
    this.token = localStorage.getItem("moon_lol_auth_token");
  }

  // Helper for requests
  private async request<T = any>(path: string, options: RequestInit = {}): Promise<T> {
    if (!this.token && !path.startsWith("/api/auth/")) {
      await this.ensureAuth();
    }

    const headers = new Headers(options.headers);
    if (this.token) {
      headers.set("Authorization", `Bearer ${this.token}`);
    }
    headers.set("Content-Type", "application/json");

    const url = `${BASE_URL}${path}`;
    const response = await fetch(url, {
      ...options,
      headers
    });

    if (response.status === 401 && !path.startsWith("/api/auth/")) {
      this.token = null;
      localStorage.removeItem("moon_lol_auth_token");
      await this.ensureAuth();
      headers.set("Authorization", `Bearer ${this.token}`);
      const retryResponse = await fetch(url, {
        ...options,
        headers
      });
      return this.handleResponse<T>(retryResponse);
    }

    return this.handleResponse<T>(response);
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      const errText = await response.text();
      let errMsg = `Request failed: ${response.status}`;
      try {
        const errJson = JSON.parse(errText);
        errMsg = errJson.error?.message || errMsg;
      } catch {
        // ignore
      }
      throw new Error(errMsg);
    }
    const json = await response.json();
    return json.data;
  }

  private async ensureAuth(): Promise<void> {
    const phone = "13800000000";
    const password = "admin_password";
    
    // Try login
    try {
      const res = await fetch(`${BASE_URL}/api/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ phone, password })
      });
      if (res.ok) {
        const data = await res.json();
        this.token = data.data.token;
        localStorage.setItem("moon_lol_auth_token", this.token!);
        return;
      }
    } catch (e) {
      console.warn("[WebBackend] Login failed, attempting auto-register", e);
    }

    // Try register
    const regRes = await fetch(`${BASE_URL}/api/auth/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ phone, password, code: "111111" })
    });
    if (!regRes.ok) {
      throw new Error(`Auto-authentication registration failed: ${regRes.status}`);
    }
    const regData = await regRes.json();
    this.token = regData.data.token;
    localStorage.setItem("moon_lol_auth_token", this.token!);
  }

  // AI Config
  async getAiConfig(): Promise<AiConfig> {
    return this.request<AiConfig>("/api/config");
  }

  async setAiConfig(config: AiConfig): Promise<void> {
    await this.request("/api/config", {
      method: "POST",
      body: JSON.stringify(config)
    });
  }

  // Spawn Presets
  async listSpawnPresets(): Promise<SpawnPreset[]> {
    const list = await this.request<any[]>("/api/spawn-presets");
    return list.map(item => {
      this.spawnPresetIds.set(item.name, item.id);
      return {
        name: item.name,
        x: item.x,
        z: item.z,
        team: item.team === "order" ? "Order" : "Chaos"
      };
    });
  }

  async saveSpawnPreset(preset: SpawnPreset): Promise<void> {
    const id = this.spawnPresetIds.get(preset.name);
    const body = {
      name: preset.name,
      x: preset.x,
      z: preset.z,
      team: preset.team.toLowerCase(),
      visibility: "private"
    };
    if (id) {
      await this.request(`/api/spawn-presets/${id}`, {
        method: "PUT",
        body: JSON.stringify(body)
      });
    } else {
      const created = await this.request("/api/spawn-presets", {
        method: "POST",
        body: JSON.stringify(body)
      });
      this.spawnPresetIds.set(created.name, created.id);
    }
  }

  async deleteSpawnPreset(name: string): Promise<void> {
    const id = this.spawnPresetIds.get(name);
    if (!id) {
      throw new Error(`Spawn preset not found: ${name}`);
    }
    await this.request(`/api/spawn-presets/${id}`, {
      method: "DELETE"
    });
    this.spawnPresetIds.delete(name);
  }

  // Agent Presets
  async listAgentPresets(): Promise<AgentPreset[]> {
    const list = await this.request<any[]>("/api/agent-configs");
    return list.map(item => {
      this.agentPresetIds.set(item.name, item.id);
      return {
        name: item.name,
        agent_type: item.agent_type,
        prompt: item.prompt,
        preamble: item.preamble || undefined,
        model: item.model || undefined
      };
    });
  }

  async saveAgentPreset(preset: AgentPreset): Promise<void> {
    const id = this.agentPresetIds.get(preset.name);
    const body = {
      name: preset.name,
      agent_type: preset.agent_type,
      prompt: preset.prompt,
      preamble: preset.preamble || "",
      model: preset.model || "",
      config_json: {},
      visibility: "private"
    };
    if (id) {
      await this.request(`/api/agent-configs/${id}`, {
        method: "PUT",
        body: JSON.stringify(body)
      });
    } else {
      const created = await this.request("/api/agent-configs", {
        method: "POST",
        body: JSON.stringify(body)
      });
      this.agentPresetIds.set(created.name, created.id);
    }
  }

  async deleteAgentPreset(name: string): Promise<void> {
    const id = this.agentPresetIds.get(name);
    if (!id) {
      throw new Error(`Agent preset not found: ${name}`);
    }
    await this.request(`/api/agent-configs/${id}`, {
      method: "DELETE"
    });
    this.agentPresetIds.delete(name);
  }

  // Hero Presets
  async listHeroPresets(): Promise<HeroPreset[]> {
    await Promise.all([this.listSpawnPresets(), this.listAgentPresets()]);
    
    const list = await this.request<any[]>("/api/agents");
    return list.map(item => {
      this.heroPresetIds.set(item.name, item.id);
      
      let agentPresetName = "";
      for (const [name, id] of this.agentPresetIds.entries()) {
        if (id === item.agent_config_id) {
          agentPresetName = name;
          break;
        }
      }
      let spawnPresetName = "";
      if (item.spawn_preset_id) {
        for (const [name, id] of this.spawnPresetIds.entries()) {
          if (id === item.spawn_preset_id) {
            spawnPresetName = name;
            break;
          }
        }
      }

      return {
        name: item.name,
        champion: item.champion,
        agent_preset_name: agentPresetName,
        spawn_preset_name: spawnPresetName
      };
    });
  }

  async saveHeroPreset(preset: HeroPreset): Promise<void> {
    if (this.agentPresetIds.size === 0) await this.listAgentPresets();
    if (this.spawnPresetIds.size === 0) await this.listSpawnPresets();

    const agent_config_id = this.agentPresetIds.get(preset.agent_preset_name);
    if (!agent_config_id) {
      throw new Error(`Agent preset not found: ${preset.agent_preset_name}`);
    }
    const spawn_preset_id = preset.spawn_preset_name ? this.spawnPresetIds.get(preset.spawn_preset_name) : null;

    const id = this.heroPresetIds.get(preset.name);
    const body = {
      name: preset.name,
      champion: preset.champion,
      agent_config_id,
      spawn_preset_id,
      visibility: "private"
    };

    if (id) {
      await this.request(`/api/agents/${id}`, {
        method: "PUT",
        body: JSON.stringify(body)
      });
    } else {
      const created = await this.request("/api/agents", {
        method: "POST",
        body: JSON.stringify(body)
      });
      this.heroPresetIds.set(created.name, created.id);
    }
  }

  async deleteHeroPreset(name: string): Promise<void> {
    const id = this.heroPresetIds.get(name);
    if (!id) {
      throw new Error(`Hero preset not found: ${name}`);
    }
    await this.request(`/api/agents/${id}`, {
      method: "DELETE"
    });
    this.heroPresetIds.delete(name);
  }

  // Custom Scenarios & Win Conditions
  async listCustomScenarios(): Promise<string[]> {
    const list = await this.request<any[]>("/api/scenarios");
    return list.map(item => {
      this.scenarioIds.set(item.name, item.id);
      return item.name;
    });
  }

  async loadCustomScenario(sceneName: string): Promise<FrontAgentConfig[]> {
    const id = this.scenarioIds.get(sceneName);
    if (!id) {
      await this.listCustomScenarios();
    }
    const activeId = this.scenarioIds.get(sceneName);
    if (!activeId) {
      throw new Error(`Scenario not found: ${sceneName}`);
    }
    const scenario = await this.request<any>(`/api/scenarios/${activeId}`);
    return scenario.agents;
  }

  async saveCustomScenario(sceneName: string, agents: FrontAgentConfig[]): Promise<void> {
    let id = this.scenarioIds.get(sceneName);
    if (!id) {
      await this.listCustomScenarios();
      id = this.scenarioIds.get(sceneName);
    }
    const body = {
      name: sceneName,
      agents
    };
    if (id) {
      await this.request(`/api/scenarios/${id}`, {
        method: "PUT",
        body: JSON.stringify(body)
      });
    } else {
      const created = await this.request("/api/scenarios", {
        method: "POST",
        body: JSON.stringify(body)
      });
      this.scenarioIds.set(created.name, created.id);
    }
  }

  async deleteCustomScenario(sceneName: string): Promise<void> {
    const id = this.scenarioIds.get(sceneName);
    if (!id) {
      throw new Error(`Scenario not found: ${sceneName}`);
    }
    await this.request(`/api/scenarios/${id}`, {
      method: "DELETE"
    });
    this.scenarioIds.delete(sceneName);
  }

  async loadScenarioWinCondition(sceneName: string): Promise<any> {
    const id = this.scenarioIds.get(sceneName);
    if (!id) {
      await this.listCustomScenarios();
    }
    const activeId = this.scenarioIds.get(sceneName);
    if (!activeId) return null;
    return this.request(`/api/scenarios/${activeId}/win-condition`);
  }

  async saveScenarioWinCondition(sceneName: string, condition: any): Promise<void> {
    const id = this.scenarioIds.get(sceneName);
    if (!id) {
      await this.listCustomScenarios();
    }
    const activeId = this.scenarioIds.get(sceneName);
    if (!activeId) {
      throw new Error(`Scenario not found for win condition: ${sceneName}`);
    }
    await this.request(`/api/scenarios/${activeId}/win-condition`, {
      method: "PUT",
      body: JSON.stringify(condition)
    });
  }

  // Game Control
  async startGame(_config: GameConfig): Promise<void> {
    throw new Error("Local battle is not supported in the Web environment.");
  }

  async stopGame(): Promise<void> {
    // noop on web
  }

  // Game Histories (Web environment does not support local histories)
  async listGameHistories(): Promise<any[]> {
    return [];
  }

  async getGameHistoryDetail(_datetime: string): Promise<any[]> {
    return [];
  }

  async deleteGameHistory(_datetime: string): Promise<void> {
    // noop on web
  }

  // Logs Querying (Web environment does not support logs querying)
  async queryLogEntities(): Promise<{ entity_id: number | null; entity_name: string | null }[]> {
    return [];
  }

  async queryLogCategories(): Promise<{ category: string | null }[]> {
    return [];
  }

  async queryLogs(_params: {
    offset: number;
    limit: number;
    levels: string[] | null;
    entityId: number | null;
    category: string | null;
    searchText: string | null;
  }): Promise<QueryLogsResult> {
    return { rows: [], total_count: 0 };
  }

  async clearLogs(): Promise<void> {
    // noop on web
  }

  // WebSocket proxy communication (Not supported in Web environment)
  async connectWs(): Promise<void> {
    throw new Error("Local game WebSocket connection is not supported in the Web environment.");
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
    return Promise.resolve();
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
