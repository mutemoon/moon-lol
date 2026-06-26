// ── ICloudService ──
// 云端完整服务接口：Desktop 和 Web 共用同一份实现（直接 HTTP fetch）
// 合并了原 webBackend.ts 中的预设 CRUD 和 cloudApi.ts 中的房间/Rank/精粹等高阶 API

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

export interface ICloudService {
  // ── Auth ──
  login(phone: string, password: string): Promise<AuthToken>
  codeLogin(phone: string, code: string): Promise<AuthToken>
  register(phone: string, password: string, code: string): Promise<AuthToken>
  resetPassword(phone: string, code: string, newPassword: string): Promise<void>
  isAuthenticated(): boolean
  getToken(): string | null
  logout(): void
  getCurrentUser(): Promise<{ id: number; phone: string }>

  // ── AI Config (云端同步) ──
  getAiConfig(): Promise<AiConfig>
  setAiConfig(config: AiConfig): Promise<void>

  // ── Agent (选手) CRUD ──
  listAgents(): Promise<Agent[]>
  getAgent(id: string): Promise<Agent>
  createAgent(data: CreateAgentDto): Promise<Agent>
  updateAgent(id: string, data: UpdateAgentDto): Promise<Agent>
  deleteAgent(id: string): Promise<void>
  updateAgentVisibility(id: string, visibility: Visibility): Promise<void>

  // ── Agent 快照 ──
  publishSnapshot(agentId: string): Promise<AgentSnapshot>
  listSnapshots(agentId: string): Promise<AgentSnapshot[]>

  // ── 社区 ──
  browseCommunityAgents(sort?: string, limit?: number): Promise<Agent[]>
  forkAgent(agentId: string, newName?: string): Promise<Agent>
  pullUpstream(agentId: string): Promise<Agent>

  // ── Spawn Presets ──
  listSpawnPresets(): Promise<SpawnPreset[]>
  createSpawnPreset(data: CreateSpawnPresetDto): Promise<SpawnPreset>
  updateSpawnPreset(id: string, data: UpdateSpawnPresetDto): Promise<SpawnPreset>
  deleteSpawnPreset(id: string): Promise<void>

  // ── Scenarios ──
  listScenarios(): Promise<Scenario[]>
  getScenario(id: string): Promise<Scenario>
  createScenario(data: CreateScenarioDto): Promise<Scenario>
  updateScenario(id: string, data: UpdateScenarioDto): Promise<Scenario>
  deleteScenario(id: string): Promise<void>
  getScenarioWinCondition(id: string): Promise<WinCondition | null>
  setScenarioWinCondition(id: string, condition: WinCondition): Promise<void>

  // ── Game History (云端对话历史同步) ──
  listGameHistories(): Promise<GameHistorySummary[]>
  getGameHistoryDetail(id: string): Promise<SavedAgentHistory[]>
  uploadGameHistory(history: SavedAgentHistory[]): Promise<void>
  deleteGameHistory(id: string): Promise<void>

  // ── Rooms ──
  listMyRooms(): Promise<Room[]>
  listLobbyRooms(): Promise<Room[]>
  getRoom(id: string): Promise<Room>
  createRoom(name: string, constraints: RoomConstraints): Promise<Room>
  joinRoom(id: string): Promise<void>
  joinRoomByCode(code: string): Promise<Room>
  leaveRoom(id: string): Promise<void>
  dissolveRoom(id: string): Promise<void>
  updateRoomConstraints(id: string, constraints: RoomConstraints): Promise<void>
  listRoomSlots(roomId: string): Promise<RoomAgentSlot[]>
  addRoomSlot(roomId: string, agentId: string, team: Team): Promise<RoomAgentSlot>
  removeRoomSlot(roomId: string, slotId: string): Promise<void>
  startRoomMatch(roomId: string): Promise<{ match_id: string; ws_port: number }>

  // ── Matches ──
  listMyMatches(): Promise<Match[]>
  getMatch(id: string): Promise<Match>
  getMatchEvents(id: string, fromSeq?: number, limit?: number): Promise<MatchEvent[]>
  stopMatch(id: string): Promise<void>

  // ── Rank ──
  enqueueRank(agentId: string, snapshotId: string, mode: string): Promise<RankQueueEntry>
  getRankStatus(): Promise<RankQueueEntry[]>
  getLeaderboard(mode?: string, limit?: number): Promise<EloRating[]>
  getCurrentSeason(): Promise<Season>

  // ── Essence & Subscriptions ──
  getEssenceBalance(): Promise<number>
  checkInEssence(): Promise<CheckInResult>
  getEssenceTransactions(limit?: number, offset?: number): Promise<EssenceTransaction[]>
  getCurrentSubscription(): Promise<BillingPlan>
  subscribe(planId: string): Promise<void>
  listBillingPlans(): Promise<BillingPlan[]>

  // ── Admin ──
  getAdminMetrics(): Promise<AdminMetrics>
  listRunningMatches(): Promise<Match[]>
  forceAbortMatch(id: string): Promise<void>
}
