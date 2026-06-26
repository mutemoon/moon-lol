<script setup lang="ts">
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import { PlusIcon } from "@lucide/vue";
import { useLocale } from "../composables/useLocale";

// 阵营列容器：标题/点/徽标 + 自定义槽位列表 + 新增槽位。红蓝复用同一组件。

interface Props {
  count: number;
  label: string;
  color: "blue" | "red";
  showAdd?: boolean;
}

withDefaults(defineProps<Props>(), {
  showAdd: true,
});

const emit = defineEmits<{
  add: [];
}>();

const { t } = useLocale();
</script>

<template>
  <div class="bg-card flex min-h-0 flex-col overflow-hidden rounded-lg border border-border">
    <!-- 头部 -->
    <div class="border-border flex shrink-0 items-center justify-between border-b px-3.5 py-2.5">
      <div class="flex items-center gap-2">
        <span class="size-2.5 rounded-full" :class="color === 'blue' ? 'bg-blue-500' : 'bg-red-500'"></span>
        <span class="text-foreground text-xs font-bold tracking-wide uppercase">{{ label }}</span>
        <Badge variant="outline" class="border-border text-muted-foreground text-[9px]">
          {{ count }}
        </Badge>
      </div>
      <Button v-if="showAdd" variant="outline" size="xs" class="text-muted-foreground hover:bg-muted hover:text-foreground h-6 text-[10px]" @click="emit('add')" :data-testid="`add-slot-${color}`">
        <PlusIcon class="size-3" />
        {{ t("common.teamSlots.addSlotBtn") }}
      </Button>
    </div>
    <!-- 槽位列表 -->
    <div class="min-h-0 flex-1 overflow-y-auto p-2.5">
      <div class="flex flex-col gap-2">
        <slot />
      </div>
    </div>
  </div>
</template>
