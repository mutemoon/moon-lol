// ── ILocalService ──
// Desktop 独占接口：游戏进程控制、WS 通信、本地日志查询、调试控制、本地文件操作
// 仅在 Tauri 环境下可用

import type {
  GameConfig,
  LogEntity,
  LogCategory,
  QueryLogsResult,
  WsEvent,
  UnsubscribeFn,
  RunningGame,
} from './types'

export interface ILocalService {
  // ── 游戏进程控制 ──
  startGame(config: GameConfig): Promise<{ id: string; port: number }>
  stopGame(id: string): Promise<void>

  // ── 实时事件与控制 ──
  subscribeMatchEvents(id: string, callback: (event: any) => void): Promise<UnsubscribeFn>
  pauseMatch(id: string): Promise<boolean>
  resumeMatch(id: string): Promise<boolean>
  setGodMode(id: string, enabled: boolean): Promise<void>
  toggleCooldown(id: string, enabled: boolean): Promise<void>
  resetPosition(id: string): Promise<void>
  switchChampion(id: string, name: string): Promise<void>
  setScript(id: string, entityId: number, source: string): Promise<void>

  // ── 运行中对局列表 ──
  listRunningGames(): Promise<RunningGame[]>
  getRunningGame(id: string): Promise<RunningGame | null>

  // ── 本地游戏日志查询（SQLite） ──
  queryLogs(params: { gameId: string; offset: number; limit: number; levels: string[] | null; entityId: number | null; category: string | null; searchText: string | null }): Promise<QueryLogsResult>
  queryLogEntities(gameId: string): Promise<LogEntity[]>
  queryLogCategories(gameId: string): Promise<LogCategory[]>
  clearLogs(gameId: string): Promise<void>

  // ── 事件监听（桥接到 EventBus） ──
  onWsEvent(callback: (event: WsEvent) => void): Promise<UnsubscribeFn>
  onAgentFinished(callback: (data: { minion_kills: number; gold: number }) => void): Promise<UnsubscribeFn>
  onAgentHistoryUpdated(callback: (data: { agent_id: string; champion: string; history: any[] }) => void): Promise<UnsubscribeFn>
}
