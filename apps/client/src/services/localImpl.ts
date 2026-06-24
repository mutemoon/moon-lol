// ── TauriLocalServiceImpl ──
// 基于 Tauri invoke() 的 ILocalService 实现

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ILocalService } from './local'
import type {
  GameConfig,
  LogEntity,
  LogCategory,
  LogQueryParams,
  QueryLogsResult,
  WsEvent,
  UnsubscribeFn,
} from './types'

export class TauriLocalServiceImpl implements ILocalService {
  // ── 游戏进程控制 ──
  async startGame(config: GameConfig): Promise<void> {
    return invoke('start_game', { config })
  }

  async stopGame(): Promise<void> {
    return invoke('stop_game')
  }

  // ── WS 连接与命令 ──
  async connectWs(): Promise<void> {
    return invoke('connect_ws')
  }

  async disconnectWs(): Promise<void> {
    return invoke('disconnect_ws')
  }

  async sendWsCmd(cmd: string, params: Record<string, any> = {}): Promise<any> {
    return invoke('send_ws_cmd', { cmd, params })
  }

  // ── 本地游戏日志查询 ──
  async queryLogEntities(): Promise<LogEntity[]> {
    return invoke<LogEntity[]>('query_log_entities')
  }

  async queryLogCategories(): Promise<LogCategory[]> {
    return invoke<LogCategory[]>('query_log_categories')
  }

  async queryLogs(params: LogQueryParams): Promise<QueryLogsResult> {
    return invoke<QueryLogsResult>('query_logs', params as any)
  }

  async clearLogs(): Promise<void> {
    return invoke('clear_logs')
  }

  // ── 事件监听 ──
  async onWsEvent(callback: (event: WsEvent) => void): Promise<UnsubscribeFn> {
    return listen<WsEvent>('ws-event', (e) => callback(e.payload))
  }

  async onAgentFinished(callback: (data: { minion_kills: number; gold: number }) => void): Promise<UnsubscribeFn> {
    return listen<{ minion_kills: number; gold: number }>('agent-finished', (e) => callback(e.payload))
  }

  async onAgentHistoryUpdated(callback: (data: { agent_id: string; champion: string; history: any[] }) => void): Promise<UnsubscribeFn> {
    return listen<{ agent_id: string; champion: string; history: any[] }>('agent-history-updated', (e) => callback(e.payload))
  }

  // ── 本地离线缓存 ──
  async getLocalCache<T>(key: string): Promise<T | null> {
    try {
      return await invoke<T | null>('get_local_cache', { key })
    } catch {
      return null
    }
  }

  async setLocalCache<T>(key: string, data: T): Promise<void> {
    return invoke('set_local_cache', { key, data })
  }

  // ── Bash 工具 ──
  async runBashTool(cmd: string): Promise<string> {
    return invoke<string>('run_bash_tool', { cmd })
  }
}
