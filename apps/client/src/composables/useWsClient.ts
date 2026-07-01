import { ref } from "vue";
import { services } from "../services/provider";
import type { WsResponse, WsEvent, UnsubscribeFn } from "../services/types";

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
  const matchId = ref<string | null>(null);
  const gameState = ref({
    champion: "",
    godMode: false,
    cooldownDisabled: false,
    paused: false,
  });

  const selectedEntityId = ref<number | null>(null);

  let unlisten: UnsubscribeFn | null = null;

  /**
   * 订阅对局事件流。
   * @param id 对局 ID (match_id)
   */
  async function connect(id: string): Promise<void> {
    connecting.value = true;
    connectTimeout.value = false;

    try {
      if (unlisten) {
        unlisten();
      }
      unlisten = await services.local.subscribeMatchEvents(id, (event) => {
        handleEvent(event);
      });

      matchId.value = id;
      connected.value = true;
      connecting.value = false;
    } catch (err) {
      matchId.value = null;
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
    matchId.value = null;
    connected.value = false;
    connecting.value = false;
    connectTimeout.value = false;
  }

  async function send(cmd: string, params: Record<string, unknown> = {}): Promise<WsResponse> {
    if (!matchId.value) {
      return { id: 0, type: "result", ok: false, error: "对局未连接" };
    }
    try {
      let data: any = null;
      if (cmd === 'god_mode') {
        await services.local.setGodMode(matchId.value, params.enabled as boolean);
        gameState.value.godMode = params.enabled as boolean;
      } else if (cmd === 'toggle_cooldown') {
        await services.local.toggleCooldown(matchId.value, params.enabled as boolean);
        gameState.value.cooldownDisabled = params.enabled as boolean;
      } else if (cmd === 'toggle_pause') {
        const nextPaused = !gameState.value.paused;
        if (gameState.value.paused) {
          await services.local.resumeMatch(matchId.value);
        } else {
          await services.local.pauseMatch(matchId.value);
        }
        gameState.value.paused = nextPaused;
      } else if (cmd === 'reset_position') {
        await services.local.resetPosition(matchId.value);
      } else if (cmd === 'switch_champion') {
        await services.local.switchChampion(matchId.value, params.name as string);
        gameState.value.champion = params.name as string;
      } else if (cmd === 'set_script') {
        await services.local.setScript(matchId.value, params.entity_id as number, params.source as string);
      } else {
        console.warn(`Unknown control command: ${cmd}`);
      }

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
        // 游戏加载后更新状态
        break;
      case "game_paused":
        gameState.value.paused = event.data.paused as boolean;
        break;
      case "champion_changed":
        gameState.value.champion = event.data.name as string;
        break;
      case "game_close":
        connected.value = false;
        matchId.value = null;
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
    matchId,
    gameState,
    selectedEntityId,
    connect,
    disconnect,
    send,
  };
}

