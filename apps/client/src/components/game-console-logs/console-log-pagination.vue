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
    class="border-border flex shrink-0 items-center justify-between gap-3 border-t bg-muted/20 px-4 py-2 select-none"
  >
    <div class="flex items-center gap-2">
      <span class="text-muted-foreground text-[11px] whitespace-nowrap">
        显示第
        <strong class="text-primary">{{ totalLogsCount > 0 ? (currentPage - 1) * pageSize + 1 : 0 }}</strong>
        -
        <strong class="text-primary">{{ Math.min(currentPage * pageSize, totalLogsCount) }}</strong>
        条， 共
        <strong class="text-primary">{{ totalLogsCount }}</strong>
        条
      </span>

      <span class="bg-border mx-2 h-3.5 w-[1px]"></span>

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
          class="bg-muted/40 text-foreground border-border flex h-7 w-28 items-center justify-between px-2 text-xs"
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent class="bg-popover border-border text-foreground/80">
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
        class="border-border text-muted-foreground hover:text-primary hover:border-primary/40 flex size-8 items-center justify-center bg-transparent p-0"
        :disabled="currentPage === 1"
        @click="currentPage = 1"
        title="第一页"
      >
        <ChevronsLeftIcon class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="outline"
        class="border-border text-muted-foreground hover:text-primary hover:border-primary/40 flex size-8 items-center justify-center bg-transparent p-0"
        :disabled="currentPage === 1"
        @click="currentPage--"
        title="上一页"
      >
        <ChevronLeftIcon class="h-3.5 w-3.5" />
      </Button>

      <div class="mx-1 flex items-center gap-1.5 text-[11px]">
        <span class="text-muted-foreground">页码</span>
        <span
          class="text-primary rounded border border-primary/20 bg-primary/10 px-1.5 py-0.5 font-mono font-bold shadow-sm"
        >
          {{ currentPage }}
        </span>
        <span class="text-muted-foreground">/</span>
        <span class="text-foreground/80 font-mono">{{ totalPages }}</span>
      </div>

      <Button
        variant="outline"
        class="border-border text-muted-foreground hover:text-primary hover:border-primary/40 flex size-8 items-center justify-center bg-transparent p-0"
        :disabled="currentPage === totalPages"
        @click="currentPage++"
        title="下一页"
      >
        <ChevronRightIcon class="h-3.5 w-3.5" />
      </Button>
      <Button
        variant="outline"
        class="border-border text-muted-foreground hover:text-primary hover:border-primary/40 flex size-8 items-center justify-center bg-transparent p-0"
        :disabled="currentPage === totalPages"
        @click="currentPage = totalPages"
        title="最后一页"
      >
        <ChevronsRightIcon class="h-3.5 w-3.5" />
      </Button>
    </div>
  </div>
</template>
