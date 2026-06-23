<script setup lang="ts">
import { computed } from "vue";

// 游戏坐标系：15000 x 15000。地图 Y 轴向上，SVG Y 轴向下，因此做翻转。
const MAP_SIZE = 15000;

export interface RiftAgent {
  id: string;
  champion: string;
  team: string; // "Order" | "Chaos"
  spawnPoint: [number, number];
}

const props = withDefaults(
  defineProps<{
    agents: RiftAgent[];
    selectedId?: string | null;
    /** 启用点击/拖拽选点；为 false 时仅展示。 */
    interactive?: boolean;
    /** 视图盒尺寸，组件按比例缩放。 */
    viewBox?: number;
  }>(),
  {
    selectedId: null,
    interactive: true,
    viewBox: 500,
  },
);

const emit = defineEmits<{
  pick: [x: number, z: number];
  select: [id: string];
}>();

function toSvgX(x: number) {
  return (x / MAP_SIZE) * props.viewBox;
}
function toSvgY(z: number) {
  return (1 - z / MAP_SIZE) * props.viewBox;
}

const markers = computed(() =>
  props.agents.map((a) => ({
    id: a.id,
    champion: a.champion,
    team: a.team,
    cx: toSvgX(a.spawnPoint[0]),
    cy: toSvgY(a.spawnPoint[1]),
    selected: a.id === props.selectedId,
  })),
);

function handleClick(event: MouseEvent) {
  if (!props.interactive) return;
  const rect = (event.currentTarget as SVGElement).getBoundingClientRect();
  const clickX = (event.clientX - rect.left) / rect.width;
  const clickY = (event.clientY - rect.top) / rect.height;
  // 转换回游戏坐标：X 同向，Z 因 Y 轴翻转取反。
  const x = Math.round(clickX * MAP_SIZE);
  const z = Math.round((1 - clickY) * MAP_SIZE);
  emit("pick", x, z);
}
</script>

<template>
  <svg
    :viewBox="`0 0 ${viewBox} ${viewBox}`"
    class="relative h-full w-full select-none"
    :class="interactive ? 'cursor-crosshair' : 'cursor-default'"
    @click="handleClick"
  >
    <!-- 网格 -->
    <defs>
      <pattern id="riftGrid" width="25" height="25" patternUnits="userSpaceOnUse">
        <path
          d="M 25 0 L 0 0 0 25"
          fill="none"
          stroke="currentColor"
          class="text-border/20"
          stroke-width="0.75"
        />
      </pattern>
    </defs>
    <rect width="100%" height="100%" fill="url(#riftGrid)" />

    <!-- 三条兵线 -->
    <path
      d="M 60 440 L 60 60 L 440 60"
      fill="none"
      stroke="currentColor"
      class="text-border"
      stroke-width="5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <line
      x1="60"
      y1="440"
      x2="440"
      y2="60"
      stroke="currentColor"
      class="text-border"
      stroke-width="5"
      stroke-linecap="round"
    />
    <path
      d="M 60 440 L 440 440 L 440 60"
      fill="none"
      stroke="currentColor"
      class="text-border"
      stroke-width="5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M 60 440 L 60 60 L 440 60 M 60 440 L 440 60 M 60 440 L 440 440 L 440 60"
      fill="none"
      stroke="currentColor"
      class="text-primary/40"
      stroke-width="1.5"
      stroke-dasharray="4,6"
      stroke-linecap="round"
      stroke-linejoin="round"
    />

    <!-- 蓝方基地 -->
    <circle
      cx="60"
      cy="440"
      r="28"
      fill="rgba(59, 130, 246, 0.03)"
      stroke="#3b82f6"
      stroke-width="1"
      stroke-dasharray="3,3"
    />
    <rect
      x="48"
      y="428"
      width="24"
      height="24"
      fill="rgba(59, 130, 246, 0.12)"
      stroke="#3b82f6"
      stroke-width="1.5"
    />
    <text
      x="60"
      y="415"
      text-anchor="middle"
      fill="#3b82f6"
      font-size="8"
      font-family="sans-serif"
      font-weight="bold"
      letter-spacing="1"
    >
      BLUE
    </text>

    <!-- 红方基地 -->
    <circle
      cx="440"
      cy="60"
      r="28"
      fill="rgba(239, 68, 68, 0.03)"
      stroke="#ef4444"
      stroke-width="1"
      stroke-dasharray="3,3"
    />
    <rect
      x="428"
      y="48"
      width="24"
      height="24"
      fill="rgba(239, 68, 68, 0.12)"
      stroke="#ef4444"
      stroke-width="1.5"
    />
    <text
      x="440"
      y="92"
      text-anchor="middle"
      fill="#ef4444"
      font-size="8"
      font-family="sans-serif"
      font-weight="bold"
      letter-spacing="1"
    >
      RED
    </text>

    <!-- 所有 Agent 出生点 -->
    <g
      v-for="m in markers"
      :key="m.id"
      :transform="`translate(${m.cx} ${m.cy})`"
      class="cursor-pointer"
      @click.stop="emit('select', m.id)"
    >
      <!-- 选中态脉冲环 -->
      <circle
        v-if="m.selected"
        r="16"
        fill="none"
        stroke="currentColor"
        :class="m.team === 'Order' ? 'text-blue-500' : 'text-red-500'"
        stroke-width="1"
        class="animate-ping"
      />
      <circle
        r="8"
        fill="none"
        :stroke="m.team === 'Order' ? '#3b82f6' : '#ef4444'"
        :class="m.team === 'Order' ? 'fill-blue-500/20' : 'fill-red-500/20'"
        stroke-width="1.5"
      />
      <circle r="2.5" :fill="m.team === 'Order' ? '#3b82f6' : '#ef4444'" />
      <text
        v-if="viewBox >= 400"
        y="-14"
        text-anchor="middle"
        :fill="m.team === 'Order' ? '#60a5fa' : '#f87171'"
        font-size="9"
        font-family="sans-serif"
        font-weight="bold"
      >
        {{ m.champion }}
      </text>
    </g>
  </svg>
</template>
