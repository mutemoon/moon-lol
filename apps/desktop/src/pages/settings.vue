<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "../components/ui/button";

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

每次必须且只能调用一个工具命令，不要输出无关 of 普通文本。
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
  <div class="flex items-center justify-center min-h-full py-10 px-6">
    <div class="flex flex-col items-center gap-8 w-full max-w-[36rem]">
      <!-- Header -->
      <div class="text-center">
        <h1 class="font-display text-[2.5rem] font-bold text-gold-bright tracking-[0.06em] leading-[1.15]" style="text-shadow: 0 0 30px rgba(212, 175, 92, 0.15);">Settings</h1>
        <p class="mt-1.5 text-xs text-text-muted tracking-[0.1rem] uppercase">AI Agent Configuration</p>
      </div>

      <!-- Settings Card -->
      <div class="relative w-full bg-bg-surface border border-border-subtle rounded-[0.625rem] p-7 shadow-[0_4px_12px_rgba(0,0,0,0.5),0_0_2px_rgba(120,91,40,0.2)]">
        <!-- Inner Border Gradient Overlay -->
        <div class="pointer-events-none absolute inset-0 rounded-[0.625rem] border border-transparent [background:linear-gradient(180deg,rgba(185,145,71,0.15),transparent_60%)_border-box] [mask:linear-gradient(#fff_0_0)_content-box,linear-gradient(#fff_0_0)] [mask-composite:exclude] [-webkit-mask-composite:xor]"></div>

        <!-- Anthropic API Key -->
        <div class="mb-5">
          <label class="block mb-1.5 text-xs font-medium text-text-muted tracking-[0.03em] uppercase">Anthropic API Key</label>
          <div class="relative">
            <input
              v-model="apiKey"
              type="password"
              placeholder="Enter your Anthropic API Key"
              :disabled="isSaving"
              class="w-full appearance-none bg-bg-deep text-text-bright border border-gold-dimmer rounded-[0.375rem] px-3.5 py-2.5 text-sm font-normal shadow-[inset_0_2px_4px_rgba(0,0,0,0.5)] placeholder:text-text-muted/60 hover:border-gold-muted focus:border-gold-default focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-gold-default/30 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            />
          </div>
        </div>

        <!-- Anthropic Base URL -->
        <div class="mb-5">
          <label class="block mb-1.5 text-xs font-medium text-text-muted tracking-[0.03em] uppercase">Anthropic Base URL</label>
          <div class="relative">
            <input
              v-model="baseUrl"
              type="text"
              placeholder="e.g. https://api.deepseek.com/anthropic"
              :disabled="isSaving"
              class="w-full appearance-none bg-bg-deep text-text-bright border border-gold-dimmer rounded-[0.375rem] px-3.5 py-2.5 text-sm font-normal shadow-[inset_0_2px_4px_rgba(0,0,0,0.5)] placeholder:text-text-muted/60 hover:border-gold-muted focus:border-gold-default focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-gold-default/30 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            />
          </div>
        </div>

        <!-- Agent Preamble (Prompt) -->
        <div class="mb-6">
          <label class="block mb-1.5 text-xs font-medium text-text-muted tracking-[0.03em] uppercase">Agent Preamble (Prompt)</label>
          <div class="relative">
            <textarea
              v-model="preamble"
              placeholder="输入 AI 代理的系统预设提示词（Preamble Prompt）"
              :disabled="isSaving"
              rows="12"
              class="w-full appearance-none bg-bg-deep text-text-bright border border-gold-dimmer rounded-[0.375rem] px-3.5 py-3 text-sm font-normal font-mono leading-relaxed shadow-[inset_0_2px_4px_rgba(0,0,0,0.5)] placeholder:text-text-muted/60 hover:border-gold-muted focus:border-gold-default focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-gold-default/30 transition-all disabled:opacity-50 disabled:cursor-not-allowed resize-y min-h-[200px]"
            ></textarea>
          </div>
        </div>

        <!-- Save Button -->
        <Button
          class="relative w-full overflow-hidden py-6 text-sm font-semibold tracking-[0.08em] uppercase text-gold-bright bg-bg-surface border border-transparent rounded-[0.375rem] shadow-sm hover:text-gold-glow hover:shadow-glow-gold active:scale-[0.97] transition-all disabled:opacity-50 disabled:cursor-not-allowed group/btn cursor-pointer"
          :disabled="isSaving"
          @click="saveConfig"
        >
          <!-- Border gradient via overlay child -->
          <div class="pointer-events-none absolute inset-0 rounded-[0.375rem] border border-transparent [background:linear-gradient(135deg,var(--color-gold-dimmer),var(--color-gold-default),var(--color-gold-bright))_border-box] [mask:linear-gradient(#fff_0_0)_content-box,linear-gradient(#fff_0_0)] [mask-composite:exclude] [-webkit-mask-composite:xor]"></div>
          
          <!-- Shine sweep -->
          <span class="absolute top-0 left-[-50%] w-1/2 h-full bg-gradient-to-r from-transparent via-[rgba(232,201,122,0.15)] to-transparent -skew-x-[20deg] pointer-events-none transition-none group-hover/btn:left-[150%] group-hover/btn:transition-[left] group-hover/btn:duration-700 group-hover/btn:ease-[cubic-bezier(0.16,1,0.3,1)]" aria-hidden="true"></span>
          
          <span class="relative z-1">{{ isSaving ? "Saving…" : "Save Config" }}</span>
        </Button>

        <Transition
          enter-active-class="transition-opacity duration-200 ease-out"
          leave-active-class="transition-opacity duration-200 ease-out"
          enter-from-class="opacity-0"
          leave-to-class="opacity-0"
        >
          <p v-if="saveSuccess" class="mt-3 text-xs text-green text-center">Config saved to ~/.moon-lol/.env successfully!</p>
        </Transition>

        <Transition
          enter-active-class="transition-opacity duration-200 ease-out"
          leave-active-class="transition-opacity duration-200 ease-out"
          enter-from-class="opacity-0"
          leave-to-class="opacity-0"
        >
          <p v-if="saveError" class="mt-3 text-xs text-red text-center">{{ saveError }}</p>
        </Transition>
      </div>
    </div>
  </div>
</template>
