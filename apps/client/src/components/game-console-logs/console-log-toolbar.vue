<script setup lang="ts">
import { useLog } from "../../composables/useLogPoller";

// ── Shadcn UI Components ──
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

// ── Lucide Icons ──
import {
  ClockIcon,
  FolderOpenIcon,
  BarChart3Icon,
  SlidersHorizontalIcon,
  DownloadIcon,
  RefreshCwIcon,
  Trash2Icon,
} from "@lucide/vue";

const {
  logs,
  clearLogs,
  refresh,
  exportLogs,
} = useLog();

defineProps<{
  viewMode: "timeline" | "groups";
  showAnalytics: boolean;
  table: any;
}>();

defineEmits<{
  (e: "update:viewMode", value: "timeline" | "groups"): void;
  (e: "update:showAnalytics", value: boolean): void;
}>();
</script>

<template>
  <!-- 日志工具栏 -->
  <div class="bg-muted/50 border-border flex shrink-0 items-center justify-between border-b px-3.5 py-2">
    <div class="flex items-baseline gap-2">
      <h2 class="text-foreground/80 text-xs font-semibold uppercase">控制台运行日志</h2>
      <span class="text-muted-foreground text-[11px]">{{ logs.length }} 条记录</span>
    </div>
    <div class="flex items-center gap-1">
      <Button
        size="xs"
        variant="outline"
        class="h-6 cursor-pointer border px-2 py-0.5 text-[11px] transition-all duration-200 inline-flex items-center gap-1"
        :class="
          viewMode === 'timeline'
            ? 'text-primary border-primary bg-primary/10 hover:bg-primary/20'
            : 'text-muted-foreground border-border hover:text-foreground hover:border-primary/40 bg-transparent'
        "
        @click="$emit('update:viewMode', 'timeline')"
      >
        <ClockIcon class="h-3 w-3" />
        时间线
      </Button>
      <Button
        size="xs"
        variant="outline"
        class="h-6 cursor-pointer border px-2 py-0.5 text-[11px] transition-all duration-200 inline-flex items-center gap-1"
        :class="
          viewMode === 'groups'
            ? 'text-primary border-primary bg-primary/10 hover:bg-primary/20'
            : 'text-muted-foreground border-border hover:text-foreground hover:border-primary/40 bg-transparent'
        "
        @click="$emit('update:viewMode', 'groups')"
      >
        <FolderOpenIcon class="h-3 w-3" />
        实体分组
      </Button>
      <Button
        size="xs"
        variant="outline"
        class="h-6 cursor-pointer border px-2 py-0.5 text-[11px] transition-all duration-200 inline-flex items-center gap-1"
        :class="
          showAnalytics
            ? 'text-primary border-primary bg-primary/10 hover:bg-primary/20'
            : 'text-muted-foreground border-border hover:text-foreground hover:border-primary/40 bg-transparent'
        "
        @click="$emit('update:showAnalytics', !showAnalytics)"
      >
        <BarChart3Icon class="h-3 w-3" />
        智能分析
      </Button>

      <span class="bg-border mx-1 h-3.5 w-[1px]"></span>

      <!-- 🔧 Customize Columns Dropdown -->
      <DropdownMenu v-if="viewMode === 'timeline' && table">
        <DropdownMenuTrigger as-child>
          <Button
            variant="outline"
            class="border-border text-muted-foreground hover:text-foreground hover:border-primary/40 h-6 gap-1 bg-transparent px-2 text-[11px] transition-all inline-flex items-center"
          >
            <SlidersHorizontalIcon class="h-3 w-3" />
            自定义显示列
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" class="bg-popover border-border text-foreground/80 w-44">
          <template
            v-for="column in table
              .getAllColumns()
              .filter((col: any) => typeof col.accessorFn !== 'undefined' || col.id === 'entity' || col.id === 'source')"
            :key="column.id"
          >
            <DropdownMenuCheckboxItem
              class="text-foreground cursor-pointer text-xs capitalize hover:bg-muted"
              :model-value="column.getIsVisible()"
              @update:model-value="(value) => column.toggleVisibility(!!value)"
            >
              {{
                (
                  {
                    timestamp: "时间",
                    level: "级别",
                    entity: "实体",
                    category: "模块",
                    message: "内容",
                    source: "源码位置",
                  } as Record<string, string>
                )[column.id] || column.id
              }}
            </DropdownMenuCheckboxItem>
          </template>
        </DropdownMenuContent>
      </DropdownMenu>

      <Button
        size="xs"
        variant="outline"
        class="text-primary border-primary bg-transparent flex h-6 cursor-pointer items-center gap-1 rounded border px-1.5 py-0.5 text-[11px] transition-all duration-200 hover:bg-primary/10"
        @click="exportLogs('json')"
      >
        <DownloadIcon class="h-3 w-3" />
        导出 JSON
      </Button>
      <Button
        size="xs"
        variant="outline"
        class="text-muted-foreground border-border hover:text-primary hover:border-primary/40 flex h-6 cursor-pointer items-center gap-1 rounded border bg-transparent px-1.5 py-0.5 text-[11px] transition-all duration-200"
        @click="exportLogs('txt')"
      >
        <DownloadIcon class="h-3 w-3" />
        导出 TXT
      </Button>
      <Button
        size="xs"
        variant="outline"
        class="text-muted-foreground border-border hover:text-primary hover:border-primary/40 h-6 cursor-pointer rounded border bg-transparent px-1.5 py-0.5 text-[11px] transition-all duration-200 inline-flex items-center gap-1"
        @click="refresh()"
      >
        <RefreshCwIcon class="h-3 w-3" />
        刷新
      </Button>
      <Button
        size="xs"
        variant="destructive"
        class="text-destructive h-6 cursor-pointer rounded border border-destructive/30 bg-transparent px-1.5 py-0.5 text-[11px] transition-all duration-200 hover:bg-destructive/10 inline-flex items-center gap-1"
        @click="clearLogs()"
      >
        <Trash2Icon class="h-3 w-3" />
        清空
      </Button>
    </div>
  </div>
</template>
