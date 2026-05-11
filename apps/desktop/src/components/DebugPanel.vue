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
  <div class="debug">
    <!-- Status Bar -->
    <div class="status-bar">
      <div class="status-left">
        <span class="status-badge" :class="connected ? 'on' : 'off'">
          <span class="badge-dot"></span>
          {{ connected ? 'Connected' : 'Disconnected' }}
        </span>
        <span class="status-divider"></span>
        <span class="status-champ">
          <span class="champ-label">Champion</span>
          <span class="champ-value">{{ gameState.champion || '—' }}</span>
        </span>
      </div>
      <button class="stop-btn" @click="$emit('stop')">Stop Game</button>
    </div>

    <!-- Controls -->
    <section class="controls-section">
      <div class="control-row">
        <div class="control-group">
          <span class="group-label">Toggles</span>
          <div class="toggle-row">
            <button
              class="toggle-btn"
              :class="{ active: gameState.godMode }"
              @click="toggleGodMode"
            >
              <span class="toggle-indicator"></span>
              God Mode
            </button>
            <button
              class="toggle-btn"
              :class="{ active: gameState.cooldownDisabled }"
              @click="toggleCooldown"
            >
              <span class="toggle-indicator"></span>
              No Cooldown
            </button>
            <button
              class="toggle-btn"
              :class="{ active: gameState.paused }"
              @click="togglePause"
            >
              <span class="toggle-indicator"></span>
              {{ gameState.paused ? 'Resume' : 'Pause' }}
            </button>
          </div>
        </div>

        <div class="control-group">
          <span class="group-label">Champion</span>
          <div class="champ-switch-row">
            <div class="select-wrap small">
              <select v-model="switchTarget">
                <option v-for="c in champions" :key="c" :value="c">{{ c }}</option>
              </select>
            </div>
            <button class="btn-ghost" @click="switchChampion">Switch</button>
          </div>
        </div>

        <div class="control-group action-group">
          <button class="btn-ghost" @click="resetPosition">Reset Position</button>
        </div>
      </div>
    </section>

    <!-- Log Panel -->
    <section class="log-section">
      <div class="log-header">
        <div class="log-header-left">
          <h2 class="log-title">Game Log</h2>
          <span class="log-count">{{ logs.length }} entries</span>
        </div>
        <button
          class="btn-ghost small"
          @click="logs.splice(0, logs.length)"
        >
          Clear
        </button>
      </div>

      <div
        class="log-entries"
        ref="logContainer"
        @scroll="onLogScroll"
      >
        <div v-if="logs.length === 0" class="log-empty">
          <svg class="empty-icon" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <path d="M4 4 L20 4 L20 20 L4 20 Z" />
            <path d="M8 8 L16 8" />
            <path d="M8 12 L16 12" />
            <path d="M8 16 L12 16" />
          </svg>
          <span>Waiting for log output…</span>
        </div>
        <template v-for="entry in logs" :key="entry.id">
          <div class="log-entry" :class="entry.level" :title="entry.raw">
            <span v-if="entry.timestamp" class="entry-ts">{{ entry.timestamp }}</span>
            <span v-else class="entry-ts dim">--:--:--.---</span>
            <span class="entry-badge" :class="entry.level">{{ entry.level }}</span>
            <span v-if="entry.source" class="entry-src">{{ entry.source }}:{{ entry.sourceLine }}</span>
            <span class="entry-msg">{{ entry.message || entry.raw }}</span>
            <span v-if="entry.count > 1" class="entry-count">{{ entry.count }}×</span>
          </div>
        </template>
      </div>
    </section>
  </div>
</template>

<style scoped>
.debug {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 20px 24px;
  gap: 16px;
}

/* ── Status Bar ── */
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  padding: 10px 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-sm);
}

.status-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 3px 10px;
  font-size: var(--fs-tiny);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
}

.status-badge.on {
  color: var(--green);
  background: rgba(74,158,90,0.08);
  border-color: rgba(74,158,90,0.2);
}

.status-badge.off {
  color: var(--red);
  background: rgba(200,74,74,0.08);
  border-color: rgba(200,74,74,0.2);
}

.badge-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
}

.status-badge.on .badge-dot {
  background: var(--green);
  box-shadow: 0 0 6px rgba(74,158,90,0.6);
}

.status-badge.off .badge-dot {
  background: var(--red);
  box-shadow: 0 0 6px rgba(200,74,74,0.4);
  animation: pulse-red 2s ease-in-out infinite;
}

@keyframes pulse-red {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.status-divider {
  width: 1px;
  height: 16px;
  background: var(--border-subtle);
}

.status-champ {
  display: flex;
  align-items: center;
  gap: 8px;
}

.champ-label {
  font-size: var(--fs-tiny);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.champ-value {
  font-size: var(--fs-small);
  font-weight: 600;
  color: var(--gold-bright);
}

.stop-btn {
  padding: 5px 16px;
  font-size: var(--fs-small);
  font-weight: 500;
  color: var(--red);
  border: 1px solid rgba(200,74,74,0.3);
  border-radius: var(--radius-sm);
  background: rgba(200,74,74,0.06);
  transition: all var(--dur-fast) ease-out;
}

.stop-btn:hover {
  background: rgba(200,74,74,0.14);
  border-color: rgba(200,74,74,0.5);
  box-shadow: 0 0 10px rgba(200,74,74,0.15);
}

/* ── Controls Section ── */
.controls-section {
  flex-shrink: 0;
}

.control-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.control-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
}

.group-label {
  font-size: var(--fs-tiny);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.toggle-row {
  display: flex;
  gap: 6px;
}

.toggle-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  font-size: var(--fs-small);
  color: var(--text-muted);
  background: var(--bg-deep);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  transition: all var(--dur-fast) ease-out;
}

.toggle-btn:hover {
  color: var(--text-default);
  border-color: var(--gold-muted);
}

.toggle-btn.active {
  color: var(--gold-bright);
  border-color: var(--gold-dimmer);
  background: rgba(185,145,71,0.08);
  box-shadow: inset 0 0 0 1px rgba(185,145,71,0.15);
}

.toggle-indicator {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--border-default);
  transition: all var(--dur-fast) ease-out;
}

.toggle-btn.active .toggle-indicator {
  background: var(--gold-default);
  box-shadow: 0 0 6px rgba(185,145,71,0.5);
}

/* Champion switch */
.champ-switch-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.select-wrap.small select {
  appearance: none;
  background: var(--bg-deep);
  color: var(--text-bright);
  border: 1px solid var(--gold-dimmer);
  border-radius: var(--radius-sm);
  padding: 5px 28px 5px 10px;
  font-size: var(--fs-small);
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.5);
  cursor: pointer;
}

.select-wrap.small select:focus {
  border-color: var(--gold-default);
  outline: none;
}

/* Ghost button */
.btn-ghost {
  padding: 5px 14px;
  font-size: var(--fs-small);
  font-weight: 500;
  color: var(--text-muted);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  transition: all var(--dur-fast) ease-out;
}

.btn-ghost:hover {
  color: var(--gold-bright);
  border-color: var(--gold-muted);
}

.btn-ghost.small {
  padding: 3px 10px;
  font-size: var(--fs-tiny);
}

.action-group {
  justify-content: center;
}

/* ── Log Section ── */
.log-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  padding: 10px 16px;
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border-subtle);
}

.log-header-left {
  display: flex;
  align-items: baseline;
  gap: 10px;
}

.log-title {
  font-size: var(--fs-small);
  font-weight: 600;
  color: var(--text-default);
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.log-count {
  font-size: var(--fs-tiny);
  color: var(--text-muted);
}

.log-entries {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.6;
}

/* Empty state */
.log-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 100%;
  min-height: 120px;
  color: var(--text-muted);
  font-size: var(--fs-small);
}

.empty-icon {
  opacity: 0.3;
}

/* ── Log Entry ── */
.log-entry {
  display: flex;
  align-items: baseline;
  gap: 6px;
  padding: 2px 16px;
  white-space: nowrap;
  transition: background var(--dur-instant) ease-out;
}

.log-entry:hover {
  background: rgba(255,255,255,0.02);
}

.log-entry.info {
  color: var(--text-default);
}

.log-entry.warn {
  color: #ca8;
}

.log-entry.error {
  color: var(--red);
}

.entry-ts {
  flex-shrink: 0;
  min-width: 76px;
  font-size: 11px;
  color: var(--border-default);
}

.entry-ts.dim {
  opacity: 0.35;
}

.entry-badge {
  flex-shrink: 0;
  min-width: 34px;
  padding: 0 5px;
  font-size: 10px;
  font-weight: 600;
  text-align: center;
  line-height: 1.7;
  border-radius: var(--radius-sm);
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.entry-badge.info {
  background: rgba(154,146,130,0.1);
  color: var(--text-muted);
}

.entry-badge.warn {
  background: rgba(204,170,136,0.12);
  color: #ca8;
}

.entry-badge.error {
  background: rgba(200,74,74,0.12);
  color: var(--red);
}

.entry-src {
  flex-shrink: 0;
  color: var(--border-default);
  font-size: 11px;
}

.entry-msg {
  overflow: hidden;
  text-overflow: ellipsis;
}

.entry-count {
  flex-shrink: 0;
  padding: 0 6px;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
  background: rgba(255,255,255,0.04);
  border-radius: 8px;
  line-height: 1.6;
}
</style>
