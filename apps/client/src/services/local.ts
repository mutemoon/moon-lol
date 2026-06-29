// ── ILocalService ──
// Desktop 独占接口：游戏进程控制、WS 通信、本地日志查询、调试控制、本地文件操作
// 仅在 Tauri 环境下可用

import type {
  GameConfig,
  LogEntity,
  LogCategory,
  LogQueryParams,
  QueryLogsResult,
  WsEvent,
  UnsubscribeFn,
} from './types'

export interface ILocalService {
  // ── 游戏进程控制 ──
  startGame(config: GameConfig): Promise<void>
  stopGame(): Promise<void>

  // ── WS 连接与命令 ──
  connectWs(): Promise<void>
  /** 仅建连 WS、不启动 AI 编排器，供观战/回放场景使用。 */
  connectWsObserve(): Promise<void>
  disconnectWs(): Promise<void>
  sendWsCmd(cmd: string, params?: Record<string, any>): Promise<any>

  // ── 本地游戏日志查询（SQLite） ──
  queryLogEntities(): Promise<LogEntity[]>
  queryLogCategories(): Promise<LogCategory[]>
  queryLogs(params: LogQueryParams): Promise<QueryLogsResult>
  clearLogs(): Promise<void>

  // ── 事件监听（桥接到 EventBus） ──
  onWsEvent(callback: (event: WsEvent) => void): Promise<UnsubscribeFn>
  onAgentFinished(callback: (data: { minion_kills: number; gold: number }) => void): Promise<UnsubscribeFn>
  onAgentHistoryUpdated(callback: (data: { agent_id: string; champion: string; history: any[] }) => void): Promise<UnsubscribeFn>

  // ── 本地离线缓存读写（离线降级用） ──
  getLocalCache<T>(key: string): Promise<T | null>
  setLocalCache<T>(key: string, data: T): Promise<void>

  // ── Bash 工具 ──
  runBashTool(cmd: string): Promise<string>
}
