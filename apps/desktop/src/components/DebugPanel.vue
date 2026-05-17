<script setup lang="ts">
import { ref, nextTick, watch, computed } from "vue";
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
  agentObserve?: any;
  agentThinking?: string;
  agentAction?: string;
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

// ── Agent Data Computed ──
const myself = computed(() => props.agentObserve?.myself);
const minions = computed(() => props.agentObserve?.minions || []);
const simulationTime = computed(() => props.agentObserve?.time || 0);

// 计算进入补刀斩杀线的小兵
const lasthitableMinions = computed(() => {
  if (!myself.value || !minions.value.length) return [];
  const ad = myself.value.attack_damage || 0;
  return minions.value.map((m: any, idx: number) => ({
    ...m,
    index: idx + 1,
    lasthitable: m.health <= ad,
  }));
});

// 计算当前的生命比率
const hpPercent = computed(() => {
  if (!myself.value || myself.value.max_health <= 0) return 0;
  return Math.min(100, Math.max(0, (myself.value.health / myself.value.max_health) * 100));
});

// 计算能量蓝量比率
const mpPercent = computed(() => {
  if (!myself.value || !myself.value.ability_resource) return 0;
  const [val, max] = myself.value.ability_resource;
  if (max <= 0) return 0;
  return Math.min(100, Math.max(0, (val / max) * 100));
});
</script>

<template>
  <div class="debug-container">
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
        <span class="status-divider" v-if="simulationTime > 0"></span>
        <span class="status-champ" v-if="simulationTime > 0">
          <span class="champ-label">Sim Time</span>
          <span class="champ-value gold">{{ simulationTime.toFixed(1) }}s</span>
        </span>
      </div>
      <button class="stop-btn" @click="$emit('stop')">Stop Game</button>
    </div>

    <!-- Main Workspace Layout: 2 Columns -->
    <div class="workspace">
      
      <!-- LEFT COLUMN: Controller & Original Logs -->
      <div class="left-col">
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
              <h2 class="log-title">Game Console Log</h2>
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

      <!-- RIGHT COLUMN: Agent Decision Dashboard -->
      <div class="right-col">
        <section class="agent-panel">
          <div class="panel-header">
            <div class="panel-title-area">
              <span class="panel-icon pulse-gold"></span>
              <h2 class="panel-title">AI Agent Decision Cockpit</h2>
            </div>
            <span class="agent-mode-tag">Rig + DeepSeek-R1</span>
          </div>

          <!-- Empty State -->
          <div v-if="!myself" class="agent-empty">
            <div class="radar-scan"></div>
            <p>等待 AI Agent 启动思考与观察数据流...</p>
          </div>

          <!-- Dashboard Content -->
          <div v-else class="agent-dashboard">
            
            <!-- Row 1: Myself State Overview -->
            <div class="dashboard-section card">
              <div class="section-header">
                <span class="label-glow-gold">Actor Perception</span>
                <span class="badge">Lv.{{ myself.level }}</span>
              </div>
              
              <!-- HP and Resource Bars -->
              <div class="resources">
                <div class="resource-row">
                  <div class="bar-label">HP</div>
                  <div class="bar-outer hp">
                    <div class="bar-inner hp" :style="{ width: hpPercent + '%' }"></div>
                    <span class="bar-text">{{ myself.health.toFixed(0) }} / {{ myself.max_health.toFixed(0) }} ({{ hpPercent.toFixed(0) }}%)</span>
                  </div>
                </div>
                
                <div class="resource-row" v-if="myself.ability_resource">
                  <div class="bar-label">MP</div>
                  <div class="bar-outer mp">
                    <div class="bar-inner mp" :style="{ width: mpPercent + '%' }"></div>
                    <span class="bar-text">{{ myself.ability_resource[0].toFixed(0) }} / {{ myself.ability_resource[1].toFixed(0) }}</span>
                  </div>
                </div>
              </div>

              <!-- Gold Assets Bar -->
              <div class="gold-monitor-bar" v-if="myself.gold !== undefined">
                <div class="gold-label-area">
                  <svg class="gold-coin-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="10" />
                    <circle cx="12" cy="12" r="6" />
                    <path d="M12 8 L12 16" />
                    <path d="M10 10 L14 10" />
                  </svg>
                  <span class="gold-title">GOLD ASSETS</span>
                </div>
                <span class="gold-value-neon">{{ myself.gold.toFixed(0) }} <span class="gold-currency">g</span></span>
              </div>

              <!-- Fight stats -->
              <div class="stats-grid">
                <div class="stat-card">
                  <span class="stat-lbl">Damage</span>
                  <span class="stat-val gold">{{ myself.attack_damage.toFixed(0) }}</span>
                </div>
                <div class="stat-card">
                  <span class="stat-lbl">Speed</span>
                  <span class="stat-val">{{ myself.attack_speed.toFixed(2) }}</span>
                </div>
                <div class="stat-card">
                  <span class="stat-lbl">Range</span>
                  <span class="stat-val">{{ myself.attack_range.toFixed(0) }}</span>
                </div>
                <div class="stat-card">
                  <span class="stat-lbl">Armor</span>
                  <span class="stat-val">{{ myself.armor.toFixed(0) }}</span>
                </div>
              </div>

              <!-- Skills Grid -->
              <div class="skills-section" v-if="myself.skills && myself.skills.length">
                <div class="skills-header-row">
                  <span class="sub-label">Active Skills</span>
                  <span class="skill-pts" v-if="myself.skill_points > 0">
                    <span class="pts-dot"></span> {{ myself.skill_points }} Pts Available
                  </span>
                </div>
                <div class="skills-grid">
                  <div 
                    v-for="skill in myself.skills" 
                    :key="skill.index" 
                    class="skill-orb"
                    :class="{ 
                      locked: skill.level === 0, 
                      cooldown: skill.cooldown_remaining !== null 
                    }"
                  >
                    <div class="orb-name">{{ ['Q', 'W', 'E', 'R'][skill.index] || '?' }}</div>
                    <div class="orb-level">L{{ skill.level }}</div>
                    <div class="orb-overlay" v-if="skill.cooldown_remaining !== null">
                      <span class="cd-text">{{ skill.cooldown_remaining.toFixed(1) }}s</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Row 2: Minion perceptions -->
            <div class="dashboard-section card flex-1">
              <div class="section-header">
                <span class="label-glow-gold">Minion Perceptions (Range 2000)</span>
                <span class="badge blue">{{ minions.length }} Minions</span>
              </div>

              <div class="minions-table-wrap">
                <div v-if="lasthitableMinions.length === 0" class="minions-empty">
                  当前雷达探测范围内没有敌方小兵
                </div>
                <div v-else class="minions-list">
                  <div 
                    v-for="minion in lasthitableMinions" 
                    :key="minion.entity" 
                    class="minion-row"
                    :class="{ 'lasthit-ready': minion.lasthitable }"
                  >
                    <span class="minion-index">#{{ minion.index }}</span>
                    <span class="minion-dist">Distance: {{ minion.distance.toFixed(0) }}</span>
                    
                    <div class="minion-hp-box">
                      <span class="minion-hp-text">HP: {{ minion.health.toFixed(0) }}</span>
                      <div class="minion-hp-outer">
                        <div class="minion-hp-inner" :style="{ width: Math.min(100, (minion.health / 500) * 100) + '%' }"></div>
                      </div>
                    </div>

                    <span class="minion-status-tag" :class="{ lasthit: minion.lasthitable }">
                      {{ minion.lasthitable ? 'READY TO LASTHIT' : 'ATTACKABLE' }}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Row 3: Reasoning and Action -->
            <div class="dashboard-section card flex-2 min-h-0">
              <div class="section-header">
                <span class="label-glow-gold">Agent Reasoning Chain & Decision Action</span>
              </div>

              <div class="reasoning-split">
                <!-- Action output -->
                <div class="action-card">
                  <span class="action-lbl">Dispatched Command</span>
                  <div class="action-content neon-gold">
                    {{ agentAction || 'WAITING...' }}
                  </div>
                </div>

                <!-- Thinking Process -->
                <div class="thinking-box">
                  <div class="thinking-header">
                    <span class="thinking-icon">🤖</span>
                    <span>DeepSeek R1 思维推导链</span>
                  </div>
                  <div class="thinking-text">
                    {{ agentThinking || 'AI 正在分析自身状态与小兵血量，规划最佳战术路线...' }}
                  </div>
                </div>
              </div>
            </div>

          </div>
        </section>
      </div>

    </div>
  </div>
</template>

<style scoped>
/* ── Main Layout ── */
.debug-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  padding: 16px 20px;
  gap: 12px;
  background: var(--bg-deep);
  overflow: hidden;
}

.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  padding: 8px 14px;
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
  gap: 6px;
  padding: 3px 8px;
  font-size: var(--fs-tiny);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
}

.status-badge.on {
  color: var(--green);
  background: rgba(74, 158, 90, 0.08);
  border-color: rgba(74, 158, 90, 0.15);
}

.status-badge.off {
  color: var(--red);
  background: rgba(200, 74, 74, 0.08);
  border-color: rgba(200, 74, 74, 0.15);
}

.badge-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
}

.status-badge.on .badge-dot {
  background: var(--green);
  box-shadow: 0 0 6px rgba(74, 158, 90, 0.6);
}

.status-badge.off .badge-dot {
  background: var(--red);
  box-shadow: 0 0 6px rgba(200, 74, 74, 0.4);
}

.status-divider {
  width: 1px;
  height: 14px;
  background: var(--border-subtle);
}

.status-champ {
  display: flex;
  align-items: center;
  gap: 6px;
}

.champ-label {
  font-size: var(--fs-tiny);
  color: var(--text-muted);
  text-transform: uppercase;
}

.champ-value {
  font-size: var(--fs-small);
  font-weight: 600;
  color: var(--text-bright);
}

.champ-value.gold {
  color: var(--gold-bright);
}

.stop-btn {
  padding: 4px 12px;
  font-size: var(--fs-small);
  font-weight: 500;
  color: var(--red);
  border: 1px solid rgba(200, 74, 74, 0.25);
  border-radius: var(--radius-sm);
  background: rgba(200, 74, 74, 0.04);
  transition: all var(--dur-fast) ease-out;
}

.stop-btn:hover {
  background: rgba(200, 74, 74, 0.12);
  border-color: rgba(200, 74, 74, 0.45);
}

/* ── Workspace ── */
.workspace {
  display: flex;
  flex: 1;
  gap: 14px;
  min-height: 0;
}

.left-col {
  width: 44%;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
}

.right-col {
  width: 56%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* ── Controls Section ── */
.controls-section {
  flex-shrink: 0;
}

.control-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.control-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  flex: 1;
}

.group-label {
  font-size: var(--fs-tiny);
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
}

.toggle-row {
  display: flex;
  gap: 4px;
}

.toggle-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  font-size: var(--fs-small);
  color: var(--text-muted);
  background: var(--bg-deep);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  transition: all var(--dur-fast) ease-out;
  flex: 1;
  white-space: nowrap;
}

.toggle-btn:hover {
  color: var(--text-default);
  border-color: var(--gold-muted);
}

.toggle-btn.active {
  color: var(--gold-bright);
  border-color: var(--gold-dimmer);
  background: rgba(185, 145, 71, 0.06);
}

.toggle-indicator {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--border-default);
}

.toggle-btn.active .toggle-indicator {
  background: var(--gold-default);
  box-shadow: 0 0 6px rgba(185, 145, 71, 0.5);
}

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
  padding: 4px 20px 4px 8px;
  font-size: var(--fs-small);
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.4);
  cursor: pointer;
}

.btn-ghost {
  padding: 4px 10px;
  font-size: var(--fs-small);
  color: var(--text-muted);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  transition: all var(--dur-fast) ease-out;
  background: transparent;
  cursor: pointer;
}

.btn-ghost:hover {
  color: var(--gold-bright);
  border-color: var(--gold-muted);
}

.btn-ghost.small {
  padding: 2px 6px;
  font-size: var(--fs-tiny);
}

/* ── Log Section ── */
.log-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
  min-height: 0;
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  padding: 8px 14px;
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border-subtle);
}

.log-header-left {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.log-title {
  font-size: var(--fs-small);
  font-weight: 600;
  color: var(--text-default);
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
  font-size: 11px;
  line-height: 1.5;
}

.log-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 100%;
  color: var(--text-muted);
}

.empty-icon {
  opacity: 0.3;
}

.log-entry {
  display: flex;
  align-items: baseline;
  gap: 6px;
  padding: 2px 14px;
  white-space: nowrap;
  transition: background var(--dur-instant) ease-out;
}

.log-entry:hover {
  background: rgba(255, 255, 255, 0.02);
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
  min-width: 70px;
  font-size: 10px;
  color: var(--border-default);
}

.entry-ts.dim {
  opacity: 0.3;
}

.entry-badge {
  flex-shrink: 0;
  min-width: 32px;
  padding: 0 4px;
  font-size: 9px;
  font-weight: 600;
  text-align: center;
  border-radius: var(--radius-sm);
  text-transform: uppercase;
}

.entry-badge.info {
  background: rgba(154, 146, 130, 0.08);
  color: var(--text-muted);
}

.entry-badge.warn {
  background: rgba(204, 170, 136, 0.1);
  color: #ca8;
}

.entry-badge.error {
  background: rgba(200, 74, 74, 0.1);
  color: var(--red);
}

.entry-src {
  flex-shrink: 0;
  color: var(--border-default);
  font-size: 10px;
}

.entry-msg {
  overflow: hidden;
  text-overflow: ellipsis;
}

.entry-count {
  flex-shrink: 0;
  padding: 0 5px;
  font-size: 9px;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.03);
  border-radius: 6px;
}

/* ── RIGHT COLUMN: Agent Panel ── */
.agent-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  min-height: 0;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.panel-title-area {
  display: flex;
  align-items: center;
  gap: 8px;
}

.panel-icon {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--gold-default);
}

.pulse-gold {
  box-shadow: 0 0 8px var(--gold-default);
  animation: pulse-glow-gold 1.8s ease-in-out infinite;
}

@keyframes pulse-glow-gold {
  0%, 100% { transform: scale(1); opacity: 1; box-shadow: 0 0 6px var(--gold-bright); }
  50% { transform: scale(1.2); opacity: 0.5; box-shadow: 0 0 2px var(--gold-dimmer); }
}

.panel-title {
  font-family: var(--font-display);
  font-size: var(--fs-h3);
  font-weight: 700;
  color: var(--gold-bright);
  letter-spacing: 0.04em;
}

.agent-mode-tag {
  font-size: var(--fs-tiny);
  color: var(--text-muted);
  background: rgba(185, 145, 71, 0.05);
  border: 1px solid rgba(185, 145, 71, 0.15);
  padding: 1px 8px;
  border-radius: 10px;
}

/* Radar Empty State */
.agent-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--text-muted);
  gap: 16px;
  position: relative;
}

.radar-scan {
  width: 64px;
  height: 64px;
  border: 1px solid rgba(185, 145, 71, 0.15);
  border-radius: 50%;
  position: relative;
  overflow: hidden;
}

.radar-scan::after {
  content: "";
  position: absolute;
  inset: 0;
  background: conic-gradient(from 0deg, rgba(185, 145, 71, 0.18) 0deg, transparent 90deg);
  animation: rotate-radar 2.5s linear infinite;
  transform-origin: center;
}

@keyframes rotate-radar {
  100% { transform: rotate(360deg); }
}

/* ── Dashboard Layout ── */
.agent-dashboard {
  display: flex;
  flex-direction: column;
  flex: 1;
  padding: 12px;
  gap: 10px;
  min-height: 0;
  overflow-y: auto;
}

.card {
  background: rgba(18, 16, 19, 0.55);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  padding-bottom: 6px;
  flex-shrink: 0;
}

.label-glow-gold {
  font-size: var(--fs-tiny);
  font-weight: 700;
  color: var(--gold-bright);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(185, 145, 71, 0.1);
  color: var(--gold-default);
  border: 1px solid rgba(185, 145, 71, 0.2);
}

.badge.blue {
  background: rgba(74, 126, 196, 0.1);
  color: var(--blue);
  border-color: rgba(74, 126, 196, 0.2);
}

/* Resources & HP Bars */
.resources {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.resource-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.bar-label {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 700;
  width: 20px;
  color: var(--text-muted);
}

.bar-outer {
  flex: 1;
  height: 18px;
  background: var(--bg-deep);
  border-radius: 4px;
  border: 1px solid var(--border-subtle);
  position: relative;
  overflow: hidden;
}

.bar-inner {
  height: 100%;
  transition: width 0.3s ease-out;
}

.bar-inner.hp {
  background: linear-gradient(90deg, #c84a4a 0%, #e25c5c 100%);
}

.bar-inner.mp {
  background: linear-gradient(90deg, #4a7ec4 0%, #5d92e0 100%);
}

.bar-text {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-family: var(--font-mono);
  font-weight: 600;
  color: #fff;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
}

/* Stats Grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 6px;
}

.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: var(--bg-deep);
  border: 1px solid rgba(255, 255, 255, 0.02);
  border-radius: 4px;
  padding: 4px;
}

.stat-lbl {
  font-size: 9px;
  color: var(--text-muted);
  text-transform: uppercase;
}

.stat-val {
  font-family: var(--font-mono);
  font-size: var(--fs-small);
  font-weight: 700;
  color: var(--text-bright);
}

.stat-val.gold {
  color: var(--gold-bright);
}

/* Skills */
.skills-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
  border-top: 1px solid rgba(255, 255, 255, 0.02);
  padding-top: 6px;
}

.skills-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.sub-label {
  font-size: 10px;
  color: var(--text-muted);
  font-weight: 600;
}

.skill-pts {
  font-size: 10px;
  color: var(--gold-bright);
  font-weight: 700;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.pts-dot {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--gold-bright);
  box-shadow: 0 0 6px var(--gold-bright);
}

.skills-grid {
  display: flex;
  gap: 8px;
}

.skill-orb {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  border: 1.5px solid var(--border-default);
  background: var(--bg-deep);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
  cursor: default;
}

.skill-orb.locked {
  opacity: 0.35;
  border-style: dashed;
}

.skill-orb.cooldown {
  border-color: rgba(200, 74, 74, 0.4);
}

.orb-name {
  font-family: var(--font-display);
  font-weight: 800;
  font-size: var(--fs-small);
  color: var(--text-bright);
}

.orb-level {
  font-size: 8px;
  color: var(--text-muted);
  line-height: 1;
}

.orb-overlay {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: rgba(7, 6, 8, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
}

.cd-text {
  font-family: var(--font-mono);
  font-size: 10px;
  color: #fff;
  font-weight: 700;
}

/* ── Minions Perception ── */
.minions-table-wrap {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.minions-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
  font-size: var(--fs-small);
  min-height: 50px;
}

.minions-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.minion-row {
  display: flex;
  align-items: center;
  background: var(--bg-deep);
  border: 1px solid var(--border-subtle);
  border-radius: 4px;
  padding: 6px 12px;
  gap: 12px;
  transition: all 0.2s ease;
}

.minion-row.lasthit-ready {
  border-color: rgba(200, 74, 74, 0.45);
  background: rgba(200, 74, 74, 0.04);
  box-shadow: 0 0 8px rgba(200, 74, 74, 0.08);
}

.minion-index {
  font-family: var(--font-mono);
  font-weight: 700;
  color: var(--text-muted);
  width: 24px;
}

.minion-dist {
  font-size: var(--fs-small);
  color: var(--text-default);
  width: 100px;
}

.minion-hp-box {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
}

.minion-hp-text {
  font-family: var(--font-mono);
  font-size: var(--fs-small);
  color: var(--text-bright);
  min-width: 50px;
}

.minion-hp-outer {
  flex: 1;
  height: 6px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 3px;
  overflow: hidden;
  max-width: 120px;
}

.minion-hp-inner {
  height: 100%;
  background: var(--text-muted);
}

.minion-row.lasthit-ready .minion-hp-inner {
  background: var(--red);
}

.minion-status-tag {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-muted);
}

.minion-status-tag.lasthit {
  background: rgba(200, 74, 74, 0.12);
  color: var(--red);
  border: 1px solid rgba(200, 74, 74, 0.25);
  animation: pulse-red-bg 2s infinite;
}

@keyframes pulse-red-bg {
  0%, 100% { box-shadow: 0 0 0 rgba(200, 74, 74, 0); }
  50% { box-shadow: 0 0 6px rgba(200, 74, 74, 0.3); }
}

/* ── Reasoning Card ── */
.reasoning-split {
  display: flex;
  flex: 1;
  gap: 12px;
  min-height: 0;
}

.action-card {
  width: 35%;
  background: var(--bg-deep);
  border: 1px solid rgba(185, 145, 71, 0.12);
  border-radius: var(--radius-md);
  padding: 10px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  flex-shrink: 0;
}

.action-lbl {
  font-size: var(--fs-tiny);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-weight: 700;
}

.action-content {
  font-family: var(--font-mono);
  font-size: 15px;
  font-weight: 800;
  text-align: center;
  word-break: break-all;
}

.neon-gold {
  color: var(--gold-bright);
  text-shadow: 0 0 10px rgba(212, 175, 92, 0.3);
}

.thinking-box {
  flex: 1;
  background: var(--bg-deep);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 0;
}

.thinking-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: var(--fs-small);
  font-weight: 700;
  color: var(--text-default);
  border-bottom: 1px solid rgba(255, 255, 255, 0.02);
  padding-bottom: 4px;
  flex-shrink: 0;
}

.thinking-text {
  flex: 1;
  overflow-y: auto;
  font-size: var(--fs-small);
  color: var(--text-muted);
  line-height: 1.6;
  white-space: pre-wrap;
  font-style: italic;
}

/* ── Gold Assets Bar Styling ── */
.gold-monitor-bar {
  margin: 12px 0 16px 0;
  padding: 10px 14px;
  background: linear-gradient(135deg, rgba(212, 175, 92, 0.08) 0%, rgba(185, 145, 71, 0.03) 100%);
  border: 1px dashed rgba(212, 175, 92, 0.35);
  border-radius: var(--radius-md);
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: inset 0 0 10px rgba(212, 175, 92, 0.03);
}

.gold-label-area {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--gold-bright);
}

.gold-coin-icon {
  filter: drop-shadow(0 0 4px rgba(212, 175, 92, 0.3));
}

.gold-title {
  font-size: var(--fs-tiny);
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.gold-value-neon {
  font-family: var(--font-mono);
  font-size: 18px;
  font-weight: 800;
  color: var(--gold-bright);
  text-shadow: 0 0 10px rgba(212, 175, 92, 0.45);
  display: flex;
  align-items: baseline;
  gap: 2px;
}

.gold-currency {
  font-size: 11px;
  font-weight: 600;
  color: var(--gold-muted);
}
</style>
