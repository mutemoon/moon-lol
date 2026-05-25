import { ref, watch } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { useWsClient } from "../composables/useWsClient";
import { createLogContext } from "../composables/useLogPoller";
import { router } from "../router";

export const useGameStore = defineStore("game", () => {
  const champion = ref(localStorage.getItem("moon_lol_last_champion") || "Riven");
  const mode = ref(localStorage.getItem("moon_lol_last_mode") || "sandbox");
  const launchError = ref("");
  const isStarting = ref(false);
  const showStatsModal = ref(false);
  const statsResult = ref({ minionKills: 0, gold: 0.0 });

  const ws = useWsClient();
  const log = createLogContext();

  const champions = ["Riven", "Fiora"];

  // Sync log filter when ws selected entity changes
  watch(
    () => ws.selectedEntityId.value,
    (entityId) => {
      if (entityId !== null) {
        log.setEntityFilter(entityId);
      }
    },
  );

  watch(champion, (val) => {
    localStorage.setItem("moon_lol_last_champion", val);
  });

  watch(mode, (val) => {
    localStorage.setItem("moon_lol_last_mode", val);
  });

  async function startGame(sceneName?: string) {
    launchError.value = "";
    isStarting.value = true;

    try {
      await invoke("start_game", {
        config: {
          mode: mode.value,
          champion: champion.value,
          sceneName: sceneName || null,
        },
      });
    } catch (e: any) {
      launchError.value = typeof e === "string" ? e : e.message || "Unknown error";
      isStarting.value = false;
      return;
    }

    ws.connect().then(() => {
      log.start();
      isStarting.value = false;
      router.push("/debug");
    });
  }

  function stopGame() {
    ws.disconnect();
    log.stop();
    invoke("stop_game").catch(() => {});
    router.push("/");
  }

  return {
    champion,
    mode,
    launchError,
    isStarting,
    showStatsModal,
    statsResult,
    ws,
    log,
    champions,
    startGame,
    stopGame,
  };
});
