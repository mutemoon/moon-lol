<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const apiKey = ref("");
const baseUrl = ref("");
const preamble = ref("");
const isSaving = ref(false);
const saveError = ref("");
const saveSuccess = ref(false);

const DEFAULT_PREAMBLE = `你是一个运行在 MOBA 游戏环境中的 AI Agent。你的使命是使用 lol_cli 来观测游戏状态并执行动作，取得更好的游戏表现。

【核心目标】击杀敌方小兵（补刀）并保证自身生存。

【战术规则】
1. 双方小兵会在 (2649, 12875) 交汇。开局或空闲时必须 move 到此位置。
2. 补刀优先：当某小兵生命值 <= 你的攻击力时，立即用 attack 指令指定该小兵 ID 进行补刀。
3. 技能升级：若拥有技能点(skill_points > 0)，优先升级技能 0 (通常是 Q 技能)。
4. 生存意识：当生命值 hp 低于 30% 时，应使用 move 指令撤退到安全位置。

【去重原则】
如果你的当前移动/攻击状态已经完全符合预期，调用 stop 指令或无需进行任何操作。

【工具调用指南 (lol_cli 使用帮助)】
你必须通过调用 \`BashTool\` 运行以下 \`lol_cli\` 命令行指令来与游戏交互：

1. **获取观测 (Observe)**:
   - 运行：\`cargo run -p lol_cli -- observe\`（或简写为 \`cargo run -p lol_cli -- obs\`）
   - 说明：返回包含当前英雄状态（生命值 hp, 攻击力 attack_power, 技能点 skill_points）、小兵列表 (minions) 等局势的 JSON 数据。

2. **下达动作 (Action)**:
   - **移动到指定坐标**:
     - 运行：\`cargo run -p lol_cli -- action move <X> <Y>\`（或简写为 \`cargo run -p lol_cli -- act move <X> <Y>\`）
     - 示例：\`cargo run -p lol_cli -- act move 2649 12875\`
   - **攻击指定目标小兵/实体 (如补刀)**:
     - 运行：\`cargo run -p lol_cli -- action attack <ENTITY_ID>\`（或简写为 \`cargo run -p lol_cli -- act attack <ENTITY_ID>\`）
     - 示例：\`cargo run -p lol_cli -- act attack 12345\`
   - **停止当前动作**:
     - 运行：\`cargo run -p lol_cli -- action stop\`（或简写为 \`cargo run -p lol_cli -- act stop\`）
   - **释放技能到指定坐标**:
     - 运行：\`cargo run -p lol_cli -- action skill <INDEX> <X> <Y>\`（或简写为 \`cargo run -p lol_cli -- act skill <INDEX> <X> <Y>\`，其中 INDEX 为 0-3 代表 QWER 技能）
     - 示例：\`cargo run -p lol_cli -- act skill 0 2649 12875\`
   - **升级指定索引的技能**:
     - 运行：\`cargo run -p lol_cli -- action skill-level-up <INDEX>\`（或简写为 \`cargo run -p lol_cli -- act upgrade <INDEX>\`，其中 INDEX 为 0-3）
     - 示例：\`cargo run -p lol_cli -- act upgrade 0\`

每次必须且只能调用一个工具命令，不要输出无关的普通文本。
`;

async function loadConfig() {
  try {
    const config: any = await invoke("get_ai_config");
    apiKey.value = config.api_key;
    baseUrl.value = config.base_url;
    preamble.value = config.preamble || DEFAULT_PREAMBLE;
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
        preamble: preamble.value,
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
      <!-- Header -->
      <div class="text-center">
        <h1 class="settings-title">Settings</h1>
        <p class="settings-subtitle">AI Agent Configuration</p>
      </div>

      <!-- Settings Card -->
      <div class="settings-card relative w-full">
        <div class="field">
          <label class="field-label">Anthropic API Key</label>
          <div class="input-wrap">
            <input v-model="apiKey" type="password" placeholder="Enter your Anthropic API Key" :disabled="isSaving" />
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

        <div class="field">
          <label class="field-label">Agent Preamble (Prompt)</label>
          <div class="input-wrap">
            <textarea
              v-model="preamble"
              placeholder="输入 AI 代理的系统预设提示词（Preamble Prompt）"
              :disabled="isSaving"
              rows="12"
            ></textarea>
          </div>
        </div>

        <button class="btn-primary" :disabled="isSaving" @click="saveConfig">
          <span class="btn-shine" aria-hidden="true"></span>
          <span class="btn-label">{{ isSaving ? "Saving…" : "Save Config" }}</span>
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
  height: 100%;
  padding: 2.5rem 1.5rem;
}

.settings-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2rem;
  width: 100%;
  max-width: 36rem;
}

.settings-title {
  font-family: var(--font-display);
  font-size: 2.5rem;
  font-weight: 700;
  color: var(--color-gold-bright);
  letter-spacing: 0.06em;
  line-height: 1.15;
  text-shadow: 0 0 30px rgba(212, 175, 92, 0.15);
}

.settings-subtitle {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  color: var(--color-text-muted);
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

/* Settings Card */
.settings-card {
  position: relative;
  width: 100%;
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  border-radius: 0.625rem;
  padding: 1.75rem;
  box-shadow:
    0 4px 12px rgba(0, 0, 0, 0.5),
    0 0 2px rgba(120, 91, 40, 0.2);
}

.settings-card::before {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: 0.625rem;
  padding: 1px;
  background: linear-gradient(180deg, rgba(185, 145, 71, 0.15), transparent 60%);
  -webkit-mask:
    linear-gradient(#fff 0 0) content-box,
    linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  pointer-events: none;
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

.input-wrap {
  position: relative;
}

input {
  width: 100%;
  appearance: none;
  background: var(--color-bg-deep);
  color: var(--color-text-bright);
  border: 1px solid var(--color-gold-dimmer);
  border-radius: 0.375rem;
  padding: 0.625rem 0.875rem;
  font-size: 0.875rem;
  font-weight: 400;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.5);
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
}

input::placeholder {
  color: var(--color-text-muted);
  opacity: 0.6;
}

input:hover {
  border-color: var(--color-gold-muted);
}

input:focus {
  border-color: var(--color-gold-default);
  box-shadow:
    inset 0 2px 4px rgba(0, 0, 0, 0.5),
    0 0 12px rgba(201, 170, 113, 0.25),
    0 0 4px rgba(201, 170, 113, 0.4);
  outline: none;
}

input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Textarea Premium Styling */
textarea {
  width: 100%;
  appearance: none;
  background: var(--color-bg-deep);
  color: var(--color-text-bright);
  border: 1px solid var(--color-gold-dimmer);
  border-radius: 0.375rem;
  padding: 0.75rem 0.875rem;
  font-size: 0.875rem;
  font-weight: 400;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.5);
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
  font-family: var(--font-mono), monospace;
  line-height: 1.6;
  resize: vertical;
  min-height: 200px;
}

textarea::placeholder {
  color: var(--color-text-muted);
  opacity: 0.6;
}

textarea:hover {
  border-color: var(--color-gold-muted);
}

textarea:focus {
  border-color: var(--color-gold-default);
  box-shadow:
    inset 0 2px 4px rgba(0, 0, 0, 0.5),
    0 0 12px rgba(201, 170, 113, 0.25),
    0 0 4px rgba(201, 170, 113, 0.4);
  outline: none;
}

textarea:disabled {
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
  box-shadow:
    0 1px 2px rgba(0, 0, 0, 0.4),
    0 0 1px rgba(120, 91, 40, 0.15),
    inset 0 1px 0 rgba(212, 175, 92, 0.12);
  transition: all 0.2s;
  cursor: pointer;
}

.btn-primary::before {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: 0.375rem;
  padding: 1px;
  background: linear-gradient(135deg, var(--color-gold-dimmer), var(--color-gold-default), var(--color-gold-bright));
  -webkit-mask:
    linear-gradient(#fff 0 0) content-box,
    linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  pointer-events: none;
}

.btn-primary:hover:not(:disabled) {
  color: var(--color-gold-glow);
  box-shadow:
    0 0 12px rgba(201, 170, 113, 0.25),
    0 0 4px rgba(201, 170, 113, 0.4),
    inset 0 1px 0 rgba(232, 201, 122, 0.2);
  text-shadow: 0 0 8px rgba(232, 201, 122, 0.3);
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
  background: linear-gradient(90deg, transparent, rgba(232, 201, 122, 0.1), transparent);
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
  margin-top: 0.75rem;
  font-size: 0.75rem;
  color: var(--color-green);
  text-align: center;
}

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
