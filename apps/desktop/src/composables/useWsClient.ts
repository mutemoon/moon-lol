import { ref } from "vue";

// ── WS Protocol types ──

export interface WsRequest {
  id: number;
  type: "cmd";
  cmd: string;
  params: Record<string, unknown>;
}

export interface WsResponse {
  id: number;
  type: "result";
  ok: boolean;
  data?: unknown;
  error?: string;
}

export interface WsEvent {
  type: "event";
  event: string;
  data: Record<string, unknown>;
}

// ── Composable ──

export function useWsClient(url: string) {
  const connected = ref(false);
  const connecting = ref(false);
  const connectTimeout = ref(false);
  const gameState = ref({
    champion: "",
    godMode: false,
    cooldownDisabled: false,
    paused: false,
  });

  const agentObserve = ref<any>(null);
  const agentThinking = ref<string>("");
  const agentAction = ref<string>("");
  const selectedEntityId = ref<number | null>(null);

  let ws: WebSocket | null = null;
  let nextId = 1;
  const pending = new Map<number, (res: WsResponse) => void>();
  let cancelRetry: (() => void) | null = null;

  /** Attempt a single WS connection. Rejects on failure. */
  function tryConnect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (ws) {
        ws.onclose = null;
        ws.close();
        ws = null;
      }
      ws = new WebSocket(url);
      ws.onopen = () => {
        connected.value = true;
        resolve();
      };
      ws.onerror = () => {
        connected.value = false;
        reject(new Error("WS connection failed"));
      };
      ws.onclose = () => {
        connected.value = false;
      };
      ws.onmessage = (msg) => {
        const data = JSON.parse(msg.data);
        if (data.type === "result") {
          const resolve = pending.get(data.id);
          if (resolve) {
            pending.delete(data.id);
            resolve(data);
          }
        } else if (data.type === "event") {
          handleEvent(data);
        }
      };
    });
  }

  /** Keep retrying WS connection until it succeeds or cancelled. */
  async function connect(): Promise<void> {
    connecting.value = true;
    connectTimeout.value = false;
    const startTime = Date.now();

    let cancelled = false;
    cancelRetry = () => {
      cancelled = true;
    };

    while (!cancelled) {
      try {
        await tryConnect();
        connecting.value = false;
        cancelRetry = null;
        return;
      } catch {
        if (cancelled) break;
        if (Date.now() - startTime > 30_000) {
          connectTimeout.value = true;
        }
        await new Promise((r) => setTimeout(r, 1500));
      }
    }

    connecting.value = false;
    cancelRetry = null;
  }

  function disconnect() {
    if (cancelRetry) {
      cancelRetry();
      cancelRetry = null;
    }
    if (ws) {
      ws.onclose = null;
      ws.close();
      ws = null;
    }
    connected.value = false;
    connecting.value = false;
    connectTimeout.value = false;
  }

  function send(cmd: string, params: Record<string, unknown> = {}): Promise<WsResponse> {
    return new Promise((resolve, reject) => {
      if (!ws || ws.readyState !== WebSocket.OPEN) {
        reject(new Error("WS not connected"));
        return;
      }
      const id = nextId++;
      pending.set(id, resolve);
      ws.send(JSON.stringify({ id, type: "cmd", cmd, params }));

      setTimeout(() => {
        if (pending.has(id)) {
          pending.delete(id);
          reject(new Error(`Command ${cmd} timed out`));
        }
      }, 5000);
    });
  }

  function handleEvent(event: WsEvent) {
    switch (event.event) {
      case "game_loaded":
        send("get_state").then((res) => {
          if (res.ok && res.data) {
            gameState.value = { ...gameState.value, ...(res.data as any) };
          }
        });
        break;
      case "game_paused":
        gameState.value.paused = event.data.paused as boolean;
        break;
      case "champion_changed":
        gameState.value.champion = event.data.name as string;
        break;
      case "game_close":
        connected.value = false;
        break;
      case "agent_update":
        agentObserve.value = event.data.observe;
        agentThinking.value = event.data.thinking as string;
        agentAction.value = event.data.action as string;
        break;
      case "entity_selected":
        selectedEntityId.value = event.data.entity_id as number;
        break;
    }
  }

  return {
    connected,
    connecting,
    connectTimeout,
    gameState,
    agentObserve,
    agentThinking,
    agentAction,
    selectedEntityId,
    connect,
    disconnect,
    send,
  };
}
