<script setup lang="ts">
defineProps<{
  champion: string;
  mode: string;
  isStarting: boolean;
  error: string;
  connecting: boolean;
  connectTimeout: boolean;
  champions: string[];
}>();

defineEmits<{
  (e: "update:champion", v: string): void;
  (e: "update:mode", v: string): void;
  (e: "start"): void;
  (e: "cancel"): void;
}>();
</script>

<template>
  <div class="launcher">
    <div class="launcher-inner">
      <!-- Hero branding -->
      <div class="hero">
        <div class="hero-emblem">
          <svg viewBox="0 0 64 64" width="64" height="64" fill="none" aria-hidden="true">
            <circle cx="32" cy="32" r="28" stroke="url(#goldGrad)" stroke-width="1.5" />
            <path d="M16 32 L48 32 M32 16 L32 48" stroke="url(#goldGrad)" stroke-width="2" stroke-linecap="round" />
            <circle cx="32" cy="32" r="8" fill="none" stroke="url(#goldGrad)" stroke-width="1" />
            <defs>
              <linearGradient id="goldGrad" x1="0" y1="0" x2="64" y2="64">
                <stop offset="0%" stop-color="#927136" />
                <stop offset="50%" stop-color="#d4af5c" />
                <stop offset="100%" stop-color="#785b28" />
              </linearGradient>
            </defs>
          </svg>
        </div>
        <h1 class="hero-title">MoonLOL</h1>
        <p class="hero-subtitle">League Launcher</p>
      </div>

      <!-- Launch Card -->
      <div class="launch-card">
        <!-- Connecting overlay -->
        <Transition name="fade">
          <div v-if="connecting" class="connecting-overlay">
            <div class="spinner"></div>
            <p class="connecting-text" :class="{ timeout: connectTimeout }">
              {{ connectTimeout ? 'Still connecting… game may be slow to start.' : 'Connecting to game…' }}
            </p>
            <button class="cancel-btn" @click="$emit('cancel')">Cancel</button>
          </div>
        </Transition>

        <div class="field">
          <label class="field-label">Champion</label>
          <div class="select-wrap">
            <select
              :value="champion"
              :disabled="isStarting"
              @change="$emit('update:champion', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="c in champions" :key="c" :value="c">{{ c }}</option>
            </select>
          </div>
        </div>

        <div class="field">
          <label class="field-label">Mode</label>
          <div class="select-wrap">
            <select
              :value="mode"
              :disabled="isStarting"
              @change="$emit('update:mode', ($event.target as HTMLSelectElement).value)"
            >
              <option value="sandbox">Sandbox</option>
            </select>
          </div>
        </div>

        <button
          class="btn-primary"
          :disabled="isStarting"
          @click="$emit('start')"
        >
          <span class="btn-shine" aria-hidden="true"></span>
          <span class="btn-label">{{ isStarting ? 'Starting…' : 'Launch Game' }}</span>
        </button>

        <Transition name="fade">
          <p v-if="error" class="error-msg">{{ error }}</p>
        </Transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
.launcher {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100%;
  padding: 40px 24px;
}

.launcher-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 32px;
  width: 100%;
  max-width: 400px;
}

/* ── Hero ── */
.hero {
  text-align: center;
}

.hero-emblem {
  margin-bottom: 16px;
  filter: drop-shadow(0 0 20px rgba(212,175,92,0.15));
}

.hero-title {
  font-family: var(--font-display);
  font-size: var(--fs-display);
  font-weight: 700;
  color: var(--gold-bright);
  letter-spacing: 0.06em;
  line-height: 1.15;
  text-shadow: 0 0 30px rgba(212,175,92,0.15);
}

.hero-subtitle {
  margin-top: 6px;
  font-size: var(--fs-small);
  color: var(--text-muted);
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

/* ── Launch Card ── */
.launch-card {
  position: relative;
  width: 100%;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: 28px;
  box-shadow: var(--shadow-md);
}

.launch-card::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: var(--radius-lg);
  padding: 1px;
  background: linear-gradient(180deg, rgba(185,145,71,0.15), transparent 60%);
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  pointer-events: none;
}

/* Connecting overlay */
.connecting-overlay {
  position: absolute;
  inset: -1px;
  background: var(--bg-surface);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  z-index: 5;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 2px solid var(--border-subtle);
  border-top-color: var(--gold-default);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.connecting-text {
  font-size: var(--fs-small);
  color: var(--text-muted);
  text-align: center;
  padding: 0 16px;
}

.connecting-text.timeout {
  color: var(--gold-dimmer);
}

.cancel-btn {
  padding: 6px 24px;
  font-size: var(--fs-small);
  color: var(--text-muted);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  transition: all var(--dur-fast) ease-out;
}

.cancel-btn:hover {
  color: var(--text-default);
  border-color: var(--gold-muted);
}

/* Fields */
.field {
  margin-bottom: 20px;
}

.field:last-of-type {
  margin-bottom: 24px;
}

.field-label {
  display: block;
  margin-bottom: 6px;
  font-size: var(--fs-small);
  font-weight: 500;
  color: var(--text-muted);
  letter-spacing: 0.03em;
  text-transform: uppercase;
}

.select-wrap {
  position: relative;
}

.select-wrap::after {
  content: '';
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 0;
  height: 0;
  border-left: 4px solid transparent;
  border-right: 4px solid transparent;
  border-top: 5px solid var(--gold-muted);
  pointer-events: none;
}

select {
  width: 100%;
  appearance: none;
  background: var(--bg-deep);
  color: var(--text-bright);
  border: 1px solid var(--gold-dimmer);
  border-radius: var(--radius-md);
  padding: 10px 36px 10px 14px;
  font-size: var(--fs-body);
  font-weight: 400;
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.5);
  transition: border-color var(--dur-fast) ease-out, box-shadow var(--dur-fast) ease-out;
  cursor: pointer;
}

select:hover {
  border-color: var(--gold-muted);
}

select:focus {
  border-color: var(--gold-default);
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.5), var(--shadow-glow-gold);
  outline: none;
}

select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ── Primary Button ── */
.btn-primary {
  position: relative;
  width: 100%;
  overflow: hidden;
  padding: 12px 24px;
  font-size: var(--fs-body);
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--gold-bright);
  background: var(--bg-surface);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  background-clip: padding-box;
  box-shadow:
    var(--shadow-sm),
    inset 0 1px 0 rgba(212,175,92,0.12);
  transition: all var(--dur-fast) ease-out;
  cursor: pointer;

  /* Gold gradient border via pseudo */
}

.btn-primary::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: var(--radius-md);
  padding: 1px;
  background: linear-gradient(135deg, var(--gold-dimmer), var(--gold-default), var(--gold-bright));
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  pointer-events: none;
}

.btn-primary:hover:not(:disabled) {
  color: var(--gold-glow);
  box-shadow:
    var(--shadow-glow-gold),
    inset 0 1px 0 rgba(232,201,122,0.2);
  text-shadow: 0 0 8px rgba(232,201,122,0.3);
}

.btn-primary:active:not(:disabled) {
  transform: scale(0.97);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Shine sweep on active */
.btn-shine {
  position: absolute;
  top: 0;
  left: -50%;
  width: 50%;
  height: 100%;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(232,201,122,0.1),
    transparent
  );
  transform: skewX(-20deg);
  transition: none;
  pointer-events: none;
}

.btn-primary:not(:disabled):hover .btn-shine {
  left: 150%;
  transition: left 0.6s cubic-bezier(0.16, 1, 0.3, 1);
}

.btn-label {
  position: relative;
  z-index: 1;
}

/* Error */
.error-msg {
  margin-top: 12px;
  font-size: var(--fs-small);
  color: var(--red);
  text-align: center;
}

/* ── Transitions ── */
.fade-enter-active,
.fade-leave-active {
  transition: opacity var(--dur-fast) ease-out;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
