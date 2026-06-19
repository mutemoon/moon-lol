<script setup lang="ts">
import { ref, computed } from "vue";
import { useLog } from "../../composables/useLogPoller";

// ── Shadcn UI Components ──
import { Badge } from "@/components/ui/badge";

// ── Lucide Icons ──
import { SearchIcon, ChevronRightIcon, BotIcon, FolderIcon } from "@lucide/vue";

const { logs } = useLog();

defineEmits<{
  (e: "copy", message: string): void;
}>();

const collapsedGroups = ref<(number | string)[]>([]);

const logsByEntity = computed(() => {
  const groups: Record<string, { id: number | string; name?: string; logs: typeof logs.value }> = {};
  logs.value.forEach((l) => {
    const key = l.entity_id !== undefined ? l.entity_id.toString() : "system";
    if (!groups[key]) {
      groups[key] = {
        id: l.entity_id !== undefined ? l.entity_id : "system",
        name: l.entity_name,
        logs: [],
      };
    }
    groups[key].logs.push(l);
  });
  return Object.values(groups);
});

function toggleGroup(id: number | string) {
  if (collapsedGroups.value.includes(id)) {
    collapsedGroups.value = collapsedGroups.value.filter((x) => x !== id);
  } else {
    collapsedGroups.value.push(id);
  }
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
</script>

<template>
  <!-- 视图 2: 实体分组折叠列表 -->
  <div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto p-1.5">
    <div
      v-if="logsByEntity.length === 0"
      class="text-muted-foreground flex h-full flex-col items-center justify-center gap-2 text-xs"
    >
      <SearchIcon class="h-5 w-5 animate-pulse text-muted-foreground" />
      <span>暂无匹配的实体分组日志...</span>
    </div>

    <div
      v-for="group in logsByEntity"
      :key="group.id"
      class="border-border rounded border bg-muted/10 transition-colors"
    >
      <div
        class="flex cursor-pointer items-center justify-between bg-muted/20 p-2 select-none hover:bg-muted/40"
        @click="toggleGroup(group.id)"
      >
        <div class="flex items-center gap-2">
          <ChevronRightIcon
            class="text-muted-foreground h-3.5 w-3.5 transition-transform duration-200"
            :class="{ 'rotate-90': !collapsedGroups.includes(group.id as number) }"
          />
          <span class="text-foreground text-xs font-semibold flex items-center">
            <template v-if="group.name">
              <Badge
                variant="outline"
                class="text-primary border-primary/30 mr-1.5 bg-primary/10 px-2 py-0.5 text-xs font-semibold inline-flex items-center gap-1"
              >
                <BotIcon class="h-2.5 w-2.5 shrink-0 text-primary" /> 实体 {{ group.id }}
              </Badge>
              <span class="text-foreground text-xs font-semibold">{{ group.name }}</span>
            </template>
            <template v-else>💻 系统与常规运行日志</template>
          </span>
        </div>
        <Badge
          variant="secondary"
          class="text-muted-foreground inline-flex h-4 items-center rounded-xl border-none bg-muted px-1.5 py-0.5 text-[10px] hover:bg-muted/80"
        >
          {{ group.logs.length }} 个事件
        </Badge>
      </div>

      <div
        v-show="!collapsedGroups.includes(group.id as number)"
        class="border-border border-t bg-muted/10 py-1"
      >
        <div
          v-for="entry in group.logs"
          :key="entry.id"
          class="flex cursor-pointer items-center gap-2 border-b border-border/20 py-0.5 pr-3.5 pl-7 whitespace-nowrap transition-colors duration-100 hover:bg-muted/20 active:bg-primary/10"
          :class="
            {
              info: 'text-foreground/80',
              warn: 'bg-yellow-500/5 text-yellow-600',
              error: 'bg-destructive/5 text-destructive',
              debug: 'text-blue-500',
            }[entry.level]
          "
          @dblclick="$emit('copy', entry.message)"
          title="双击可直接复制日志内容"
        >
          <span class="border-border min-w-20 shrink-0 text-[10px]">{{ formatTime(entry.timestamp) }}</span>
          <Badge
            variant="outline"
            class="inline-flex h-4 min-w-9 shrink-0 items-center justify-center border-none px-1 py-0 text-center text-[9px] font-bold uppercase"
            :class="
              {
                info: 'text-muted-foreground bg-muted/40',
                warn: 'bg-yellow-500/10 text-yellow-600',
                error: 'bg-destructive/10 text-destructive',
                debug: 'bg-blue-500/10 text-blue-500',
              }[entry.level]
            "
          >
            {{ entry.level }}
          </Badge>
          <Badge
            v-if="entry.category"
            variant="outline"
            class="text-muted-foreground border-border inline-flex h-4 shrink-0 items-center justify-center bg-muted/30 px-1.5 py-0 text-[9px] gap-1"
          >
            <FolderIcon class="h-2.5 w-2.5 shrink-0 text-muted-foreground" />
            {{ entry.category }}
          </Badge>
          <span class="flex-1 overflow-hidden text-ellipsis text-foreground">{{ entry.message }}</span>
          <span v-if="entry.file" class="border-border ml-2 shrink-0 text-[10px]" :title="entry.file">
            {{ getShortFile(entry.file) }}:{{ entry.line }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
