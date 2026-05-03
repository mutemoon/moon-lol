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
    await invoke("start_game", {
      config: {
        mode: mode.value,
        champion: champion.value,
      },
    });
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message || "Unknown error";
    isStarting.value = false;
    return;
  }

  // Start connecting (retries internally, never throws)
  ws.connect().then(() => {
    isStarting.value = false;
    view.value = "debug";
  });
}

function stopGame() {
  ws.disconnect();
  invoke("stop_game").catch(() => {});
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
        <!-- Connecting overlay -->
        <div v-if="ws.connecting.value" class="connecting-overlay">
          <div class="spinner"></div>
          <p v-if="!ws.connectTimeout.value" class="connecting-text">
            Connecting to game...
          </p>
          <p v-else class="connecting-text timeout">
            Still connecting... game may be slow to start.
          </p>
          <button class="cancel-btn" @click="stopGame">Cancel</button>
        </div>

        <div class="field">
          <label>Champion</label>
          <select v-model="champion" :disabled="isStarting">
            <option v-for="c in champions" :key="c" :value="c">{{ c }}</option>
          </select>
        </div>

        <div class="field">
          <label>Mode</label>
          <select v-model="mode" :disabled="isStarting">
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
  position: relative;
  background: #1a1a2e;
  padding: 32px;
  border-radius: 12px;
  width: 100%;
  max-width: 400px;
}

/* Connecting spinner overlay */
.connecting-overlay {
  position: absolute;
  inset: 0;
  background: #1a1a2e;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  z-index: 10;
}

.spinner {
  width: 36px;
  height: 36px;
  border: 3px solid #333;
  border-top-color: #00ffff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.connecting-text {
  color: #aaa;
  font-size: 14px;
}

.connecting-text.timeout {
  color: #ffaa00;
}

.cancel-btn {
  margin-top: 4px;
  background: #333;
  color: #999;
  border: 1px solid #444;
  padding: 6px 20px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
}

.cancel-btn:hover {
  background: #444;
  color: #ccc;
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
