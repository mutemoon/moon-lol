<script setup lang="ts">
import { ref } from "vue";
import type { LogEntry } from "../composables/useWsClient";

const props = defineProps<{
  connected: boolean;
  gameState: {
    champion: string;
    godMode: boolean;
    cooldownDisabled: boolean;
    paused: boolean;
  };
  logs: LogEntry[];
}>();

const emit = defineEmits<{
  (e: "send", cmd: string, params: Record<string, unknown>): void;
  (e: "stop"): void;
}>();

const switchTarget = ref("Riven");

function toggleGodMode() {
  emit("send", "god_mode", { enabled: !props.gameState.godMode });
}

function toggleCooldown() {
  emit("send", "toggle_cooldown", { enabled: !props.gameState.cooldownDisabled });
}

function togglePause() {
  emit("send", "toggle_pause", {});
}

function resetPosition() {
  emit("send", "reset_position", {});
}

function switchChampion() {
  emit("send", "switch_champion", { name: switchTarget.value });
}

const champions = ["Riven", "Fiora"];
</script>

<template>
  <div class="panel">
    <!-- Status bar -->
    <div class="status-bar">
      <span class="dot" :class="connected ? 'on' : 'off'"></span>
      <span>{{ connected ? 'Connected' : 'Disconnected' }}</span>
      <span class="champ">Champion: {{ gameState.champion || '—' }}</span>
      <button class="stop-btn" @click="$emit('stop')">Stop Game</button>
    </div>

    <!-- Controls -->
    <div class="controls">
      <div class="control-group">
        <label>Switch Champion</label>
        <div class="row">
          <select v-model="switchTarget">
            <option v-for="c in champions" :key="c" :value="c">{{ c }}</option>
          </select>
          <button @click="switchChampion">Switch</button>
        </div>
      </div>

      <div class="control-group toggles">
        <button
          :class="{ active: gameState.godMode }"
          @click="toggleGodMode"
        >
          God Mode: {{ gameState.godMode ? 'ON' : 'OFF' }}
        </button>
        <button
          :class="{ active: gameState.cooldownDisabled }"
          @click="toggleCooldown"
        >
          No Cooldown: {{ gameState.cooldownDisabled ? 'ON' : 'OFF' }}
        </button>
        <button
          :class="{ active: gameState.paused }"
          @click="togglePause"
        >
          {{ gameState.paused ? '▶ Resume' : '⏸ Pause' }}
        </button>
      </div>

      <div class="control-group">
        <button @click="resetPosition">Reset Position</button>
      </div>
    </div>

    <!-- Log panel -->
    <div class="log-panel">
      <h3>Log</h3>
      <div class="log-entries" ref="logContainer">
        <div
          v-for="entry in logs"
          :key="entry.id"
          class="log-entry"
          :class="entry.level"
        >
          <span class="level">{{ entry.level }}</span>
          <span class="msg">{{ entry.msg }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel {
  max-width: 900px;
  margin: 0 auto;
  padding: 20px;
}

.status-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  background: #1a1a2e;
  border-radius: 8px;
  margin-bottom: 16px;
  font-size: 14px;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot.on { background: #39ff14; }
.dot.off { background: #ff4444; }

.champ {
  color: #00ffff;
  font-weight: 600;
  margin-left: auto;
}

.stop-btn {
  background: #ff4444;
  color: white;
  border: none;
  padding: 4px 12px;
  border-radius: 4px;
  cursor: pointer;
}

.controls {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 16px;
}

.control-group {
  background: #1a1a2e;
  padding: 12px;
  border-radius: 8px;
}

.control-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 12px;
  color: #888;
}

.row {
  display: flex;
  gap: 8px;
}

select {
  background: #0d0d1a;
  color: #fff;
  border: 1px solid #333;
  padding: 6px 10px;
  border-radius: 4px;
  font-size: 14px;
}

button {
  background: #2a2a4a;
  color: #ccc;
  border: 1px solid #444;
  padding: 6px 14px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  white-space: nowrap;
}

button:hover {
  background: #3a3a5a;
}

button.active {
  background: #00ffff22;
  border-color: #00ffff;
  color: #00ffff;
}

.toggles {
  display: flex;
  gap: 8px;
}

.log-panel {
  background: #0d0d1a;
  border-radius: 8px;
  padding: 12px;
  max-height: 300px;
  overflow-y: auto;
}

.log-panel h3 {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: #888;
}

.log-entry {
  font-family: monospace;
  font-size: 12px;
  padding: 2px 0;
}
.log-entry.info { color: #aaa; }
.log-entry.warn { color: #ffaa00; }
.log-entry.error { color: #ff4444; }

.level {
  color: #666;
  margin-right: 8px;
  text-transform: uppercase;
}
</style>
