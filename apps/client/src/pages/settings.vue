<route lang="yaml">
meta:
  layout: desktop
</route>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { backendClient } from "@/services/backend";
import { useLocale } from "@/composables/useLocale";
import { useSettingsTab } from "@/composables/useSettingsTab";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import {
  MoonIcon,
  TerminalIcon,
  GlobeIcon,
  BellIcon,
  SettingsIcon,
  CpuIcon,
} from "@lucide/vue";

const { t, locale, availableLocales } = useLocale();
const { currentTab } = useSettingsTab();

// General settings state (ZCode mock configs)
const selectedTheme = ref(localStorage.getItem("theme") || "dark");

watch(
  selectedTheme,
  (val) => {
    localStorage.setItem("theme", val);
    if (val === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  },
  { immediate: true },
);

// locale 切换由 useLocale().locale 的 setter 接管（写 localStorage + 切 i18n 实例）
const textScale = ref("normal"); // small | normal | large
const inheritTerminalProfile = ref(true);
const terminalFont = ref("MesloLGS NF, monospace");
const httpProxy = ref("http://127.0.0.1:7890");
const taskNotifications = ref(true);

// AI Config state (Original MoonLOL config)
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
你必须通过调用 \`BashTool\` 运行以下 \`lol_cli\` 命令行指令来与游戏交互。

1. **获取观测 (Observe)**:
   - 运行：\`lol_cli observe -e <MY_ENTITY_ID>\`
   - 说明：返回包含当前英雄状态（生命值 hp, 攻击力 attack_power, 技能点 skill_points）、小兵列表 (minions) 等局势 of JSON 数据。

2. **下达动作 (Action)**:
   - **移动到指定坐标**:
     - 运行：\`lol_cli action -e <MY_ENTITY_ID> move <X> <Y>\`
   - **攻击指定目标小兵/实体 (如补刀)**:
     - 运行：\`lol_cli action -e <MY_ENTITY_ID> attack <ENTITY_ID>\`
   - **停止当前动作**:
     - 运行：\`lol_cli action -e <MY_ENTITY_ID> stop\`
   - **释放技能到指定坐标**:
     - 运行：\`lol_cli action -e <MY_ENTITY_ID> skill <INDEX> <X> <Y>\`
   - **升级指定索引的技能**:
     - 运行：\`lol_cli action -e <MY_ENTITY_ID> skill-level-up <INDEX>\`
`;

async function loadConfig() {
  try {
    const config: any = await backendClient.getAiConfig();
    apiKey.value = config.api_key;
    baseUrl.value = config.base_url;
    preamble.value = config.preamble || DEFAULT_PREAMBLE;
  } catch (e: any) {
    saveError.value = t("settings.model.loadFailed", { error: e.message || e });
  }
}

async function saveConfig() {
  saveError.value = "";
  saveSuccess.value = false;
  isSaving.value = true;

  try {
    await backendClient.setAiConfig({
      api_key: apiKey.value,
      base_url: baseUrl.value,
      preamble: preamble.value,
    });
    saveSuccess.value = true;
    setTimeout(() => {
      saveSuccess.value = false;
    }, 3000);
  } catch (e: any) {
    saveError.value = typeof e === "string" ? e : e.message || t("settings.model.unknownError");
  } finally {
    isSaving.value = false;
  }
}

onMounted(() => {
  loadConfig();
});
</script>

<template>
  <div class="bg-background flex h-full w-full flex-col overflow-hidden">
    <main class="bg-background max-w-2xl flex-1 overflow-y-auto p-8">
      <!-- Tab 1: General (常规) -->
      <div v-show="currentTab === 'general'" class="flex flex-col gap-6">
        <div>
          <h1 class="text-foreground mb-1 text-xl font-bold tracking-tight">{{ t("settings.general.title") }}</h1>
          <div class="flex gap-2">
            <Badge variant="outline" class="border-border text-muted-foreground">
              {{ t("settings.general.darkModeBadge") }}
            </Badge>
            <Badge variant="outline" class="border-border text-muted-foreground">
              {{ t("settings.general.chineseBadge") }}
            </Badge>
          </div>
        </div>

        <!-- Appearance card -->
        <div class="border-border bg-card flex flex-col gap-5 rounded-lg border p-5">
          <h2 class="text-foreground border-border flex items-center gap-1.5 border-b pb-1.5 text-sm font-bold">
            <MoonIcon class="text-primary size-4" />
            <span>{{ t("settings.general.appearance.title") }}</span>
          </h2>

          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <!-- Theme select -->
            <div class="flex flex-col gap-1.5">
              <label class="text-muted-foreground text-xs font-semibold uppercase">
                {{ t("settings.general.appearance.themeLabel") }}
              </label>
              <Select v-model="selectedTheme">
                <SelectTrigger class="border-border bg-muted/40 h-9 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent class="border-border bg-popover text-foreground">
                  <SelectItem value="dark" class="text-xs">{{ t("settings.general.appearance.themeDark") }}</SelectItem>
                  <SelectItem value="light" class="text-xs">
                    {{ t("settings.general.appearance.themeLight") }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>

            <!-- Language select -->
            <div class="flex flex-col gap-1.5">
              <label class="text-muted-foreground text-xs font-semibold uppercase">
                {{ t("settings.general.appearance.languageLabel") }}
              </label>
              <Select v-model="locale">
                <SelectTrigger class="border-border bg-muted/40 h-9 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent class="border-border bg-popover text-foreground">
                  <SelectItem v-for="l in availableLocales" :key="l.value" :value="l.value" class="text-xs">
                    {{ l.native }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <!-- Font Scaling -->
          <div class="flex flex-col gap-1.5">
            <label class="text-muted-foreground text-xs font-semibold uppercase">
              {{ t("settings.general.appearance.scaleLabel") }}
            </label>
            <div class="bg-muted flex max-w-xs gap-1 rounded-md p-1">
              <button
                class="flex-1 rounded-sm py-1.5 text-xs font-semibold transition-colors"
                :class="
                  textScale === 'small'
                    ? 'bg-card text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                "
                @click="textScale = 'small'"
              >
                {{ t("settings.general.appearance.scaleSmall") }}
              </button>
              <button
                class="flex-1 rounded-sm py-1.5 text-xs font-semibold transition-colors"
                :class="
                  textScale === 'normal'
                    ? 'bg-card text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                "
                @click="textScale = 'normal'"
              >
                {{ t("settings.general.appearance.scaleNormal") }}
              </button>
              <button
                class="flex-1 rounded-sm py-1.5 text-xs font-semibold transition-colors"
                :class="
                  textScale === 'large'
                    ? 'bg-card text-foreground shadow-sm'
                    : 'text-muted-foreground hover:text-foreground'
                "
                @click="textScale = 'large'"
              >
                {{ t("settings.general.appearance.scaleLarge") }}
              </button>
            </div>
          </div>
        </div>

        <!-- Terminal Profile Card -->
        <div class="border-border bg-card flex flex-col gap-5 rounded-lg border p-5">
          <h2 class="text-foreground border-border flex items-center gap-1.5 border-b pb-1.5 text-sm font-bold">
            <TerminalIcon class="text-primary size-4" />
            <span>{{ t("settings.general.terminal.title") }}</span>
          </h2>

          <div class="flex items-center gap-2 select-none">
            <Checkbox
              id="terminalToggle"
              :checked="inheritTerminalProfile"
              @update:checked="(val: any) => (inheritTerminalProfile = !!val)"
            />
            <label
              for="terminalToggle"
              class="text-muted-foreground hover:text-foreground cursor-pointer text-xs font-medium"
            >
              {{ t("settings.general.terminal.inheritHint") }}
            </label>
          </div>

          <div class="flex flex-col gap-1.5">
            <label class="text-muted-foreground text-xs font-semibold uppercase">
              {{ t("settings.general.terminal.fontLabel") }}
            </label>
            <div class="flex gap-2">
              <Input
                v-model="terminalFont"
                type="text"
                class="bg-muted/40 border-border h-9 flex-1 font-mono text-xs"
                placeholder="MesloLGS NF, monospace"
              />
              <Button size="sm" class="h-9 px-4 text-xs font-semibold">
                {{ t("settings.general.terminal.saveFont") }}
              </Button>
            </div>
          </div>
        </div>

        <!-- HTTP Proxy Card -->
        <div class="border-border bg-card flex flex-col gap-5 rounded-lg border p-5">
          <h2 class="text-foreground border-border flex items-center gap-1.5 border-b pb-1.5 text-sm font-bold">
            <GlobeIcon class="text-primary size-4" />
            <span>{{ t("settings.general.proxy.title") }}</span>
          </h2>

          <div class="flex flex-col gap-1.5">
            <label class="text-muted-foreground text-xs font-semibold uppercase">
              {{ t("settings.general.proxy.addressLabel") }}
            </label>
            <div class="flex gap-2">
              <Input
                v-model="httpProxy"
                type="text"
                class="bg-muted/40 border-border h-9 flex-1 font-mono text-xs"
                placeholder="http://127.0.0.1:7890"
              />
              <Button size="sm" class="h-9 px-4 text-xs font-semibold">
                {{ t("settings.general.proxy.save") }}
              </Button>
            </div>
            <p class="text-muted-foreground mt-1 text-[10px] leading-normal">
              {{ t("settings.general.proxy.hint") }}
            </p>
          </div>
        </div>

        <!-- Notification Card -->
        <div class="border-border bg-card flex flex-col gap-4 rounded-lg border p-5">
          <h2 class="text-foreground border-border flex items-center gap-1.5 border-b pb-1.5 text-sm font-bold">
            <BellIcon class="text-primary size-4" />
            <span>{{ t("settings.general.notification.title") }}</span>
          </h2>
          <div class="flex items-center gap-2 select-none">
            <Checkbox
              id="notificationToggle"
              :checked="taskNotifications"
              @update:checked="(val: any) => (taskNotifications = !!val)"
            />
            <label
              for="notificationToggle"
              class="text-muted-foreground hover:text-foreground cursor-pointer text-xs font-medium"
            >
              {{ t("settings.general.notification.hint") }}
            </label>
          </div>
        </div>
      </div>

      <!-- Tab 2: Model Settings (模型设置) -->
      <div v-show="currentTab === 'model_settings'" class="flex flex-col gap-6">
        <div>
          <h1 class="text-foreground mb-1 text-xl font-bold tracking-tight">{{ t("settings.model.title") }}</h1>
          <p class="text-muted-foreground text-xs">{{ t("settings.model.description") }}</p>
        </div>

        <!-- Configuration Form -->
        <div class="border-border bg-card flex flex-col gap-5 rounded-lg border p-5">
          <!-- API Key -->
          <div class="flex flex-col gap-1.5">
            <label class="text-muted-foreground text-xs font-bold uppercase">
              {{ t("settings.model.apiKeyLabel") }}
            </label>
            <Input
              v-model="apiKey"
              type="password"
              :placeholder="t('settings.model.apiKeyPlaceholder')"
              :disabled="isSaving"
              class="bg-muted/40 border-border h-9 text-xs"
            />
          </div>

          <!-- Base URL -->
          <div class="flex flex-col gap-1.5">
            <label class="text-muted-foreground text-xs font-bold uppercase">
              {{ t("settings.model.baseUrlLabel") }}
            </label>
            <Input
              v-model="baseUrl"
              type="text"
              :placeholder="t('settings.model.baseUrlPlaceholder')"
              :disabled="isSaving"
              class="bg-muted/40 border-border h-9 font-mono text-xs"
            />
          </div>

          <!-- Preamble Prompt -->
          <div class="flex flex-col gap-1.5">
            <label class="text-muted-foreground text-xs font-bold uppercase">
              {{ t("settings.model.preambleLabel") }}
            </label>
            <Textarea
              v-model="preamble"
              :placeholder="t('settings.model.preamblePlaceholder')"
              :disabled="isSaving"
              rows="10"
              class="bg-muted/40 border-border min-h-56 font-mono text-xs leading-relaxed"
            />
          </div>

          <!-- Save Button -->
          <div class="flex flex-col gap-2.5">
            <Button class="w-full py-5 text-xs font-semibold" :disabled="isSaving" @click="saveConfig">
              {{ isSaving ? t("settings.model.saving") : t("settings.model.save") }}
            </Button>

            <Transition
              enter-active-class="transition-opacity duration-200 ease-out"
              leave-active-class="transition-opacity duration-200 ease-out"
              enter-from-class="opacity-0"
              leave-to-class="opacity-0"
            >
              <p v-if="saveSuccess" class="text-center text-xs font-semibold text-green-500">
                {{ t("settings.model.saveSuccess") }}
              </p>
            </Transition>

            <Transition
              enter-active-class="transition-opacity duration-200 ease-out"
              leave-active-class="transition-opacity duration-200 ease-out"
              enter-from-class="opacity-0"
              leave-to-class="opacity-0"
            >
              <p v-if="saveError" class="text-destructive text-center text-xs font-semibold">
                {{ t("settings.model.saveFailed", { error: saveError }) }}
              </p>
            </Transition>
          </div>
        </div>
      </div>

      <!-- Other Mock Tabs -->
      <div v-show="currentTab !== 'general' && currentTab !== 'model_settings'" class="py-12 text-center">
        <CpuIcon class="text-muted-foreground/30 mx-auto mb-3 size-10" />
        <h2 class="text-foreground text-sm font-semibold">{{ t("settings.placeholder.title") }}</h2>
        <p class="text-muted-foreground mx-auto mt-1.5 max-w-sm text-xs leading-relaxed">
          {{ t("settings.placeholder.body") }}
        </p>
      </div>
    </main>
  </div>
</template>

<style scoped>
/* Custom Scrollbars */
::-webkit-scrollbar {
  width: 4px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--muted-foreground);
}
</style>
