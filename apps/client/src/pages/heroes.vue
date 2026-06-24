<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { storeToRefs } from "pinia";
import { useGameStore, type HeroPreset } from "@/stores/gameStore";
import { useRouter, useRoute } from "vue-router";
import { useLocale } from "@/composables/useLocale";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { SwordsIcon, PlusIcon, Trash2Icon, SaveIcon, ArrowLeftIcon } from "@lucide/vue";

// 英雄预设管理页（产品文档 §3.0）：管理自定义选手的属性与运行策略。
// 每个选手包含：英雄、决策类型以及对应的决策配置参数（LLM、RL 或脚本配置）。

const store = useGameStore();
const { heroPresets, champions } = storeToRefs(store);
const router = useRouter();
const route = useRoute();
const { t } = useLocale();

const selectedName = ref<string | null>(null);
const errorMsg = ref("");
const showDeleteConfirm = ref(false);

const emptyDraft = (): HeroPreset => ({
  name: "",
  champion: "Riven",
  agent_type: "llm",
  prompt: "",
  preamble: "",
  model: "",
  config_json: {},
});
const draft = ref<HeroPreset>(emptyDraft());
const configJsonStr = ref("{}");

function selectPreset(name: string) {
  selectedName.value = name;
  const p = heroPresets.value.find((x) => x.name === name);
  if (p) {
    draft.value = { ...p };
    configJsonStr.value = JSON.stringify(p.config_json || {}, null, 2);
  }
}

function startNew() {
  selectedName.value = null;
  draft.value = emptyDraft();
  configJsonStr.value = "{}";
}

async function handleSave() {
  errorMsg.value = "";
  const name = draft.value.name.trim();
  if (!name) {
    errorMsg.value = t("heroes.errorFillName");
    return;
  }
  if (draft.value.agent_type !== "llm") {
    try {
      draft.value.config_json = JSON.parse(configJsonStr.value || "{}");
    } catch (e) {
      errorMsg.value = t("heroes.invalidJson");
      return;
    }
  } else {
    draft.value.config_json = {};
  }
  try {
    await store.saveHeroPreset({ ...draft.value, name });
    selectedName.value = name;
    errorMsg.value = t("heroes.successSave");
  } catch (e: any) {
    errorMsg.value = e.message || t("heroes.errorSave");
  }
}

async function confirmDelete() {
  if (!selectedName.value) return;
  try {
    await store.deleteHeroPreset(selectedName.value);
    selectedName.value = null;
    draft.value = emptyDraft();
    configJsonStr.value = "{}";
    showDeleteConfirm.value = false;
  } catch (e: any) {
    errorMsg.value = e.message || t("heroes.errorDelete");
  }
}

onMounted(async () => {
  await store.loadHeroPresets();
  // 深链编辑：编排页「编辑」按钮跳转 /heroes?edit=<name>，自动选中并填表
  const editName = route.query.edit;
  if (typeof editName === "string" && editName && heroPresets.value.some((p) => p.name === editName)) {
    selectPreset(editName);
  }
});
</script>

<template>
  <div class="bg-background flex h-full w-full flex-col overflow-hidden p-4">
    <!-- 顶部 Header -->
    <header
      class="border-border bg-card flex shrink-0 items-center justify-between rounded-lg border px-4 py-2.5 shadow-sm"
    >
      <div class="flex items-center gap-2.5">
        <Button variant="ghost" size="icon" class="size-7" @click="router.push('/')">
          <ArrowLeftIcon class="size-4" />
        </Button>
        <div class="bg-primary/10 flex size-8 items-center justify-center rounded-lg">
          <SwordsIcon class="text-primary size-4" />
        </div>
        <div class="flex items-baseline gap-2">
          <h1 class="text-foreground text-sm font-bold tracking-tight">{{ t("heroes.title") }}</h1>
          <Badge variant="secondary" class="text-[10px]">{{ heroPresets.length }}{{ t("heroes.countUnit") }}</Badge>
        </div>
      </div>
      <Button variant="outline" size="xs" class="h-7 gap-1 text-[11px]" @click="startNew">
        <PlusIcon class="size-3" />
        {{ t("heroes.newPreset") }}
      </Button>
    </header>

    <div class="mt-3 flex min-h-0 flex-1 gap-3">
      <!-- 左：预设列表 -->
      <aside class="border-border bg-card w-60 shrink-0 overflow-hidden rounded-lg border shadow-sm">
        <div class="border-border shrink-0 border-b px-3 py-2 text-[11px] font-bold tracking-wide uppercase">
          {{ t("heroes.listTitle") }}
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto p-2">
          <div v-if="heroPresets.length === 0" class="text-muted-foreground py-6 text-center text-xs italic">
            {{ t("heroes.emptyList") }}
          </div>
          <div v-else class="flex flex-col gap-1">
            <button
              v-for="p in heroPresets"
              :key="p.name"
              class="hover:bg-muted border-border flex items-center justify-between rounded border px-2 py-1.5 text-left transition-colors"
              :class="p.name === selectedName ? 'border-primary/40 bg-primary/10' : 'bg-muted/30'"
              @click="selectPreset(p.name)"
            >
              <span class="flex min-w-0 flex-col">
                <span class="text-foreground truncate text-xs font-medium">{{ p.name }}</span>
                <span class="text-muted-foreground truncate text-[10px]">{{ p.champion }} · {{ p.agent_type.toUpperCase() }}</span>
              </span>
            </button>
          </div>
        </div>
      </aside>

      <!-- 右：编辑表单 -->
      <section class="border-border bg-card min-w-0 flex-1 overflow-y-auto rounded-lg border p-5 shadow-sm">
        <div
          v-if="errorMsg"
          class="border-border mb-3 rounded border-l-2 px-3 py-1.5 text-xs"
          :class="
            errorMsg === t('heroes.successSave')
              ? 'border-green-500 bg-green-500/5 text-green-500'
              : 'border-destructive text-destructive bg-destructive/5'
          "
        >
          {{ errorMsg }}
        </div>

        <div class="mx-auto flex max-w-xl flex-col gap-4">
          <!-- 名称 + 英雄 -->
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-muted-foreground mb-1 block text-[10px] font-semibold tracking-wider uppercase">
                {{ t("heroes.presetName") }}
              </label>
              <Input
                v-model="draft.name"
                :placeholder="t('heroes.presetNamePlaceholder')"
                class="border-border bg-muted/40 h-9 text-sm"
              />
            </div>
            <div>
              <label class="text-muted-foreground mb-1 block text-[10px] font-semibold tracking-wider uppercase">
                {{ t("heroes.heroLabel") }}
              </label>
              <Select v-model="draft.champion">
                <SelectTrigger class="bg-muted/40 border-border h-9 text-sm">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent class="border-border bg-popover text-foreground">
                  <SelectItem v-for="c in champions" :key="c" :value="c" class="text-sm">
                    {{ c }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <!-- 决策类型 -->
          <div>
            <label class="text-muted-foreground mb-1 block text-[10px] font-semibold tracking-wider uppercase">
              {{ t("heroes.agentType") }}
            </label>
            <Select v-model="draft.agent_type">
              <SelectTrigger class="bg-muted/40 border-border h-9 text-sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent class="border-border bg-popover text-foreground">
                <SelectItem value="llm" class="text-sm">
                  {{ t("agents.types.llm") }}
                </SelectItem>
                <SelectItem value="rl" class="text-sm">
                  {{ t("agents.types.rl") }}
                </SelectItem>
                <SelectItem value="script" class="text-sm">
                  {{ t("agents.types.script") }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- 警告提示 (针对尚未完全集成的 RL / Script 类型) -->
          <div
            v-if="draft.agent_type !== 'llm'"
            class="border-yellow-500/20 bg-yellow-500/5 text-yellow-500 rounded border p-3 text-xs leading-normal"
            v-html="t('agents.typeWarning', { type: draft.agent_type.toUpperCase() })"
          ></div>

          <!-- 动态策略配置区域 -->
          <div v-if="draft.agent_type === 'llm'" class="flex flex-col gap-4">
            <!-- Prompt -->
            <div>
              <label class="text-muted-foreground mb-1 block text-[10px] font-semibold tracking-wider uppercase">
                {{ t("heroes.promptLabel") }}
              </label>
              <Textarea
                v-model="draft.prompt"
                :placeholder="t('heroes.promptPlaceholder')"
                class="border-border bg-muted/40 min-h-[120px] text-xs font-mono leading-relaxed"
              />
            </div>

            <!-- Model -->
            <div>
              <label class="text-muted-foreground mb-1 block text-[10px] font-semibold tracking-wider uppercase">
                {{ t("heroes.modelLabel") }}
              </label>
              <Input
                v-model="draft.model"
                :placeholder="t('heroes.modelPlaceholder')"
                class="border-border bg-muted/40 h-9 text-sm"
              />
            </div>

            <!-- Preamble -->
            <div>
              <label class="text-muted-foreground mb-1 block text-[10px] font-semibold tracking-wider uppercase">
                {{ t("heroes.preambleLabel") }}
              </label>
              <Textarea
                v-model="draft.preamble"
                :placeholder="t('heroes.preamblePlaceholder')"
                class="border-border bg-muted/40 min-h-[60px] text-xs font-mono leading-relaxed"
              />
            </div>
          </div>

          <div v-else class="flex flex-col gap-4">
            <!-- Config JSON -->
            <div>
              <label class="text-muted-foreground mb-1 block text-[10px] font-semibold tracking-wider uppercase">
                {{ t("heroes.configJsonLabel") }}
              </label>
              <Textarea
                v-model="configJsonStr"
                :placeholder="t('heroes.configJsonPlaceholder')"
                class="border-border bg-muted/40 min-h-[180px] text-xs font-mono leading-relaxed"
              />
            </div>
          </div>

          <!-- 操作 -->
          <div class="border-border mt-2 flex items-center gap-2 border-t pt-4">
            <Button class="gap-1.5" :disabled="!draft.name.trim()" @click="handleSave">
              <SaveIcon class="size-3.5" />
              {{ t("heroes.saveBtn") }}
            </Button>
            <Button
              v-if="selectedName"
              variant="outline"
              class="border-destructive/20 bg-destructive/5 text-destructive hover:bg-destructive hover:text-destructive-foreground gap-1.5"
              @click="showDeleteConfirm = true"
            >
              <Trash2Icon class="size-3.5" />
              {{ t("heroes.deleteBtn") }}
            </Button>
          </div>
        </div>
      </section>
    </div>

    <!-- 删除确认 -->
    <Dialog :open="showDeleteConfirm" @update:open="(v) => (showDeleteConfirm = v)">
      <DialogContent class="border-border bg-card text-foreground max-w-sm p-6">
        <DialogHeader>
          <DialogTitle class="text-foreground text-sm">
            {{ t("heroes.deleteConfirmTitle", { name: selectedName }) }}
          </DialogTitle>
          <DialogDescription class="text-muted-foreground text-[11px]">
            {{ t("heroes.deleteConfirmDesc") }}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter class="gap-2">
          <Button variant="outline" size="sm" @click="showDeleteConfirm = false">{{ t("heroes.cancelBtn") }}</Button>
          <Button variant="destructive" size="sm" @click="confirmDelete">{{ t("heroes.confirmDeleteBtn") }}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
