<script setup lang="ts">
import { computed } from "vue";
import { NumberField, NumberFieldContent, NumberFieldInput } from "@/components/ui/number-field";
import { Button } from "@/components/ui/button";
import { TrendingUpIcon, HashIcon } from "lucide-vue-next";
import VecInput from "./VecInput.vue";
import SamplerCurvePreview from "./SamplerCurvePreview.vue";

const props = defineProps<{
  modelValue?: any;
  type?: "number" | "vec2" | "vec3" | "vec4";
  label?: string;
  step?: number;
  isColor?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", val: any): void;
}>();

/** 判断 base_sampler 是 Constant 还是 Curve */
const isCurve = computed(() => {
  const bs = props.modelValue?.base_sampler;
  return Array.isArray(bs) && Array.isArray(bs[0]);
});

/** 节点采样数据数组 */
const curveSamples = computed(() => {
  if (isCurve.value) {
    return props.modelValue?.base_sampler || [];
  }
  return [];
});

/** 概率曲线列表 */
const probCurves = computed(() => {
  return props.modelValue?.prob_curves || [];
});

const numberValue = computed(() => {
  if (props.type === "number" && !isCurve.value) {
    let bs = props.modelValue?.base_sampler;
    if (typeof bs !== "number") return 0;
    return bs;
  }
  return 0;
});

const vectorValue = computed(() => {
  const bs = props.modelValue?.base_sampler;
  if (Array.isArray(bs) && typeof bs[0] === "number") {
    return bs;
  }
  if (props.type === "vec2") return [0, 0];
  if (props.type === "vec3") return [0, 0, 0];
  if (props.type === "vec4") return [0, 0, 0, 1];
  return [0, 0, 0];
});

function updateConstantNumber(val: number | undefined | null) {
  const num = val === undefined || val === null || isNaN(val) ? 0 : val;

  const current = props.modelValue
    ? JSON.parse(JSON.stringify(props.modelValue))
    : { base_sampler: num, prob_curves: [] };
  current.base_sampler = num;
  emit("update:modelValue", current);
}

function updateConstantVector(valVec: number[]) {
  const current = props.modelValue
    ? JSON.parse(JSON.stringify(props.modelValue))
    : { base_sampler: valVec, prob_curves: [] };
  current.base_sampler = valVec;
  emit("update:modelValue", current);
}

function updateCurveSamples(newSamples: any[]) {
  const current = props.modelValue
    ? JSON.parse(JSON.stringify(props.modelValue))
    : { base_sampler: newSamples, prob_curves: [] };
  current.base_sampler = newSamples;
  emit("update:modelValue", current);
}

function switchToCurve() {
  let initVal: any = 0;
  if (props.type === "number" || !props.type) {
    initVal = numberValue.value;
  } else {
    initVal = vectorValue.value;
  }

  // 初始化包含两个 keyframe 控制点的曲线 [[0, val], [1, val]]
  const defaultSamples = [
    [0, Array.isArray(initVal) ? [...initVal] : initVal],
    [1, Array.isArray(initVal) ? [...initVal] : initVal],
  ];

  updateCurveSamples(defaultSamples);
}

function switchToConstant() {
  let constVal: any = 0;
  if (curveSamples.value.length > 0) {
    const first = curveSamples.value[0];
    constVal = first[1];
  } else {
    if (props.type === "vec2") constVal = [0, 0];
    else if (props.type === "vec3") constVal = [0, 0, 0];
    else if (props.type === "vec4") constVal = [0, 0, 0, 1];
    else constVal = 0;
  }

  const current = props.modelValue
    ? JSON.parse(JSON.stringify(props.modelValue))
    : { base_sampler: constVal, prob_curves: [] };
  current.base_sampler = constVal;
  emit("update:modelValue", current);
}
</script>

<template>
  <div class="flex items-center gap-1.5 w-full">
    <!-- 常量模式 -->
    <template v-if="!isCurve">
      <div v-if="type === 'number' || !type" class="flex-1 min-w-0">
        <NumberField
          :model-value="numberValue"
          :step="step ?? 0.1"
          @update:model-value="updateConstantNumber"
        >
          <NumberFieldContent class="h-7">
            <NumberFieldInput class="h-7 text-xs px-2 font-mono" />
          </NumberFieldContent>
        </NumberField>
      </div>
      <div v-else class="flex-1 min-w-0">
        <VecInput
          :model-value="vectorValue"
          :dimensions="type === 'vec2' ? 2 : type === 'vec4' ? 4 : 3"
          :step="step"
          :is-color="isColor"
          @update:model-value="updateConstantVector"
        />
      </div>

      <!-- 切换为曲线模式按钮 -->
      <Button
        variant="ghost"
        size="icon"
        class="size-7 shrink-0 text-muted-foreground hover:text-primary hover:bg-accent/80 border border-border/40"
        :title="$t('particles.convertToCurve') || '转换为曲线'"
        @click="switchToCurve"
      >
        <TrendingUpIcon class="size-3.5" />
      </Button>
    </template>

    <!-- 曲线模式 -->
    <template v-else>
      <div class="flex-1 min-w-0">
        <SamplerCurvePreview
          :samples="curveSamples"
          :prob-curves="probCurves"
          :type="type || 'number'"
          :label="label"
          :is-color="isColor"
          @update:samples="updateCurveSamples"
        />
      </div>

      <!-- 切换为常量模式按钮 -->
      <Button
        variant="ghost"
        size="icon"
        class="size-7 shrink-0 text-muted-foreground hover:text-primary hover:bg-accent/80 border border-border/40"
        :title="$t('particles.convertToConstant') || '转换为常量'"
        @click="switchToConstant"
      >
        <HashIcon class="size-3.5" />
      </Button>
    </template>
  </div>
</template>
