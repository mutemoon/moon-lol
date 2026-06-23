<script setup lang="ts">
import { computed } from "vue";
import { Input } from "./ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { PlusIcon, Trash2Icon } from "@lucide/vue";
import { ATOM_TYPES, type WinCondition } from "./win-condition";

const OP_META = {
  and: { label: "AND 全部满足", cls: "text-green-500 border-green-500/30 bg-green-500/5" },
  or: { label: "OR 任一满足", cls: "text-primary border-primary/30 bg-primary/5" },
  not: { label: "NOT 取反", cls: "text-amber-500 border-amber-500/30 bg-amber-500/5" },
} as const;

const props = defineProps<{ node: WinCondition }>();
const emit = defineEmits<{ replace: [next: WinCondition] }>();

const paramDef = computed(() => ATOM_TYPES.find((t) => t.value === props.node.type));
const paramVal = computed(() => props.node.params?.[paramDef.value?.param.key ?? ""] ?? 0);

function replace(next: WinCondition) {
  emit("replace", next);
}

function setAtomType(type: string) {
  const def = ATOM_TYPES.find((t) => t.value === type);
  const next: WinCondition = {
    ...JSON.parse(JSON.stringify(props.node)),
    type,
    params: def ? { [def.param.key]: def.param.def } : {},
  };
  replace(next);
}

function setParam(val: string) {
  if (!paramDef.value) return;
  const next: WinCondition = {
    ...JSON.parse(JSON.stringify(props.node)),
    params: { ...(props.node.params ?? {}), [paramDef.value.param.key]: Number(val) },
  };
  replace(next);
}

function replaceChild(idx: number, next: WinCondition) {
  if (!props.node.children) return;
  const copy = JSON.parse(JSON.stringify(props.node)) as WinCondition;
  copy.children![idx] = next;
  replace(copy);
}

function replaceSingle(next: WinCondition) {
  // 用于 NOT 的唯一子节点
  replace({ ...JSON.parse(JSON.stringify(props.node)), child: next });
}

function removeChild(idx: number) {
  if (!props.node.children) return;
  const copy = JSON.parse(JSON.stringify(props.node)) as WinCondition;
  copy.children!.splice(idx, 1);
  if (copy.children!.length === 1 && copy.children![0]) {
    replace(copy.children![0]); // 单子节点降级
  } else {
    replace(copy);
  }
}

function addChild() {
  if (!props.node.children) return;
  const copy = JSON.parse(JSON.stringify(props.node)) as WinCondition;
  copy.children!.push({ op: "atom", type: "minion_kills", params: { n: 10 } });
  replace(copy);
}

function unwrapToFirstChild() {
  if (props.node.children && props.node.children.length > 0) {
    replace(JSON.parse(JSON.stringify(props.node.children[0])));
  }
}
</script>

<template>
  <!-- 原子节点 -->
  <div v-if="node.op === 'atom'" class="flex flex-wrap items-center gap-2">
    <Select :model-value="node.type" @update:model-value="(v) => setAtomType(v as string)">
      <SelectTrigger class="bg-muted/40 h-7 min-w-44 border-border text-[11px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent class="border-border bg-popover text-foreground">
        <SelectItem v-for="t in ATOM_TYPES" :key="t.value" :value="t.value" class="text-xs">
          {{ t.value }} — {{ t.label }}
        </SelectItem>
      </SelectContent>
    </Select>
    <div v-if="paramDef" class="flex items-center gap-1">
      <span class="text-muted-foreground text-[10px]">{{ paramDef.param.label }}</span>
      <Input
        type="number"
        :model-value="paramVal"
        class="bg-muted/40 h-7 w-16 border-border text-[11px]"
        @update:model-value="(v: string | number) => setParam(String(v))"
      />
    </div>
  </div>

  <!-- NOT 节点 -->
  <div v-else-if="node.op === 'not'" class="flex flex-col gap-1.5">
    <div
      class="flex items-center justify-between rounded border px-2 py-1 text-[10px] font-semibold"
      :class="OP_META.not.cls"
    >
      <span>{{ OP_META.not.label }}</span>
      <span class="text-muted-foreground">包裹下方</span>
    </div>
    <WinNode v-if="node.child" :node="node.child" @replace="replaceSingle" />
  </div>

  <!-- AND / OR 节点 -->
  <div v-else-if="node.op === 'and' || node.op === 'or'" class="flex flex-col gap-1.5">
    <div
      class="flex items-center justify-between rounded border px-2 py-1 text-[10px] font-semibold"
      :class="(node.op === 'and' || node.op === 'or') ? OP_META[node.op].cls : ''"
    >
      <span>{{ (node.op === 'and' || node.op === 'or') ? OP_META[node.op].label : '' }}</span>
      <button
        class="text-muted-foreground hover:text-foreground text-[10px] underline-offset-2 hover:underline"
        @click="unwrapToFirstChild"
      >
        拆解
      </button>
    </div>
    <div class="border-border flex flex-col gap-1.5 border-l pl-2.5">
      <div v-for="(_child, idx) in node.children" :key="idx" class="flex items-start gap-1.5">
        <WinNode v-if="node.children && node.children[idx]" :node="node.children[idx]" @replace="(next) => replaceChild(idx, next)" />
        <button
          class="text-muted-foreground hover:text-destructive mt-1 shrink-0 transition-colors"
          @click="removeChild(idx)"
        >
          <Trash2Icon class="size-3" />
        </button>
      </div>
    </div>
    <button
      class="text-muted-foreground hover:text-foreground hover:bg-muted flex items-center gap-1 self-start rounded px-2 py-1 text-[10px] transition-colors"
      @click="addChild"
    >
      <PlusIcon class="size-3" /> 添加子条件
    </button>
  </div>

  <div v-else class="text-muted-foreground text-[11px] italic">未知条件节点</div>
</template>
