<script setup lang="ts">
import { useLog } from "../../composables/useLogPoller";

// ── Shadcn UI Components ──
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

// ── Lucide Icons ──
import { SearchIcon } from "@lucide/vue";

const {
  selectedLevels,
  searchText,
  regexEnabled,
  autoRefresh,
  autoScroll,
} = useLog();

function toggleLevel(level: string) {
  const current = [...selectedLevels.value];
  if (current.includes(level)) {
    selectedLevels.value = current.filter((x) => x !== level);
  } else {
    selectedLevels.value = [...current, level];
  }
}
</script>

<template>
  <!-- 搜索与多维过滤 -->
  <div class="border-border flex shrink-0 flex-col gap-2 border-b bg-muted/20 px-3.5 py-2">
    <div class="flex items-center justify-between gap-2">
      <!-- 级别过滤 -->
      <div class="flex gap-1">
        <Badge
          v-for="lvl in ['info', 'warn', 'error', 'debug']"
          :key="lvl"
          variant="outline"
          class="h-auto cursor-pointer border px-1.5 py-0.5 text-[9px] font-bold transition-all duration-200"
          :class="[
            selectedLevels.includes(lvl)
              ? {
                  info: 'text-foreground border-border/40 bg-muted/40 hover:bg-muted/60',
                  warn: 'border-yellow-500/20 bg-yellow-500/10 text-yellow-600 hover:bg-yellow-500/15',
                  error: 'border-destructive/20 bg-destructive/10 text-destructive hover:bg-destructive/15',
                  debug: 'border-blue-500/20 bg-blue-500/10 text-blue-500 hover:bg-blue-500/15',
                }[lvl]
              : 'text-muted-foreground border-transparent bg-muted/20 hover:bg-muted/30',
          ]"
          @click="toggleLevel(lvl)"
        >
          {{ lvl.toUpperCase() }}
        </Badge>
      </div>

      <!-- 选择器过滤 -->
      <div class="flex items-center gap-1.5">
      </div>
    </div>

    <!-- 搜索过滤栏 -->
    <div class="flex items-center gap-3">
      <div
        class="bg-muted/40 border-border relative flex flex-1 items-center rounded border px-2 py-1 shadow-sm"
      >
        <SearchIcon class="text-muted-foreground mr-1.5 h-3.5 w-3.5 shrink-0 opacity-50" />
        <input
          v-model="searchText"
          type="text"
          placeholder="输入检索词进行模糊或正则匹配（双击日志行可直接复制）..."
          class="text-foreground placeholder:text-muted-foreground w-full bg-transparent text-xs outline-none"
        />
        <button
          v-if="searchText"
          class="text-muted-foreground hover:text-foreground absolute right-2 text-sm font-semibold transition-colors duration-100"
          @click="searchText = ''"
          title="清空检索词"
        >
          ×
        </button>
      </div>

      <div class="flex gap-1.5">
        <!-- 正则开关 -->
        <Button
          size="xs"
          variant="outline"
          class="inline-flex h-7 cursor-pointer items-center gap-1 rounded border px-2.5 py-1 text-[10px] whitespace-nowrap transition-all duration-200"
          :class="
            regexEnabled
              ? 'text-primary border-primary bg-primary/10 hover:bg-primary/20'
              : 'text-muted-foreground bg-muted/40 border-border hover:text-foreground hover:border-primary/40'
          "
          @click="regexEnabled = !regexEnabled"
          title="开启后支持使用 JS 正则语法检索日志"
        >
          <span
            class="h-1 w-1 rounded-full"
            :class="regexEnabled ? 'bg-primary' : 'bg-muted-foreground/30'"
          ></span>
          正则匹配
        </Button>

        <!-- 自动轮询开关 -->
        <Button
          size="xs"
          variant="outline"
          class="inline-flex h-7 cursor-pointer items-center gap-1 rounded border px-2.5 py-1 text-[10px] whitespace-nowrap transition-all duration-200"
          :class="
            autoRefresh
              ? 'text-primary border-primary bg-primary/10 hover:bg-primary/20'
              : 'text-muted-foreground bg-muted/40 border-border hover:text-foreground hover:border-primary/40'
          "
          @click="autoRefresh = !autoRefresh"
          title="每秒自动从 SQLite 数据库轮询最新日志"
        >
          <span
            class="h-1 w-1 rounded-full"
            :class="autoRefresh ? 'bg-primary' : 'bg-muted-foreground/30'"
          ></span>
          自动刷新
        </Button>

        <!-- 自动滚动开关 -->
        <Button
          size="xs"
          variant="outline"
          class="inline-flex h-7 cursor-pointer items-center gap-1 rounded border px-2.5 py-1 text-[10px] whitespace-nowrap transition-all duration-200"
          :class="
            autoScroll
              ? 'text-primary border-primary bg-primary/10 hover:bg-primary/20'
              : 'text-muted-foreground bg-muted/40 border-border hover:text-foreground hover:border-primary/40'
          "
          @click="autoScroll = !autoScroll"
          title="新日志产生时自动滚动到底部"
        >
          <span
            class="h-1 w-1 rounded-full"
            :class="autoScroll ? 'bg-primary' : 'bg-muted-foreground/30'"
          ></span>
          自动滚动
        </Button>
      </div>
    </div>
  </div>
</template>
