<script setup lang="ts">
import { computed } from "vue";
import { useLog } from "../../composables/useLogPoller";

// ── Shadcn UI Components ──
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

// ── Lucide Icons ──
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronsLeftIcon,
  ChevronsRightIcon,
} from "@lucide/vue";

const {
  totalLogsCount,
  currentPage,
  pageSize,
} = useLog();

const totalPages = computed(() => Math.ceil(totalLogsCount.value / pageSize.value) || 1);
</script>

<template>
  <!-- 分页控制条 -->
  <div
    class="border-border-subtle flex shrink-0 items-center justify-between gap-3 border-t bg-[rgba(0,0,0,0.18)] px-4 py-2 select-none"
  >
    <div class="flex items-center gap-2">
      <span class="text-text-muted text-[11px] whitespace-nowrap">
        显示第
        <strong class="text-gold-bright">{{ totalLogsCount > 0 ? (currentPage - 1) * pageSize + 1 : 0 }}</strong>
        -
        <strong class="text-gold-bright">{{ Math.min(currentPage * pageSize, totalLogsCount) }}</strong>
        条， 共
        <strong class="text-gold-bright">{{ totalLogsCount }}</strong>
        条
      </span>

      <span class="bg-border-subtle mx-2 h-3.5 w-[1px]"></span>

      <!-- PageSize Select -->
      <Select
        :model-value="pageSize.toString()"
        @update:model-value="
          (val) => {
            pageSize = Number(val);
            currentPage = 1;
          }
        "
      >
        <SelectTrigger
          class="bg-bg-deep text-text-bright border-gold-dimmer flex h-7 w-28 items-center justify-between px-2 text-xs"
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent class="bg-bg-surface border-border-subtle text-text-default">
          <SelectItem class="text-xs" value="50">50 条/页</SelectItem>
          <SelectItem class="text-xs" value="100">100 条/页</SelectItem>
          <SelectItem class="text-xs" value="200">200 条/页</SelectItem>
          <SelectItem class="text-xs" value="500">500 条/页</SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div class="flex items-center gap-2">
      <Button
        variant="outline"
        class="border-border-subtle text-text-muted hover:text-gold-bright hover:border-gold-muted flex size-8 items-center justify-center bg-[rgba(255,255,255,0.01)] p-0"
        :disabled="currentPage === 1"
        @click="currentPage = 1"
        title="第一页"
      >
        <ChevronsLeftIcon class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="outline"
        class="border-border-subtle text-text-muted hover:text-gold-bright hover:border-gold-muted flex size-8 items-center justify-center bg-[rgba(255,255,255,0.01)] p-0"
        :disabled="currentPage === 1"
        @click="currentPage--"
        title="上一页"
      >
        <ChevronLeftIcon class="h-3.5 w-3.5" />
      </Button>

      <div class="mx-1 flex items-center gap-1.5 text-[11px]">
        <span class="text-text-muted">页码</span>
        <span
          class="text-gold-bright rounded border border-[rgba(185,145,71,0.15)] bg-[rgba(185,145,71,0.06)] px-1.5 py-0.5 font-mono font-bold shadow-[0_0_8px_rgba(185,145,71,0.2)]"
        >
          {{ currentPage }}
        </span>
        <span class="text-text-muted">/</span>
        <span class="text-text-default font-mono">{{ totalPages }}</span>
      </div>

      <Button
        variant="outline"
        class="border-border-subtle text-text-muted hover:text-gold-bright hover:border-gold-muted flex size-8 items-center justify-center bg-[rgba(255,255,255,0.01)] p-0"
        :disabled="currentPage === totalPages"
        @click="currentPage++"
        title="下一页"
      >
        <ChevronRightIcon class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="outline"
        class="border-border-subtle text-text-muted hover:text-gold-bright hover:border-gold-muted flex size-8 items-center justify-center bg-[rgba(255,255,255,0.01)] p-0"
        :disabled="currentPage === totalPages"
        @click="currentPage = totalPages"
        title="最后一页"
      >
        <ChevronsRightIcon class="h-3.5 w-3.5" />
      </Button>
    </div>
  </div>
</template>
