<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const apiKey = ref("");
const baseUrl = ref("");
const isSaving = ref(false);
const saveError = ref("");
const saveSuccess = ref(false);

async function loadConfig() {
  try {
    const config: any = await invoke("get_ai_config");
    apiKey.value = config.api_key;
    baseUrl.value = config.base_url;
  } catch (e: any) {
    saveError.value = "Failed to load config: " + e;
  }
}

async function saveConfig() {
  saveError.value = "";
  saveSuccess.value = false;
  isSaving.value = true;

  try {
    await invoke("set_ai_config", {
      config: {
        api_key: apiKey.value,
        base_url: baseUrl.value,
      },
    });
    saveSuccess.value = true;
    setTimeout(() => {
      saveSuccess.value = false;
    }, 3000);
  } catch (e: any) {
    saveError.value = typeof e === "string" ? e : e.message || "Unknown error";
  } finally {
    isSaving.value = false;
  }
}

onMounted(() => {
  loadConfig();
});
</script>

<template>
  <div class="settings">
    <div class="settings-inner">
      <div class="header">
        <h1 class="settings-title">Settings</h1>
        <p class="settings-subtitle">AI Agent Configuration</p>
      </div>

      <div class="settings-card">
        <div class="field">
          <label class="field-label">Anthropic API Key</label>
          <div class="input-wrap">
            <input
              v-model="apiKey"
              type="password"
              placeholder="Enter your Anthropic API Key"
              :disabled="isSaving"
            />
          </div>
        </div>

        <div class="field">
          <label class="field-label">Anthropic Base URL</label>
          <div class="input-wrap">
            <input
              v-model="baseUrl"
              type="text"
              placeholder="e.g. https://api.deepseek.com/anthropic"
              :disabled="isSaving"
            />
          </div>
        </div>

        <button
          class="btn-primary"
          :disabled="isSaving"
          @click="saveConfig"
        >
          <span class="btn-shine" aria-hidden="true"></span>
          <span class="btn-label">{{ isSaving ? 'Saving…' : 'Save Config' }}</span>
        </button>

        <Transition name="fade">
          <p v-if="saveSuccess" class="success-msg">Config saved to ~/.moon-lol/.env successfully!</p>
        </Transition>

        <Transition name="fade">
          <p v-if="saveError" class="error-msg">{{ saveError }}</p>
        </Transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100%;
  padding: 40px 24px;
}

.settings-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 32px;
  width: 100%;
  max-width: 400px;
}

/* ── Header ── */
.header {
  text-align: center;
}

.settings-title {
  font-family: var(--font-display);
  font-size: var(--fs-display);
  font-weight: 700;
  color: var(--gold-bright);
  letter-spacing: 0.06em;
  line-height: 1.15;
  text-shadow: 0 0 30px rgba(212,175,92,0.15);
}

.settings-subtitle {
  margin-top: 6px;
  font-size: var(--fs-small);
  color: var(--text-muted);
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

/* ── Settings Card ── */
.settings-card {
  position: relative;
  width: 100%;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: 28px;
  box-shadow: var(--shadow-md);
}

.settings-card::before {
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

.input-wrap {
  position: relative;
}

input {
  width: 100%;
  appearance: none;
  background: var(--bg-deep);
  color: var(--text-bright);
  border: 1px solid var(--gold-dimmer);
  border-radius: var(--radius-md);
  padding: 10px 14px;
  font-size: var(--fs-body);
  font-weight: 400;
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.5);
  transition: border-color var(--dur-fast) ease-out, box-shadow var(--dur-fast) ease-out;
}

input::placeholder {
  color: var(--text-muted);
  opacity: 0.6;
}

input:hover {
  border-color: var(--gold-muted);
}

input:focus {
  border-color: var(--gold-default);
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.5), var(--shadow-glow-gold);
  outline: none;
}

input:disabled {
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

/* Success & Error */
.success-msg {
  margin-top: 12px;
  font-size: var(--fs-small);
  color: var(--green);
  text-align: center;
}

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
