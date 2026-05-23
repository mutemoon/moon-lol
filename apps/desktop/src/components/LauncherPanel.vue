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
      <div class="hero text-center">
        <div class="mb-4">
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
      <div class="launch-card relative w-full">
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
              <option value="agent">AI Agent</option>
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
  padding: 2.5rem 1.5rem;
}

.launcher-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2rem;
  width: 100%;
  max-width: 28rem;
}

.hero-title {
  font-family: var(--font-display);
  font-size: 2.5rem;
  font-weight: 700;
  color: var(--color-gold-bright);
  letter-spacing: 0.06em;
  line-height: 1.15;
  text-shadow: 0 0 30px rgba(212,175,92,0.15);
}

.hero-subtitle {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

/* Launch Card */
.launch-card {
  position: relative;
  width: 100%;
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: 0.625rem;
  padding: 1.75rem;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5), 0 0 2px rgba(120, 91, 40, 0.2);
}

.launch-card::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 0.625rem;
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
  background: var(--color-bg-surface);
  border-radius: 0.625rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  z-index: 5;
}

.spinner {
  width: 2rem;
  height: 2rem;
  border: 2px solid var(--color-border-subtle);
  border-top-color: var(--color-gold-default);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.connecting-text {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  text-align: center;
  padding: 0 1rem;
}

.connecting-text.timeout {
  color: var(--color-gold-dimmer);
}

.cancel-btn {
  padding: 0.375rem 1.5rem;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  border: 1px solid var(--color-border-subtle);
  border-radius: 0.375rem;
  transition: all 0.2s;
}

.cancel-btn:hover {
  color: var(--color-text-default);
  border-color: var(--color-gold-muted);
}

/* Fields */
.field {
  margin-bottom: 1.25rem;
}

.field:last-of-type {
  margin-bottom: 1.5rem;
}

.field-label {
  display: block;
  margin-bottom: 0.375rem;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-muted);
  letter-spacing: 0.03em;
  text-transform: uppercase;
}

.select-wrap {
  position: relative;
}

.select-wrap::after {
  content: '';
  position: absolute;
  right: 0.75rem;
  top: 50%;
  transform: translateY(-50%);
  width: 0;
  height: 0;
  border-left: 4px solid transparent;
  border-right: 4px solid transparent;
  border-top: 5px solid var(--color-gold-muted);
  pointer-events: none;
}

select {
  width: 100%;
  appearance: none;
  background: var(--color-bg-deep);
  color: var(--color-text-bright);
  border: 1px solid var(--color-gold-dimmer);
  border-radius: 0.375rem;
  padding: 0.625rem 2.25rem 0.625rem 0.875rem;
  font-size: 0.875rem;
  font-weight: 400;
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.5);
  transition: border-color 0.2s, box-shadow 0.2s;
  cursor: pointer;
}

select:hover {
  border-color: var(--color-gold-muted);
}

select:focus {
  border-color: var(--color-gold-default);
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.5), 0 0 12px rgba(201, 170, 113, 0.25), 0 0 4px rgba(201, 170, 113, 0.4);
  outline: none;
}

select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Primary Button */
.btn-primary {
  position: relative;
  width: 100%;
  overflow: hidden;
  padding: 0.75rem 1.5rem;
  font-size: 0.875rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-gold-bright);
  background: var(--color-bg-surface);
  border: 1px solid transparent;
  border-radius: 0.375rem;
  background-clip: padding-box;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.4), 0 0 1px rgba(120, 91, 40, 0.15), inset 0 1px 0 rgba(212,175,92,0.12);
  transition: all 0.2s;
  cursor: pointer;
}

.btn-primary::before {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: 0.375rem;
  padding: 1px;
  background: linear-gradient(135deg, var(--color-gold-dimmer), var(--color-gold-default), var(--color-gold-bright));
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  pointer-events: none;
}

.btn-primary:hover:not(:disabled) {
  color: var(--color-gold-glow);
  box-shadow: 0 0 12px rgba(201, 170, 113, 0.25), 0 0 4px rgba(201, 170, 113, 0.4), inset 0 1px 0 rgba(232,201,122,0.2);
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
  background: linear-gradient(90deg, transparent, rgba(232,201,122,0.1), transparent);
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
  margin-top: 0.75rem;
  font-size: 0.75rem;
  color: var(--color-red);
  text-align: center;
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease-out;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
