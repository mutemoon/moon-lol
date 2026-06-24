<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { storeToRefs } from "pinia";
import { useGameStore, type HeroPreset } from "@/stores/gameStore";
import { useRouter, useRoute } from "vue-router";
import { useLocale } from "@/composables/useLocale";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import PresetSelect from "@/components/PresetSelect.vue";
import { SwordsIcon, PlusIcon, Trash2Icon, SaveIcon, ArrowLeftIcon } from "@lucide/vue";

// 英雄预设管理页（产品文档 §3.0）：编排页槽位选择的唯一单元。
// 每个英雄预设 = 具体英雄 + Agent 预设（大脑）+ 出生点预设（坐标）。
// 后两者通过 PresetSelect 下拉选择，下拉末尾带「＋ 新建」跳转对应管理页。

const store = useGameStore();
const { heroPresets, champions, agentPresets, spawnPresets } = storeToRefs(store);
const router = useRouter();
const route = useRoute();
const { t } = useLocale();

const selectedName = ref<string | null>(null);
const errorMsg = ref("");
const showDeleteConfirm = ref(false);

const emptyDraft = (): HeroPreset => ({
  name: "",
  champion: "Riven",
  agent_preset_name: "",
  spawn_preset_name: "",
});
const draft = ref<HeroPreset>(emptyDraft());

// 展示用：当前选中英雄预设绑定的 Agent/出生点副标题
const boundAgentSubtitle = computed(() => {
  const a = agentPresets.value.find((p) => p.name === draft.value.agent_preset_name);
  return a ? a.agent_type.toUpperCase() : "";
});
const boundSpawnSubtitle = computed(() => {
  const s = spawnPresets.value.find((p) => p.name === draft.value.spawn_preset_name);
  return s ? `${Math.round(s.x)},${Math.round(s.z)}` : "";
});

function selectPreset(name: string) {
  selectedName.value = name;
  const p = heroPresets.value.find((x) => x.name === name);
  if (p) draft.value = { ...p };
}

function startNew() {
  selectedName.value = null;
  draft.value = emptyDraft();
}

async function handleSave() {
  errorMsg.value = "";
  const name = draft.value.name.trim();
  if (!name) {
    errorMsg.value = t("heroes.errorFillName");
    return;
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
    showDeleteConfirm.value = false;
  } catch (e: any) {
    errorMsg.value = e.message || t("heroes.errorDelete");
  }
}

onMounted(async () => {
  await Promise.all([store.loadHeroPresets(), store.loadAgentPresets(), store.loadSpawnPresets()]);
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
                <span class="text-muted-foreground truncate text-[10px]">{{ p.champion }}</span>
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

          <!-- 绑定的 Agent 预设 -->
          <div>
            <label class="text-muted-foreground mb-1 block text-[10px] font-semibold tracking-wider uppercase">
              {{ t("heroes.bindAgent") }}
            </label>
            <PresetSelect
              :presets="agentPresets"
              :model-value="draft.agent_preset_name"
              :placeholder="t('heroes.selectAgentPlaceholder')"
              :new-label="t('heroes.newAgentLabel')"
              :subtitle-key="'agent_type'"
              @update:model-value="(v) => (draft.agent_preset_name = v)"
              @new="router.push('/agents')"
            />
          </div>

          <!-- 绑定的出生点预设 -->
          <div>
            <label class="text-muted-foreground mb-1 block text-[10px] font-semibold tracking-wider uppercase">
              {{ t("heroes.bindSpawn") }}
            </label>
            <PresetSelect
              :presets="spawnPresets"
              :model-value="draft.spawn_preset_name"
              :placeholder="t('heroes.selectSpawnPlaceholder')"
              :new-label="t('heroes.newSpawnLabel')"
              @update:model-value="(v) => (draft.spawn_preset_name = v)"
              @new="router.push('/spawn-presets')"
            />
          </div>

          <!-- 绑定摘要 -->
          <div
            v-if="draft.agent_preset_name || draft.spawn_preset_name"
            class="bg-muted/40 border-border rounded-md border px-3 py-2"
          >
            <div class="text-muted-foreground mb-1 text-[10px] font-semibold tracking-wider uppercase">
              {{ t("heroes.previewTitle") }}
            </div>
            <div
              class="text-foreground font-mono text-xs leading-relaxed"
              v-html="
                t('heroes.previewText', {
                  champion: draft.champion,
                  agent: draft.agent_preset_name || t('heroes.notSelected'),
                  agentSub: boundAgentSubtitle ? `[${boundAgentSubtitle}]` : '',
                  spawn: draft.spawn_preset_name || t('heroes.notSelected'),
                  spawnSub: boundSpawnSubtitle ? `[${boundSpawnSubtitle}]` : '',
                })
              "
            ></div>
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
