<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { services } from "@/services/provider";
import type { EloRating } from "@/services/types";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { CrownIcon, TrophyIcon } from "@lucide/vue";

// ELO 排行榜：总榜 + 日增榜 双 tab 切换，按模式过滤。
// 数据特点：排序后的扁平结构 — 表格最合适，重点放在 rank/rating/delta 三列。

const mode = ref("top_solo");
const view = ref<"total" | "daily">("total");
const data = ref<EloRating[]>([]);
const loading = ref(true);

const MODES = [{ value: "top_solo", label: "上单 SOLO" }];

async function refresh() {
  loading.value = true;
  try {
    data.value = await services.cloud.getLeaderboard(mode.value, 100);
  } catch (e) {
    data.value = [];
  } finally {
    loading.value = false;
  }
}

const sorted = computed(() => {
  const arr = data.value.slice();
  if (view.value === "daily") {
    arr.sort((a, b) => b.daily_delta - a.daily_delta);
  } else {
    arr.sort((a, b) => b.rating - a.rating);
  }
  return arr;
});

function tierMedal(idx: number): string {
  if (idx === 0) return "text-yellow-500";
  if (idx === 1) return "text-zinc-400";
  if (idx === 2) return "text-amber-700";
  return "text-muted-foreground";
}

watch(mode, refresh);
onMounted(refresh);
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-4xl flex-col gap-6 px-8 py-8">
    <header class="space-y-1">
      <h1 class="flex items-center gap-2 text-2xl font-semibold tracking-tight">
        <TrophyIcon class="size-6" />
        排行榜
      </h1>
      <p class="text-muted-foreground text-sm">Agent × 模式 维度的 ELO 排名</p>
    </header>

    <div class="flex items-center justify-between gap-3">
      <Tabs v-model="view">
        <TabsList>
          <TabsTrigger value="total">总排行</TabsTrigger>
          <TabsTrigger value="daily">今日增量</TabsTrigger>
        </TabsList>
      </Tabs>

      <Select v-model="mode">
        <SelectTrigger class="w-40">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="m in MODES" :key="m.value" :value="m.value">{{ m.label }}</SelectItem>
        </SelectContent>
      </Select>
    </div>

    <div v-if="loading" class="text-muted-foreground py-12 text-center text-sm">加载中…</div>
    <div v-else-if="sorted.length === 0" class="text-muted-foreground py-12 text-center text-sm">
      暂无上榜 Agent
    </div>
    <div v-else class="overflow-hidden rounded-lg border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead class="w-14 text-center">#</TableHead>
            <TableHead>Agent</TableHead>
            <TableHead class="text-right">{{ view === "daily" ? "今日 Δ" : "ELO" }}</TableHead>
            <TableHead class="text-right">胜 / 负</TableHead>
            <TableHead class="text-right">胜率</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="(r, idx) in sorted" :key="r.agent_id">
            <TableCell class="text-center">
              <CrownIcon v-if="idx < 3" :class="['mx-auto size-4', tierMedal(idx)]" />
              <span v-else class="text-muted-foreground tabular-nums">{{ idx + 1 }}</span>
            </TableCell>
            <TableCell>
              <div class="flex items-center gap-2">
                <span class="font-medium">{{ r.agent_name }}</span>
                <Badge variant="outline" class="text-[10px]">{{ r.mode }}</Badge>
              </div>
            </TableCell>
            <TableCell class="text-right font-mono tabular-nums">
              <template v-if="view === 'daily'">
                <span :class="r.daily_delta >= 0 ? 'text-emerald-600' : 'text-destructive'">
                  {{ r.daily_delta >= 0 ? "+" : "" }}{{ r.daily_delta }}
                </span>
              </template>
              <template v-else>
                {{ r.rating }}
              </template>
            </TableCell>
            <TableCell class="text-muted-foreground text-right tabular-nums">
              {{ r.wins }} / {{ r.losses }}
            </TableCell>
            <TableCell class="text-right tabular-nums">
              {{ r.games_played > 0 ? ((r.wins / r.games_played) * 100).toFixed(1) : "—" }}%
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  </div>
</template>
