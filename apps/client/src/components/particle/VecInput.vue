<script setup lang="ts">
import { computed } from "vue";
import { NumberField, NumberFieldContent, NumberFieldInput } from "@/components/ui/number-field";

const props = defineProps<{
  modelValue?: number[] | null;
  labels?: string[];
  dimensions?: number;
  step?: number;
  min?: number;
  max?: number;
  isColor?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", val: number[]): void;
}>();

const dim = computed(() => props.dimensions ?? props.modelValue?.length ?? 3);

const labelsList = computed(() => {
  if (props.labels) {
    return props.labels;
  }
  if (props.isColor) {
    return ["R", "G", "B", "A"].slice(0, dim.value);
  }
  const defaultLabels = ["X", "Y", "Z", "W"];
  return defaultLabels.slice(0, dim.value);
});

const numericValues = computed(() => {
  const vals = props.modelValue ?? [];
  const res: number[] = [];
  for (let i = 0; i < dim.value; i++) {
    res.push(vals[i] ?? 0);
  }
  return res;
});

const colorStyle = computed(() => {
  if (!props.isColor || !numericValues.value) return null;
  const vals = numericValues.value;
  const isNorm = vals.some((v) => v > 0 && v <= 1.0) && vals.every((v) => v <= 1.0);
  const factor = isNorm ? 255 : 1;

  const r = Math.min(255, Math.max(0, Math.round((vals[0] ?? 0) * factor)));
  const g = Math.min(255, Math.max(0, Math.round((vals[1] ?? 0) * factor)));
  const b = Math.min(255, Math.max(0, Math.round((vals[2] ?? 0) * factor)));

  let a = 1;
  if (vals.length >= 4) {
    const rawA = vals[3] ?? 1;
    a = isNorm ? rawA : rawA / 255;
    a = Math.min(1, Math.max(0, a));
  }
  return `rgba(${r}, ${g}, ${b}, ${a.toFixed(2)})`;
});

function onInput(index: number, val: number | undefined | null) {
  const num = val === undefined || val === null || isNaN(val) ? 0 : val;
  const newArr: number[] = [];
  for (let i = 0; i < dim.value; i++) {
    newArr.push(numericValues.value[i] ?? 0);
  }
  newArr[index] = num;
  emit("update:modelValue", newArr);
}
</script>

<template>
  <div class="flex items-center gap-1.5 w-full">
    <div
      v-for="(_, idx) in dim"
      :key="idx"
      class="flex flex-1 items-center gap-1 min-w-0"
    >
      <span class="text-[11px] font-medium text-muted-foreground uppercase shrink-0 font-mono">
        {{ labelsList[idx] || `V${idx}` }}:
      </span>
      <NumberField
        :model-value="numericValues[idx]"
        :step="step ?? 0.1"
        :min="min"
        :max="max"
        class="flex-1 min-w-0"
        @update:model-value="onInput(idx, $event)"
      >
        <NumberFieldContent class="h-7">
          <NumberFieldInput class="h-7 text-xs px-1.5 font-mono min-w-0" />
        </NumberFieldContent>
      </NumberField>
    </div>

    <!-- 常数颜色模式预览方块 -->
    <div
      v-if="isColor && colorStyle"
      class="size-6 rounded border border-border shrink-0 overflow-hidden shadow-xs checkerboard-bg"
      :title="`RGBA颜色: ${colorStyle}`"
    >
      <div class="w-full h-full" :style="{ backgroundColor: colorStyle }" />
    </div>
  </div>
</template>

<style scoped>
.checkerboard-bg {
  background-color: #ffffff;
  background-image: linear-gradient(45deg, #e5e7eb 25%, transparent 25%),
                    linear-gradient(-45deg, #e5e7eb 25%, transparent 25%),
                    linear-gradient(45deg, transparent 75%, #e5e7eb 75%),
                    linear-gradient(-45deg, transparent 75%, #e5e7eb 75%);
  background-size: 8px 8px;
  background-position: 0 0, 0 4px, 4px -4px, -4px 0px;
}
.dark .checkerboard-bg {
  background-color: #1f2937;
  background-image: linear-gradient(45deg, #374151 25%, transparent 25%),
                    linear-gradient(-45deg, #374151 25%, transparent 25%),
                    linear-gradient(45deg, transparent 75%, #374151 75%),
                    linear-gradient(-45deg, transparent 75%, #374151 75%);
}
</style>
