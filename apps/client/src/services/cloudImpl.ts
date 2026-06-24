// ── CloudServiceImpl ──
// ICloudService 实现：合并原 webBackend.ts 的预设 CRUD + cloudApi.ts 的房间/Rank/精粹等
// Desktop 和 Web 共用，通过 fetch() 直接调用 lol_web_server REST API

import type { ICloudService } from './cloud'
import type {
  AiConfig,
  Agent,
  CreateAgentDto,
  UpdateAgentDto,
  AgentSnapshot,
  SpawnPreset,
  CreateSpawnPresetDto,
  UpdateSpawnPresetDto,
  Scenario,
  CreateScenarioDto,
  UpdateScenarioDto,
  GameHistorySummary,
  SavedAgentHistory,
  Room,
  RoomConstraints,
  RoomAgentSlot,
  Team,
  Match,
  MatchEvent,
  RankQueueEntry,
  EloRating,
  Season,
  EssenceTransaction,
  CheckInResult,
  BillingPlan,
  AdminMetrics,
  AuthToken,
  Visibility,
  WinCondition,
} from './types'

const TOKEN_KEY = 'moon_lol_auth_token'

export class CloudServiceImpl implements ICloudService {
  private baseUrl: string
  private token: string | null = null

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || (import.meta as any).env?.VITE_BASE_URL || 'http://localhost:3000'
    this.token = typeof localStorage !== 'undefined' ? localStorage.getItem(TOKEN_KEY) : null
  }

  // ── HTTP 请求基础设施 ──

  private async request<T = any>(path: string, options: RequestInit = {}): Promise<T> {
    const headers = new Headers(options.headers)
    if (this.token) {
      headers.set('Authorization', `Bearer ${this.token}`)
    }
    if (options.body && !headers.has('Content-Type')) {
      headers.set('Content-Type', 'application/json')
    }

    const url = `${this.baseUrl}${path}`
    const response = await fetch(url, { ...options, headers })

    if (response.status === 401 && !path.startsWith('/api/auth/')) {
      this.token = null
      localStorage.removeItem(TOKEN_KEY)
      throw new Error('Authentication required')
    }

    return this.handleResponse<T>(response)
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    const text = await response.text()
    let parsed: any = null
    try {
      parsed = text ? JSON.parse(text) : null
    } catch {
      // non-JSON
    }
    if (!response.ok) {
      const msg = parsed?.error?.message || `HTTP ${response.status}`
      throw new Error(msg)
    }
    return parsed?.data as T
  }

  // ── Auth ──

  async login(phone: string, password: string): Promise<AuthToken> {
    const res = await this.request<AuthToken>('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify({ phone, password }),
    })
    this.token = res.token
    localStorage.setItem(TOKEN_KEY, res.token)
    return res
  }

  async codeLogin(phone: string, code: string): Promise<AuthToken> {
    const res = await this.request<AuthToken>('/api/auth/code-login', {
      method: 'POST',
      body: JSON.stringify({ phone, code }),
    })
    this.token = res.token
    localStorage.setItem(TOKEN_KEY, res.token)
    return res
  }

  async register(phone: string, password: string, code: string): Promise<AuthToken> {
    const res = await this.request<AuthToken>('/api/auth/register', {
      method: 'POST',
      body: JSON.stringify({ phone, password, code }),
    })
    this.token = res.token
    localStorage.setItem(TOKEN_KEY, res.token)
    return res
  }

  async resetPassword(phone: string, code: string, newPassword: string): Promise<void> {
    await this.request('/api/auth/reset-password', {
      method: 'POST',
      body: JSON.stringify({ phone, code, new_password: newPassword }),
    })
  }

  isAuthenticated(): boolean {
    return !!this.token
  }

  getToken(): string | null {
    return this.token
  }

  logout(): void {
    this.token = null
    localStorage.removeItem(TOKEN_KEY)
  }

  async getCurrentUser(): Promise<{ id: number; phone: string }> {
    return this.request<{ id: number; phone: string }>('/api/auth/me')
  }

  // ── AI Config ──

  async getAiConfig(): Promise<AiConfig> {
    return this.request<AiConfig>('/api/config')
  }

  async setAiConfig(config: AiConfig): Promise<void> {
    await this.request('/api/config', {
      method: 'POST',
      body: JSON.stringify(config),
    })
  }

  // ── Agent CRUD ──

  async listAgents(): Promise<Agent[]> {
    return this.request<Agent[]>('/api/agents')
  }

  async getAgent(id: string): Promise<Agent> {
    return this.request<Agent>(`/api/agents/${id}`)
  }

  async createAgent(data: CreateAgentDto): Promise<Agent> {
    return this.request<Agent>('/api/agents', {
      method: 'POST',
      body: JSON.stringify({ ...data, visibility: data.visibility || 'private' }),
    })
  }

  async updateAgent(id: string, data: UpdateAgentDto): Promise<Agent> {
    return this.request<Agent>(`/api/agents/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async deleteAgent(id: string): Promise<void> {
    await this.request(`/api/agents/${id}`, { method: 'DELETE' })
  }

  async updateAgentVisibility(id: string, visibility: Visibility): Promise<void> {
    await this.request(`/api/agents/${id}/visibility`, {
      method: 'PATCH',
      body: JSON.stringify({ visibility }),
    })
  }

  // ── Agent 快照 ──

  async publishSnapshot(agentId: string): Promise<AgentSnapshot> {
    return this.request<AgentSnapshot>(`/api/agents/${agentId}/publish`, { method: 'POST' })
  }

  async listSnapshots(agentId: string): Promise<AgentSnapshot[]> {
    return this.request<AgentSnapshot[]>(`/api/agents/${agentId}/snapshots`)
  }

  // ── 社区 ──

  async browseCommunityAgents(sort = 'recent', limit = 50): Promise<Agent[]> {
    return this.request<Agent[]>(`/api/agents/community?sort=${sort}&limit=${limit}`)
  }

  async forkAgent(agentId: string, newName?: string): Promise<Agent> {
    return this.request<Agent>(`/api/agents/${agentId}/fork`, {
      method: 'POST',
      body: JSON.stringify({ new_name: newName ?? null }),
    })
  }

  // ── Spawn Presets ──

  async listSpawnPresets(): Promise<SpawnPreset[]> {
    return this.request<SpawnPreset[]>('/api/spawn-presets')
  }

  async createSpawnPreset(data: CreateSpawnPresetDto): Promise<SpawnPreset> {
    return this.request<SpawnPreset>('/api/spawn-presets', {
      method: 'POST',
      body: JSON.stringify({ ...data, visibility: data.visibility || 'private' }),
    })
  }

  async updateSpawnPreset(id: string, data: UpdateSpawnPresetDto): Promise<SpawnPreset> {
    return this.request<SpawnPreset>(`/api/spawn-presets/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async deleteSpawnPreset(id: string): Promise<void> {
    await this.request(`/api/spawn-presets/${id}`, { method: 'DELETE' })
  }

  // ── Scenarios ──

  async listScenarios(): Promise<Scenario[]> {
    return this.request<Scenario[]>('/api/scenarios')
  }

  async getScenario(id: string): Promise<Scenario> {
    return this.request<Scenario>(`/api/scenarios/${id}`)
  }

  async createScenario(data: CreateScenarioDto): Promise<Scenario> {
    return this.request<Scenario>('/api/scenarios', {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async updateScenario(id: string, data: UpdateScenarioDto): Promise<Scenario> {
    return this.request<Scenario>(`/api/scenarios/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async deleteScenario(id: string): Promise<void> {
    await this.request(`/api/scenarios/${id}`, { method: 'DELETE' })
  }

  async getScenarioWinCondition(id: string): Promise<WinCondition | null> {
    return this.request<WinCondition | null>(`/api/scenarios/${id}/win-condition`)
  }

  async setScenarioWinCondition(id: string, condition: WinCondition): Promise<void> {
    await this.request(`/api/scenarios/${id}/win-condition`, {
      method: 'PUT',
      body: JSON.stringify(condition),
    })
  }

  // ── Game History ──

  async listGameHistories(): Promise<GameHistorySummary[]> {
    return this.request<GameHistorySummary[]>('/api/histories')
  }

  async getGameHistoryDetail(id: string): Promise<SavedAgentHistory[]> {
    return this.request<SavedAgentHistory[]>(`/api/histories/${id}`)
  }

  async uploadGameHistory(history: SavedAgentHistory[]): Promise<void> {
    await this.request('/api/histories', {
      method: 'POST',
      body: JSON.stringify({ histories: history }),
    })
  }

  async deleteGameHistory(id: string): Promise<void> {
    await this.request(`/api/histories/${id}`, { method: 'DELETE' })
  }

  // ── Rooms ──

  async listMyRooms(): Promise<Room[]> {
    return this.request<Room[]>('/api/rooms')
  }

  async listLobbyRooms(): Promise<Room[]> {
    return this.request<Room[]>('/api/rooms/lobby')
  }

  async getRoom(id: string): Promise<Room> {
    return this.request<Room>(`/api/rooms/${id}`)
  }

  async createRoom(name: string, constraints: RoomConstraints): Promise<Room> {
    return this.request<Room>('/api/rooms', {
      method: 'POST',
      body: JSON.stringify({ name, constraints }),
    })
  }

  async joinRoom(id: string): Promise<void> {
    await this.request(`/api/rooms/${id}/join`, { method: 'POST' })
  }

  async joinRoomByCode(code: string): Promise<Room> {
    return this.request<Room>('/api/rooms/join-by-code', {
      method: 'POST',
      body: JSON.stringify({ code }),
    })
  }

  async leaveRoom(id: string): Promise<void> {
    await this.request(`/api/rooms/${id}/leave`, { method: 'POST' })
  }

  async dissolveRoom(id: string): Promise<void> {
    await this.request(`/api/rooms/${id}`, { method: 'DELETE' })
  }

  async updateRoomConstraints(id: string, constraints: RoomConstraints): Promise<void> {
    await this.request(`/api/rooms/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(constraints),
    })
  }

  async listRoomSlots(roomId: string): Promise<RoomAgentSlot[]> {
    return this.request<RoomAgentSlot[]>(`/api/rooms/${roomId}/agents`)
  }

  async addRoomSlot(roomId: string, agentId: string, team: Team): Promise<RoomAgentSlot> {
    return this.request<RoomAgentSlot>(`/api/rooms/${roomId}/agents`, {
      method: 'POST',
      body: JSON.stringify({ agent_id: agentId, team }),
    })
  }

  async removeRoomSlot(roomId: string, slotId: string): Promise<void> {
    await this.request(`/api/rooms/${roomId}/agents/${slotId}`, { method: 'DELETE' })
  }

  async startRoomMatch(roomId: string): Promise<{ match_id: string; ws_port: number }> {
    return this.request<{ match_id: string; ws_port: number }>(`/api/rooms/${roomId}/start`, { method: 'POST' })
  }

  // ── Matches ──

  async listMyMatches(): Promise<Match[]> {
    return this.request<Match[]>('/api/matches')
  }

  async getMatch(id: string): Promise<Match> {
    return this.request<Match>(`/api/matches/${id}`)
  }

  async getMatchEvents(id: string, fromSeq = 0, limit = 200): Promise<MatchEvent[]> {
    return this.request<MatchEvent[]>(`/api/matches/${id}/events?from_seq=${fromSeq}&limit=${limit}`)
  }

  async stopMatch(id: string): Promise<void> {
    await this.request(`/api/matches/${id}/stop`, { method: 'POST' })
  }

  // ── Rank ──

  async enqueueRank(agentId: string, snapshotId: string, mode: string): Promise<RankQueueEntry> {
    return this.request<RankQueueEntry>('/api/rank/queue', {
      method: 'POST',
      body: JSON.stringify({ agent_id: agentId, agent_snapshot_id: snapshotId, mode }),
    })
  }

  async getRankStatus(): Promise<RankQueueEntry[]> {
    return this.request<RankQueueEntry[]>('/api/rank/queue/status')
  }

  async getLeaderboard(mode = 'top_solo', limit = 50): Promise<EloRating[]> {
    return this.request<EloRating[]>(`/api/rank/leaderboard?mode=${mode}&limit=${limit}`)
  }

  async getCurrentSeason(): Promise<Season> {
    return this.request<Season>('/api/rank/seasons/current')
  }

  // ── Essence & Subscriptions ──

  async getEssenceBalance(): Promise<number> {
    return this.request<number>('/api/essence/balance')
  }

  async checkInEssence(): Promise<CheckInResult> {
    return this.request<CheckInResult>('/api/essence/check-in', { method: 'POST' })
  }

  async getEssenceTransactions(limit = 50, offset = 0): Promise<EssenceTransaction[]> {
    return this.request<EssenceTransaction[]>(`/api/essence/transactions?limit=${limit}&offset=${offset}`)
  }

  async getCurrentSubscription(): Promise<BillingPlan> {
    return this.request<BillingPlan>('/api/subscriptions')
  }

  async subscribe(planId: string): Promise<void> {
    await this.request('/api/subscriptions', {
      method: 'POST',
      body: JSON.stringify({ plan_id: planId }),
    })
  }

  async listBillingPlans(): Promise<BillingPlan[]> {
    return this.request<BillingPlan[]>('/api/billing/plans')
  }

  // ── Admin ──

  async getAdminMetrics(): Promise<AdminMetrics> {
    return this.request<AdminMetrics>('/api/admin/metrics')
  }

  async listRunningMatches(): Promise<Match[]> {
    return this.request<Match[]>('/api/admin/matches/running')
  }

  async forceAbortMatch(id: string): Promise<void> {
    await this.request(`/api/admin/matches/${id}/abort`, { method: 'POST' })
  }
}
