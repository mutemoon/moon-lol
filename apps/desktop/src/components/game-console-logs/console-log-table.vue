<script setup lang="ts">
import { ref, h, watch, nextTick } from "vue";
import { useLog } from "../../composables/useLogPoller";
import {
  useVueTable,
  getCoreRowModel,
  FlexRender,
  type ColumnDef,
  type SortingState,
  type VisibilityState,
} from "@tanstack/vue-table";

// ── Shadcn UI Components ──
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";

// ── Lucide Icons ──
import { SearchIcon, BotIcon, FolderIcon } from "@lucide/vue";

const { logs, autoScroll, filterEntityId } = useLog();

defineEmits<{
  (e: "copy", message: string): void;
}>();

// 自动滚动到最新
watch(
  () => logs.value,
  () => {
    if (autoScroll.value) {
      nextTick(() => {
        const el = document.getElementById("log-list-container");
        if (el) {
          el.scrollTo({
            top: el.scrollHeight,
            behavior: "smooth",
          });
        }
      });
    }
  },
  { flush: "post" },
);

function setEntityFilter(id: number | null) {
  filterEntityId.value = id;
}

function formatTime(ts?: number | string) {
  if (!ts) return "--:--:--.---";
  if (typeof ts === "string") return ts;
  const date = new Date(ts);
  const pad = (n: number) => n.toString().padStart(2, "0");
  const ms = date.getMilliseconds().toString().padStart(3, "0");
  return `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}.${ms}`;
}

function getShortFile(path?: string) {
  if (!path) return "";
  return path.replace(/^.*[\\/]([^\\/]+[\\/][^\\/]+)$/, "$1");
}

// ── TanStack Table Definitions ──
interface StructuredLog {
  id: number;
  timestamp: number;
  level: "info" | "warn" | "error" | "debug";
  entity_id?: number;
  entity_name?: string;
  category?: string;
  message: string;
  count: number;
  file?: string;
  line?: number;
}

const sorting = ref<SortingState>([]);
const columnVisibility = ref<VisibilityState>({
  entity: true,
  category: true,
  source: false,
});

const columns: ColumnDef<StructuredLog>[] = [
  {
    accessorKey: "timestamp",
    header: "时间",
    cell: ({ row }) =>
      h("span", { class: "text-[10px] font-mono text-text-muted shrink-0" }, formatTime(row.getValue("timestamp"))),
  },
  {
    accessorKey: "level",
    header: "级别",
    cell: ({ row }) => {
      const lvl = row.getValue("level") as string;
      return h(
        Badge,
        {
          variant: "outline",
          class: `min-w-[45px] inline-flex items-center justify-center shrink-0 rounded px-1.5 py-0.5 text-[9px] font-bold uppercase border-none h-4 ${
            {
              info: "text-text-muted bg-[rgba(154,146,130,0.08)]",
              warn: "bg-[rgba(251,191,36,0.12)] text-[#fbbf24]",
              error: "bg-[rgba(248,113,113,0.15)] text-[#f87171]",
              debug: "bg-[rgba(56,189,248,0.12)] text-[#38bdf8]",
            }[lvl] || ""
          }`,
        },
        () => lvl,
      );
    },
  },
  {
    id: "entity",
    accessorFn: (row) => (row.entity_name ? `🤖 ${row.entity_name} (${row.entity_id})` : ""),
    header: "实体",
    cell: ({ row }) => {
      const original = row.original;
      if (!original.entity_name) return null;
      return h(
        Badge,
        {
          variant: "outline",
          class:
            "text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.08)] px-1.5 py-0 text-[9px] font-semibold transition-all duration-200 hover:bg-[rgba(185,145,71,0.2)] hover:shadow-[0_0_6px_rgba(185,145,71,0.2)] cursor-pointer h-4 inline-flex items-center justify-center gap-1",
          onClick: (e: MouseEvent) => {
            e.stopPropagation();
            if (original.entity_id !== undefined) {
              setEntityFilter(original.entity_id);
            }
          },
        },
        () => [
          h(BotIcon, { class: "h-2.5 w-2.5 shrink-0 text-gold-bright" }),
          h("span", {}, `${original.entity_name} (${original.entity_id})`)
        ]
      );
    },
  },
  {
    accessorKey: "category",
    header: "模块",
    cell: ({ row }) => {
      const val = row.getValue("category") as string;
      if (!val) return null;
      return h(
        Badge,
        {
          variant: "outline",
          class:
            "text-text-muted border-border-subtle shrink-0 bg-[rgba(255,255,255,0.04)] px-1.5 py-0 text-[9px] h-4 inline-flex items-center justify-center gap-1",
        },
        () => [
          h(FolderIcon, { class: "h-2.5 w-2.5 shrink-0 text-text-muted" }),
          h("span", {}, val)
        ]
      );
    },
  },
  {
    accessorKey: "message",
    header: "日志内容",
    cell: ({ row }) => {
      const original = row.original;
      return h("div", { class: "flex items-center gap-1.5 min-w-0" }, [
        h("span", { class: "flex-1 text-left text-ellipsis overflow-hidden break-all font-mono" }, original.message),
        original.count > 1
          ? h(
              "span",
              { class: "text-text-muted shrink-0 rounded-md bg-[rgba(255,255,255,0.05)] px-1 py-0 text-[9px]" },
              `${original.count}×`,
            )
          : null,
      ]);
    },
  },
  {
    id: "source",
    accessorFn: (row) => (row.file ? `${row.file}:${row.line}` : ""),
    header: "源码位置",
    cell: ({ row }) => {
      const original = row.original;
      if (!original.file) return null;
      return h(
        "span",
        {
          class: "text-text-muted text-[10px] shrink-0 font-mono",
          title: original.file,
        },
        `${getShortFile(original.file)}:${original.line}`,
      );
    },
  },
];

const table = useVueTable({
  get data() {
    return logs.value;
  },
  columns,
  getCoreRowModel: getCoreRowModel(),
  onSortingChange: (updaterOrValue) => {
    sorting.value = typeof updaterOrValue === "function" ? updaterOrValue(sorting.value) : updaterOrValue;
  },
  onColumnVisibilityChange: (updaterOrValue) => {
    columnVisibility.value =
      typeof updaterOrValue === "function" ? updaterOrValue(columnVisibility.value) : updaterOrValue;
  },
  state: {
    get sorting() {
      return sorting.value;
    },
    get columnVisibility() {
      return columnVisibility.value;
    },
  },
});

defineExpose({
  table,
});
</script>

<template>
  <!-- 视图 1: Rich TanStack Table Timeline -->
  <div id="log-list-container" class="relative min-h-0 flex-1 overflow-y-auto">
    <div
      v-if="logs.length === 0"
      class="text-text-muted flex h-full flex-col items-center justify-center gap-2 py-10 text-xs"
    >
      <SearchIcon class="text-text-muted h-5 w-5 animate-pulse" />
      <span>没有匹配当前过滤条件的日志...</span>
    </div>
    <div v-else class="min-w-full">
      <Table>
        <TableHeader class="bg-bg-elevated border-border-subtle hover:bg-bg-elevated sticky top-0 z-10">
          <TableRow
            v-for="headerGroup in table.getHeaderGroups()"
            :key="headerGroup.id"
            class="border-border-subtle hover:bg-transparent"
          >
            <TableHead
              v-for="header in headerGroup.headers"
              :key="header.id"
              :class="[
                header.id === 'level' ? 'w-20' : '',
                header.id === 'timestamp' ? 'w-28' : '',
                header.id === 'entity' ? 'w-36' : '',
                header.id === 'category' ? 'w-24' : '',
                header.id === 'source' ? 'w-44' : '',
                'text-text-bright border-border-subtle h-8 py-1 text-[11px] font-bold',
              ]"
            >
              <FlexRender
                v-if="!header.isPlaceholder"
                :render="header.column.columnDef.header"
                :props="header.getContext()"
              />
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow
            v-for="row in table.getRowModel().rows"
            :key="row.id"
            class="cursor-pointer border-b border-[rgba(255,255,255,0.01)] hover:bg-[rgba(255,255,255,0.02)] active:bg-[rgba(185,145,71,0.06)]"
            :class="
              (
                {
                  info: 'text-text-default',
                  warn: 'bg-[rgba(251,191,36,0.02)] text-[#fbbf24]',
                  error: 'bg-[rgba(248,113,113,0.03)] text-[#f87171]',
                  debug: 'text-[#38bdf8]',
                } as Record<string, string>
              )[row.original.level]
            "
            @dblclick="$emit('copy', row.original.message)"
            title="双击可直接复制日志内容"
          >
            <TableCell
              v-for="cell in row.getVisibleCells()"
              :key="cell.id"
              class="h-8 overflow-hidden border-none px-3 py-1 text-xs text-ellipsis whitespace-nowrap"
            >
              <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  </div>
</template>
