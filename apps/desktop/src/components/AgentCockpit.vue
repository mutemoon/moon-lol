<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  agentObserve?: any;
  agentThinking?: string;
  agentAction?: string;
}>();

// ── Agent Data Computed ──
const myself = computed(() => props.agentObserve?.myself);
const minions = computed(() => props.agentObserve?.minions || []);

// 计算进入最后补刀斩杀线的小兵
const lasthitableMinions = computed(() => {
  if (!myself.value || !minions.value.length) return [];
  const ad = myself.value.attack_damage || 0;
  return minions.value.map((m: any, idx: number) => ({
    ...m,
    index: idx + 1,
    lasthitable: m.health <= ad,
  }));
});

// 计算当前生命值百分比
const hpPercent = computed(() => {
  if (!myself.value || myself.value.max_health <= 0) return 0;
  return Math.min(100, Math.max(0, (myself.value.health / myself.value.max_health) * 100));
});

// 计算魔法值百分比
const mpPercent = computed(() => {
  if (!myself.value || !myself.value.ability_resource) return 0;
  const [val, max] = myself.value.ability_resource;
  if (max <= 0) return 0;
  return Math.min(100, Math.max(0, (val / max) * 100));
});
</script>

<template>
  <section class="agent-panel">
    <div class="panel-header">
      <div class="panel-title-area">
        <span class="panel-icon pulse-gold"></span>
        <h2 class="panel-title">AI Agent Decision Cockpit</h2>
      </div>
      <span class="agent-mode-tag">Rig + DeepSeek-R1</span>
    </div>

    <!-- 空数据状态 -->
    <div v-if="!myself" class="agent-empty">
      <div class="radar-scan"></div>
      <p>等待 AI Agent 启动思考与观察数据流...</p>
    </div>

    <!-- 看板主面板 -->
    <div v-else class="agent-dashboard">
      <!-- 区域一: 玩家自身状态属性 -->
      <div class="dashboard-section card">
        <div class="section-header">
          <span class="label-glow-gold">Actor Perception</span>
          <span class="badge">Lv.{{ myself.level }}</span>
        </div>

        <!-- 血量与能量条 -->
        <div class="resources">
          <div class="resource-row">
            <div class="bar-label">HP</div>
            <div class="bar-outer hp">
              <div class="bar-inner hp" :style="{ width: hpPercent + '%' }"></div>
              <span class="bar-text">
                {{ myself.health.toFixed(0) }} / {{ myself.max_health.toFixed(0) }} ({{
                  hpPercent.toFixed(0)
                }}%)
              </span>
            </div>
          </div>

          <div v-if="myself.ability_resource" class="resource-row">
            <div class="bar-label">MP</div>
            <div class="bar-outer mp">
              <div class="bar-inner mp" :style="{ width: mpPercent + '%' }"></div>
              <span class="bar-text">
                {{ myself.ability_resource[0].toFixed(0) }} / {{ myself.ability_resource[1].toFixed(0) }}
              </span>
            </div>
          </div>
        </div>

        <!-- 战局金币资产监控 -->
        <div v-if="myself.gold !== undefined" class="gold-monitor-bar">
          <div class="gold-label-area">
            <svg
              class="gold-coin-icon"
              viewBox="0 0 24 24"
              width="16"
              height="16"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="10" />
              <circle cx="12" cy="12" r="6" />
              <path d="M12 8 L12 16" />
              <path d="M10 10 L14 10" />
            </svg>
            <span class="gold-title">GOLD ASSETS</span>
          </div>
          <span class="gold-value-neon">
            {{ myself.gold.toFixed(0) }}
            <span class="gold-currency">g</span>
          </span>
        </div>

        <!-- KDA & 补刀数监控 -->
        <div v-if="myself.minion_kills !== undefined" class="kda-cs-monitor">
          <div class="kda-box">
            <span class="kda-lbl">KDA</span>
            <span class="kda-val">
              <span class="k-val">{{ myself.kills }}</span>
              <span class="divider">/</span>
              <span class="d-val">{{ myself.deaths }}</span>
              <span class="divider">/</span>
              <span class="a-val">{{ myself.assists }}</span>
            </span>
          </div>
          <div class="cs-box">
            <span class="cs-lbl">Creep Score (CS)</span>
            <span class="cs-val">{{ myself.minion_kills }}</span>
          </div>
        </div>

        <!-- 详细面板属性 -->
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

        <!-- 活跃技能状态 -->
        <div v-if="myself.skills && myself.skills.length" class="skills-section">
          <div class="skills-header-row">
            <span class="sub-label">Active Skills</span>
            <span v-if="myself.skill_points > 0" class="skill-pts">
              <span class="pts-dot"></span>
              {{ myself.skill_points }} Pts Available
            </span>
          </div>
          <div class="skills-grid">
            <div
              v-for="skill in myself.skills"
              :key="skill.index"
              class="skill-orb"
              :class="{ locked: skill.level === 0, cooldown: skill.cooldown_remaining !== null }"
            >
              <div class="orb-name">{{ ["Q", "W", "E", "R"][skill.index] || "?" }}</div>
              <div class="orb-level">L{{ skill.level }}</div>
              <div v-if="skill.cooldown_remaining !== null" class="orb-overlay">
                <span class="cd-text">{{ skill.cooldown_remaining.toFixed(1) }}s</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 区域二: 小兵雷达感知 (2000码半径) -->
      <div class="dashboard-section card min-h-0 flex-1">
        <div class="section-header">
          <span class="label-glow-gold">Minion Perceptions (Range 2000)</span>
          <span class="badge blue">{{ minions.length }} Minions</span>
        </div>

        <div class="minions-table-wrap">
          <div v-if="lasthitableMinions.length === 0" class="minions-empty">当前雷达探测范围内没有敌方小兵</div>
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
                  <div
                    class="minion-hp-inner"
                    :style="{ width: Math.min(100, (minion.health / 500) * 100) + '%' }"
                  ></div>
                </div>
              </div>

              <span class="minion-status-tag" :class="{ lasthit: minion.lasthitable }">
                {{ minion.lasthitable ? "READY TO LASTHIT" : "ATTACKABLE" }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- 区域三: 决策推导链与指令下发 -->
      <div class="dashboard-section card min-h-0 flex-2">
        <div class="section-header">
          <span class="label-glow-gold">Agent Reasoning Chain & Decision Action</span>
        </div>

        <div class="reasoning-split">
          <!-- 动作输出 -->
          <div class="action-card">
            <span class="action-lbl">Dispatched Command</span>
            <div class="action-content neon-gold">
              {{ agentAction || "WAITING..." }}
            </div>
          </div>

          <!-- 思维过程 -->
          <div class="thinking-box">
            <div class="thinking-header">
              <span>🤖</span>
              <span>DeepSeek R1 思维推导链</span>
            </div>
            <div class="thinking-text">
              {{ agentThinking || "AI 正在分析自身状态与小兵血量，规划最佳战术路线..." }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
@reference "tailwindcss";
@reference "../style.css";

.agent-panel {
  @apply bg-bg-surface border-border-subtle flex h-full flex-col overflow-hidden rounded border shadow-[0_12px_32px_rgba(0,0,0,0.6),0_0_4px_rgba(120,91,40,0.15)];
}
.panel-header {
  @apply bg-bg-elevated border-border-subtle flex shrink-0 items-center justify-between border-b px-4 py-2;
}
.panel-title-area {
  @apply flex items-center gap-2;
}
.panel-icon {
  @apply bg-gold-default h-1.5 w-1.5 rounded-full;
}
.pulse-gold {
  @apply animate-[pulse-glow-gold_1.8s_ease-in-out_infinite] shadow-[0_0_8px_var(--text-gold-default)];
}
.panel-title {
  @apply font-display text-gold-bright text-sm font-bold tracking-wider;
}
.agent-mode-tag {
  @apply text-text-muted rounded-xl border border-[rgba(185,145,71,0.15)] bg-[rgba(185,145,71,0.05)] px-2 py-0.5 text-[11px];
}
.agent-empty {
  @apply text-text-muted relative flex flex-1 flex-col items-center justify-center gap-4;
}
.radar-scan {
  @apply relative h-16 w-16 overflow-hidden rounded-full border border-[rgba(185,145,71,0.15)];
  &::after {
    @apply absolute inset-0 origin-center animate-[rotate-radar_2.5s_linear_infinite] bg-[conic-gradient(from_0deg,rgba(185,145,71,0.18)_0deg,transparent_90deg)] content-[''];
  }
}
.agent-dashboard {
  @apply flex min-h-0 flex-1 flex-col gap-2.5 overflow-y-auto p-3;
}
.dashboard-section {
  @apply border-border-subtle flex flex-col gap-2.5 rounded border bg-[rgba(12,10,13,0.55)] p-3;
}
.section-header {
  @apply flex shrink-0 items-center justify-between border-b border-[rgba(255,255,255,0.03)] pb-1.5;
}
.label-glow-gold {
  @apply text-gold-bright text-[11px] font-bold tracking-widest uppercase;
}
.badge {
  @apply text-gold-default rounded border border-[rgba(185,145,71,0.2)] bg-[rgba(185,145,71,0.1)] px-1.5 py-0.5 text-[10px];
  &.blue {
    @apply text-blue border-[rgba(74,126,196,0.2)] bg-[rgba(74,126,196,0.1)];
  }
}
.resources {
  @apply flex flex-col gap-1.5;
}
.resource-row {
  @apply flex items-center gap-2.5;
}
.bar-label {
  @apply text-text-muted w-5 font-mono text-[11px] font-bold;
}
.bar-outer {
  @apply bg-bg-deep border-border-subtle relative h-4.5 flex-1 overflow-hidden rounded border;
}
.bar-inner {
  @apply h-full transition-[width] duration-300 ease-out;
  &.hp {
    @apply bg-gradient-to-r from-[#c84a4a] to-[#e25c5c];
  }
  &.mp {
    @apply bg-gradient-to-r from-[#4a7ec4] to-[#5d92e0];
  }
}
.bar-text {
  @apply absolute inset-0 flex items-center justify-center font-mono text-[10px] font-semibold text-white drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)];
}
.stats-grid {
  @apply grid grid-cols-4 gap-1.5;
}
.stat-card {
  @apply bg-bg-deep flex flex-col items-center justify-center rounded border border-[rgba(255,255,255,0.02)] p-1;
}
.stat-lbl {
  @apply text-text-muted text-[9px] uppercase;
}
.stat-val {
  @apply text-text-bright font-mono text-xs font-bold;
  &.gold {
    @apply text-gold-bright;
  }
}
.gold-monitor-bar {
  @apply mx-0 my-3 flex items-center justify-between rounded border border-dashed border-[rgba(212,175,92,0.35)] bg-gradient-to-br from-[rgba(212,175,92,0.08)] to-[rgba(185,145,71,0.03)] p-2.5 shadow-[inset_0_0_10px_rgba(212,175,92,0.03)];
}
.gold-label-area {
  @apply text-gold-bright flex items-center gap-2;
}
.gold-coin-icon {
  @apply drop-shadow-[0_0_4px_rgba(212,175,92,0.3)];
}
.gold-title {
  @apply text-[11px] font-extrabold tracking-widest uppercase;
}
.gold-value-neon {
  @apply text-gold-bright flex items-baseline gap-0.5 font-mono text-[18px] font-extrabold drop-shadow-[0_0_10px_rgba(212,175,92,0.45)];
}
.gold-currency {
  @apply text-gold-muted text-[11px] font-semibold;
}
.kda-cs-monitor {
  @apply -mt-1 mb-4 grid grid-cols-2 gap-3;
}
.kda-box,
.cs-box {
  @apply border-border-subtle flex flex-col gap-1 rounded border bg-[rgba(255,255,255,0.01)] p-2;
}
.kda-lbl,
.cs-lbl {
  @apply text-text-muted text-[9px] font-bold tracking-wider uppercase;
}
.kda-val {
  @apply text-text-bright flex items-center gap-1.5 font-mono text-[15px] font-bold;
}
.k-val {
  @apply text-green drop-shadow-[0_0_8px_rgba(74,158,90,0.3)];
}
.d-val {
  @apply text-red drop-shadow-[0_0_8px_rgba(200,74,74,0.3)];
}
.cs-val {
  @apply text-green font-mono text-base font-extrabold drop-shadow-[0_0_8px_rgba(74,158,90,0.4)];
}
.divider {
  @apply border-border-default font-normal;
}
.skills-section {
  @apply flex flex-col gap-1 border-t border-[rgba(255,255,255,0.02)] pt-1.5;
}
.skills-header-row {
  @apply flex items-center justify-between;
}
.sub-label {
  @apply text-text-muted text-[10px] font-semibold;
}
.skill-pts {
  @apply text-gold-bright inline-flex items-center gap-1 text-[10px] font-bold;
}
.pts-dot {
  @apply bg-gold-bright h-1 w-1 rounded-full shadow-[0_0_6px_var(--text-gold-bright)];
}
.skills-grid {
  @apply flex gap-2;
}
.skill-orb {
  @apply border-border-default bg-bg-deep relative flex h-9 w-9 flex-col items-center justify-center rounded-full border select-none;
  &.locked {
    @apply border-dashed opacity-35;
  }
  &.cooldown {
    @apply border-[rgba(200,74,74,0.4)];
  }
}
.orb-name {
  @apply font-display text-text-bright text-xs font-extrabold;
}
.orb-level {
  @apply text-text-muted text-[8px] leading-none;
}
.orb-overlay {
  @apply absolute inset-0 flex items-center justify-center rounded-full bg-[rgba(7,6,8,0.75)];
}
.cd-text {
  @apply font-mono text-[10px] font-bold text-white;
}
.minions-table-wrap {
  @apply min-h-0 flex-1 overflow-y-auto;
}
.minions-empty {
  @apply text-text-muted flex h-full min-h-[50px] items-center justify-center text-xs;
}
.minions-list {
  @apply flex flex-col gap-1.5;
}
.minion-row {
  @apply bg-bg-deep border-border-subtle flex items-center gap-3 rounded border px-3 py-1.5 transition-all duration-200;
  &.lasthit-ready {
    @apply border-[rgba(200,74,74,0.45)] bg-[rgba(200,74,74,0.04)] shadow-[0_0_8px_rgba(200,74,74,0.08)];
  }
}
.minion-index {
  @apply text-text-muted w-6 font-mono font-bold;
}
.minion-dist {
  @apply text-text-default w-25 text-xs;
}
.minion-hp-box {
  @apply flex items-center gap-1.5;
}
.minion-hp-text {
  @apply text-text-bright min-w-[50px] font-mono text-xs;
}
.minion-hp-outer {
  @apply h-1.5 max-w-[120px] flex-1 overflow-hidden rounded-sm bg-[rgba(255,255,255,0.04)];
}
.minion-hp-inner {
  @apply bg-text-muted h-full;
  .minion-row.lasthit-ready & {
    @apply bg-red;
  }
}
.minion-status-tag {
  @apply text-text-muted rounded-sm bg-[rgba(255,255,255,0.05)] px-2 py-0.5 text-[9px] font-bold;
  &.lasthit {
    @apply text-red animate-[pulse-red-bg_2s_infinite] border border-[rgba(200,74,74,0.25)] bg-[rgba(200,74,74,0.12)];
  }
}
.reasoning-split {
  @apply flex min-h-0 flex-1 gap-3;
}
.action-card {
  @apply bg-bg-deep flex w-[35%] shrink-0 flex-col items-center justify-center gap-1.5 rounded border border-[rgba(185,145,71,0.12)] p-2.5;
}
.action-lbl {
  @apply text-text-muted text-[11px] font-bold tracking-wider uppercase;
}
.action-content {
  @apply text-center font-mono text-[15px] font-extrabold break-all;
}
.neon-gold {
  @apply text-gold-bright drop-shadow-[0_0_10px_rgba(212,175,92,0.3)];
}
.thinking-box {
  @apply bg-bg-deep border-border-subtle flex min-h-0 flex-1 flex-col gap-1.5 rounded border p-2.5;
}
.thinking-header {
  @apply text-text-default flex shrink-0 items-center gap-1.5 border-b border-[rgba(255,255,255,0.02)] pb-1 text-xs font-bold;
}
.thinking-text {
  @apply text-text-muted flex-1 overflow-y-auto text-xs leading-relaxed whitespace-pre-wrap italic;
}

@keyframes rotate-radar {
  100% {
    transform: rotate(360deg);
  }
}
@keyframes pulse-red-bg {
  0%,
  100% {
    box-shadow: 0 0 0 rgba(200, 74, 74, 0);
  }
  50% {
    box-shadow: 0 0 6px rgba(200, 74, 74, 0.3);
  }
}
</style>
