<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useWsClient } from "./composables/useWsClient";
import LauncherPanel from "./components/LauncherPanel.vue";
import DebugPanel from "./components/DebugPanel.vue";

const win = getCurrentWindow();

type View = "launcher" | "debug";

const currentView = ref<View>("launcher");
const champion = ref("Riven");
const mode = ref("sandbox");
const launchError = ref("");
const isStarting = ref(false);

const champions = ["Riven", "Fiora"];

const ws = useWsClient("ws://127.0.0.1:9001");

function minimize() {
  win.minimize();
}
function toggleMaximize() {
  win.toggleMaximize();
}
function closeWindow() {
  win.close();
}

async function onNavMouseDown(e: MouseEvent) {
  // Don't start drag when clicking interactive elements
  const target = e.target as HTMLElement;
  if (target.closest("button, select, input, a")) return;
  try {
    await win.startDragging();
  } catch {
    /* ignore */
  }
}

async function startGame() {
  launchError.value = "";
  isStarting.value = true;

  try {
    await invoke("start_game", {
      config: { mode: mode.value, champion: champion.value },
    });
  } catch (e: any) {
    launchError.value = typeof e === "string" ? e : e.message || "Unknown error";
    isStarting.value = false;
    return;
  }

  ws.connect().then(() => {
    isStarting.value = false;
    currentView.value = "debug";
  });
}

function stopGame() {
  ws.disconnect();
  invoke("stop_game").catch(() => {});
  currentView.value = "launcher";
}
</script>

<template>
  <div class="app">
    <!-- Ambient background texture -->
    <div class="bg-texture" aria-hidden="true"></div>
    <div class="bg-glow" aria-hidden="true"></div>

    <!-- Navigation -->
    <nav class="navbar" @mousedown="onNavMouseDown">
      <div class="nav-brand">
        <!-- <svg class="nav-logo" viewBox="0 0 28 28" width="28" height="28" fill="none" aria-hidden="true">
          <circle cx="14" cy="14" r="12" stroke="currentColor" stroke-width="1.5" />
          <path d="M8 14 L20 14 M14 8 L14 20" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg> -->
        <span class="nav-title">MoonLOL</span>
      </div>

      <div class="nav-tabs">
        <button class="nav-tab" :class="{ active: currentView === 'launcher' }" @click="currentView = 'launcher'">
          Home
        </button>
        <button
          class="nav-tab"
          :class="{ active: currentView === 'debug' }"
          :disabled="!ws.connected.value"
          @click="currentView = 'debug'"
        >
          Debug
        </button>
        <button class="nav-tab" disabled>Stats</button>
        <button class="nav-tab" disabled>Settings</button>
      </div>

      <div class="nav-status">
        <span class="status-dot" :class="ws.connected.value ? 'connected' : 'disconnected'"></span>
        <span class="status-label">{{ ws.connected.value ? "Connected" : "Offline" }}</span>
      </div>

      <!-- Window Controls -->
      <div class="window-controls">
        <button class="win-btn" @click="minimize" aria-label="Minimize">
          <svg viewBox="0 0 12 12" width="12" height="12" fill="none">
            <path d="M2 6 L10 6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
        </button>
        <button class="win-btn" @click="toggleMaximize" aria-label="Toggle maximize">
          <svg viewBox="0 0 12 12" width="12" height="12" fill="none">
            <rect x="1.5" y="1.5" width="9" height="9" rx="1" stroke="currentColor" stroke-width="1.2" />
          </svg>
        </button>
        <button class="win-btn win-close" @click="closeWindow" aria-label="Close">
          <svg viewBox="0 0 12 12" width="12" height="12" fill="none">
            <path
              d="M2.5 2.5 L9.5 9.5 M9.5 2.5 L2.5 9.5"
              stroke="currentColor"
              stroke-width="1.2"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
    </nav>

    <!-- Content -->
    <main class="content">
      <Transition name="fade" mode="out-in">
        <LauncherPanel
          v-if="currentView === 'launcher'"
          :champion="champion"
          :mode="mode"
          :is-starting="isStarting"
          :error="launchError"
          :connecting="ws.connecting.value"
          :connect-timeout="ws.connectTimeout.value"
          :champions="champions"
          @update:champion="champion = $event"
          @update:mode="mode = $event"
          @start="startGame"
          @cancel="stopGame"
        />
        <DebugPanel
          v-else
          :connected="ws.connected.value"
          :game-state="ws.gameState.value"
          :logs="ws.logs.value"
          :agent-observe="ws.agentObserve.value"
          :agent-thinking="ws.agentThinking.value"
          :agent-action="ws.agentAction.value"
          @send="(cmd, params) => ws.send(cmd, params)"
          @stop="stopGame"
        />
      </Transition>
    </main>
  </div>
</template>

<style>
/* ── Design Tokens ── */
:root {
  /* Neutral palette */
  --bg-deep: #070608;
  --bg-surface: #121013;
  --bg-elevated: #1c1820;
  --bg-raised: #292231;
  --border-subtle: #352c3d;
  --border-default: #443a50;
  --text-muted: #685e5a;
  --text-default: #9a9282;
  --text-bright: #dbd6c5;

  /* Gold accent */
  --gold-dimmer: #785b28;
  --gold-muted: #927136;
  --gold-default: #b99147;
  --gold-bright: #d4af5c;
  --gold-glow: #e8c97a;

  /* Semantic */
  --red: #c84a4a;
  --green: #4a9e5a;
  --blue: #4a7ec4;

  /* Typography */
  --font-display: "Noto Serif SC", "Noto Serif", Georgia, serif;
  --font-body: "Noto Sans SC", "Noto Sans", "Segoe UI", Arial, sans-serif;
  --font-mono: "Cascadia Code", "JetBrains Mono", "Fira Code", Consolas, monospace;

  --fs-display: 40px;
  --fs-h1: 28px;
  --fs-h2: 20px;
  --fs-h3: 16px;
  --fs-body: 14px;
  --fs-small: 12px;
  --fs-tiny: 11px;
  --fs-mono: 13px;

  /* Shadows */
  --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.4), 0 0 1px rgba(120, 91, 40, 0.15);
  --shadow-md: 0 4px 12px rgba(0, 0, 0, 0.5), 0 0 2px rgba(120, 91, 40, 0.2);
  --shadow-lg: 0 12px 32px rgba(0, 0, 0, 0.6), 0 0 4px rgba(120, 91, 40, 0.15);
  --shadow-glow-gold: 0 0 12px rgba(201, 170, 113, 0.25), 0 0 4px rgba(201, 170, 113, 0.4);

  /* Radii */
  --radius-sm: 3px;
  --radius-md: 6px;
  --radius-lg: 10px;
  --radius-xl: 16px;

  /* Motion */
  --dur-instant: 0.1s;
  --dur-fast: 0.2s;
  --dur-normal: 0.3s;
  --dur-slow: 0.5s;
}

*,
*::before,
*::after {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body {
  height: 100%;
  overflow: hidden;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  font-family: var(--font-body);
  font-size: var(--fs-body);
  color: var(--text-default);
  background: var(--bg-deep);
  line-height: 1.5;
}

/* Selection */
::selection {
  background: rgba(185, 145, 71, 0.3);
  color: var(--text-bright);
}

/* Scrollbar */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: var(--border-default);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--gold-muted);
}

/* Focus ring */
:focus-visible {
  outline: 1px solid var(--gold-default);
  outline-offset: 2px;
}

/* Button reset */
button {
  font-family: inherit;
  cursor: pointer;
  border: none;
  background: none;
}

/* Select reset */
select {
  font-family: inherit;
}
</style>

<style scoped>
.app {
  height: 100vh;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}

/* Background texture overlay */
.bg-texture {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  opacity: 0.035;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
  background-repeat: repeat;
  background-size: 256px 256px;
  mix-blend-mode: overlay;
}

/* Warm gold ambient glow */
.bg-glow {
  position: fixed;
  top: -30vh;
  left: 50%;
  transform: translateX(-50%);
  width: 80vw;
  height: 60vh;
  pointer-events: none;
  z-index: 0;
  background: radial-gradient(ellipse at center, rgba(120, 91, 40, 0.08) 0%, transparent 70%);
}

/* ── Nav Bar ── */
.navbar {
  position: relative;
  z-index: 10;
  display: flex;
  align-items: center;
  height: 64px;
  padding: 0 24px;
  gap: 8px;
  background: rgba(7, 6, 8, 0.85);
  backdrop-filter: blur(40px);
  -webkit-backdrop-filter: blur(40px);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.nav-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-right: 32px;
  flex-shrink: 0;
}

.nav-logo {
  color: var(--gold-bright);
}

.nav-title {
  font-family: var(--font-display);
  font-size: var(--fs-h3);
  font-weight: 700;
  color: var(--gold-bright);
  letter-spacing: 0.06em;
}

.nav-tabs {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: auto;
}

.nav-tab {
  position: relative;
  padding: 8px 18px;
  font-size: var(--fs-small);
  font-weight: 500;
  color: var(--text-muted);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  transition: color var(--dur-fast) ease-out;
  border-radius: var(--radius-sm);
}

.nav-tab:hover:not(:disabled) {
  color: var(--text-bright);
}

.nav-tab.active {
  color: var(--gold-bright);
}

.nav-tab.active::after {
  content: "";
  position: absolute;
  bottom: -2px;
  left: 50%;
  transform: translateX(-50%);
  width: 24px;
  height: 2px;
  background: var(--gold-default);
  border-radius: 1px;
}

.nav-tab:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.nav-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 16px;
  padding-left: 16px;
  border-left: 1px solid var(--border-subtle);
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  transition: box-shadow var(--dur-normal) ease-out;
}

.status-dot.connected {
  background: var(--green);
  box-shadow: 0 0 8px rgba(74, 158, 90, 0.5);
}

.status-dot.disconnected {
  background: var(--red);
  box-shadow: 0 0 8px rgba(200, 74, 74, 0.3);
}

.status-label {
  font-size: var(--fs-tiny);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-weight: 500;
}

/* ── Window Controls ── */
.window-controls {
  display: flex;
  align-items: center;
  margin-right: -16px;
  height: 100%;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 100%;
  color: var(--text-muted);
  transition: all var(--dur-instant) ease-out;
  border-radius: 0;
}

.win-btn:hover {
  background: var(--bg-elevated);
  color: var(--text-bright);
}

.win-btn.win-close:hover {
  background: var(--red);
  color: #fff;
}

/* ── Content ── */
.content {
  position: relative;
  z-index: 1;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

/* ── Transitions ── */
.fade-enter-active,
.fade-leave-active {
  transition: opacity var(--dur-normal) ease-out;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
