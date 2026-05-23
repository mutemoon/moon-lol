<script setup lang="ts">
import { computed } from "vue";
import { useLog } from "../../composables/useLogPoller";

// ── Shadcn UI Components ──
import { Badge } from "@/components/ui/badge";

// ── Lucide Icons ──
import { BarChart3Icon, BotIcon, FolderIcon } from "@lucide/vue";

const {
  logs,
  selectedLevels,
  filterEntityId,
  filterCategory,
  setEntityFilter,
} = useLog();

defineEmits<{
  (e: "close"): void;
}>();

// ── 智能分析数据统计 ──
const levelsStats = computed(() => {
  const stats: Record<string, number> = { info: 0, warn: 0, error: 0, debug: 0 };
  logs.value.forEach((l) => {
    if (stats[l.level] !== undefined) {
      stats[l.level]++;
    }
  });
  const total = logs.value.length || 1;
  return Object.entries(stats).map(([level, count]) => ({
    level,
    count,
    percent: Math.round((count / total) * 100),
  }));
});

const topEntities = computed(() => {
  const counts: Record<string, { id: number; name: string; count: number }> = {};
  logs.value.forEach((l) => {
    if (l.entity_id !== undefined && l.entity_name) {
      const key = `${l.entity_id}_${l.entity_name}`;
      if (!counts[key]) {
        counts[key] = { id: l.entity_id, name: l.entity_name, count: 0 };
      }
      counts[key].count++;
    }
  });
  const sorted = Object.values(counts).sort((a, b) => b.count - a.count);
  const total = sorted.reduce((sum, item) => sum + item.count, 0) || 1;
  return sorted.slice(0, 5).map((item) => ({
    ...item,
    percent: Math.round((item.count / total) * 100),
  }));
});

const topCategories = computed(() => {
  const counts: Record<string, number> = {};
  logs.value.forEach((l) => {
    if (l.category) {
      counts[l.category] = (counts[l.category] || 0) + 1;
    }
  });
  const sorted = Object.entries(counts).sort((a, b) => b[1] - a[1]);
  const total = sorted.reduce((sum, item) => sum + item[1], 0) || 1;
  return sorted.slice(0, 5).map(([category, count]) => ({
    category,
    count,
    percent: Math.round((count / total) * 100),
  }));
});

function toggleLevel(level: string) {
  const current = [...selectedLevels.value];
  if (current.includes(level)) {
    selectedLevels.value = current.filter((x) => x !== level);
  } else {
    selectedLevels.value = [...current, level];
  }
}

function setCategoryFilter(cat: string | null) {
  filterCategory.value = cat;
}
</script>

<template>
  <!-- 可折叠的日志多维分析面板 -->
  <div
    class="bg-bg-surface border-border-subtle flex flex-col gap-2.5 overflow-hidden border-b p-3 shadow-[inset_0_1px_4px_rgba(0,0,0,0.5)]"
  >
    <div class="flex items-center justify-between border-b border-[rgba(255,255,255,0.02)] pb-1">
      <span class="text-gold-bright text-[10px] font-extrabold tracking-widest uppercase inline-flex items-center gap-1">
        <BarChart3Icon class="h-3.5 w-3.5" />
        运行日志智能分析面板
      </span>
      <button
        class="text-text-muted hover:text-text-bright cursor-pointer text-[10px]"
        @click="$emit('close')"
      >
        收起 ✕
      </button>
    </div>

    <div class="grid grid-cols-3 gap-3">
      <!-- 分析1: 级别比例 -->
      <div class="bg-bg-deep border-border-subtle flex min-w-0 flex-col gap-2 rounded border p-2">
        <span class="text-text-muted text-[9px] font-bold tracking-wider uppercase">级别分布比例</span>
        <div class="flex flex-col gap-1.5">
          <div
            v-for="stat in levelsStats"
            :key="stat.level"
            class="flex cursor-pointer items-center justify-between gap-2.5 rounded px-1 py-0.5 transition-colors duration-150 hover:bg-[rgba(255,255,255,0.02)]"
            :class="[!selectedLevels.includes(stat.level) ? 'opacity-35' : '']"
            @click="toggleLevel(stat.level)"
            title="点击直接启用/禁用该级别的过滤"
          >
            <div class="flex min-w-[55px] items-center gap-1.5 text-[9px] font-bold">
              <Badge
                variant="outline"
                class="inline-flex h-4 items-center gap-1 border-none px-1.5 py-0 text-[9px] font-bold"
                :class="
                  {
                    info: 'text-text-muted bg-[rgba(154,146,130,0.08)]',
                    warn: 'bg-[rgba(251,191,36,0.12)] text-[#fbbf24]',
                    error: 'bg-[rgba(248,113,113,0.15)] text-[#f87171]',
                    debug: 'bg-[rgba(56,189,248,0.12)] text-[#38bdf8]',
                  }[stat.level]
                "
              >
                {{ stat.level.toUpperCase() }}
              </Badge>
            </div>
            <div class="h-1 flex-1 overflow-hidden rounded-full bg-[rgba(255,255,255,0.03)]">
              <div
                class="h-full rounded-full transition-[width] duration-500 ease-out"
                :class="
                  {
                    info: 'bg-text-muted',
                    warn: 'bg-[#fbbf24]',
                    error: 'bg-[#f87171]',
                    debug: 'bg-[#38bdf8]',
                  }[stat.level]
                "
                :style="{ width: stat.percent + '%' }"
              ></div>
            </div>
            <div class="flex min-w-[42px] items-center justify-end gap-1 font-mono text-[9px]">
              <span class="text-text-muted">{{ stat.count }}条</span>
              <span class="text-text-bright font-bold">{{ stat.percent }}%</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 分析2: 最活跃实体 -->
      <div class="bg-bg-deep border-border-subtle flex min-w-0 flex-col gap-2 rounded border p-2">
        <span class="text-text-muted text-[9px] font-bold tracking-wider uppercase">
          高频活跃实体排行榜 (Top 5)
        </span>
        <div
          v-if="topEntities.length === 0"
          class="text-text-muted flex items-center justify-center py-4 text-[10px] italic"
        >
          暂无带实体 ID 的日志...
        </div>
        <div v-else class="flex flex-col gap-1.5">
          <div
            v-for="ent in topEntities"
            :key="ent.id"
            class="flex cursor-pointer items-center justify-between gap-2.5 rounded px-1 py-0.5 transition-colors duration-150 hover:bg-[rgba(255,255,255,0.02)]"
            :class="{
              'border border-[rgba(185,145,71,0.15)] bg-[rgba(185,145,71,0.08)]': filterEntityId === ent.id,
            }"
            @click="setEntityFilter(filterEntityId === ent.id ? null : ent.id)"
            title="点击直接过滤该实体，再次点击取消"
          >
            <div class="flex min-w-[55px] items-center gap-1.5 truncate text-[9px] font-bold" :title="ent.name">
              <Badge
                variant="outline"
                class="text-gold-bright border-gold-dimmer inline-flex h-4 max-w-full items-center truncate bg-[rgba(185,145,71,0.08)] px-1.5 py-0 text-[9px] font-semibold gap-1"
              >
                <BotIcon class="h-2.5 w-2.5 shrink-0 text-gold-bright" /> [{{ ent.id }}] {{ ent.name }}
              </Badge>
            </div>
            <div class="h-1 flex-1 overflow-hidden rounded-full bg-[rgba(255,255,255,0.03)]">
              <div
                class="bg-gold-default h-full rounded-full transition-[width] duration-500 ease-out"
                :style="{ width: ent.percent + '%' }"
              ></div>
            </div>
            <div class="flex min-w-[42px] items-center justify-end gap-1 font-mono text-[9px]">
              <span class="text-text-muted">{{ ent.count }}条</span>
              <span class="text-text-bright font-bold">{{ ent.percent }}%</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 分析3: 活跃模块 -->
      <div class="bg-bg-deep border-border-subtle flex min-w-0 flex-col gap-2 rounded border p-2">
        <span class="text-text-muted text-[9px] font-bold tracking-wider uppercase">系统模块活跃度 (Top 5)</span>
        <div
          v-if="topCategories.length === 0"
          class="text-text-muted flex items-center justify-center py-4 text-[10px] italic"
        >
          暂无带系统分类的日志...
        </div>
        <div v-else class="flex flex-col gap-1.5">
          <div
            v-for="cat in topCategories"
            :key="cat.category"
            class="flex cursor-pointer items-center justify-between gap-2.5 rounded px-1 py-0.5 transition-colors duration-150 hover:bg-[rgba(255,255,255,0.02)]"
            :class="{
              'border border-[rgba(185,145,71,0.15)] bg-[rgba(185,145,71,0.08)]': filterCategory === cat.category,
            }"
            @click="setCategoryFilter(filterCategory === cat.category ? null : cat.category)"
            title="点击直接过滤该模块，再次点击取消"
          >
            <div class="flex min-w-[55px] items-center gap-1.5 truncate text-[9px] font-bold" :title="cat.category">
              <Badge
                variant="outline"
                class="text-text-default border-border-subtle inline-flex h-4 max-w-full items-center truncate bg-[rgba(255,255,255,0.04)] px-1.5 py-0 text-[9px] gap-1"
              >
                <FolderIcon class="h-2.5 w-2.5 shrink-0 text-text-muted" /> {{ cat.category }}
              </Badge>
            </div>
            <div class="h-1 flex-1 overflow-hidden rounded-full bg-[rgba(255,255,255,0.03)]">
              <div
                class="bg-blue h-full rounded-full transition-[width] duration-500 ease-out"
                :style="{ width: cat.percent + '%' }"
              ></div>
            </div>
            <div class="flex min-w-[42px] items-center justify-end gap-1 font-mono text-[9px]">
              <span class="text-text-muted">{{ cat.count }}条</span>
              <span class="text-text-bright font-bold">{{ cat.percent }}%</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
