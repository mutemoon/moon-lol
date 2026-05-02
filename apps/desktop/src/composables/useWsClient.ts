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

export interface LogEntry {
  id: number;
  level: "info" | "warn" | "error";
  msg: string;
}

// ── Composable ──

export function useWsClient(url: string) {
  const connected = ref(false);
  const logs = ref<LogEntry[]>([]);
  const gameState = ref({
    champion: "",
    godMode: false,
    cooldownDisabled: false,
    paused: false,
  });

  let ws: WebSocket | null = null;
  let nextId = 1;
  const pending = new Map<number, (res: WsResponse) => void>();
  let logIdCounter = 0;

  function connect(): Promise<void> {
    return new Promise((resolve, reject) => {
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

  function disconnect() {
    if (ws) {
      ws.close();
      ws = null;
      connected.value = false;
    }
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

      // Timeout after 5s
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
        addLog("info", "Game loaded");
        // Fetch initial state
        send("get_state").then((res) => {
          if (res.ok && res.data) {
            gameState.value = { ...gameState.value, ...(res.data as any) };
          }
        });
        break;
      case "game_paused":
        gameState.value.paused = event.data.paused as boolean;
        addLog("info", event.data.paused ? "Game paused" : "Game resumed");
        break;
      case "champion_changed":
        gameState.value.champion = event.data.name as string;
        addLog("info", `Champion changed to ${event.data.name}`);
        break;
      case "game_close":
        addLog("warn", `Game closed: ${event.data.reason}`);
        connected.value = false;
        break;
      case "log":
        addLog(
          (event.data.level as "info" | "warn" | "error") || "info",
          event.data.msg as string
        );
        break;
    }
  }

  function addLog(level: "info" | "warn" | "error", msg: string) {
    logs.value.push({ id: logIdCounter++, level, msg });
    // Keep last 100 entries
    if (logs.value.length > 100) {
      logs.value.splice(0, logs.value.length - 100);
    }
  }

  return { connected, logs, gameState, connect, disconnect, send };
}
