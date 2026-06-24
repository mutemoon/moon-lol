<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import { PencilIcon, Trash2Icon } from "@lucide/vue";
import PresetSelect from "./PresetSelect.vue";
import type { Slot } from "../composables/useSlotConfig";
import { bindHeroPreset, selectSpawn, slotSubtitle } from "../composables/useSlotConfig";
import type { HeroPreset, SpawnPreset } from "../stores/gameStore";
import { useLocale } from "../composables/useLocale";

const props = defineProps<{
  slot: Slot;
  index: number;
  heroPresets: HeroPreset[];
  spawnPresets: SpawnPreset[];
  /** 阵营强调色工具类集合 */
  accent: {
    /** 槽位边框 */
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
}>();

const router = useRouter();
const { t } = useLocale();

const subtitle = computed(() => slotSubtitle(props.slot, props.heroPresets));

function onHero(name: string) {
  bindHeroPreset(props.slot, name, props.heroPresets);
}
function onSpawn(name: string) {
  selectSpawn(props.slot, name);
}
function editHeroPreset() {
  if (!props.slot.heroPresetName) return;
  router.push({ path: "/heroes", query: { edit: props.slot.heroPresetName } });
}
</script>

<template>
  <div
    class="bg-background/50 rounded-md border p-2.5"
    :class="accent.border"
  >
    <div class="mb-2 flex items-center gap-1.5">
      <span class="text-[10px] font-semibold uppercase" :class="accent.indexText">#{{ index + 1 }}</span>
      <span class="text-muted-foreground text-[10px] font-semibold uppercase">{{ t('common.slotCard.slotLabel') }}</span>
      <Badge
        v-if="slot.champion"
        variant="outline"
        class="ml-auto text-[9px]"
        :class="accent.inheritBadge"
      >
        {{ t('common.slotCard.inheritBadge') }}
      </Badge>
      <button
        class="text-muted-foreground hover:text-destructive transition-colors"
        :title="t('common.slotCard.deleteSlot')"
        @click="emit('remove')"
      >
        <Trash2Icon class="size-3.5" />
      </button>
    </div>
    <!-- 我的选手 (选手预设) -->
    <div class="flex items-center justify-between gap-1.5">
      <span class="text-muted-foreground w-14 shrink-0 text-[10px] font-semibold uppercase">{{ t('common.slotCard.heroLabel') }}</span>
      <div class="flex items-center gap-1.5">
        <Button
          v-if="slot.heroPresetName"
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
    <!-- 独立的出生点选择 -->
    <template v-if="slot.champion">
      <div class="mt-2 grid grid-cols-1 gap-1.5">
        <div class="flex items-center justify-between gap-1.5">
          <span class="text-muted-foreground w-14 shrink-0 text-[10px] font-semibold uppercase">{{ t('common.slotCard.spawnLabel') }}</span>
          <PresetSelect
            :presets="spawnPresets"
            :model-value="slot.spawnPresetName"
            :placeholder="t('common.slotCard.selectSpawnPlaceholder')"
            :new-label="t('common.slotCard.newSpawnLabel')"
            trigger-class="h-7"
            @update:model-value="onSpawn"
            @new="router.push('/')"
          />
        </div>
      </div>
      <div class="text-muted-foreground mt-1.5 truncate font-mono text-[10px]">
        {{ subtitle }}
      </div>
    </template>
  </div>
</template>
