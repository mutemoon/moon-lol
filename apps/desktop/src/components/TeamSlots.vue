<script setup lang="ts">
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import { PlusIcon } from "@lucide/vue";
import SlotCard from "./SlotCard.vue";
import type { Slot } from "../composables/useSlotConfig";
import type { HeroPreset, AgentPreset, SpawnPreset } from "../stores/gameStore";

// 阵营列：标题/点/徽标 + 槽位列表 + 新增槽位。红蓝复用同一组件，差异只在 accent 配置。

defineProps<{
  slots: Slot[];
  heroPresets: HeroPreset[];
  agentPresets: AgentPreset[];
  spawnPresets: SpawnPreset[];
  /** 阵营显示配置（标题、点、徽标、强调色）。 */
  team: {
    label: string;
    /** 圆点 + 头部文字色，如 "bg-blue-500" / "text-blue-500" */
    dot: string;
    titleText: string;
    /** 列容器边框/底色，如 "border-blue-500/20 bg-blue-500/5" */
    panel: string;
    /** 头部分隔线，如 "border-blue-500/10" */
    divider: string;
    /** 「槽位数」徽标，如 "border-blue-500/20 text-blue-500" */
    countBadge: string;
    /** 「＋ 槽位」按钮，如 "border-blue-500/30 text-blue-500 hover:bg-blue-500/10" */
    addButton: string;
  };
  /** 传给 SlotCard 的强调色（色相相关 class）。 */
  accent: {
    border: string;
    edit: string;
    indexText: string;
    inheritBadge: string;
  };
}>();

const emit = defineEmits<{
  add: [];
  remove: [index: number];
  saveAs: [slot: Slot];
}>();
</script>

<template>
  <div class="flex min-h-0 flex-col overflow-hidden rounded-lg border shadow-sm" :class="team.panel">
    <!-- 头部 -->
    <div
      class="flex shrink-0 items-center justify-between border-b px-3.5 py-2.5"
      :class="team.divider"
    >
      <div class="flex items-center gap-2">
        <span class="size-2.5 rounded-full" :class="team.dot"></span>
        <span class="text-xs font-bold tracking-wide uppercase" :class="team.titleText">{{ team.label }}</span>
        <Badge variant="outline" class="text-[9px]" :class="team.countBadge">
          {{ slots.length }}
        </Badge>
      </div>
      <Button variant="outline" size="xs" class="h-6 text-[10px]" :class="team.addButton" @click="emit('add')">
        <PlusIcon class="size-3" />
        槽位
      </Button>
    </div>
    <!-- 槽位列表 -->
    <div class="min-h-0 flex-1 overflow-y-auto p-2.5">
      <div class="flex flex-col gap-2">
        <SlotCard
          v-for="(slot, idx) in slots"
          :key="slot.id"
          :slot="slot"
          :index="idx"
          :hero-presets="heroPresets"
          :agent-presets="agentPresets"
          :spawn-presets="spawnPresets"
          :accent="accent"
          @remove="emit('remove', idx)"
          @save-as="emit('saveAs', slot)"
        />
      </div>
    </div>
  </div>
</template>
