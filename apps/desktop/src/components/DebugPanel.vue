<script setup lang="ts">
import { ref, computed } from "vue";
import AgentCockpit from "./AgentCockpit.vue";
import GameConsoleLogs from "./GameConsoleLogs.vue";

const props = defineProps<{
  connected: boolean;
  gameState: {
    champion: string;
    godMode: boolean;
    cooldownDisabled: boolean;
    paused: boolean;
  };
  agentObserve?: any;
  agentThinking?: string;
  agentAction?: string;
}>();

const emit = defineEmits<{
  (e: "send", cmd: string, params: Record<string, unknown>): void;
  (e: "stop"): void;
}>();

const switchTarget = ref("Riven");

// 仅用于状态栏的简单计算
const simulationTime = computed(() => props.agentObserve?.time || 0);

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
const activeTab = ref<"cockpit" | "logs">("cockpit");
</script>

<template>
  <div class="debug-container">
    <!-- Status Bar -->
    <div class="status-bar">
      <div class="status-left">
        <span class="status-badge" :class="connected ? 'on' : 'off'">
          <span class="badge-dot"></span>
          {{ connected ? "Connected" : "Disconnected" }}
        </span>
        <span class="status-divider"></span>
        <span class="status-champ">
          <span class="champ-label">Champion</span>
          <span class="champ-value">{{ gameState.champion || "—" }}</span>
        </span>
        <template v-if="simulationTime > 0">
          <span class="status-divider"></span>
          <span class="status-champ">
            <span class="champ-label">Sim Time</span>
            <span class="champ-value gold">{{ simulationTime.toFixed(1) }}s</span>
          </span>
        </template>
      </div>
      <button class="stop-btn" @click="$emit('stop')">Stop Game</button>
    </div>

    <!-- Main Workspace Layout -->
    <div class="workspace">
      <!-- LEFT COLUMN: Global Control Sidebar -->
      <div class="sidebar-col">
        <div class="control-group">
          <span class="group-label">Toggles</span>
          <div class="toggle-column">
            <button class="toggle-btn" :class="{ active: gameState.godMode }" @click="toggleGodMode">
              <span class="toggle-indicator"></span>
              God Mode
            </button>
            <button class="toggle-btn" :class="{ active: gameState.cooldownDisabled }" @click="toggleCooldown">
              <span class="toggle-indicator"></span>
              No Cooldown
            </button>
            <button class="toggle-btn" :class="{ active: gameState.paused }" @click="togglePause">
              <span class="toggle-indicator"></span>
              {{ gameState.paused ? "Resume" : "Pause" }}
            </button>
          </div>
        </div>

        <div class="control-group">
          <span class="group-label">Champion</span>
          <div class="champ-switch-column">
            <div class="select-wrap small full-width">
              <select v-model="switchTarget">
                <option v-for="c in champions" :key="c" :value="c">{{ c }}</option>
              </select>
            </div>
            <button class="btn-ghost full-width" @click="switchChampion">Switch Champion</button>
          </div>
        </div>

        <div class="control-group">
          <span class="group-label">Actions</span>
          <button class="btn-ghost full-width" @click="resetPosition">Reset Position</button>
        </div>
      </div>

      <!-- RIGHT COLUMN: Interactive Tabs Workspace -->
      <div class="main-tabs-col">
        <!-- Tab Navigation Bar -->
        <div class="tabs-nav-bar">
          <button class="tab-nav-btn" :class="{ active: activeTab === 'cockpit' }" @click="activeTab = 'cockpit'">
            <span>🤖</span>
            AI Agent Cockpit
          </button>
          <button class="tab-nav-btn" :class="{ active: activeTab === 'logs' }" @click="activeTab = 'logs'">
            <span>📋</span>
            Game Console Logs
          </button>
        </div>

        <!-- Tab Contents Viewport -->
        <div class="tab-viewport">
          <!-- Cockpit Tab -->
          <AgentCockpit
            v-show="activeTab === 'cockpit'"
            :agent-observe="agentObserve"
            :agent-thinking="agentThinking"
            :agent-action="agentAction"
          />

          <!-- Logs Tab -->
          <GameConsoleLogs v-show="activeTab === 'logs'" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
@reference "tailwindcss";
@reference "../style.css";

.debug-container {
  @apply bg-bg-deep flex h-full flex-col gap-3 overflow-hidden p-4;
}
.status-bar {
  @apply bg-bg-surface border-border-subtle flex shrink-0 items-center justify-between rounded border px-3.5 py-2 shadow-[0_1px_2px_rgba(0,0,0,0.4)];
}
.status-left {
  @apply flex items-center gap-3;
}
.status-badge {
  @apply inline-flex items-center gap-1.5 rounded border border-transparent px-2 py-0.5 text-[11px] font-semibold tracking-wider uppercase;
  &.on {
    @apply text-green border-[rgba(74,158,90,0.15)] bg-[rgba(74,158,90,0.08)];
  }
  &.off {
    @apply text-red border-[rgba(200,74,74,0.15)] bg-[rgba(200,74,74,0.08)];
  }
}
.badge-dot {
  @apply h-1.5 w-1.5 rounded-full;
  .status-badge.on & {
    @apply bg-green shadow-[0_0_6px_rgba(74,158,90,0.6)];
  }
  .status-badge.off & {
    @apply bg-red shadow-[0_0_6px_rgba(200,74,74,0.4)];
  }
}
.status-divider {
  @apply bg-border-subtle h-3.5 w-px;
}
.status-champ {
  @apply flex items-center gap-1.5;
}
.champ-label {
  @apply text-text-muted text-[11px] uppercase;
}
.champ-value {
  @apply text-text-bright text-xs font-semibold;
  &.gold {
    @apply text-gold-bright;
  }
}
.stop-btn {
  @apply text-red rounded border border-[rgba(200,74,74,0.25)] bg-[rgba(200,74,74,0.04)] px-3 py-1 text-xs font-medium transition-all duration-200 hover:border-[rgba(200,74,74,0.45)] hover:bg-[rgba(200,74,74,0.12)];
}
.workspace {
  @apply flex min-h-0 flex-1 gap-3.5 overflow-hidden;
}
.sidebar-col {
  @apply flex min-h-0 w-44 flex-col gap-3 overflow-hidden;
}
.main-tabs-col {
  @apply flex min-h-0 flex-1 flex-col overflow-hidden;
}
.control-group {
  @apply bg-bg-surface border-border-subtle flex flex-1 flex-col gap-1.5 rounded border p-2.5;
}
.group-label {
  @apply text-text-muted text-[11px] font-semibold uppercase;
}
.toggle-column {
  @apply flex flex-col gap-1;
}
.toggle-btn {
  @apply text-text-muted bg-bg-deep border-border-subtle hover:text-text-default hover:border-gold-muted inline-flex flex-1 shrink-0 items-center gap-1 rounded border px-2.5 py-1 text-xs whitespace-nowrap transition-all duration-200;
  &.active {
    @apply text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.06)];
  }
}
.toggle-indicator {
  @apply bg-border-default h-1.5 w-1.5 rounded-full;
  .toggle-btn.active & {
    @apply bg-gold-default shadow-[0_0_6px_rgba(185,145,71,0.5)];
  }
}
.champ-switch-column {
  @apply flex flex-col gap-1.5;
}
.select-wrap.small select {
  @apply bg-bg-deep text-text-bright border-gold-dimmer w-full cursor-pointer appearance-none rounded border py-1 pr-5 pl-2 text-xs shadow-[inset_0_2px_4px_rgba(0,0,0,0.4)];
}
.btn-ghost {
  @apply text-text-muted border-border-subtle hover:text-gold-bright hover:border-gold-muted cursor-pointer rounded border bg-transparent px-2.5 py-1 text-xs transition-all duration-200;
  &.small {
    @apply px-1.5 py-0.5 text-[11px];
  }
}
.full-width {
  @apply w-full;
}
.tabs-nav-bar {
  @apply bg-bg-elevated border-border-subtle flex shrink-0 border-b;
}
.tab-nav-btn {
  @apply text-text-muted hover:text-text-default flex items-center gap-1.5 border-b-2 border-transparent px-4 py-2 text-xs font-medium transition-all duration-200;
  &.active {
    @apply text-gold-bright border-b-gold-default;
  }
}
.tab-viewport {
  @apply flex min-h-0 flex-1 flex-col overflow-hidden;
}
</style>
