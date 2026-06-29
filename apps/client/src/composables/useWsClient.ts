import { ref } from "vue";
import { backendClient } from "../services/backend";
import type { WsResponse, WsEvent, UnsubscribeFn } from "../services/backend";

// ── WS Protocol types ──

export interface WsRequest {
  id: number;
  type: "cmd";
  cmd: string;
  params: Record<string, unknown>;
}

export type { WsResponse, WsEvent };

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

  let unlisten: UnsubscribeFn | null = null;

  /**
   * 建立 WS 连接。
   * @param observe 若为 true，使用 connectWsObserve（不启动 AI 编排器），供观战/回放。
   */
  async function connect(observe = false): Promise<void> {
    connecting.value = true;
    connectTimeout.value = false;

    try {
      if (unlisten) {
        unlisten();
      }
      unlisten = await backendClient.onWsEvent((event) => {
        handleEvent(event);
      });

      if (observe) {
        await backendClient.connectWsObserve();
      } else {
        await backendClient.connectWs();
      }
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
      await backendClient.disconnectWs();
    } catch {
      /* ignore */
    }
    connected.value = false;
    connecting.value = false;
    connectTimeout.value = false;
  }

  async function send(cmd: string, params: Record<string, unknown> = {}): Promise<WsResponse> {
    try {
      const data = await backendClient.sendWsCmd(cmd, params);
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

