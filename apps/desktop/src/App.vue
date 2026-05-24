<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useWsClient } from "./composables/useWsClient";
import { createLogContext } from "./composables/useLogPoller";
import LauncherPanel from "./components/LauncherPanel.vue";
import DebugPanel from "./components/DebugPanel.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import "./style.css";

const win = getCurrentWindow();

type View = "launcher" | "debug" | "settings";

const currentView = ref<View>("launcher");
const champion = ref("Riven");
const mode = ref("sandbox");
const launchError = ref("");
const isStarting = ref(false);
const showStatsModal = ref(false);
const statsResult = ref({ minionKills: 0, gold: 0.0 });

listen<any>("agent-finished", (event) => {
  statsResult.value = {
    minionKills: event.payload.minion_kills,
    gold: event.payload.gold,
  };
  showStatsModal.value = true;
});

const champions = ["Riven", "Fiora"];

const ws = useWsClient();
const log = createLogContext();
watch(
  () => ws.selectedEntityId.value,
  (entityId) => {
    if (entityId !== null) {
      log.setEntityFilter(entityId);
    }
  },
);

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
    log.start();
    isStarting.value = false;
    currentView.value = "debug";
  });
}

function stopGame() {
  ws.disconnect();
  log.stop();
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
      <div class="mr-8 flex shrink-0 items-center gap-2.5">
        <span class="font-display text-gold-bright text-base font-bold tracking-widest">MoonLOL</span>
      </div>

      <div class="ml-auto flex items-center gap-0.5">
        <button
          class="nav-tab"
          :class="{ 'nav-tab-active': currentView === 'launcher' }"
          @click="currentView = 'launcher'"
        >
          Home
        </button>
        <button
          class="nav-tab"
          :class="{ 'nav-tab-active': currentView === 'debug' }"
          :disabled="!ws.connected.value"
          @click="currentView = 'debug'"
        >
          Debug
        </button>
        <button class="nav-tab disabled:cursor-not-allowed disabled:opacity-35" disabled>Stats</button>
        <button
          class="nav-tab"
          :class="{ 'nav-tab-active': currentView === 'settings' }"
          @click="currentView = 'settings'"
        >
          Settings
        </button>
      </div>

      <div class="border-border-subtle ml-4 flex items-center gap-2 border-l pl-4">
        <span
          class="h-1.5 w-1.5 shrink-0 rounded-full transition-shadow"
          :class="
            ws.connected.value
              ? 'bg-green shadow-[0_0_8px_rgba(74,158,90,0.5)]'
              : 'bg-red shadow-[0_0_8px_rgba(200,74,74,0.3)]'
          "
        ></span>
        <span class="text-text-muted text-xs font-medium tracking-wider uppercase">
          {{ ws.connected.value ? "Connected" : "Offline" }}
        </span>
      </div>

      <!-- Window Controls -->
      <div class="-mr-4 flex h-full items-center">
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
        <button class="win-btn win-btn-close" @click="closeWindow" aria-label="Close">
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
          v-else-if="currentView === 'debug'"
          :connected="ws.connected.value"
          :game-state="ws.gameState.value"
          @send="(cmd, params) => ws.send(cmd, params)"
          @stop="stopGame"
        />
        <SettingsPanel v-else-if="currentView === 'settings'" />
      </Transition>
    </main>

    <!-- Premium Agent Stats Modal -->
    <Transition name="fade">
      <div v-if="showStatsModal" class="stats-modal-overlay">
        <div class="stats-modal-card">
          <div class="stats-modal-glow"></div>
          <div class="stats-modal-header">
            <span class="gold-trophy-icon">🏆</span>
            <h2 class="stats-modal-title">AI Agent 模拟测试报告</h2>
          </div>
          
          <div class="stats-modal-divider"></div>
          
          <div class="stats-modal-body">
            <p class="stats-modal-subtitle">AI 代理已成功运行并自主决策满 2 分钟，累计运行数据统计如下：</p>
            <div class="stats-modal-grid">
              <div class="stats-item card">
                <span class="stats-item-lbl">总击杀小兵 (补刀)</span>
                <span class="stats-item-val gold-neon">{{ statsResult.minionKills }}</span>
              </div>
              <div class="stats-item card">
                <span class="stats-item-lbl">总累计金币 (Gold)</span>
                <span class="stats-item-val gold-neon">{{ statsResult.gold.toFixed(0) }} <span class="gold-unit">g</span></span>
              </div>
            </div>
          </div>
          
          <div class="stats-modal-footer">
            <button class="stats-modal-btn" @click="showStatsModal = false">确认并返回</button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

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
  pointer-events: none;
  z-index: 0;
  top: -30vh;
  left: 50%;
  transform: translateX(-50%);
  width: 80vw;
  height: 60vh;
  background: radial-gradient(ellipse at center, rgba(120, 91, 40, 0.08) 0%, transparent 70%);
}

/* Nav Bar */
.navbar {
  position: relative;
  z-index: 10;
  display: flex;
  align-items: center;
  height: 4rem;
  padding: 0 1.5rem;
  gap: 0.5rem;
  background: rgba(7, 6, 8, 0.85);
  backdrop-filter: blur(40px);
  -webkit-backdrop-filter: blur(40px);
  border-bottom: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

/* Nav Tab */
.nav-tab {
  position: relative;
  padding: 0.5rem 1rem;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-muted);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  transition: color 0.2s;
  border-radius: 3px;
}

.nav-tab:hover:not(:disabled) {
  color: var(--color-text-bright);
}

.nav-tab-active {
  color: var(--color-gold-bright);
}

.nav-tab-active::after {
  content: "";
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 1.5rem;
  height: 2px;
  background: var(--color-gold-default);
  border-radius: 1px;
}

/* Window Controls */
.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.5rem;
  height: 100%;
  color: var(--color-text-muted);
  transition: all 0.1s;
  border-radius: 0;
}

.win-btn:hover {
  background: var(--color-bg-elevated);
  color: var(--color-text-bright);
}

.win-btn-close:hover {
  background: var(--color-red);
  color: #fff;
}

/* Content */
.content {
  position: relative;
  z-index: 1;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease-out;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* Stats Modal Premium Styling */
.stats-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(7, 6, 8, 0.85);
  backdrop-filter: blur(12px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1.5rem;
}

.stats-modal-card {
  position: relative;
  background: #110e14;
  border: 1px solid var(--color-gold-muted);
  border-radius: 8px;
  width: 100%;
  max-width: 480px;
  padding: 2.2rem;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.8), 0 0 40px rgba(185, 145, 71, 0.15);
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  overflow: hidden;
}

.stats-modal-glow {
  position: absolute;
  top: -100px;
  left: 50%;
  transform: translateX(-50%);
  width: 300px;
  height: 150px;
  background: radial-gradient(circle, rgba(185, 145, 71, 0.25) 0%, transparent 70%);
  pointer-events: none;
}

.stats-modal-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  z-index: 1;
}

.gold-trophy-icon {
  font-size: 2rem;
  filter: drop-shadow(0 0 8px rgba(185, 145, 71, 0.5));
  animation: float 2.5s ease-in-out infinite;
}

.stats-modal-title {
  font-family: var(--font-display);
  color: var(--color-gold-bright);
  font-size: 1.4rem;
  font-weight: 700;
  letter-spacing: 0.05em;
  margin: 0;
}

.stats-modal-divider {
  height: 1px;
  background: linear-gradient(to right, transparent, var(--color-gold-muted) 20%, var(--color-gold-muted) 80%, transparent);
}

.stats-modal-body {
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
  z-index: 1;
}

.stats-modal-subtitle {
  color: var(--color-text-muted);
  font-size: 0.9rem;
  line-height: 1.5;
  margin: 0;
}

.stats-modal-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}

.stats-item.card {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(185, 145, 71, 0.12);
  border-radius: 6px;
  padding: 1.2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
}

.stats-item-lbl {
  color: var(--color-text-muted);
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.stats-item-val.gold-neon {
  font-family: var(--font-mono);
  color: var(--color-gold-bright);
  font-size: 2.2rem;
  font-weight: 800;
  line-height: 1;
  text-shadow: 0 0 16px rgba(212, 175, 92, 0.4);
}

.gold-unit {
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-gold-muted);
}

.stats-modal-footer {
  display: flex;
  justify-content: center;
  z-index: 1;
}

.stats-modal-btn {
  background: linear-gradient(135deg, var(--color-gold-default) 0%, var(--color-gold-dark) 100%);
  border: 1px solid var(--color-gold-bright);
  color: #fff;
  font-family: var(--font-display);
  font-size: 0.95rem;
  font-weight: 700;
  padding: 0.75rem 2.5rem;
  border-radius: 4px;
  cursor: pointer;
  box-shadow: 0 4px 16px rgba(185, 145, 71, 0.25);
  transition: all 0.2s;
}

.stats-modal-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 24px rgba(185, 145, 71, 0.45);
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-6px); }
}
</style>
