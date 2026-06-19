<script setup lang="ts">
import { computed, ref } from "vue";
import { CheckIcon, ChevronsUpDownIcon, PlusIcon } from "@lucide/vue";
import { Button } from "./ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "./ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";
import { cn } from "@/lib/utils";

// 通用「预设选择器」：combobox（Popover + Command），支持搜索，末尾内置「＋ 新建预设」入口。
// 这是产品文档 §3.0 设计原则的核心交互：编排页只做下拉选择，新建走独立管理页。
//
// 用法：
//   <PresetSelect :presets="agentPresets" v-model="slot.agentPresetName"
//     subtitle-key="champion" @new="router.push('/agents')" />

interface Preset {
  name: string;
  [k: string]: any;
}

const props = withDefaults(
  defineProps<{
    presets: Preset[];
    modelValue: string | undefined;
    placeholder?: string;
    /** 搜索框 placeholder。 */
    searchPlaceholder?: string;
    /** 新建入口文案，如「新建 Agent 预设」。 */
    newLabel?: string;
    /** 展示每个预设时附加的副标题字段名（取该项对应字段的值，转为字符串）。 */
    subtitleKey?: string;
    disabled?: boolean;
    triggerClass?: string;
  }>(),
  {
    placeholder: "选择预设…",
    searchPlaceholder: "搜索预设…",
    newLabel: "新建预设",
    disabled: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  new: [];
}>();

const open = ref(false);

// 直接使用父组件传入的 presets（store 的 load* 已把内置预设 merge 进去）。
// 不再在此按 subtitleKey 猜测补全——那会把出生点内置预设错塞进 Agent 选择器。
const selectedLabel = computed(() => props.presets.find((p) => p.name === props.modelValue)?.name ?? "");

function subtitle(p: Preset): string {
  if (!props.subtitleKey) return "";
  const v = p[props.subtitleKey];
  return v == null || v === "" ? "" : String(v);
}

function pick(name: string) {
  emit("update:modelValue", name === props.modelValue ? "" : name);
  open.value = false;
}

function onNew() {
  open.value = false;
  emit("new");
}
</script>

<template>
  <Popover v-model:open="open">
    <PopoverTrigger as-child>
      <Button
        variant="outline"
        role="combobox"
        :aria-expanded="open"
        :disabled="disabled"
        :class="cn('justify-between font-normal', triggerClass)"
        class="bg-muted/40 border-border text-foreground hover:bg-muted/40 h-8 px-2.5 text-xs"
      >
        <span v-if="selectedLabel" class="text-foreground truncate">{{ selectedLabel }}</span>
        <span v-else class="text-muted-foreground truncate">{{ placeholder }}</span>
        <ChevronsUpDownIcon class="text-muted-foreground ml-2 size-3.5 shrink-0 opacity-50" />
      </Button>
    </PopoverTrigger>
    <PopoverContent
      class="popover-content-width bg-popover text-popover-foreground w-[var(--reka-popover-trigger-width)] min-w-52 p-0"
    >
      <Command>
        <CommandInput :placeholder="searchPlaceholder" class="h-8" />
        <CommandList>
          <CommandEmpty>未找到匹配的预设</CommandEmpty>
          <CommandGroup>
            <CommandItem
              v-for="p in presets"
              :key="p.name"
              :value="p.name"
              @select="() => pick(p.name)"
              class="text-xs"
            >
              <CheckIcon :class="cn('mr-2 size-3.5', modelValue === p.name ? 'opacity-100' : 'opacity-0')" />
              <span class="flex min-w-0 flex-1 items-center justify-between gap-2">
                <span class="truncate font-medium">{{ p.name }}</span>
                <span v-if="subtitle(p)" class="text-muted-foreground truncate text-[10px]">
                  {{ subtitle(p) }}
                </span>
              </span>
            </CommandItem>
          </CommandGroup>

          <CommandSeparator />
          <!-- 新建入口：刻意放在 CommandGroup 之外，避免被搜索过滤或当作可选预设 -->
          <CommandGroup>
            <CommandItem :value="`__new__${newLabel}`" @select="onNew" class="text-xs">
              <PlusIcon class="text-primary mr-2 size-3.5" />
              <span class="text-primary font-semibold">{{ newLabel }}</span>
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    </PopoverContent>
  </Popover>
</template>
