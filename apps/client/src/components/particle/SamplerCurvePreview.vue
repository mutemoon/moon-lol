<script setup lang="ts">
import { computed, ref, onUnmounted } from "vue";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { NumberField, NumberFieldContent, NumberFieldInput } from "@/components/ui/number-field";
import {
  TrendingUpIcon,
  PaletteIcon,
  PlusIcon,
  Trash2Icon,
} from "lucide-vue-next";

const props = defineProps<{
  /** 节点采样数组 `[[t, val], ...]` */
  samples: any[];
  /** 概率/随机辅助曲线列表 */
  probCurves?: any[];
  /** 维度类型 */
  type?: "number" | "vec2" | "vec3" | "vec4";
  label?: string;
  /** 是否是颜色采样器 */
  isColor?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:samples", val: any[]): void;
}>();

// 向量线段颜色
const COMPONENT_COLORS = ["#ef4444", "#22c55e", "#3b82f6", "#a855f7"];
const COMPONENT_LABELS = ["X", "Y", "Z", "W"];
const COLOR_LABELS = ["R", "G", "B", "A"];

// SVG 渲染常量
const SVG_WIDTH = 320;
const SVG_HEIGHT = 130;
const PAD_X = 14;
const PAD_Y = 14;
const GRAPH_WIDTH = SVG_WIDTH - PAD_X * 2;
const GRAPH_HEIGHT = SVG_HEIGHT - PAD_Y * 2;

// 交互状态
const svgRef = ref<SVGSVGElement | null>(null);
const isDragging = ref(false);
const dragSampleIdx = ref<number | null>(null);
const dragCompIdx = ref<number | null>(null);
const hoveredSampleIdx = ref<number | null>(null);

/** 标准化解析节点: Array<{ t: number, values: number[] }> */
const parsedSamples = computed(() => {
  if (!Array.isArray(props.samples) || props.samples.length === 0) return [];
  return props.samples.map((item) => {
    if (!Array.isArray(item) || item.length < 2) return { t: 0, values: [0] };
    const t = Number(item[0]) || 0;
    const rawVal = item[1];
    let values: number[] = [];
    if (Array.isArray(rawVal)) {
      values = rawVal.map((v) => Number(v) || 0);
    } else {
      values = [Number(rawVal) || 0];
    }
    return { t, values };
  });
});

function cloneParsedSamples(): Array<{ t: number; values: number[] }> {
  return JSON.parse(JSON.stringify(parsedSamples.value));
}

/** 确定维度数量 */
const numComponents = computed(() => {
  if (parsedSamples.value.length === 0) return 1;
  return parsedSamples.value[0]?.values.length ?? 1;
});

/** 转换颜色值辅助方法 */
function parseColor(values: number[]) {
  if (!values || values.length === 0) {
    return { r: 0, g: 0, b: 0, a: 1, rgbaStr: "rgba(0,0,0,1)", hexStr: "#000000" };
  }
  const isNorm = values.some((v) => v > 0 && v <= 1.0) && values.every((v) => v <= 1.0);
  const factor = isNorm ? 255 : 1;

  const r = Math.min(255, Math.max(0, Math.round((values[0] ?? 0) * factor)));
  const g = Math.min(255, Math.max(0, Math.round((values[1] ?? 0) * factor)));
  const b = Math.min(255, Math.max(0, Math.round((values[2] ?? 0) * factor)));

  let a = 1;
  if (values.length >= 4) {
    const rawA = values[3] ?? 1;
    a = isNorm ? rawA : rawA / 255;
    a = Math.min(1, Math.max(0, a));
  }

  const rgbaStr = `rgba(${r}, ${g}, ${b}, ${a.toFixed(2)})`;
  const hexR = r.toString(16).padStart(2, "0");
  const hexG = g.toString(16).padStart(2, "0");
  const hexB = b.toString(16).padStart(2, "0");
  const hexStr = `#${hexR}${hexG}${hexB}`.toUpperCase();

  return { r, g, b, a, rgbaStr, hexStr };
}

/** 计算颜色渐变的 CSS linear-gradient */
const cssGradient = computed(() => {
  if (!props.isColor || parsedSamples.value.length === 0) return "";
  const { min: tMin, max: tMax } = rangeT.value;
  const stops = parsedSamples.value.map((s) => {
    const pct = (((s.t - tMin) / (tMax - tMin)) * 100).toFixed(1);
    const col = parseColor(s.values);
    return `${col.rgbaStr} ${pct}%`;
  });
  return `linear-gradient(to right, ${stops.join(", ")})`;
});

/** 计算全局 Min / Max 用于 SVG Y轴归一化 */
const rangeY = computed(() => {
  if (parsedSamples.value.length === 0) return { min: 0, max: 1 };
  let min = Infinity;
  let max = -Infinity;
  for (const s of parsedSamples.value) {
    for (const v of s.values) {
      if (v < min) min = v;
      if (v > max) max = v;
    }
  }
  if (min === Infinity || max === -Infinity || min === max) {
    min = min === Infinity ? 0 : min - 1;
    max = max === -Infinity ? 1 : max + 1;
  }
  const padding = (max - min) * 0.1 || 0.1;
  return { min: min - padding, max: max + padding };
});

/** 计算全局 tMin / tMax */
const rangeT = computed(() => {
  if (parsedSamples.value.length === 0) return { min: 0, max: 1 };
  const first = parsedSamples.value[0];
  const last = parsedSamples.value[parsedSamples.value.length - 1];
  const min = first?.t ?? 0;
  let max = last?.t ?? 1;
  if (min === max) max = min + 1;
  return { min, max };
});

/** 获取某个点的 SVG 坐标 */
function getPointCoords(s: { t: number; values: number[] }, compIdx: number, width = SVG_WIDTH, height = SVG_HEIGHT) {
  const { min: yMin, max: yMax } = rangeY.value;
  const { min: tMin, max: tMax } = rangeT.value;
  const padX = (PAD_X / SVG_WIDTH) * width;
  const padY = (PAD_Y / SVG_HEIGHT) * height;
  const graphW = width - padX * 2;
  const graphH = height - padY * 2;

  const tNorm = (tMax - tMin) === 0 ? 0 : (s.t - tMin) / (tMax - tMin);
  const x = padX + Math.max(0, Math.min(1, tNorm)) * graphW;

  const val = s.values[compIdx] ?? 0;
  const valNorm = (yMax - yMin) === 0 ? 0.5 : (val - yMin) / (yMax - yMin);
  const y = padY + graphH - Math.max(0, Math.min(1, valNorm)) * graphH;
  return { x, y };
}

/** 生成指定组件 (compIdx) 的 SVG 折线 Path */
function generatePathD(compIdx: number, width = SVG_WIDTH, height = SVG_HEIGHT): string {
  const samples = parsedSamples.value;
  if (samples.length === 0) return "";
  const points = samples.map((s) => {
    const { x, y } = getPointCoords(s, compIdx, width, height);
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  });
  return `M ${points.join(" L ")}`;
}

/** 生成多边形闭合 Fill Path */
function generateAreaPathD(compIdx: number, width = SVG_WIDTH, height = SVG_HEIGHT): string {
  const lineD = generatePathD(compIdx, width, height);
  if (!lineD) return "";
  const { min: yMin, max: yMax } = rangeY.value;
  const padY = (PAD_Y / SVG_HEIGHT) * height;
  const graphH = height - padY * 2;
  const zeroNorm = (yMax - yMin) === 0 ? 0.5 : (0 - yMin) / (yMax - yMin);
  const zeroY = padY + graphH - Math.max(0, Math.min(1, zeroNorm)) * graphH;
  const padX = (PAD_X / SVG_WIDTH) * width;
  const graphW = width - padX * 2;
  return `${lineD} L ${(padX + graphW).toFixed(1)},${zeroY.toFixed(1)} L ${padX.toFixed(1)},${zeroY.toFixed(1)} Z`;
}

/** 发送更改数据给父级 */
function emitSamplesUpdate(list: Array<{ t: number; values: number[] }>) {
  const sorted = [...list].sort((a, b) => a.t - b.t);
  const raw = sorted.map((s) => {
    if (props.type === "number" || (s.values.length === 1 && props.type !== "vec2" && props.type !== "vec3" && props.type !== "vec4")) {
      return [s.t, s.values[0] ?? 0];
    }
    return [s.t, [...s.values]];
  });
  emit("update:samples", raw);
}

// ---------------------------------------------------------------------------
// SVG 交互拖拽 & 双击新增
// ---------------------------------------------------------------------------

function startDrag(sIdx: number, cIdx: number, e: MouseEvent) {
  e.stopPropagation();
  e.preventDefault();
  isDragging.value = true;
  dragSampleIdx.value = sIdx;
  dragCompIdx.value = cIdx;
  hoveredSampleIdx.value = sIdx;
  window.addEventListener("mousemove", onDragMove);
  window.addEventListener("mouseup", stopDrag);
}

function onDragMove(e: MouseEvent) {
  if (!isDragging.value || dragSampleIdx.value === null || dragCompIdx.value === null || !svgRef.value) return;
  const rect = svgRef.value.getBoundingClientRect();
  const scaleX = rect.width > 0 ? SVG_WIDTH / rect.width : 1;
  const scaleY = rect.height > 0 ? SVG_HEIGHT / rect.height : 1;
  const svgMouseX = (e.clientX - rect.left) * scaleX;
  const svgMouseY = (e.clientY - rect.top) * scaleY;

  const { min: yMin, max: yMax } = rangeY.value;
  const { min: tMin, max: tMax } = rangeT.value;

  const tNorm = Math.max(0, Math.min(1, (svgMouseX - PAD_X) / GRAPH_WIDTH));
  const newT = Number((tMin + tNorm * (tMax - tMin)).toFixed(3));

  const valNorm = Math.max(0, Math.min(1, (PAD_Y + GRAPH_HEIGHT - svgMouseY) / GRAPH_HEIGHT));
  const newVal = Number((yMin + valNorm * (yMax - yMin)).toFixed(3));

  const samplesCopy = cloneParsedSamples();
  const target = samplesCopy[dragSampleIdx.value];
  if (target) {
    target.t = newT;
    target.values[dragCompIdx.value] = newVal;
    emitSamplesUpdate(samplesCopy);
  }
}

function stopDrag() {
  if (isDragging.value) {
    isDragging.value = false;
    dragSampleIdx.value = null;
    dragCompIdx.value = null;
    window.removeEventListener("mousemove", onDragMove);
    window.removeEventListener("mouseup", stopDrag);
  }
}

function onSvgDblClick(e: MouseEvent) {
  if (!svgRef.value) return;
  const rect = svgRef.value.getBoundingClientRect();
  const scaleX = rect.width > 0 ? SVG_WIDTH / rect.width : 1;
  const scaleY = rect.height > 0 ? SVG_HEIGHT / rect.height : 1;
  const svgMouseX = (e.clientX - rect.left) * scaleX;
  const svgMouseY = (e.clientY - rect.top) * scaleY;

  const { min: yMin, max: yMax } = rangeY.value;
  const { min: tMin, max: tMax } = rangeT.value;

  const tNorm = Math.max(0, Math.min(1, (svgMouseX - PAD_X) / GRAPH_WIDTH));
  const newT = Number((tMin + tNorm * (tMax - tMin)).toFixed(3));

  const valNorm = Math.max(0, Math.min(1, (PAD_Y + GRAPH_HEIGHT - svgMouseY) / GRAPH_HEIGHT));
  const newVal = Number((yMin + valNorm * (yMax - yMin)).toFixed(3));

  const samplesCopy = cloneParsedSamples();
  const newValues = Array.from({ length: numComponents.value }, () => newVal);
  samplesCopy.push({ t: newT, values: newValues });
  emitSamplesUpdate(samplesCopy);
}

onUnmounted(() => {
  stopDrag();
});

// ---------------------------------------------------------------------------
// 表单与预设操作
// ---------------------------------------------------------------------------

function updateSampleT(sIdx: number, val: number | undefined | null) {
  const t = val === undefined || val === null || isNaN(val) ? 0 : Math.max(0, Math.min(1, val));
  const samplesCopy = cloneParsedSamples();
  if (samplesCopy[sIdx]) {
    samplesCopy[sIdx].t = t;
    emitSamplesUpdate(samplesCopy);
  }
}

function updateSampleVal(sIdx: number, cIdx: number, val: number | undefined | null) {
  const num = val === undefined || val === null || isNaN(val) ? 0 : val;
  const samplesCopy = cloneParsedSamples();
  if (samplesCopy[sIdx]) {
    samplesCopy[sIdx].values[cIdx] = num;
    emitSamplesUpdate(samplesCopy);
  }
}

function updateColorHex(sIdx: number, e: Event) {
  const hex = (e.target as HTMLInputElement).value;
  const samplesCopy = cloneParsedSamples();
  const target = samplesCopy[sIdx];
  if (!target) return;

  const rHex = parseInt(hex.substring(1, 3), 16) || 0;
  const gHex = parseInt(hex.substring(3, 5), 16) || 0;
  const bHex = parseInt(hex.substring(5, 7), 16) || 0;

  const isNorm = target.values.some((v) => v > 0 && v <= 1.0) && target.values.every((v) => v <= 1.0);
  const factor = isNorm ? 1 / 255 : 1;

  target.values[0] = Number((rHex * factor).toFixed(3));
  target.values[1] = Number((gHex * factor).toFixed(3));
  target.values[2] = Number((bHex * factor).toFixed(3));
  emitSamplesUpdate(samplesCopy);
}

function addKeyframe() {
  const samplesCopy = cloneParsedSamples();
  let newT = 0.5;
  if (samplesCopy.length > 0) {
    let maxGap = 0;
    let insertAt = 0;
    for (let i = 0; i < samplesCopy.length - 1; i++) {
      const sNext = samplesCopy[i + 1];
      const sCurr = samplesCopy[i];
      if (sNext && sCurr) {
        const gap = sNext.t - sCurr.t;
        if (gap > maxGap) {
          maxGap = gap;
          insertAt = i;
        }
      }
    }
    const sTarget1 = samplesCopy[insertAt];
    const sTarget2 = samplesCopy[insertAt + 1];
    if (maxGap > 0 && sTarget1 && sTarget2) {
      newT = Number(((sTarget1.t + sTarget2.t) / 2).toFixed(3));
    }
  }
  const defaultVals = Array.from({ length: numComponents.value }, () => 1.0);
  samplesCopy.push({ t: newT, values: defaultVals });
  emitSamplesUpdate(samplesCopy);
}

function removeKeyframe(sIdx: number) {
  const samplesCopy = cloneParsedSamples();
  if (samplesCopy.length <= 1) return;
  samplesCopy.splice(sIdx, 1);
  emitSamplesUpdate(samplesCopy);
}

function applyPreset(presetName: "flat" | "linearUp" | "linearDown" | "ease" | "pulse") {
  const { min: yMin, max: yMax } = rangeY.value;
  const midY = (yMin + yMax) / 2;
  const dim = numComponents.value;

  const makeVal = (v: number) => Array.from({ length: dim }, () => v);

  let newSamples: Array<{ t: number; values: number[] }> = [];
  if (presetName === "flat") {
    newSamples = [
      { t: 0, values: makeVal(midY) },
      { t: 1, values: makeVal(midY) },
    ];
  } else if (presetName === "linearUp") {
    newSamples = [
      { t: 0, values: makeVal(yMin) },
      { t: 1, values: makeVal(yMax) },
    ];
  } else if (presetName === "linearDown") {
    newSamples = [
      { t: 0, values: makeVal(yMax) },
      { t: 1, values: makeVal(yMin) },
    ];
  } else if (presetName === "ease") {
    newSamples = [
      { t: 0, values: makeVal(yMin) },
      { t: 0.25, values: makeVal(yMin + (yMax - yMin) * 0.1) },
      { t: 0.75, values: makeVal(yMin + (yMax - yMin) * 0.9) },
      { t: 1, values: makeVal(yMax) },
    ];
  } else if (presetName === "pulse") {
    newSamples = [
      { t: 0, values: makeVal(yMin) },
      { t: 0.5, values: makeVal(yMax) },
      { t: 1, values: makeVal(yMin) },
    ];
  }
  emitSamplesUpdate(newSamples);
}
</script>

<template>
  <Popover>
    <PopoverTrigger as-child>
      <button
        type="button"
        class="flex items-center gap-1.5 px-2 py-1 h-7 rounded border border-border bg-muted/40 hover:bg-accent/60 transition-colors cursor-pointer text-xs font-mono group w-full"
      >
        <PaletteIcon v-if="isColor" class="size-3.5 text-primary shrink-0" />
        <TrendingUpIcon v-else class="size-3.5 text-primary shrink-0" />
        
        <!-- 迷你 CSS 颜色渐变条 (若是 Color 模式) -->
        <div
          v-if="isColor"
          class="relative flex-1 h-5 overflow-hidden rounded border border-border/50 checkerboard-bg"
        >
          <div
            class="w-full h-full"
            :style="{ background: cssGradient }"
          />
        </div>

        <!-- 迷你 SVG 走势图 (若非 Color 模式) -->
        <div v-else class="relative flex-1 h-5 overflow-hidden rounded bg-background/60 border border-border/50 px-0.5">
          <svg class="w-full h-full" viewBox="0 0 100 24" preserveAspectRatio="none">
            <line x1="0" y1="12" x2="100" y2="12" stroke="currentColor" stroke-dasharray="2 2" class="text-border/40" stroke-width="0.5" />
            
            <path
              v-if="numComponents === 1"
              :d="generateAreaPathD(0, 100, 24)"
              fill="currentColor"
              class="text-primary/15"
            />
            
            <path
              v-for="cIdx in numComponents"
              :key="cIdx"
              :d="generatePathD(cIdx - 1, 100, 24)"
              fill="none"
              :stroke="numComponents === 1 ? 'var(--primary, #3b82f6)' : COMPONENT_COLORS[(cIdx - 1) % COMPONENT_COLORS.length]"
              stroke-width="1.2"
              stroke-linejoin="round"
            />

            <g v-for="cIdx in numComponents" :key="`pts_${cIdx}`">
              <circle
                v-for="(s, sIdx) in parsedSamples"
                :key="sIdx"
                :cx="getPointCoords(s, cIdx - 1, 100, 24).x"
                :cy="getPointCoords(s, cIdx - 1, 100, 24).y"
                r="1.5"
                :fill="numComponents === 1 ? 'var(--primary, #3b82f6)' : COMPONENT_COLORS[(cIdx - 1) % COMPONENT_COLORS.length]"
              />
            </g>
          </svg>
        </div>

        <span class="text-[10px] text-muted-foreground shrink-0 font-medium group-hover:text-foreground">
          {{ parsedSamples.length }} 节点
        </span>
      </button>
    </PopoverTrigger>

    <PopoverContent class="w-[410px] p-3 space-y-3 shadow-xl" align="start">
      <!-- 1. 头部标题 & 快捷预设按钮 -->
      <div class="space-y-2 border-b pb-2">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-1.5 text-xs font-semibold text-foreground">
            <PaletteIcon v-if="isColor" class="size-4 text-primary" />
            <TrendingUpIcon v-else class="size-4 text-primary" />
            <span>{{ label || (isColor ? $t('particles.curveEditor') || '采样曲线编辑器' : $t('particles.curveEditor') || '采样曲线编辑器') }}</span>
          </div>
          <Badge variant="outline" class="text-[10px] px-1.5 py-0 font-mono">
            {{ parsedSamples.length }} {{ $t('particles.keyframePoints') || '控制节点' }}
          </Badge>
        </div>

        <!-- 曲线预设按钮栏 -->
        <div class="flex items-center gap-1 overflow-x-auto text-[10px] font-medium pt-0.5">
          <span class="text-muted-foreground text-[10px] shrink-0 font-sans">预设:</span>
          <button
            type="button"
            class="px-1.5 py-0.5 rounded border border-border bg-muted/30 hover:bg-accent text-foreground transition-colors shrink-0"
            @click="applyPreset('flat')"
          >
            {{ $t('particles.presetFlat') || '平坦' }}
          </button>
          <button
            type="button"
            class="px-1.5 py-0.5 rounded border border-border bg-muted/30 hover:bg-accent text-foreground transition-colors shrink-0"
            @click="applyPreset('linearUp')"
          >
            {{ $t('particles.presetLinearUp') || '线性上升' }}
          </button>
          <button
            type="button"
            class="px-1.5 py-0.5 rounded border border-border bg-muted/30 hover:bg-accent text-foreground transition-colors shrink-0"
            @click="applyPreset('linearDown')"
          >
            {{ $t('particles.presetLinearDown') || '线性下降' }}
          </button>
          <button
            type="button"
            class="px-1.5 py-0.5 rounded border border-border bg-muted/30 hover:bg-accent text-foreground transition-colors shrink-0"
            @click="applyPreset('ease')"
          >
            {{ $t('particles.presetEase') || '缓入缓出' }}
          </button>
          <button
            type="button"
            class="px-1.5 py-0.5 rounded border border-border bg-muted/30 hover:bg-accent text-foreground transition-colors shrink-0"
            @click="applyPreset('pulse')"
          >
            {{ $t('particles.presetPulse') || '脉冲' }}
          </button>
        </div>
      </div>

      <!-- 2. 可交互 SVG 图表 / 颜色渐变栏 -->
      <div class="space-y-1.5">
        <!-- 颜色模式下的全宽渐变预览 -->
        <div v-if="isColor" class="space-y-1">
          <div class="relative w-full h-7 rounded border shadow-inner checkerboard-bg overflow-hidden">
            <div class="w-full h-full" :style="{ background: cssGradient }" />
          </div>
        </div>

        <!-- 交互式 SVG 图表 -->
        <div class="relative w-full h-34 bg-muted/20 border rounded p-1 overflow-hidden select-none">
          <!-- 刻度线 -->
          <div class="absolute top-1 right-2 text-[9px] font-mono text-muted-foreground/70 pointer-events-none">
            Max: {{ rangeY.max.toFixed(2) }}
          </div>
          <div class="absolute bottom-1 right-2 text-[9px] font-mono text-muted-foreground/70 pointer-events-none">
            Min: {{ rangeY.min.toFixed(2) }}
          </div>
          <div class="absolute bottom-1 left-2 text-[9px] font-mono text-muted-foreground/70 pointer-events-none">
            t = {{ rangeT.min.toFixed(1) }}
          </div>
          <div class="absolute bottom-1 left-1/2 -translate-x-1/2 text-[9px] font-mono text-muted-foreground/70 pointer-events-none">
            t = {{ ((rangeT.min + rangeT.max) / 2).toFixed(1) }}
          </div>
          <div class="absolute top-1 left-2 text-[9px] font-mono text-muted-foreground/50 pointer-events-none">
            {{ $t('particles.dragTip') || '拖拽节点调整数值 / 双击空白处新增节点' }}
          </div>

          <svg
            ref="svgRef"
            class="w-full h-full cursor-crosshair"
            viewBox="0 0 320 130"
            preserveAspectRatio="none"
            @dblclick="onSvgDblClick"
          >
            <!-- 背景网格 -->
            <line x1="14" y1="36" x2="306" y2="36" stroke="currentColor" stroke-dasharray="2 2" class="text-border/30" stroke-width="0.5" />
            <line x1="14" y1="65" x2="306" y2="65" stroke="currentColor" stroke-dasharray="2 2" class="text-border/40" stroke-width="0.5" />
            <line x1="14" y1="94" x2="306" y2="94" stroke="currentColor" stroke-dasharray="2 2" class="text-border/30" stroke-width="0.5" />
            <line x1="160" y1="14" x2="160" y2="116" stroke="currentColor" stroke-dasharray="2 2" class="text-border/30" stroke-width="0.5" />

            <!-- 单分量 Fill -->
            <path
              v-if="numComponents === 1"
              :d="generateAreaPathD(0, 320, 130)"
              fill="currentColor"
              class="text-primary/10"
            />

            <!-- 绘制线段 -->
            <path
              v-for="cIdx in numComponents"
              :key="cIdx"
              :d="generatePathD(cIdx - 1, 320, 130)"
              fill="none"
              :stroke="numComponents === 1 ? 'var(--primary, #3b82f6)' : COMPONENT_COLORS[(cIdx - 1) % COMPONENT_COLORS.length]"
              stroke-width="1.8"
              stroke-linejoin="round"
            />

            <!-- 可拖拽控制节点圆点 (透明 Hitbox + 视觉 Circle 隔离防闪烁) -->
            <g v-for="cIdx in numComponents" :key="`lg_pts_${cIdx}`">
              <g v-for="(s, sIdx) in parsedSamples" :key="sIdx">
                <!-- 透明碰撞代理 (固定 r=8px，防 mouseenter/mouseleave 震荡) -->
                <circle
                  :cx="getPointCoords(s, cIdx - 1, 320, 130).x"
                  :cy="getPointCoords(s, cIdx - 1, 320, 130).y"
                  r="8"
                  fill="transparent"
                  class="cursor-grab active:cursor-grabbing"
                  @mousedown="startDrag(sIdx, cIdx - 1, $event)"
                  @mouseenter="hoveredSampleIdx = sIdx"
                  @mouseleave="hoveredSampleIdx = null"
                />
                <!-- 纯外观渲染层 -->
                <circle
                  :cx="getPointCoords(s, cIdx - 1, 320, 130).x"
                  :cy="getPointCoords(s, cIdx - 1, 320, 130).y"
                  :r="hoveredSampleIdx === sIdx ? 5.5 : 4"
                  :fill="numComponents === 1 ? 'var(--primary, #3b82f6)' : COMPONENT_COLORS[(cIdx - 1) % COMPONENT_COLORS.length]"
                  stroke="var(--background, #fff)"
                  :stroke-width="hoveredSampleIdx === sIdx ? 2 : 1"
                  class="pointer-events-none transition-all duration-100"
                />
              </g>
            </g>
          </svg>
        </div>

        <!-- 向量维度图例 -->
        <div v-if="numComponents > 1" class="flex items-center gap-3 px-1 pt-0.5">
          <div
            v-for="cIdx in numComponents"
            :key="cIdx"
            class="flex items-center gap-1 text-[11px] font-mono"
          >
            <span
              class="size-2 rounded-full inline-block"
              :style="{ backgroundColor: COMPONENT_COLORS[(cIdx - 1) % COMPONENT_COLORS.length] }"
            />
            <span class="text-muted-foreground">
              {{ isColor ? (COLOR_LABELS[cIdx - 1] || `C${cIdx}`) : (COMPONENT_LABELS[(cIdx - 1) % COMPONENT_LABELS.length]) }}
            </span>
          </div>
        </div>
      </div>

      <!-- 3. 控制节点数据表单列表 -->
      <div class="space-y-1.5 pt-1">
        <div class="text-[11px] font-semibold text-muted-foreground flex items-center justify-between">
          <span>{{ $t('particles.keyframePoints') || '控制节点' }} (Keyframes)</span>
          <span class="text-[10px] font-mono text-muted-foreground font-normal">t: 0.0 ~ 1.0</span>
        </div>

        <div class="max-h-44 overflow-y-auto rounded border bg-background text-[11px] font-mono divide-y">
          <div
            v-for="(s, sIdx) in parsedSamples"
            :key="sIdx"
            class="flex items-center justify-between px-2 py-1.5 hover:bg-muted/40 transition-colors gap-2"
            :class="{ 'bg-accent/30': hoveredSampleIdx === sIdx }"
            @mouseenter="hoveredSampleIdx = sIdx"
            @mouseleave="hoveredSampleIdx = null"
          >
            <!-- 节点时间比 t -->
            <div class="flex items-center gap-1 shrink-0">
              <span class="text-muted-foreground text-[10px]">t:</span>
              <NumberField
                :model-value="s.t"
                :step="0.05"
                :min="0"
                :max="1"
                class="w-14"
                @update:model-value="updateSampleT(sIdx, $event)"
              >
                <NumberFieldContent class="h-6">
                  <NumberFieldInput class="h-6 text-[11px] px-1 font-mono" />
                </NumberFieldContent>
              </NumberField>
            </div>

            <!-- 节点颜色方块 & Color Picker (Color 模式) -->
            <div v-if="isColor" class="flex items-center gap-2 flex-1 justify-end min-w-0">
              <span class="text-[10px] text-muted-foreground font-mono truncate">
                {{ parseColor(s.values).hexStr }}
              </span>
              <div class="relative size-5 rounded border checkerboard-bg shrink-0 overflow-hidden shadow-xs cursor-pointer">
                <input
                  type="color"
                  :value="parseColor(s.values).hexStr"
                  class="absolute -top-2 -left-2 w-9 h-9 cursor-pointer opacity-0"
                  @input="updateColorHex(sIdx, $event)"
                />
                <div
                  class="w-full h-full pointer-events-none"
                  :style="{ backgroundColor: parseColor(s.values).rgbaStr }"
                />
              </div>
            </div>

            <!-- 普通数字/向量编辑 (非 Color 模式) -->
            <div v-else class="flex items-center gap-1.5 flex-1 justify-end min-w-0">
              <div
                v-for="(v, cIdx) in s.values"
                :key="cIdx"
                class="flex items-center gap-0.5 min-w-0"
              >
                <span
                  v-if="numComponents > 1"
                  class="text-[9px] font-semibold uppercase shrink-0"
                  :style="{ color: COMPONENT_COLORS[cIdx % COMPONENT_COLORS.length] }"
                >
                  {{ COMPONENT_LABELS[cIdx % COMPONENT_LABELS.length] }}:
                </span>
                <NumberField
                  :model-value="v"
                  :step="0.1"
                  class="w-15 min-w-0"
                  @update:model-value="updateSampleVal(sIdx, cIdx, $event)"
                >
                  <NumberFieldContent class="h-6">
                    <NumberFieldInput class="h-6 text-[11px] px-1 font-mono min-w-0" />
                  </NumberFieldContent>
                </NumberField>
              </div>
            </div>

            <!-- 删除控制点按钮 -->
            <button
              type="button"
              class="size-5 inline-flex items-center justify-center rounded text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors shrink-0 disabled:opacity-30 disabled:cursor-not-allowed"
              :disabled="parsedSamples.length <= 1"
              :title="$t('common.delete') || '删除'"
              @click="removeKeyframe(sIdx)"
            >
              <Trash2Icon class="size-3" />
            </button>
          </div>
        </div>

        <!-- 添加控制点按钮 -->
        <Button
          variant="outline"
          size="sm"
          class="w-full h-7 text-xs gap-1 border-dashed text-muted-foreground hover:text-foreground"
          @click="addKeyframe"
        >
          <PlusIcon class="size-3.5 text-primary" />
          {{ $t('particles.addKeyframe') || '添加控制节点' }}
        </Button>
      </div>
    </PopoverContent>
  </Popover>
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
