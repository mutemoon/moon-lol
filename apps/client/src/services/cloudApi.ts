// ── cloudApi.ts (兼容层) ──
// 重新导出新架构的类型和 API helper，保持现有消费者的 import 路径不变
// 最终目标是消费者直接 import services/provider

import { services } from './provider'
import type { CreateAgentDto, UpdateAgentDto } from './types'

// ── 类型重导出 ──
export type {
  Visibility,
  Team,
  Agent,
  AgentSnapshot,
  RoomConstraints,
  Room,
  RoomAgentSlot,
  Match,
  MatchEvent,
  RankQueueEntry,
  EloRating,
  Season,
  EssenceTransaction,
  BillingPlan,
  AdminMetrics,
} from './types'

// ── 便捷 API 对象（保持现有消费者的 agentsApi.xxx() 调用模式） ──

export const agentsApi = {
  list: () => services.cloud.listAgents(),
  get: (id: string) => services.cloud.getAgent(id),
  create: (data: CreateAgentDto) => services.cloud.createAgent(data),
  update: (id: string, data: UpdateAgentDto) => services.cloud.updateAgent(id, data),
  updateVisibility: (id: string, visibility: 'private' | 'friends' | 'public') =>
    services.cloud.updateAgentVisibility(id, visibility),
  publishSnapshot: (id: string) => services.cloud.publishSnapshot(id),
  listSnapshots: (id: string) => services.cloud.listSnapshots(id),
  browseCommunity: (sort: 'recent' | 'popular' | 'elo' = 'recent', limit = 50) =>
    services.cloud.browseCommunityAgents(sort, limit),
  fork: (id: string, newName?: string) => services.cloud.forkAgent(id, newName),
  pullUpstream: (id: string) => services.cloud.pullUpstream(id),
}

export const roomsApi = {
  listMine: () => services.cloud.listMyRooms(),
  listLobby: () => services.cloud.listLobbyRooms(),
  create: (name: string, constraints: any) => services.cloud.createRoom(name, constraints),
  joinByCode: (code: string) => services.cloud.joinRoomByCode(code),
  get: (id: string) => services.cloud.getRoom(id),
  dissolve: (id: string) => services.cloud.dissolveRoom(id),
  updateConstraints: (id: string, c: any) => services.cloud.updateRoomConstraints(id, c),
  join: (id: string) => services.cloud.joinRoom(id),
  leave: (id: string) => services.cloud.leaveRoom(id),
  listSlots: (id: string) => services.cloud.listRoomSlots(id),
  addSlot: (id: string, agent_id: string, team: any) => services.cloud.addRoomSlot(id, agent_id, team),
  removeSlot: (id: string, slotId: string) => services.cloud.removeRoomSlot(id, slotId),
  start: (id: string) => services.cloud.startRoomMatch(id),
}

export const matchesApi = {
  listMine: () => services.cloud.listMyMatches(),
  listByStatus: (status: string) => services.cloud.listMyMatches().then(ms => ms.filter(m => m.status === status)),
  get: (id: string) => services.cloud.getMatch(id),
  getEvents: (id: string, fromSeq = 0, limit = 200) => services.cloud.getMatchEvents(id, fromSeq, limit),
  stop: (id: string) => services.cloud.stopMatch(id),
}

export const rankApi = {
  enqueue: (agent_id: string, agent_snapshot_id: string, mode: string) =>
    services.cloud.enqueueRank(agent_id, agent_snapshot_id, mode),
  status: () => services.cloud.getRankStatus(),
  leaderboard: (mode = 'top_solo', limit = 50) => services.cloud.getLeaderboard(mode, limit),
  currentSeason: () => services.cloud.getCurrentSeason(),
}

export const essenceApi = {
  balance: () => services.cloud.getEssenceBalance(),
  checkIn: () => services.cloud.checkInEssence(),
  transactions: (limit = 50, offset = 0) => services.cloud.getEssenceTransactions(limit, offset),
}

export const subscriptionsApi = {
  current: () => services.cloud.getCurrentSubscription(),
  subscribe: (plan_id: string) => services.cloud.subscribe(plan_id),
  listPlans: () => services.cloud.listBillingPlans(),
}

export const adminApi = {
  metrics: () => services.cloud.getAdminMetrics(),
  listRunning: () => services.cloud.listRunningMatches(),
  forceAbort: (id: string) => services.cloud.forceAbortMatch(id),
}

const BASE_URL = (import.meta as any).env?.VITE_BASE_URL || 'http://localhost:3000'
export const BASE = BASE_URL
