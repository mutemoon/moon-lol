<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useWsClient } from "./composables/useWsClient";
import DebugPanel from "./components/DebugPanel.vue";

type View = "launcher" | "debug";

const view = ref<View>("launcher");
const champion = ref("Riven");
const mode = ref("sandbox");
const error = ref("");
const isStarting = ref(false);

const champions = ["Riven", "Fiora"];

const ws = useWsClient("ws://127.0.0.1:9001");

async function startGame() {
  error.value = "";
  isStarting.value = true;

  try {
    // 1. Start Bevy process via Tauri
    await invoke("start_game", {
      config: {
        mode: mode.value,
        champion: champion.value,
      },
    });

    // 2. Wait briefly for Bevy to bind the port
    await new Promise((r) => setTimeout(r, 1000));

    // 3. Connect WS
    await ws.connect();

    view.value = "debug";
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message || "Unknown error";
  } finally {
    isStarting.value = false;
  }
}

async function stopGame() {
  ws.disconnect();
  try {
    await invoke("stop_game");
  } catch (_) {
    // ignore errors
  }
  view.value = "launcher";
}
</script>

<template>
  <main>
    <!-- Launcher -->
    <div v-if="view === 'launcher'" class="launcher">
      <h1>MOON-LOL</h1>
      <p class="subtitle">Desktop Launcher</p>

      <div class="launcher-card">
        <div class="field">
          <label>Champion</label>
          <select v-model="champion">
            <option v-for="c in champions" :key="c" :value="c">{{ c }}</option>
          </select>
        </div>

        <div class="field">
          <label>Mode</label>
          <select v-model="mode">
            <option value="sandbox">Sandbox</option>
          </select>
        </div>

        <button
          class="start-btn"
          :disabled="isStarting"
          @click="startGame"
        >
          {{ isStarting ? 'Starting...' : 'Start Game' }}
        </button>

        <p v-if="error" class="error">{{ error }}</p>
      </div>
    </div>

    <!-- Debug Panel -->
    <DebugPanel
      v-if="view === 'debug'"
      :connected="ws.connected.value"
      :game-state="ws.gameState.value"
      :logs="ws.logs.value"
      @send="(cmd, params) => ws.send(cmd, params)"
      @stop="stopGame"
    />
  </main>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  color: #f0f0f0;
  background-color: #0a0a14;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  min-height: 100vh;
}

main {
  min-height: 100vh;
}
</style>

<style scoped>
.launcher {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  padding: 40px;
}

h1 {
  font-size: 48px;
  font-family: monospace;
  color: #00ffff;
  text-shadow: 0 0 20px rgba(0, 255, 255, 0.3);
}

.subtitle {
  color: #666;
  margin-bottom: 32px;
}

.launcher-card {
  background: #1a1a2e;
  padding: 32px;
  border-radius: 12px;
  width: 100%;
  max-width: 400px;
}

.field {
  margin-bottom: 20px;
}

.field label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  color: #888;
}

select {
  width: 100%;
  background: #0d0d1a;
  color: #fff;
  border: 1px solid #333;
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 16px;
}

.start-btn {
  width: 100%;
  margin-top: 8px;
  padding: 12px;
  background: #00ffff22;
  color: #00ffff;
  border: 1px solid #00ffff66;
  border-radius: 6px;
  font-size: 18px;
  font-weight: 600;
  cursor: pointer;
}

.start-btn:hover:not(:disabled) {
  background: #00ffff33;
}

.start-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error {
  margin-top: 12px;
  color: #ff4444;
  font-size: 14px;
  text-align: center;
}
</style>
