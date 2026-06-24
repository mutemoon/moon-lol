<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ChevronsUpDownIcon, PlusIcon, SearchIcon } from "@lucide/vue";
import { Button } from "./ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
  DropdownMenuCheckboxItem,
  DropdownMenuSeparator,
  DropdownMenuItem,
} from "./ui/dropdown-menu";
import { InputGroup, InputGroupAddon, InputGroupInput } from "./ui/input-group";
import { cn } from "@/lib/utils";
import { useLocale } from "../composables/useLocale";

// 通用「预设选择器」：采用 DropdownMenu + DropdownMenuCheckboxItem 结构，支持搜索，末尾内置「＋ 新建预设」入口。
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
    disabled: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  new: [];
}>();

const { t, te } = useLocale();

const open = ref(false);
const searchQuery = ref("");

// 直接使用父组件传入的 presets（store 的 load* 已把内置预设 merge 进去）。
const selectedLabel = computed(() => props.presets.find((p) => p.name === props.modelValue)?.name ?? "");

const filteredPresets = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) return props.presets;
  return props.presets.filter((p) => {
    const nameMatch = p.name.toLowerCase().includes(query);
    if (props.subtitleKey) {
      const sub = p[props.subtitleKey];
      if (sub) {
        const subStr = String(sub).toLowerCase();
        const translatedSub = subtitle(p).toLowerCase();
        if (subStr.includes(query) || translatedSub.includes(query)) {
          return true;
        }
      }
    }
    return nameMatch;
  });
});

watch(open, (val) => {
  if (!val) {
    searchQuery.value = "";
  }
});

function subtitle(p: Preset): string {
  if (!props.subtitleKey) return "";
  const v = p[props.subtitleKey];
  if (v == null || v === "") return "";
  const str = String(v);
  if (props.subtitleKey === "champion") {
    const key = `champions.${str}`;
    return te(key) ? t(key) : str;
  }
  if (props.subtitleKey === "agent_type") {
    const key = `agents.types.${str.toLowerCase()}`;
    return te(key) ? t(key) : str;
  }
  return str;
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
  <DropdownMenu v-model:open="open">
    <DropdownMenuTrigger as-child>
      <Button
        variant="outline"
        :disabled="disabled"
        :class="cn('justify-between font-normal', triggerClass)"
        class="bg-muted/40 border-border text-foreground hover:bg-muted/40 h-8 px-2.5 text-xs"
      >
        <span v-if="selectedLabel" class="text-foreground truncate">{{ selectedLabel }}</span>
        <span v-else class="text-muted-foreground truncate">{{ placeholder ?? t('common.presetSelect.placeholder') }}</span>
        <ChevronsUpDownIcon class="text-muted-foreground ml-2 size-3.5 shrink-0 opacity-50" />
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent
      class="bg-popover text-popover-foreground w-[var(--reka-dropdown-menu-trigger-width)] min-w-52 p-0"
    >
      <div data-slot="command-input-wrapper" class="p-1 border-b border-border">
        <InputGroup class="border-0 !bg-transparent hover:border-input-border-hover h-8">
          <InputGroupInput
            v-model="searchQuery"
            data-slot="command-input"
            :placeholder="searchPlaceholder ?? t('common.presetSelect.searchPlaceholder')"
            class="w-full text-[13px]/relaxed text-foreground outline-hidden placeholder:text-foreground-subtlest disabled:cursor-not-allowed disabled:opacity-50"
          />
          <InputGroupAddon align="inline-start" class="pl-2">
            <SearchIcon class="size-4 shrink-0 text-foreground-subtlest" />
          </InputGroupAddon>
        </InputGroup>
      </div>

      <div class="p-1">
        <div v-if="filteredPresets.length === 0" class="text-muted-foreground p-3 text-center text-xs">
          {{ t('common.presetSelect.noMatch') }}
        </div>

        <DropdownMenuCheckboxItem
          v-for="p in filteredPresets"
          :key="p.name"
          :checked="modelValue === p.name"
          @select="() => pick(p.name)"
          class="text-xs"
        >
          <span class="flex min-w-0 flex-1 items-center justify-between gap-2">
            <span class="truncate font-medium">{{ p.name }}</span>
            <span v-if="subtitle(p)" class="text-muted-foreground truncate text-[10px]">
              {{ subtitle(p) }}
            </span>
          </span>
        </DropdownMenuCheckboxItem>

        <DropdownMenuSeparator />

        <DropdownMenuItem
          @select="onNew"
          class="text-xs"
        >
          <PlusIcon class="text-primary mr-2 size-3.5" />
          <span class="text-primary font-semibold">{{ newLabel ?? t('common.presetSelect.newLabel') }}</span>
        </DropdownMenuItem>
      </div>
    </DropdownMenuContent>
  </DropdownMenu>
</template>

