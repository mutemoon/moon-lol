<script setup lang="ts">
import { ref, nextTick, watch } from "vue";
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
const logContainer = ref<HTMLElement | null>(null);
const autoScroll = ref(true);

// Auto-scroll to bottom when new logs arrive (unless user scrolled up)
watch(
  () => props.logs.length,
  () => {
    if (autoScroll.value) {
      nextTick(() => {
        if (logContainer.value) {
          logContainer.value.scrollTop = logContainer.value.scrollHeight;
        }
      });
    }
  }
);

function onLogScroll() {
  if (!logContainer.value) return;
  const el = logContainer.value;
  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
  autoScroll.value = atBottom;
}

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
      <div class="log-header">
        <h3>Log</h3>
        <button
          class="clear-btn"
          title="Clear logs"
          @click="logs.splice(0, logs.length)"
        >Clear</button>
      </div>
      <div
        class="log-entries"
        ref="logContainer"
        @scroll="onLogScroll"
      >
        <div
          v-for="entry in logs"
          :key="entry.id"
          class="log-entry"
          :class="entry.level"
        >
          <!-- Timestamp -->
          <span v-if="entry.timestamp" class="ts">{{ entry.timestamp }}</span>
          <span v-else class="ts ts-empty">--:--:--.---</span>

          <!-- Level badge -->
          <span class="badge" :class="entry.level">{{ entry.level.toUpperCase() }}</span>

          <!-- Source -->
          <span v-if="entry.source" class="src">{{ entry.source }}:{{ entry.sourceLine }}</span>

          <!-- Message -->
          <span class="msg">{{ entry.message || entry.raw }}</span>

          <!-- Repeat count -->
          <span v-if="entry.count > 1" class="count">{{ entry.count }}×</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel {
  max-width: 960px;
  margin: 0 auto;
  padding: 20px;
  height: 100vh;
  display: flex;
  flex-direction: column;
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
  flex-shrink: 0;
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
  flex-shrink: 0;
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

/* ── Log panel ── */

.log-panel {
  flex: 1;
  min-height: 0;
  background: #0d0d1a;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: #111128;
  border-bottom: 1px solid #1a1a3a;
  flex-shrink: 0;
}

.log-header h3 {
  margin: 0;
  font-size: 13px;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.clear-btn {
  padding: 2px 10px;
  font-size: 11px;
  background: transparent;
  color: #666;
  border: 1px solid #333;
  border-radius: 3px;
}

.clear-btn:hover {
  color: #aaa;
  border-color: #555;
}

.log-entries {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
  font-family: "Cascadia Code", "JetBrains Mono", "Fira Code", "Consolas", monospace;
  font-size: 12px;
  line-height: 1.5;
}

/* ── Log entry layout ── */

.log-entry {
  display: flex;
  align-items: baseline;
  gap: 6px;
  padding: 3px 14px;
  white-space: nowrap;
  transition: background 0.15s;
}

.log-entry:hover {
  background: #ffffff06;
}

.log-entry.info  { color: #99a; }
.log-entry.warn  { color: #ca8; }
.log-entry.error { color: #c66; }

/* Timestamp */
.ts {
  flex-shrink: 0;
  min-width: 82px;
  color: #556;
  font-size: 11px;
}

.ts-empty {
  opacity: 0.3;
}

/* Level badge */
.badge {
  flex-shrink: 0;
  min-width: 40px;
  padding: 0 5px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 600;
  text-align: center;
  line-height: 1.6;
}

.badge.info  { background: #223; color: #88b; }
.badge.warn  { background: #320; color: #d94; }
.badge.error { background: #300; color: #f66; }

/* Source */
.src {
  flex-shrink: 0;
  color: #556;
  font-size: 11px;
}

/* Message */
.msg {
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Repeat count badge */
.count {
  flex-shrink: 0;
  padding: 0 5px;
  border-radius: 8px;
  background: #ffffff10;
  color: #779;
  font-size: 10px;
  font-weight: 600;
}
</style>
