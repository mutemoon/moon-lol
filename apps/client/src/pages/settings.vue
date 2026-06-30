<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useLocale } from "@/composables/useLocale";
import { useSettingsTab } from "@/composables/useSettingsTab";
import { useProviders } from "@/stores/providersStore";
import { providerPresets, PLATFORM_PROVIDER_ID } from "@/config/providerPresets";
import type { ApiFormat, ProviderCategory } from "@/services/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { MoonIcon, RefreshCwIcon, PlusIcon, TrashIcon, PackageIcon } from "@lucide/vue";

const { t, locale, availableLocales } = useLocale();
const { currentTab } = useSettingsTab();
const providersStore = useProviders();

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

// ── 模型供应商管理 ──

interface ProviderForm {
  id?: string;
  name: string;
  category: ProviderCategory;
  preset_type: string;
  base_url: string;
  api_key: string;
  api_format: ApiFormat;
  models: string[];
  enabled: boolean;
  website_url: string;
  api_key_url: string;
  icon: string;
  icon_color: string;
  sort_order: number;
}

const API_FORMATS: { value: ApiFormat; labelKey: string }[] = [
  { value: "anthropic", labelKey: "settings.providers.formatAnthropic" },
  { value: "openai_chat", labelKey: "settings.providers.formatOpenaiChat" },
  { value: "openai_responses", labelKey: "settings.providers.formatOpenaiResponses" },
  { value: "gemini_native", labelKey: "settings.providers.formatGemini" },
];

const selectedKey = ref<string>(PLATFORM_PROVIDER_ID);
const form = ref<ProviderForm>(emptyForm());
const hasApiKey = ref(false);
const providerError = ref("");
const providerSaving = ref(false);

function emptyForm(): ProviderForm {
  return {
    name: "",
    category: "custom",
    preset_type: "",
    base_url: "",
    api_key: "",
    api_format: "anthropic",
    models: [],
    enabled: true,
    website_url: "",
    api_key_url: "",
    icon: "",
    icon_color: "",
    sort_order: 0,
  };
}

const dbProviders = computed(() => providersStore.providers);

/** 新建表单中选中的预设；__custom__ 表示完全自定义。 */
const newPresetChoice = ref<string>("__custom__");

function applyPreset(choice: string) {
  if (choice === "__custom__") {
    form.value = emptyForm();
    return;
  }
  const preset = providerPresets.find((p) => p.presetType === choice);
  if (!preset) return;
  form.value = {
    ...emptyForm(),
    name: preset.name,
    category: "preset",
    preset_type: preset.presetType,
    base_url: preset.baseUrl,
    api_format: preset.apiFormat,
    models: [...preset.defaultModels],
    website_url: preset.websiteUrl ?? "",
    api_key_url: preset.apiKeyUrl ?? "",
    icon: preset.icon ?? "",
    icon_color: preset.iconColor ?? "",
  };
}

watch(newPresetChoice, (v) => applyPreset(v));

const isPlatform = computed(() => selectedKey.value === PLATFORM_PROVIDER_ID);
const isNew = computed(() => selectedKey.value === "new");

function selectKey(key: string) {
  selectedKey.value = key;
  loadForm(key);
}

function loadForm(key: string) {
  providerError.value = "";
  if (key === PLATFORM_PROVIDER_ID || key === "new") {
    form.value = emptyForm();
    hasApiKey.value = false;
    newPresetChoice.value = "__custom__";
    return;
  }
  // 既存 DB 供应商
  const p = dbProviders.value.find((x) => x.id === key);
  if (!p) return;
  form.value = {
    id: p.id,
    name: p.name,
    category: p.category,
    preset_type: p.preset_type,
    base_url: p.base_url,
    api_key: "",
    api_format: p.api_format,
    models: [...p.models],
    enabled: p.enabled,
    website_url: p.website_url ?? "",
    api_key_url: p.api_key_url ?? "",
    icon: p.icon ?? "",
    icon_color: p.icon_color ?? "",
    sort_order: p.sort_order,
  };
  hasApiKey.value = !!p.has_api_key;
}

function addModel() {
  form.value.models.push("");
}

function removeModel(i: number) {
  form.value.models.splice(i, 1);
}

async function saveProvider() {
  providerError.value = "";
  if (!form.value.name.trim()) {
    providerError.value = t("settings.providers.nameLabel") + "不能为空";
    return;
  }
  providerSaving.value = true;
  try {
    const input = {
      name: form.value.name.trim(),
      category: form.value.category,
      preset_type: form.value.preset_type,
      base_url: form.value.base_url.trim(),
      api_key: form.value.api_key,
      api_format: form.value.api_format,
      models: form.value.models.map((m) => m.trim()).filter(Boolean),
      enabled: form.value.enabled,
      website_url: form.value.website_url,
      api_key_url: form.value.api_key_url,
      icon: form.value.icon,
      icon_color: form.value.icon_color,
      sort_order: form.value.sort_order,
    };
    const saved = await providersStore.save(input, form.value.id);
    if (saved) selectKey(saved.id);
  } catch (e: any) {
    providerError.value = typeof e === "string" ? e : e.message || String(e);
  } finally {
    providerSaving.value = false;
  }
}

async function deleteProvider() {
  if (!form.value.id) return;
  if (!confirm(t("settings.providers.deleteConfirm"))) return;
  await providersStore.remove(form.value.id);
  selectKey(PLATFORM_PROVIDER_ID);
}

async function refreshModels() {
  providerError.value = "";
  if (!form.value.base_url.trim()) return;
  try {
    const url = form.value.base_url.trim().replace(/\/$/, "") + "/v1/models";
    const res = await fetch(url, { headers: form.value.api_key ? { Authorization: `Bearer ${form.value.api_key}` } : {} });
    const data = await res.json();
    const remote: string[] = (data?.data ?? data?.models ?? [])
      .map((m: any) => (typeof m === "string" ? m : m?.id))
      .filter(Boolean);
    const merged = Array.from(new Set([...form.value.models, ...remote]));
    form.value.models = merged;
  } catch (e: any) {
    providerError.value = t("settings.providers.refreshFailed", { error: e.message || e });
  }
}

onMounted(() => {
  providersStore.load();
});
</script>

<template>
  <div class="bg-background flex h-full w-full flex-col overflow-hidden">
    <main class="bg-background mx-auto max-w-3xl flex-1 overflow-y-auto p-8">
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

        <div class="border-border bg-card flex flex-col gap-5 rounded-lg border p-5">
          <h2 class="text-foreground border-border flex items-center gap-1.5 border-b pb-1.5 text-sm font-bold">
            <MoonIcon class="text-primary size-4" />
            <span>{{ t("settings.general.appearance.title") }}</span>
          </h2>
          <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
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
                  <SelectItem value="light" class="text-xs">{{ t("settings.general.appearance.themeLight") }}</SelectItem>
                </SelectContent>
              </Select>
            </div>
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
        </div>
      </div>

      <!-- Tab 2: Model Settings (模型设置) -->
      <div v-show="currentTab === 'model_settings'" class="flex flex-col gap-6">
        <div class="flex items-start justify-between gap-3">
          <p class="text-muted-foreground text-[13px] leading-6">{{ t("settings.providers.description") }}</p>
          <Button variant="ghost" size="icon" class="size-6 shrink-0" @click="providersStore.load()">
            <RefreshCwIcon class="size-3.5" />
          </Button>
        </div>

        <div class="border-border bg-card rounded-xl border">
          <div class="grid grid-cols-[200px_minmax(0,1fr)] gap-0">
            <!-- 左侧供应商导航：仅列出已配置供应商，预设通过新建表单的下拉选择预填 -->
            <aside class="border-border flex flex-col gap-3 border-r px-2 py-3">
              <!-- 平台模型 -->
              <div class="flex flex-col gap-1">
                <h3 class="text-muted-foreground px-2 py-1 text-[13px] font-semibold">
                  {{ t("settings.providers.groupPlatform") }}
                </h3>
                <button
                  type="button"
                  class="flex h-8 items-center gap-2 rounded-lg border px-2 text-left text-[13px] font-medium transition-colors"
                  :class="selectedKey === PLATFORM_PROVIDER_ID ? 'border-border-hover bg-muted text-foreground' : 'border-transparent text-foreground hover:bg-muted/60'"
                  @click="selectKey(PLATFORM_PROVIDER_ID)"
                >
                  <PackageIcon class="size-4 shrink-0" />
                  <span class="truncate">{{ t("settings.providers.platformName") }}</span>
                </button>
              </div>

              <!-- 我的供应商 -->
              <div class="flex flex-col gap-1">
                <h3 class="text-muted-foreground px-2 py-1 text-[13px] font-semibold">
                  {{ t("settings.providers.groupMine") }}
                </h3>
                <p v-if="dbProviders.length === 0" class="text-muted-foreground px-2 text-[12px] leading-5">
                  {{ t("settings.providers.empty") }}
                </p>
                <button
                  v-for="p in dbProviders"
                  :key="p.id"
                  type="button"
                  class="flex h-8 items-center gap-2 rounded-lg border px-2 text-left text-[13px] font-medium transition-colors"
                  :class="selectedKey === p.id ? 'border-border-hover bg-muted text-foreground' : 'border-transparent text-foreground hover:bg-muted/60'"
                  @click="selectKey(p.id)"
                >
                  <span class="size-2 shrink-0 rounded-full" :class="p.enabled ? 'bg-emerald-500' : 'bg-muted-foreground/30'" />
                  <span class="truncate">{{ p.name }}</span>
                </button>
                <button
                  type="button"
                  class="flex h-8 items-center gap-2 rounded-lg border px-2 text-left text-[13px] font-medium transition-colors"
                  :class="isNew ? 'border-border-hover bg-muted text-foreground' : 'border-transparent text-foreground hover:bg-muted/60'"
                  @click="selectKey('new')"
                >
                  <PlusIcon class="size-4 shrink-0" />
                  <span class="truncate">{{ t("settings.providers.addProvider") }}</span>
                </button>
              </div>
            </aside>

            <!-- 右侧表单 -->
            <div class="min-w-0 p-4 sm:p-6">
              <!-- 平台模型说明 -->
              <div v-if="isPlatform" class="space-y-3">
                <div class="text-base font-semibold text-foreground">{{ t("settings.providers.platformName") }}</div>
                <p class="text-muted-foreground text-[13px] leading-6">{{ t("settings.providers.platformDesc") }}</p>
              </div>

              <!-- 供应商表单 -->
              <div v-else class="space-y-4">
                <div>
                  <div class="text-base font-semibold text-foreground">
                    {{ isNew ? t("settings.providers.addTitle") : t("settings.providers.editTitle") }}
                  </div>
                  <p v-if="isNew" class="text-muted-foreground mt-1 text-[13px]">{{ t("settings.providers.addDesc") }}</p>
                </div>

                <div class="space-y-3">
                  <div v-if="isNew">
                    <label class="text-muted-foreground mb-1 block text-[13px] font-medium">{{ t("settings.providers.presetTypeLabel") }}</label>
                    <Select v-model="newPresetChoice">
                      <SelectTrigger class="bg-muted/40 border-border h-9 w-full text-[13px]">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent class="border-border bg-popover text-foreground">
                        <SelectItem value="__custom__" class="text-[13px]">{{ t("settings.providers.presetCustom") }}</SelectItem>
                        <SelectItem v-for="preset in providerPresets" :key="preset.presetType" :value="preset.presetType" class="text-[13px]">
                          {{ preset.name }}
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div>
                    <label class="text-muted-foreground mb-1 block text-[13px] font-medium">{{ t("settings.providers.nameLabel") }}</label>
                    <Input v-model="form.name" :placeholder="t('settings.providers.namePlaceholder')" class="bg-muted/40 border-border h-9 text-[13px]" />
                  </div>
                  <div>
                    <label class="text-muted-foreground mb-1 block text-[13px] font-medium">{{ t("settings.providers.baseUrlLabel") }}</label>
                    <Input v-model="form.base_url" :placeholder="t('settings.providers.baseUrlPlaceholder')" class="bg-muted/40 border-border h-9 font-mono text-[13px]" />
                  </div>
                  <div>
                    <label class="text-muted-foreground mb-1 block text-[13px] font-medium">{{ t("settings.providers.apiKeyLabel") }}</label>
                    <Input
                      v-model="form.api_key"
                      type="password"
                      :placeholder="hasApiKey ? t('settings.providers.apiKeyKept') : t('settings.providers.apiKeyPlaceholder')"
                      class="bg-muted/40 border-border h-9 text-[13px]"
                    />
                  </div>
                  <div>
                    <label class="text-muted-foreground mb-1 block text-[13px] font-medium">{{ t("settings.providers.apiFormatLabel") }}</label>
                    <Select v-model="form.api_format">
                      <SelectTrigger class="bg-muted/40 border-border h-9 text-[13px] w-full">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent class="border-border bg-popover text-foreground">
                        <SelectItem v-for="f in API_FORMATS" :key="f.value" :value="f.value" class="text-[13px]">
                          {{ t(f.labelKey) }}
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div>
                    <label class="text-muted-foreground mb-1 block text-[13px]">{{ t("settings.providers.modelsLabel") }}</label>
                    <div class="flex flex-col gap-1.5">
                      <div v-for="(_, i) in form.models" :key="i" class="flex items-center gap-1.5">
                        <Input v-model="form.models[i]" :placeholder="t('settings.providers.modelPlaceholder')" class="bg-muted/40 border-border h-8 font-mono text-[13px]" />
                        <Button variant="ghost" size="icon" class="size-8 shrink-0" @click="removeModel(i)">
                          <TrashIcon class="size-3.5" />
                        </Button>
                      </div>
                    </div>
                    <Button variant="secondary" class="mt-1.5 h-8 gap-1 text-[13px]" @click="addModel">
                      <PlusIcon class="size-3.5" /> {{ t("settings.providers.addModel") }}
                    </Button>
                  </div>
                </div>

                <div class="flex items-center gap-2">
                  <Button :disabled="providerSaving" class="h-8 text-[13px]" @click="saveProvider">
                    {{ t("settings.providers.save") }}
                  </Button>
                  <Button variant="secondary" class="h-8 gap-1 text-[13px]" @click="refreshModels">
                    <RefreshCwIcon class="size-3.5" /> {{ t("settings.providers.refresh") }}
                  </Button>
                  <Button v-if="form.id" variant="ghost" class="text-destructive h-8 text-[13px]" @click="deleteProvider">
                    {{ t("settings.providers.delete") }}
                  </Button>
                </div>
                <p v-if="providerError" class="text-destructive text-xs">{{ providerError }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
::-webkit-scrollbar { width: 4px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border); border-radius: 2px; }
::-webkit-scrollbar-thumb:hover { background: var(--muted-foreground); }
</style>
