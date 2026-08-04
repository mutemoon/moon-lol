<script setup lang="ts">
import { computed } from "vue";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";
import { NumberField, NumberFieldContent, NumberFieldInput } from "@/components/ui/number-field";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import StochasticSamplerInput from "./StochasticSamplerInput.vue";
import { FlameIcon, PlusIcon, Trash2Icon } from "lucide-vue-next";

const props = defineProps<{
  modelValue?: any;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", val: any): void;
  (e: "change"): void;
}>();

const isEnabled = computed(() => {
  return props.modelValue !== undefined && props.modelValue !== null;
});

function enableDefinition() {
  const defaultDef = {
    erosion_map_name: "",
    erosion_drive_curve: { base_sampler: 0, prob_curves: [] },
    erosion_feather_in: 0,
    erosion_feather_out: 0,
    erosion_slice_width: 0,
    erosion_map_address_mode: 0,
    use_linger_erosion_drive_curve: false,
  };
  emit("update:modelValue", defaultDef);
  emit("change");
}

function disableDefinition() {
  emit("update:modelValue", null);
  emit("change");
}

function updateField(key: string, val: any) {
  if (!props.modelValue) return;
  const current = JSON.parse(JSON.stringify(props.modelValue));
  current[key] = val;
  emit("update:modelValue", current);
  emit("change");
}

function onNumberInput(key: string, val: number | undefined | null) {
  const num = val === undefined || val === null || isNaN(val) ? 0 : val;
  updateField(key, num);
}
</script>

<template>
  <div class="rounded-lg border bg-card/60 overflow-hidden transition-all duration-200">
    <!-- Header / 卡片头部 -->
    <div class="flex items-center justify-between px-3 py-2 bg-muted/40 border-b">
      <div class="flex items-center gap-2">
        <FlameIcon class="size-4 text-orange-500" />
        <span class="text-xs font-semibold text-foreground">
          {{ $t('particles.alphaErosionTitle') }}
        </span>
        <span
          class="text-[10px] px-1.5 py-0.5 rounded font-mono font-medium"
          :class="isEnabled ? 'bg-orange-500/10 text-orange-600 dark:text-orange-400 border border-orange-500/20' : 'bg-muted text-muted-foreground'"
        >
          {{ isEnabled ? 'Active' : 'Disabled' }}
        </span>
      </div>

      <div class="flex items-center gap-2">
        <Button
          v-if="!isEnabled"
          variant="outline"
          size="sm"
          class="h-6 px-2 text-[11px] gap-1 text-primary border-primary/30 hover:bg-primary/10"
          @click="enableDefinition"
        >
          <PlusIcon class="size-3" />
          {{ $t('particles.enableAlphaErosion') }}
        </Button>
        <Button
          v-else
          variant="ghost"
          size="sm"
          class="h-6 px-2 text-[11px] gap-1 text-destructive hover:bg-destructive/10 hover:text-destructive"
          :title="$t('particles.disableAlphaErosion')"
          @click="disableDefinition"
        >
          <Trash2Icon class="size-3" />
          {{ $t('particles.disableAlphaErosion') }}
        </Button>
      </div>
    </div>

    <!-- Panel Content / 卡片内容区 -->
    <div v-if="isEnabled" class="p-3 space-y-3.5 text-xs">
      <!-- 侵蚀贴图路径 -->
      <div class="space-y-1">
        <Label class="text-[11px] font-medium text-muted-foreground">
          {{ $t('particles.erosionMapName') }}
        </Label>
        <Input
          :model-value="modelValue?.erosion_map_name ?? ''"
          class="h-7 text-xs font-mono"
          placeholder="ASSETS/Textures/particles/erosion.dds"
          @update:model-value="updateField('erosion_map_name', String($event))"
        />
      </div>

      <!-- 侵蚀驱动曲线 -->
      <div class="space-y-1.5 p-2 rounded border bg-muted/20">
        <Label class="text-[11px] font-semibold text-foreground">
          {{ $t('particles.erosionDriveCurve') }}
        </Label>
        <StochasticSamplerInput
          :model-value="modelValue?.erosion_drive_curve"
          type="number"
          :step="0.05"
          @update:model-value="updateField('erosion_drive_curve', $event)"
        />
      </div>

      <!-- 数字控制字段 (Feather In / Feather Out / Slice Width) -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
        <div class="space-y-1">
          <Label class="text-[11px] text-muted-foreground">
            {{ $t('particles.erosionFeatherIn') }}
          </Label>
          <NumberField
            :model-value="modelValue?.erosion_feather_in ?? 0"
            :step="0.05"
            :min="0"
            :max="1"
            @update:model-value="onNumberInput('erosion_feather_in', $event)"
          >
            <NumberFieldContent class="h-7">
              <NumberFieldInput class="h-7 text-xs font-mono" />
            </NumberFieldContent>
          </NumberField>
        </div>

        <div class="space-y-1">
          <Label class="text-[11px] text-muted-foreground">
            {{ $t('particles.erosionFeatherOut') }}
          </Label>
          <NumberField
            :model-value="modelValue?.erosion_feather_out ?? 0"
            :step="0.05"
            :min="0"
            :max="1"
            @update:model-value="onNumberInput('erosion_feather_out', $event)"
          >
            <NumberFieldContent class="h-7">
              <NumberFieldInput class="h-7 text-xs font-mono" />
            </NumberFieldContent>
          </NumberField>
        </div>

        <div class="space-y-1">
          <Label class="text-[11px] text-muted-foreground">
            {{ $t('particles.erosionSliceWidth') }}
          </Label>
          <NumberField
            :model-value="modelValue?.erosion_slice_width ?? 0"
            :step="0.05"
            :min="0"
            :max="1"
            @update:model-value="onNumberInput('erosion_slice_width', $event)"
          >
            <NumberFieldContent class="h-7">
              <NumberFieldInput class="h-7 text-xs font-mono" />
            </NumberFieldContent>
          </NumberField>
        </div>
      </div>

      <!-- 寻址模式选择 -->
      <div class="space-y-1">
        <Label class="text-[11px] text-muted-foreground">
          {{ $t('particles.erosionMapAddressMode') }}
        </Label>
        <Select
          :model-value="String(modelValue?.erosion_map_address_mode ?? 0)"
          @update:model-value="updateField('erosion_map_address_mode', Number($event))"
        >
          <SelectTrigger class="h-7 text-xs font-mono">
            <SelectValue placeholder="Address Mode" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="0">0 - {{ $t('particles.addressModeWrap') }}</SelectItem>
            <SelectItem value="1">1 - {{ $t('particles.addressModeClamp') }}</SelectItem>
            <SelectItem value="2">2 - {{ $t('particles.addressModeMirror') }}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <!-- 残余 (Linger) 侵蚀控制 -->
      <div class="space-y-2 pt-1 border-t">
        <div class="flex items-center gap-2">
          <Checkbox
            id="chk_use_linger_erosion"
            :model-value="!!modelValue?.use_linger_erosion_drive_curve"
            @update:model-value="updateField('use_linger_erosion_drive_curve', Boolean($event))"
          />
          <Label for="chk_use_linger_erosion" class="text-xs cursor-pointer">
            {{ $t('particles.useLingerErosionDriveCurve') }}
          </Label>
        </div>

        <div
          v-if="modelValue?.use_linger_erosion_drive_curve"
          class="space-y-1.5 p-2 rounded border bg-muted/20"
        >
          <Label class="text-[11px] font-semibold text-foreground">
            {{ $t('particles.lingerErosionDriveCurve') }}
          </Label>
          <StochasticSamplerInput
            :model-value="modelValue?.linger_erosion_drive_curve"
            type="number"
            :step="0.05"
            @update:model-value="updateField('linger_erosion_drive_curve', $event)"
          />
        </div>
      </div>
    </div>

    <!-- 未启用时的简约说明背景 -->
    <div v-else class="p-3 text-center text-xs text-muted-foreground/70 bg-muted/10">
      Alpha 消融切割通过侵蚀纹理驱动粒子 Alpha 值的渐变消融效果。
    </div>
  </div>
</template>
