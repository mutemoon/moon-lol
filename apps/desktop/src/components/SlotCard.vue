<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import { PencilIcon, CopyPlusIcon, Trash2Icon } from "@lucide/vue";
import PresetSelect from "./PresetSelect.vue";
import type { Slot } from "../composables/useSlotConfig";
import { bindHeroPreset, overrideAgent, overrideSpawn, slotSubtitle } from "../composables/useSlotConfig";
import type { HeroPreset, AgentPreset, SpawnPreset } from "../stores/gameStore";
import { useLocale } from "../composables/useLocale";

// 单个槽位卡片：英雄预设 + 始终展开的 Agent/出生点覆盖 + 编辑/存为新预设入口。
// 阵营差异仅体现在颜色 class 上，由 accentClass 注入，模板主体只此一份。

const props = defineProps<{
  slot: Slot;
  index: number;
  heroPresets: HeroPreset[];
  agentPresets: AgentPreset[];
  spawnPresets: SpawnPreset[];
  /** 阵营强调色工具类集合（border/text/bg 的色相差异，统一以 Tailwind class 传入）。 */
  accent: {
    /** 槽位边框（绑定态） */
    border: string;
    /** 编辑按钮 */
    edit: string;
    /** 头部 #序号 文字色 */
    indexText: string;
    /** 「继承」徽标 */
    inheritBadge: string;
  };
}>();

const emit = defineEmits<{
  remove: [];
  saveAs: [];
}>();

const router = useRouter();
const { t } = useLocale();

const subtitle = computed(() => slotSubtitle(props.slot, props.agentPresets));

function onHero(name: string) {
  bindHeroPreset(props.slot, name, props.heroPresets);
}
function onAgent(name: string) {
  overrideAgent(props.slot, name);
}
function onSpawn(name: string) {
  overrideSpawn(props.slot, name);
}
function editHeroPreset() {
  if (!props.slot.heroPresetName) return;
  router.push({ path: "/heroes", query: { edit: props.slot.heroPresetName } });
}
</script>

<template>
  <div
    class="bg-background/50 rounded-md border p-2.5"
    :class="slot.dirty ? 'border-amber-500/40 bg-amber-500/5' : accent.border"
  >
    <div class="mb-2 flex items-center gap-1.5">
      <span class="text-[10px] font-semibold uppercase" :class="accent.indexText">#{{ index + 1 }}</span>
      <span class="text-muted-foreground text-[10px] font-semibold uppercase">{{ t('common.slotCard.slotLabel') }}</span>
      <Badge
        v-if="slot.champion"
        variant="outline"
        class="ml-auto text-[9px]"
        :class="slot.dirty ? 'border-amber-500/40 text-amber-500' : accent.inheritBadge"
      >
        {{ slot.dirty ? t('common.slotCard.dirtyBadge') : t('common.slotCard.inheritBadge') }}
      </Badge>
      <button
        class="text-muted-foreground hover:text-destructive transition-colors"
        :title="t('common.slotCard.deleteSlot')"
        @click="emit('remove')"
      >
        <Trash2Icon class="size-3.5" />
      </button>
    </div>
    <!-- 英雄预设 -->
    <div class="flex items-center justify-between gap-1.5">
      <span class="text-muted-foreground w-14 shrink-0 text-[10px] font-semibold uppercase">{{ t('common.slotCard.heroLabel') }}</span>
      <div class="flex items-center gap-1.5">
        <Button
          v-if="slot.heroPresetName && !slot.dirty"
          variant="ghost"
          size="icon-xs"
          :class="accent.edit"
          :title="t('common.slotCard.editPreset')"
          @click="editHeroPreset"
        >
          <PencilIcon class="size-3" />
        </Button>
        <PresetSelect
          :presets="heroPresets"
          :model-value="slot.heroPresetName"
          :placeholder="t('common.slotCard.selectHeroPlaceholder')"
          :new-label="t('common.slotCard.newHeroLabel')"
          subtitle-key="champion"
          @update:model-value="onHero"
          @new="router.push('/heroes')"
        />
      </div>
    </div>
    <!-- 覆盖：Agent 预设 / 出生点（始终展开） -->
    <template v-if="slot.champion">
      <div class="mt-2 grid grid-cols-1 gap-1.5">
        <div class="flex items-center justify-between gap-1.5">
          <span class="text-muted-foreground w-14 shrink-0 text-[10px] font-semibold uppercase">{{ t('common.slotCard.brainLabel') }}</span>
          <PresetSelect
            :presets="agentPresets"
            :model-value="slot.agentPresetName"
            :placeholder="t('common.slotCard.selectAgentPlaceholder')"
            :new-label="t('common.slotCard.newAgentLabel')"
            subtitle-key="agent_type"
            trigger-class="h-7"
            @update:model-value="onAgent"
            @new="router.push('/agents')"
          />
        </div>
        <div class="flex items-center justify-between gap-1.5">
          <span class="text-muted-foreground w-14 shrink-0 text-[10px] font-semibold uppercase">{{ t('common.slotCard.spawnLabel') }}</span>
          <PresetSelect
            :presets="spawnPresets"
            :model-value="slot.spawnPresetName"
            :placeholder="t('common.slotCard.selectSpawnPlaceholder')"
            :new-label="t('common.slotCard.newSpawnLabel')"
            trigger-class="h-7"
            @update:model-value="onSpawn"
            @new="router.push('/spawn-presets')"
          />
        </div>
      </div>
      <div class="text-muted-foreground mt-1.5 truncate font-mono text-[10px]">
        {{ subtitle }}
      </div>
      <Button
        v-if="slot.dirty"
        variant="outline"
        size="xs"
        class="mt-2 h-6 gap-1 border-amber-500/40 text-[10px] text-amber-600 hover:bg-amber-500/10 dark:text-amber-500"
        @click="emit('saveAs')"
      >
        <CopyPlusIcon class="size-3" />
        {{ t('common.slotCard.saveAsBtn') }}
      </Button>
    </template>
  </div>
</template>
