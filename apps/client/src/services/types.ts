// ── 共享类型定义 ──
// 从 backend.ts 和 cloudApi.ts 提取，供 ILocalService / ICloudService / EventBus 共用

// ── 基础数据类型 ──

export type Visibility = 'private' | 'friends' | 'public'
export type Team = 'order' | 'chaos'

export interface SpawnPreset {
  id?: string
  name: string
  x: number
  z: number
  team: string
  visibility?: Visibility
}

export interface HeroPreset {
  id?: string
  name: string
  champion: string
  agent_type: string
  prompt: string
  model?: string
  config_json?: any
  visibility?: Visibility
}

export interface FrontAgentConfig {
  id?: string
  champion: string
  team: string
  prompt: string
  spawn_point: number[]
  agent_type: string
  model?: string
  provider_id?: string
  config_json?: any
}

// ── 模型供应商 ──

export type ApiFormat = 'anthropic' | 'openai_chat' | 'openai_responses' | 'gemini_native'
export type ProviderCategory = 'preset' | 'custom' | 'platform'

export interface ModelConfig {
  name: string
  max_tokens: number
}

export interface ModelProvider {
  id: string
  name: string
  category: ProviderCategory
  preset_type: string
  base_url: string
  /** 写入时携带；云端读取时为空串，用 has_api_key 判断是否已设置。 */
  api_key: string
  has_api_key?: boolean
  api_format: ApiFormat
  models: ModelConfig[]
  enabled: boolean
  website_url?: string
  api_key_url?: string
  icon?: string
  icon_color?: string
  sort_order: number
}

/** 创建/更新供应商的输入（不含 id）。api_key 空串表示更新时保留旧值。 */
export type ModelProviderInput = Omit<ModelProvider, 'id' | 'has_api_key'>

// ── 游戏配置 ──

export interface GameConfig {
  mode: string
  champion: string
  sceneName: string | null
  agents?: FrontAgentConfig[]
  providers?: ModelProvider[]
}

// ── 运行中对局 ──

export interface RunningGame {
  id: string
  port: number
  status: string
}

// ── WS 协议类型 ──

export interface WsResponse {
  id: number
  type: 'result'
  ok: boolean
  data?: any
  error?: string
}

export interface WsEvent {
  type: 'event'
  event: 'game_loaded' | 'game_paused' | 'champion_changed' | 'game_close' | 'entity_selected'
  data: Record<string, any>
}

// ── 日志类型 ──

export interface LogRow {
  id: number
  timestamp: number
  level: string
  file: string | null
  line: number | null
  entity_id: number | null
  entity_name: string | null
  category: string | null
  message: string
}

export interface QueryLogsResult {
  rows: LogRow[]
  total_count: number
}

export interface LogEntity {
  entity_id: number | null
  entity_name: string | null
}

export interface LogCategory {
  category: string | null
}

export interface LogQueryParams {
  offset: number
  limit: number
  levels: string[] | null
  entityId: number | null
  category: string | null
  searchText: string | null
}

// ── Agent (云端完整模型) ──

export interface Agent {
  id: string
  owner_id: number
  name: string
  champion: string
  agent_type: string
  prompt: string
  model?: string
  config_json?: any
  visibility: Visibility
  forked_from: string | null
  upstream_agent_id: string | null
  created_at: string
  updated_at: string
}

export interface CreateAgentDto {
  name: string
  champion: string
  agent_type: string
  prompt: string
  model?: string
  config_json?: any
  visibility?: Visibility
}

export interface UpdateAgentDto {
  name?: string
  champion?: string
  agent_type?: string
  prompt?: string
  model?: string
  config_json?: any
  visibility?: Visibility
}

export interface AgentSnapshot {
  id: string
  agent_id: string
  version: number
  config_freeze: Record<string, any>
  created_at: string
}

// ── 场景 ──

export interface Scenario {
  id: string
  name: string
  agents: FrontAgentConfig[]
  win_condition?: any
  created_at?: string
  updated_at?: string
}

export interface CreateScenarioDto {
  name: string
  agents: FrontAgentConfig[]
}

export interface UpdateScenarioDto {
  name?: string
  agents?: FrontAgentConfig[]
}

// ── 出生点预设 DTO ──

export interface CreateSpawnPresetDto {
  name: string
  x: number
  z: number
  team: string
  visibility?: Visibility
}

export interface UpdateSpawnPresetDto {
  name?: string
  x?: number
  z?: number
  team?: string
  visibility?: Visibility
}

// ── 游戏历史 ──

export interface GameHistorySummary {
  id?: string
  datetime: string
  duration: number
  agents: AgentSummary[]
}

export interface AgentSummary {
  agent_id: string
  champion: string
  team: string
}

export interface SavedAgentHistory {
  agent_id: string
  champion: string
  team: string
  prompt: string
  system_prompt: string
  history: any[]
  game_duration: number
  datetime: string
}

// ── 房间 ──

export interface RoomConstraints {
  max_members: number
  max_agents_per_member: number
  team_policy: 'single_team' | 'free'
  lobby_visible: boolean
  prompt_visible: boolean
}

export interface Room {
  id: string
  name: string
  owner_id: number
  constraints: RoomConstraints
  invite_code: string
  member_count?: number
  status: 'lobby' | 'running' | 'closed'
  created_at?: string
}

export interface RoomAgentSlot {
  id: string
  room_id: string
  member_user_id: number
  agent_id: string
  team: Team
}

// ── 对局 ──

export interface Match {
  id: string
  mode: string
  status: 'pending' | 'running' | 'finished' | 'aborted'
  owner_user_id: number | null
  room_id: string | null
  ws_port: number | null
  created_at: string
  finished_at: string | null
}

export interface MatchEvent {
  id: string
  match_id: string
  seq: number
  payload: Record<string, any>
  recorded_at: string
}

// ── Rank ──

export interface RankQueueEntry {
  user_id: number
  agent_id: string
  agent_snapshot_id: string
  mode: string
  rating: number
  enqueued_at: string
}

export interface EloRating {
  agent_id: string
  agent_name: string
  mode: string
  rating: number
  games_played: number
  wins: number
  losses: number
  daily_delta: number
}

export interface Season {
  id: string
  mode: string
  starts_at: string
  ends_at: string | null
}

// ── 精粹与订阅 ──

export interface EssenceTransaction {
  id: string
  user_id: number
  amount: number
  reason: string
  created_at: string
}

export interface CheckInResult {
  already_checked_in: boolean
  granted: number
  balance: number
}

export interface BillingPlan {
  id: string
  name: string
  monthly_essence: number
  agent_limit: number
  price_cents: number
}

// ── Admin ──

export interface AdminMetrics {
  running_matches: number
  total_memory_mb: number
  avg_match_memory_mb: number
  cpu_usage_percent: number
}

// ── Auth ──

export interface AuthToken {
  token: string
}

// ── 通用 ──

export type UnsubscribeFn = () => void

export interface WinCondition {
  type: string
  [key: string]: any
}
