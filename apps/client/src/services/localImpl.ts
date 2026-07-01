import { invoke, Channel } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ILocalService } from './local'
import type {
  GameConfig,
  LogEntity,
  LogCategory,
  QueryLogsResult,
  WsEvent,
  UnsubscribeFn,
  RunningGame,
} from './types'

export class TauriLocalService implements ILocalService {
  // ── 游戏进程控制 ──
  async startGame(config: GameConfig): Promise<{ id: string; port: number }> {
    return invoke('start_game', { config })
  }

  async stopGame(id: string): Promise<void> {
    return invoke('stop_game', { id })
  }

  // ── 实时事件与控制 ──
  async subscribeMatchEvents(id: string, callback: (event: any) => void): Promise<UnsubscribeFn> {
    const channel = new Channel<any>()
    channel.onmessage = callback
    await invoke('subscribe_match_events', { id, channel })
    return () => {
      channel.onmessage = () => {}
    }
  }

  async pauseMatch(id: string): Promise<boolean> {
    return invoke('pause_match', { id })
  }

  async resumeMatch(id: string): Promise<boolean> {
    return invoke('resume_match', { id })
  }

  async setGodMode(id: string, enabled: boolean): Promise<void> {
    return invoke('set_god_mode', { id, enabled })
  }

  async toggleCooldown(id: string, enabled: boolean): Promise<void> {
    return invoke('toggle_cooldown', { id, enabled })
  }

  async resetPosition(id: string): Promise<void> {
    return invoke('reset_position', { id })
  }

  async switchChampion(id: string, name: string): Promise<void> {
    return invoke('switch_champion', { id, name })
  }

  async setScript(id: string, entityId: number, source: string): Promise<void> {
    return invoke('set_script', { id, entityId, source })
  }

  // ── 运行中对局列表 ──
  async listRunningGames(): Promise<RunningGame[]> {
    return invoke<RunningGame[]>('list_running_games')
  }

  async getRunningGame(id: string): Promise<RunningGame | null> {
    return invoke<RunningGame | null>('get_running_game', { id })
  }

  // ── 本地游戏日志查询 ──
  async queryLogEntities(gameId: string): Promise<LogEntity[]> {
    return invoke<LogEntity[]>('query_log_entities', { gameId })
  }

  async queryLogCategories(gameId: string): Promise<LogCategory[]> {
    return invoke<LogCategory[]>('query_log_categories', { gameId })
  }

  async queryLogs(params: { gameId: string; offset: number; limit: number; levels: string[] | null; entityId: number | null; category: string | null; searchText: string | null }): Promise<QueryLogsResult> {
    return invoke<QueryLogsResult>('query_logs', params)
  }

  async clearLogs(gameId: string): Promise<void> {
    return invoke('clear_logs', { gameId })
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
}
