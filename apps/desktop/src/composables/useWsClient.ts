import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

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
  event: "game_loaded" | "game_paused" | "champion_changed" | "game_close" | "entity_selected";
  data: Record<string, unknown>;
}

// ── Composable ──

export function useWsClient() {
  const connected = ref(false);
  const connecting = ref(false);
  const connectTimeout = ref(false);
  const gameState = ref({
    champion: "",
    godMode: false,
    cooldownDisabled: false,
    paused: false,
  });

  const selectedEntityId = ref<number | null>(null);

  let unlisten: UnlistenFn | null = null;

  async function connect(): Promise<void> {
    connecting.value = true;
    connectTimeout.value = false;

    try {
      if (unlisten) {
        unlisten();
      }
      unlisten = await listen<WsEvent>("ws-event", (event) => {
        handleEvent(event.payload);
      });

      await invoke("connect_ws");
      connected.value = true;
      connecting.value = false;
    } catch (err) {
      connected.value = false;
      connecting.value = false;
      connectTimeout.value = true;
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
      throw err;
    }
  }

  async function disconnect() {
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
    try {
      await invoke("disconnect_ws");
    } catch {
      /* ignore */
    }
    connected.value = false;
    connecting.value = false;
    connectTimeout.value = false;
  }

  async function send(cmd: string, params: Record<string, unknown> = {}): Promise<WsResponse> {
    try {
      const data = await invoke<any>("send_ws_cmd", { cmd, params });
      return {
        id: 0,
        type: "result",
        ok: true,
        data,
      };
    } catch (error: any) {
      return {
        id: 0,
        type: "result",
        ok: false,
        error: typeof error === "string" ? error : error.message || "Unknown error",
      };
    }
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
        if (unlisten) {
          unlisten();
          unlisten = null;
        }
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
    selectedEntityId,
    connect,
    disconnect,
    send,
  };
}
