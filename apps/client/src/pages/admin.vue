<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { adminApi, type AdminMetrics, type Match } from "@/services/cloudApi";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Separator } from "@/components/ui/separator";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { CpuIcon, MemoryStickIcon, ActivityIcon, OctagonXIcon } from "@lucide/vue";

// 服务器对局池管理面板。
// 数据特点：少量指标摘要 + 一张可操作的对局明细表。
// 顶部用 stat 列，明细用 table；不用花哨图表。

const metrics = ref<AdminMetrics | null>(null);
const running = ref<Match[]>([]);
const loading = ref(true);
let timer: number | null = null;

const abortTarget = ref<Match | null>(null);
const aborting = ref(false);

async function refresh() {
  try {
    const [m, r] = await Promise.all([
      adminApi.metrics().catch(() => null),
      adminApi.listRunning().catch(() => [] as Match[]),
    ]);
    metrics.value = m;
    running.value = r;
  } finally {
    loading.value = false;
  }
}

async function confirmAbort() {
  if (!abortTarget.value) return;
  aborting.value = true;
  try {
    await adminApi.forceAbort(abortTarget.value.id);
    abortTarget.value = null;
    await refresh();
  } catch (e: any) {
    console.error(e);
  } finally {
    aborting.value = false;
  }
}

const memoryPct = computed(() => {
  if (!metrics.value) return 0;
  // 假定 8GB 上限
  return Math.min(100, Math.round((metrics.value.total_memory_mb / 8192) * 100));
});

function shortId(id: string) {
  return id.slice(0, 8);
}

function ago(iso: string) {
  const diff = (Date.now() - new Date(iso).getTime()) / 1000;
  if (diff < 60) return `${Math.floor(diff)}s`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m`;
  return `${Math.floor(diff / 3600)}h`;
}

onMounted(() => {
  refresh();
  timer = window.setInterval(refresh, 3000);
});
onUnmounted(() => {
  if (timer) clearInterval(timer);
});
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-6xl flex-col gap-6 px-8 py-6">
    <header class="space-y-1">
      <h1 class="text-2xl font-semibold tracking-tight">对局池监控</h1>
      <p class="text-muted-foreground text-sm">服务器并发对局算力与内存调度（实时刷新）</p>
    </header>

    <!-- Stat 行 -->
    <section class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
      <div class="space-y-1">
        <div class="text-muted-foreground flex items-center gap-1.5 text-xs">
          <ActivityIcon class="size-3.5" />
          运行中对局
        </div>
        <div class="text-3xl font-semibold tabular-nums">{{ metrics?.running_matches ?? "—" }}</div>
      </div>

      <div class="space-y-1">
        <div class="text-muted-foreground flex items-center gap-1.5 text-xs">
          <MemoryStickIcon class="size-3.5" />
          总内存占用
        </div>
        <div class="text-3xl font-semibold tabular-nums">
          {{ metrics ? `${metrics.total_memory_mb} MB` : "—" }}
        </div>
        <Progress :model-value="memoryPct" class="h-1" />
      </div>

      <div class="space-y-1">
        <div class="text-muted-foreground text-xs">平均内存/局</div>
        <div class="text-3xl font-semibold tabular-nums">
          {{ metrics ? `${metrics.avg_match_memory_mb} MB` : "—" }}
        </div>
      </div>

      <div class="space-y-1">
        <div class="text-muted-foreground flex items-center gap-1.5 text-xs">
          <CpuIcon class="size-3.5" />
          CPU 使用率
        </div>
        <div class="text-3xl font-semibold tabular-nums">
          {{ metrics ? `${metrics.cpu_usage_percent.toFixed(1)}%` : "—" }}
        </div>
        <Progress :model-value="metrics?.cpu_usage_percent ?? 0" class="h-1" />
      </div>
    </section>

    <Separator />

    <!-- 对局明细 -->
    <section class="space-y-3">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold">进行中对局</h2>
        <Badge variant="outline">{{ running.length }}</Badge>
      </div>

      <div v-if="loading" class="text-muted-foreground py-12 text-center text-sm">加载中…</div>
      <div v-else-if="running.length === 0" class="text-muted-foreground py-12 text-center text-sm">
        当前没有运行中的对局
      </div>
      <div v-else class="overflow-hidden rounded-lg border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead class="w-[160px]">对局 ID</TableHead>
              <TableHead>模式</TableHead>
              <TableHead>所属</TableHead>
              <TableHead>端口</TableHead>
              <TableHead>已运行</TableHead>
              <TableHead class="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="m in running" :key="m.id">
              <TableCell class="font-mono text-xs">{{ shortId(m.id) }}</TableCell>
              <TableCell>
                <Badge variant="secondary">{{ m.mode }}</Badge>
              </TableCell>
              <TableCell class="text-muted-foreground text-xs">
                {{ m.room_id ? `房间 ${shortId(m.room_id)}` : `用户 #${m.owner_user_id ?? "—"}` }}
              </TableCell>
              <TableCell class="font-mono text-xs">{{ m.ws_port ?? "—" }}</TableCell>
              <TableCell class="text-muted-foreground text-xs">{{ ago(m.created_at) }}</TableCell>
              <TableCell class="text-right">
                <Button
                  variant="ghost"
                  size="sm"
                  class="text-destructive hover:text-destructive"
                  @click="abortTarget = m"
                >
                  <OctagonXIcon class="size-3.5" />
                  强制中止
                </Button>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
    </section>

    <Dialog :open="!!abortTarget" @update:open="(v) => !v && (abortTarget = null)">
      <DialogContent class="max-w-sm">
        <DialogHeader>
          <DialogTitle>强制中止对局</DialogTitle>
          <DialogDescription>
            该对局将被立即终止并释放算力。此操作不可恢复。
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="ghost" @click="abortTarget = null">取消</Button>
          <Button variant="destructive" :disabled="aborting" @click="confirmAbort">确认中止</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
