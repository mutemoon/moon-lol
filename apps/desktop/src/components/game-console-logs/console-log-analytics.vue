<script setup lang="ts">
import { computed } from "vue";
import { useLog } from "../../composables/useLogPoller";
import { Badge } from "../ui/badge";
import {
  BarChart3Icon,
  FolderIcon,
  BotIcon
} from "@lucide/vue";

defineEmits(["close"]);

const log = useLog();
const {
  filterEntityId,
  filterCategory,
  selectedLevels
} = log;

// ── 计算日志统计分析数据 ──
const statsData = computed(() => {
  const allLogs = log.logs.value || [];
  const total = allLogs.length;
  if (total === 0) {
    return {
      levels: { info: 0, warn: 0, error: 0, debug: 0 },
      entities: {} as Record<string, { name: string; count: number }>,
      categories: {} as Record<string, number>
    };
  }

  const levels = { info: 0, warn: 0, error: 0, debug: 0 };
  const entities: Record<string, { name: string; count: number }> = {};
  const categories: Record<string, number> = {};

  allLogs.forEach((item) => {
    // 1. Level Stats
    const lvl = item.level.toLowerCase() as keyof typeof levels;
    if (lvl in levels) {
      levels[lvl]++;
    }

    // 2. Entity Stats
    if (item.entity_id) {
      if (!entities[item.entity_id]) {
        entities[item.entity_id] = { name: item.entity_name || "Unknown", count: 0 };
      }
      entities[item.entity_id].count++;
    }

    // 3. Category Stats
    if (item.category) {
      categories[item.category] = (categories[item.category] || 0) + 1;
    }
  });

  return { levels, entities, categories, total };
});

const levelsStats = computed(() => {
  const total = statsData.value.total || 1;
  const lvls = statsData.value.levels;
  return (Object.keys(lvls) as Array<keyof typeof lvls>).map((key) => {
    const count = lvls[key];
    const percent = Math.round((count / total) * 100);
    return { level: key, count, percent };
  });
});

const topEntities = computed(() => {
  const total = statsData.value.total || 1;
  const ents = statsData.value.entities;
  return Object.keys(ents)
    .map((id) => {
      const data = ents[id];
      const percent = Math.round((data.count / total) * 100);
      return { id, name: data.name, count: data.count, percent };
    })
    .sort((a, b) => b.count - a.count)
    .slice(0, 5);
});

const topCategories = computed(() => {
  const total = statsData.value.total || 1;
  const cats = statsData.value.categories;
  return Object.keys(cats)
    .map((category) => {
      const count = cats[category];
      const percent = Math.round((count / total) * 100);
      return { category, count, percent };
    })
    .sort((a, b) => b.count - a.count)
    .slice(0, 5);
});

// ── 过滤触发器 ──
function toggleLevel(level: string) {
  const index = selectedLevels.value.indexOf(level);
  if (index > -1) {
    selectedLevels.value.splice(index, 1);
  } else {
    selectedLevels.value.push(level);
  }
}

function setEntityFilter(id: string | null) {
  if (id === null) {
    filterEntityId.value = null;
  } else {
    filterEntityId.value = Number(id);
  }
}

// category filter is local string or null
function setCategoryFilter(cat: string | null) {
  filterCategory.value = cat;
}
</script>

<template>
  <!-- 可折叠的日志多维分析面板 -->
  <div
    class="bg-card border-border flex flex-col gap-2.5 overflow-hidden border-b p-3 shadow-inner"
  >
    <div class="flex items-center justify-between border-b border-border pb-1">
      <span class="text-primary text-[10px] font-extrabold tracking-widest uppercase inline-flex items-center gap-1">
        <BarChart3Icon class="h-3.5 w-3.5" />
        运行日志智能分析面板
      </span>
      <button
        class="text-muted-foreground hover:text-foreground cursor-pointer text-[10px]"
        @click="$emit('close')"
      >
        收起 ✕
      </button>
    </div>

    <div class="grid grid-cols-3 gap-3">
      <!-- 分析1: 级别比例 -->
      <div class="bg-muted/30 border-border flex min-w-0 flex-col gap-2 rounded border p-2">
        <span class="text-muted-foreground text-[9px] font-bold tracking-wider uppercase">级别分布比例</span>
        <div class="flex flex-col gap-1.5">
          <div
            v-for="stat in levelsStats"
            :key="stat.level"
            class="flex cursor-pointer items-center justify-between gap-2.5 rounded px-1 py-0.5 transition-colors duration-150 hover:bg-muted/20"
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
                    info: 'text-muted-foreground bg-muted/40',
                    warn: 'bg-yellow-500/10 text-yellow-600',
                    error: 'bg-destructive/10 text-destructive',
                    debug: 'bg-blue-500/10 text-blue-500',
                  }[stat.level]
                "
              >
                {{ stat.level.toUpperCase() }}
              </Badge>
            </div>
            <div class="h-1 flex-1 overflow-hidden rounded-full bg-muted">
              <div
                class="h-full rounded-full transition-[width] duration-500 ease-out"
                :class="
                  {
                    info: 'bg-muted-foreground',
                    warn: 'bg-yellow-500',
                    error: 'bg-destructive',
                    debug: 'bg-blue-500',
                  }[stat.level]
                "
                :style="{ width: stat.percent + '%' }"
              ></div>
            </div>
            <div class="flex min-w-[42px] items-center justify-end gap-1 font-mono text-[9px]">
              <span class="text-muted-foreground">{{ stat.count }}条</span>
              <span class="text-foreground font-bold">{{ stat.percent }}%</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 分析2: 最活跃实体 -->
      <div class="bg-muted/30 border-border flex min-w-0 flex-col gap-2 rounded border p-2">
        <span class="text-muted-foreground text-[9px] font-bold tracking-wider uppercase">
          高频活跃实体排行榜 (Top 5)
        </span>
        <div
          v-if="topEntities.length === 0"
          class="text-muted-foreground flex items-center justify-center py-4 text-[10px] italic"
        >
          暂无带实体 ID 的日志...
        </div>
        <div v-else class="flex flex-col gap-1.5">
          <div
            v-for="ent in topEntities"
            :key="ent.id"
            class="flex cursor-pointer items-center justify-between gap-2.5 rounded px-1 py-0.5 transition-colors duration-150 hover:bg-muted/20"
            :class="{
              'border border-primary/20 bg-primary/10': filterEntityId === Number(ent.id),
            }"
            @click="setEntityFilter(filterEntityId === Number(ent.id) ? null : ent.id)"
            title="点击直接过滤该实体，再次点击取消"
          >
            <div class="flex min-w-[55px] items-center gap-1.5 truncate text-[9px] font-bold" :title="ent.name">
              <Badge
                variant="outline"
                class="text-primary border-primary/30 inline-flex h-4 max-w-full items-center truncate bg-primary/10 px-1.5 py-0 text-[9px] font-semibold gap-1"
              >
                <BotIcon class="h-2.5 w-2.5 shrink-0 text-primary" /> [{{ ent.id }}] {{ ent.name }}
              </Badge>
            </div>
            <div class="h-1 flex-1 overflow-hidden rounded-full bg-muted">
              <div
                class="bg-primary h-full rounded-full transition-[width] duration-500 ease-out"
                :style="{ width: ent.percent + '%' }"
              ></div>
            </div>
            <div class="flex min-w-[42px] items-center justify-end gap-1 font-mono text-[9px]">
              <span class="text-muted-foreground">{{ ent.count }}条</span>
              <span class="text-foreground font-bold">{{ ent.percent }}%</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 分析3: 活跃模块 -->
      <div class="bg-muted/30 border-border flex min-w-0 flex-col gap-2 rounded border p-2">
        <span class="text-muted-foreground text-[9px] font-bold tracking-wider uppercase">系统模块活跃度 (Top 5)</span>
        <div
          v-if="topCategories.length === 0"
          class="text-muted-foreground flex items-center justify-center py-4 text-[10px] italic"
        >
          暂无带系统分类的日志...
        </div>
        <div v-else class="flex flex-col gap-1.5">
          <div
            v-for="cat in topCategories"
            :key="cat.category"
            class="flex cursor-pointer items-center justify-between gap-2.5 rounded px-1 py-0.5 transition-colors duration-150 hover:bg-muted/20"
            :class="{
              'border border-primary/20 bg-primary/10': filterCategory === cat.category,
            }"
            @click="setCategoryFilter(filterCategory === cat.category ? null : cat.category)"
            title="点击直接过滤该模块，再次点击取消"
          >
            <div class="flex min-w-[55px] items-center gap-1.5 truncate text-[9px] font-bold" :title="cat.category">
              <Badge
                variant="outline"
                class="text-foreground/80 border-border inline-flex h-4 max-w-full items-center truncate bg-muted/30 px-1.5 py-0 text-[9px] gap-1"
              >
                <FolderIcon class="h-2.5 w-2.5 shrink-0 text-muted-foreground" /> {{ cat.category }}
              </Badge>
            </div>
            <div class="h-1 flex-1 overflow-hidden rounded-full bg-muted">
              <div
                class="bg-primary h-full rounded-full transition-[width] duration-500 ease-out"
                :style="{ width: cat.percent + '%' }"
              ></div>
            </div>
            <div class="flex min-w-[42px] items-center justify-end gap-1 font-mono text-[9px]">
              <span class="text-muted-foreground">{{ cat.count }}条</span>
              <span class="text-foreground font-bold">{{ cat.percent }}%</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
