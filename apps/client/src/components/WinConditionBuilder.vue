<script setup lang="ts">
import { Button } from "./ui/button";
import { PlusIcon, FlagIcon, LayersIcon } from "@lucide/vue";
import WinNode from "./WinNode.vue";
import type { WinCondition } from "./win-condition";

const props = defineProps<{ modelValue: WinCondition | null }>();
const emit = defineEmits<{ "update:modelValue": [value: WinCondition | null] }>();

const PRESETS: { name: string; desc: string; cond: WinCondition }[] = [
  { name: "速战速决", desc: "先拿一血获胜", cond: { op: "atom", type: "kills", params: { n: 1 } } },
  {
    name: "补刀竞赛",
    desc: "先补 10 个小兵获胜",
    cond: { op: "atom", type: "minion_kills", params: { n: 10 } },
  },
  {
    name: "推塔竞速",
    desc: "先推掉对方一塔获胜",
    cond: { op: "atom", type: "turret_destroyed", params: { tier: 1 } },
  },
  {
    name: "综合实力",
    desc: "推掉主基地获胜（标准）",
    cond: { op: "atom", type: "nexus_destroyed", params: {} },
  },
];

function update(next: WinCondition | null) {
  emit("update:modelValue", next);
}

function replaceRoot(next: WinCondition) {
  update(next);
}

function applyPreset(p: (typeof PRESETS)[number]) {
  update(JSON.parse(JSON.stringify(p.cond)));
}

function wrap(op: "and" | "or" | "not") {
  const cur = props.modelValue ?? { op: "atom" as const, type: "minion_kills", params: { n: 10 } };
  if (op === "not") {
    update({ op: "not", child: JSON.parse(JSON.stringify(cur)) });
  } else {
    update({
      op,
      children: [JSON.parse(JSON.stringify(cur)), { op: "atom", type: "minion_kills", params: { n: 10 } }],
    });
  }
}
</script>

<template>
  <div class="flex flex-col gap-2.5">
    <!-- 预设模板 -->
    <div>
      <div class="text-muted-foreground mb-1.5 text-[10px] font-semibold uppercase tracking-wider">
        预设模板
      </div>
      <div class="grid grid-cols-2 gap-1.5">
        <button
          v-for="p in PRESETS"
          :key="p.name"
          class="bg-muted/40 hover:bg-muted border-border rounded border px-2 py-1.5 text-left transition-colors"
          @click="applyPreset(p)"
        >
          <div class="text-foreground text-[11px] font-semibold">{{ p.name }}</div>
          <div class="text-muted-foreground text-[9px] leading-tight">{{ p.desc }}</div>
        </button>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-if="!modelValue"
      class="border-border bg-muted/20 flex flex-col items-center gap-2 rounded-md border border-dashed p-4 text-center"
    >
      <FlagIcon class="text-muted-foreground/40 size-6" />
      <p class="text-muted-foreground text-[11px]">尚未配置胜利条件</p>
      <Button variant="outline" size="xs" class="h-7 gap-1 text-[11px]" @click="applyPreset(PRESETS[0]!)">
        <PlusIcon class="size-3" />
        使用预设
      </Button>
    </div>

    <!-- 条件树 -->
    <div v-else class="bg-background/50 border-border flex flex-col gap-2 rounded-md border p-2.5">
      <WinNode :node="modelValue" @replace="replaceRoot" />

      <!-- 顶层逻辑包装按钮 -->
      <div class="border-border flex items-center gap-1.5 border-t pt-2">
        <span class="text-muted-foreground text-[10px] font-semibold uppercase tracking-wider">
          包装为组合：
        </span>
        <Button variant="outline" size="xs" class="h-6 gap-1 text-[10px]" @click="wrap('and')">
          <LayersIcon class="size-3" /> AND
        </Button>
        <Button variant="outline" size="xs" class="h-6 gap-1 text-[10px]" @click="wrap('or')">
          <LayersIcon class="size-3" /> OR
        </Button>
        <Button variant="outline" size="xs" class="h-6 gap-1 text-[10px]" @click="wrap('not')">
          <LayersIcon class="size-3" /> NOT
        </Button>
      </div>
    </div>

    <!-- 诚实声明 -->
    <p class="text-muted-foreground text-[10px] leading-relaxed">
      ※ 胜利条件评估逻辑尚未接入运行时，当前对局仍以 120 秒计时结束。配置将随场景保存，供未来版本使用。
    </p>
  </div>
</template>
