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
  /** Raw formatted line from Bevy tracing. */
  raw: string;
  /** Parsed timestamp string, e.g. "04:02:48.528" */
  timestamp?: string;
  /** Source file path, e.g. "crates\\lol_render\\src\\skin\\skin.rs" */
  source?: string;
  /** Source line number. */
  sourceLine?: number;
  /** The actual log message. */
  message?: string;
  /** Number of consecutive repeats of this entry. */
  count: number;
}

// ── Log line parsing ──

/** Strip ANSI escape sequences (\x1b[...m) */
function stripAnsi(s: string): string {
  return s.replace(/\x1b\[[0-9;]*m/g, "");
}

/**
 * Parse a Bevy tracing formatted line:
 *   "2026-05-03T04:02:48.528760Z  INFO crates\\lol_render\\src\\skin\\skin.rs:67: message text"
 */
function parseLogLine(raw: string): Partial<LogEntry> {
  const cleaned = stripAnsi(raw);
  const re = /^(\S+)\s+(INFO|WARN|ERROR|DEBUG|TRACE)\s+(.+?):(\d+):\s+(.+)$/;
  const m = cleaned.match(re);
  if (m) {
    // Extract HH:MM:SS.mmm from ISO timestamp
    const tsMatch = m[1].match(/T(\d{2}:\d{2}:\d{2}\.\d+)/);
    // Shorten source path to last 2 segments
    const source = m[3];
    const shortSource = source.replace(/^.*[\\/]([^\\/]+[\\/][^\\/]+)$/, "$1");
    return {
      timestamp: tsMatch ? tsMatch[1].slice(0, 12) : m[1],
      level: m[2].toLowerCase() as LogEntry["level"],
      source: shortSource,
      sourceLine: parseInt(m[4]),
      message: m[5],
    };
  }
  return {};
}

// ── Composable ──

export function useWsClient(url: string) {
  const connected = ref(false);
  const connecting = ref(false);
  const connectTimeout = ref(false);
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
  let cancelRetry: (() => void) | null = null;

  /** Attempt a single WS connection. Rejects on failure. */
  function tryConnect(): Promise<void> {
    return new Promise((resolve, reject) => {
      // Close any previous failed socket
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
    cancelRetry = () => { cancelled = true; };

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
        // Wait 1.5s between retries
        await new Promise((r) => setTimeout(r, 1500));
      }
    }

    connecting.value = false;
    cancelRetry = null;
  }

  function disconnect() {
    // Cancel any in-progress retry loop
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
        addLog("info", "Game loaded");
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

  function addLog(level: "info" | "warn" | "error", raw: string) {
    const cleaned = stripAnsi(raw);
    const parsed = parseLogLine(raw);
    const entry: LogEntry = {
      id: logIdCounter++,
      level: parsed.level || level,
      raw: cleaned,
      timestamp: parsed.timestamp,
      source: parsed.source,
      sourceLine: parsed.sourceLine,
      message: parsed.message || raw,
      count: 1,
    };

    // Group consecutive identical messages
    const last = logs.value[logs.value.length - 1];
    if (last && last.message === entry.message && last.source === entry.source && last.level === entry.level) {
      last.count++;
      last.id = entry.id;
    } else {
      logs.value.push(entry);
    }

    // Keep last 200 entries
    if (logs.value.length > 200) {
      logs.value.splice(0, logs.value.length - 200);
    }
  }

  return { connected, connecting, connectTimeout, logs, gameState, connect, disconnect, send };
}
