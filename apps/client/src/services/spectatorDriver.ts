import type { MatchEvent, UnsubscribeFn } from './types'
import type { ILocalService } from './local'
import type { ICloudService } from './cloud'

export interface SpectatorDriver {
  play(): Promise<void>
  pause(): Promise<void>
  seek(seq: number): Promise<void>
  setSpeed(speed: number): Promise<void>
  subscribe(callback: (event: MatchEvent) => void): UnsubscribeFn
  destroy(): void
}

/**
 * 桌面端本地游戏观战驱动器（控制本地 Bevy 进程）
 */
export class LocalSpectatorDriver implements SpectatorDriver {
  private matchId: string
  private localService: ILocalService
  private unsubscribeEvents?: UnsubscribeFn

  constructor(matchId: string, localService: ILocalService) {
    this.matchId = matchId
    this.localService = localService
  }

  async play(): Promise<void> {
    await this.localService.resumeMatch(this.matchId)
  }

  async pause(): Promise<void> {
    await this.localService.pauseMatch(this.matchId)
  }

  async seek(seq: number): Promise<void> {
    // 桌面端进程内暂不支持任意 Seek，记录日志即可
    console.warn(`Local OS process match does not support seeking to sequence: ${seq}`)
  }

  async setSpeed(speed: number): Promise<void> {
    // 桌面端通过 ILocalService 发送 speed 调试指令（如果底层支持）
    console.log(`Setting speed on local match: ${speed}`)
  }

  subscribe(callback: (event: MatchEvent) => void): UnsubscribeFn {
    let unsubscribed = false
    this.localService.subscribeMatchEvents(this.matchId, (rawEvent: any) => {
      if (unsubscribed) return
      // 转换原始事件为 MatchEvent DTO 格式
      if (rawEvent && rawEvent.event === 'match_event') {
        const payload = rawEvent.data
        const matchEvent: MatchEvent = {
          id: '',
          match_id: this.matchId,
          seq: payload.seq || 0,
          payload: payload,
          recorded_at: new Date().toISOString()
        }
        callback(matchEvent)
      }
    }).then(unsub => {
      this.unsubscribeEvents = unsub
    })

    return () => {
      unsubscribed = true
      if (this.unsubscribeEvents) {
        this.unsubscribeEvents()
      }
    }
  }

  destroy(): void {
    if (this.unsubscribeEvents) {
      this.unsubscribeEvents()
    }
  }
}

interface BevyWasm {
  reset(): void
  push_event(event: any): void
  set_speed(speed: number): void
}

declare global {
  interface Window {
    bevyWasm?: BevyWasm
  }
}

/**
 * Web 端 WASM 游戏观战驱动器（数据驱动 Canvas 中的 Bevy WASM 实例）
 */
export class WasmSpectatorDriver implements SpectatorDriver {
  private matchId: string
  private cloudService: ICloudService
  private eventsCache: MatchEvent[] = []
  private ws?: WebSocket
  private callbacks: Set<(event: MatchEvent) => void> = new Set()
  private speed: number = 1.0
  private isPlaying: boolean = true

  constructor(matchId: string, cloudService: ICloudService) {
    this.matchId = matchId
    this.cloudService = cloudService
    this.init()
  }

  private async init() {
    // 1. 获取所有历史事件缓存起来
    try {
      const res = await this.cloudService.getMatchEvents(this.matchId, 0, 10000)
      this.eventsCache = res.sort((a: MatchEvent, b: MatchEvent) => a.seq - b.seq)
      // 把历史事件推送给 WASM Canvas
      this.feedEventsToWasm(this.eventsCache)
      // 触发已有 callback
      for (const ev of this.eventsCache) {
        this.notifyCallbacks(ev)
      }
    } catch (e) {
      console.error('Failed to load initial match events:', e)
    }

    // 2. 建立 WebSocket 接收后续的实时事件
    const token = this.cloudService.getToken()
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const wsUrl = `${protocol}//${window.location.host}/api/matches/${this.matchId}/events/ws?token=${encodeURIComponent(token || '')}`
    
    this.ws = new WebSocket(wsUrl)
    this.ws.onmessage = (ev) => {
      try {
        const msg = JSON.parse(ev.data)
        if (msg.type === 'event') {
          const matchEvent = msg.data as MatchEvent
          // 校验是否已在缓存中，避免重复
          if (!this.eventsCache.some(e => e.seq === matchEvent.seq)) {
            this.eventsCache.push(matchEvent)
            this.eventsCache.sort((a, b) => a.seq - b.seq)
            if (window.bevyWasm) {
              window.bevyWasm.push_event(matchEvent.payload)
            }
            this.notifyCallbacks(matchEvent)
          }
        } else if (msg.type === 'close') {
          console.log('Match WS stream finished:', msg.data.reason)
          this.ws?.close()
        }
      } catch (e) {
        console.error('Failed to parse WS spectator message:', e)
      }
    }
  }

  private feedEventsToWasm(events: MatchEvent[]) {
    if (window.bevyWasm) {
      for (const ev of events) {
        window.bevyWasm.push_event(ev.payload)
      }
    }
  }

  private notifyCallbacks(ev: MatchEvent) {
    for (const cb of this.callbacks) {
      cb(ev)
    }
  }

  async play(): Promise<void> {
    this.isPlaying = true
    if (window.bevyWasm) {
      window.bevyWasm.set_speed(this.speed)
    }
  }

  async pause(): Promise<void> {
    this.isPlaying = false
    if (window.bevyWasm) {
      window.bevyWasm.set_speed(0.0)
    }
  }

  async seek(seq: number): Promise<void> {
    if (window.bevyWasm) {
      // 1. 重置 Bevy WASM 状态
      window.bevyWasm.reset()
      // 2. 筛选需要重新推送的历史事件
      const seekEvents = this.eventsCache.filter(e => e.seq <= seq)
      // 3. 一次性批量推送，WASM 内部会快速计算状态
      this.feedEventsToWasm(seekEvents)
      // 4. 恢复当前播放速度
      window.bevyWasm.set_speed(this.isPlaying ? this.speed : 0.0)
    }
  }

  async setSpeed(speed: number): Promise<void> {
    this.speed = speed
    if (this.isPlaying && window.bevyWasm) {
      window.bevyWasm.set_speed(speed)
    }
  }

  subscribe(callback: (event: MatchEvent) => void): UnsubscribeFn {
    this.callbacks.add(callback)
    // 立即向新订阅者推送已有的缓存事件
    for (const ev of this.eventsCache) {
      callback(ev)
    }
    return () => {
      this.callbacks.delete(callback)
    }
  }

  destroy(): void {
    this.callbacks.clear()
    if (this.ws) {
      this.ws.close()
    }
  }
}
