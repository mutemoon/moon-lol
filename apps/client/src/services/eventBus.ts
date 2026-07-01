// ── EventBus ──
// 统一事件系统，屏蔽底层是 Tauri event 还是 WebSocket message

export type GameEventType =
  | 'ws-event'
  | 'agent-finished'
  | 'agent-history-updated'
  | 'connection-status'
  | 'unauthorized'
  | 'match-history-ready'

export interface IEventBus {
  on<T = any>(event: GameEventType, handler: (data: T) => void): () => void
  off<T = any>(event: GameEventType, handler: (data: T) => void): void
  emit<T = any>(event: GameEventType, data: T): void
}

type Handler = (data: any) => void

export class EventBusImpl implements IEventBus {
  private handlers = new Map<GameEventType, Set<Handler>>()

  on<T = any>(event: GameEventType, handler: (data: T) => void): () => void {
    let set = this.handlers.get(event)
    if (!set) {
      set = new Set()
      this.handlers.set(event, set)
    }
    set.add(handler as Handler)
    return () => this.off(event, handler)
  }

  off<T = any>(event: GameEventType, handler: (data: T) => void): void {
    this.handlers.get(event)?.delete(handler as Handler)
  }

  emit<T = any>(event: GameEventType, data: T): void {
    this.handlers.get(event)?.forEach((h) => {
      try {
        h(data)
      } catch (e) {
        console.error(`[EventBus] ${event} handler error:`, e)
      }
    })
  }
}

// ── Tauri Event Adapter ──
// 监听 Tauri 的 listen() 事件，转发到 EventBus

export class TauriEventAdapter {
  private unlisteners: (() => void)[] = []

  async bind(bus: IEventBus): Promise<void> {
    const { listen } = await import('@tauri-apps/api/event')

    const u1 = await listen<any>('ws-event', (e) => {
      bus.emit('ws-event', e.payload)
    })
    const u2 = await listen<any>('agent-finished', (e) => {
      bus.emit('agent-finished', e.payload)
    })
    const u3 = await listen<any>('agent-history-updated', (e) => {
      bus.emit('agent-history-updated', e.payload)
    })
    const u4 = await listen<any>('match-history-ready', (e) => {
      bus.emit('match-history-ready', e.payload)
    })

    this.unlisteners.push(u1, u2, u3, u4)
  }

  dispose(): void {
    this.unlisteners.forEach((u) => u())
    this.unlisteners = []
  }
}

// ── WS Event Adapter ──
// 监听 WebSocket message，转发到 EventBus（用于云端对局观战）

export class WsEventAdapter {
  private ws: WebSocket | null = null

  bind(bus: IEventBus, wsUrl: string): void {
    this.dispose()
    this.ws = new WebSocket(wsUrl)
    this.ws.onmessage = (msg) => {
      try {
        const data = JSON.parse(msg.data)
        if (data.type === 'event') {
          bus.emit('ws-event', data)
        } else if (data.event === 'agent-finished') {
          bus.emit('agent-finished', data.data)
        } else if (data.event === 'agent-history-updated') {
          bus.emit('agent-history-updated', data.data)
        }
      } catch {
        // ignore non-JSON
      }
    }
    this.ws.onclose = () => {
      bus.emit('connection-status', { connected: false })
    }
    this.ws.onopen = () => {
      bus.emit('connection-status', { connected: true })
    }
  }

  dispose(): void {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }
}
