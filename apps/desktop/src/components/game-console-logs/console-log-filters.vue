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
  <div class="border-border-subtle flex shrink-0 flex-col gap-2 border-b bg-[rgba(0,0,0,0.15)] px-3.5 py-2">
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
                  info: 'text-text-bright border-[rgba(154,146,130,0.3)] bg-[rgba(154,146,130,0.15)] hover:bg-[rgba(154,146,130,0.2)]',
                  warn: 'border-[rgba(251,191,36,0.3)] bg-[rgba(251,191,36,0.1)] text-[#fbbf24] hover:bg-[rgba(251,191,36,0.15)]',
                  error:
                    'border-[rgba(248,113,113,0.3)] bg-[rgba(248,113,113,0.1)] text-[#f87171] hover:bg-[rgba(248,113,113,0.15)]',
                  debug:
                    'border-[rgba(56,189,248,0.3)] bg-[rgba(56,189,248,0.1)] text-[#38bdf8] hover:bg-[rgba(56,189,248,0.15)]',
                }[lvl]
              : 'text-text-muted border-transparent bg-[rgba(255,255,255,0.02)] hover:bg-[rgba(255,255,255,0.05)]',
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
        class="bg-bg-deep border-gold-dimmer relative flex flex-1 items-center rounded border px-2 py-1 shadow-[inset_0_2px_4px_rgba(0,0,0,0.4)]"
      >
        <SearchIcon class="text-text-muted mr-1.5 h-3.5 w-3.5 shrink-0 opacity-50" />
        <input
          v-model="searchText"
          type="text"
          placeholder="输入检索词进行模糊或正则匹配（双击日志行可直接复制）..."
          class="text-text-bright placeholder:text-text-muted w-full bg-transparent text-xs outline-none"
        />
        <button
          v-if="searchText"
          class="text-text-muted hover:text-text-bright absolute right-2 text-sm font-semibold transition-colors duration-100"
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
              ? 'text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.06)] hover:bg-[rgba(185,145,71,0.1)]'
              : 'text-text-muted bg-bg-deep border-border-subtle hover:text-text-default hover:border-gold-muted'
          "
          @click="regexEnabled = !regexEnabled"
          title="开启后支持使用 JS 正则语法检索日志"
        >
          <span
            class="h-1 w-1 rounded-full"
            :class="regexEnabled ? 'bg-gold-default shadow-[0_0_6px_rgba(185,145,71,0.5)]' : 'bg-border-default'"
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
              ? 'text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.06)] hover:bg-[rgba(185,145,71,0.1)]'
              : 'text-text-muted bg-bg-deep border-border-subtle hover:text-text-default hover:border-gold-muted'
          "
          @click="autoRefresh = !autoRefresh"
          title="每秒自动从 SQLite 数据库轮询最新日志"
        >
          <span
            class="h-1 w-1 rounded-full"
            :class="autoRefresh ? 'bg-gold-default shadow-[0_0_6px_rgba(185,145,71,0.5)]' : 'bg-border-default'"
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
              ? 'text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.06)] hover:bg-[rgba(185,145,71,0.1)]'
              : 'text-text-muted bg-bg-deep border-border-subtle hover:text-text-default hover:border-gold-muted'
          "
          @click="autoScroll = !autoScroll"
          title="新日志产生时自动滚动到底部"
        >
          <span
            class="h-1 w-1 rounded-full"
            :class="autoScroll ? 'bg-gold-default shadow-[0_0_6px_rgba(185,145,71,0.5)]' : 'bg-border-default'"
          ></span>
          自动滚动
        </Button>
      </div>
    </div>
  </div>
</template>
