<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { matchesApi, type Match, type MatchEvent } from "@/services/cloudApi";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  ArrowLeftIcon,
  PauseIcon,
  PlayIcon,
  AlertTriangleIcon,
  CircleDotIcon,
  SquareIcon,
} from "@lucide/vue";

// 观战与回放：操作流时间线 + 状态摘要。
// 数据特点：高频时序事件 + 关键状态指示（BYO Agent 掉线、对局暂停等）。
// 主区域用三段：摘要带 / 阵营状态对照 / 事件时间线。

const route = useRoute();
const router = useRouter();
const matchId = computed(() => String((route.params as any).id));

const match = ref<Match | null>(null);
const events = ref<MatchEvent[]>([]);
const loading = ref(true);
const paused = ref(false);
let pollTimer: number | null = null;
let lastSeq = 0;

// 模拟 BYO 异常（实际由服务端事件流上报，前端只展示）
const stalledAgents = computed(() => {
  const set = new Set<string>();
  for (const e of events.value) {
    if (e.payload?.type === "agent_stalled") set.add(e.payload.agent_id);
    if (e.payload?.type === "agent_resumed") set.delete(e.payload.agent_id);
  }
  return Array.from(set);
});

const orderAgents = computed(() => {
  // 真实场景下应从 match.snapshot 取阵容；此处用事件回填
  const m = new Map<string, { name: string; champion: string; alive: boolean }>();
  for (const e of events.value) {
    if (e.payload?.type === "agent_join" && e.payload.team === "order") {
      m.set(e.payload.agent_id, {
        name: e.payload.name,
        champion: e.payload.champion,
        alive: true,
      });
    }
  }
  return Array.from(m, ([id, v]) => ({ id, ...v }));
});

const chaosAgents = computed(() => {
  const m = new Map<string, { name: string; champion: string; alive: boolean }>();
  for (const e of events.value) {
    if (e.payload?.type === "agent_join" && e.payload.team === "chaos") {
      m.set(e.payload.agent_id, {
        name: e.payload.name,
        champion: e.payload.champion,
        alive: true,
      });
    }
  }
  return Array.from(m, ([id, v]) => ({ id, ...v }));
});

async function refresh() {
  if (paused.value) return;
  try {
    if (!match.value) {
      match.value = await matchesApi.get(matchId.value);
    }
    const delta = await matchesApi.getEvents(matchId.value, lastSeq, 200);
    if (delta.length) {
      events.value.push(...delta);
      const last = delta[delta.length - 1];
      if (last) lastSeq = last.seq + 1;
    }
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

function eventLabel(e: MatchEvent): string {
  const t = e.payload?.type as string | undefined;
  switch (t) {
    case "agent_join":
      return `${e.payload.name} (${e.payload.team}) 加入对局`;
    case "minion_kill":
      return `${e.payload.agent_name} 补刀 +1`;
    case "champion_kill":
      return `${e.payload.killer} 击杀 ${e.payload.victim}`;
    case "turret_destroyed":
      return `${e.payload.team} 推掉 ${e.payload.lane} 一塔`;
    case "agent_stalled":
      return `⚠ ${e.payload.agent_name} 动力源失联`;
    case "agent_resumed":
      return `${e.payload.agent_name} 恢复连接`;
    case "match_finished":
      return `对局结束，胜方 ${e.payload.winner}`;
    default:
      return t || "event";
  }
}

function eventTone(e: MatchEvent): string {
  const t = e.payload?.type;
  if (t === "agent_stalled") return "text-amber-600 dark:text-amber-400";
  if (t === "champion_kill" || t === "turret_destroyed") return "text-foreground font-medium";
  if (t === "match_finished") return "text-foreground font-semibold";
  return "text-muted-foreground";
}

async function stopMatch() {
  if (!confirm("结束此对局？")) return;
  try {
    await matchesApi.stop(matchId.value);
    await refresh();
  } catch (e) {
    console.error(e);
  }
}

onMounted(() => {
  refresh();
  pollTimer = window.setInterval(refresh, 1000);
});
onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<template>
  <div class="flex h-full w-full flex-col">
    <!-- 头部 -->
    <header class="flex shrink-0 items-center justify-between gap-3 px-6 py-4">
      <div class="flex items-center gap-2">
        <Button variant="ghost" size="icon" @click="router.back()">
          <ArrowLeftIcon class="size-4" />
        </Button>
        <div class="space-y-0.5">
          <div class="flex items-center gap-2">
            <h1 class="text-lg font-semibold">观战</h1>
            <span class="text-muted-foreground font-mono text-xs">{{ matchId.slice(0, 8) }}</span>
            <Badge v-if="match" variant="outline">{{ match.mode }}</Badge>
            <Badge v-if="match?.status === 'running'" class="gap-1">
              <span class="bg-emerald-500 size-1.5 animate-pulse rounded-full" />
              直播中
            </Badge>
            <Badge v-else-if="match?.status === 'aborted'" variant="destructive">已中止</Badge>
            <Badge v-else-if="match?.status === 'finished'" variant="secondary">已结束</Badge>
          </div>
        </div>
      </div>

      <div class="flex gap-2">
        <Button variant="outline" size="sm" @click="paused = !paused">
          <component :is="paused ? PlayIcon : PauseIcon" class="size-3.5" />
          {{ paused ? "继续刷新" : "暂停刷新" }}
        </Button>
        <Button
          v-if="match?.status === 'running'"
          variant="ghost"
          size="sm"
          class="text-destructive"
          @click="stopMatch"
        >
          <SquareIcon class="size-3.5" />
          结束对局
        </Button>
      </div>
    </header>

    <!-- BYO Agent 掉线告警 -->
    <div
      v-if="stalledAgents.length > 0"
      class="mx-6 mb-2 flex items-start gap-2 rounded-lg border border-amber-500/40 bg-amber-50 px-4 py-3 text-xs text-amber-900 dark:bg-amber-950/30 dark:text-amber-200"
    >
      <AlertTriangleIcon class="mt-0.5 size-4 shrink-0" />
      <div class="space-y-0.5">
        <p class="font-medium">部分 Agent 动力源失联，对局已暂停等待恢复</p>
        <p class="text-amber-700 dark:text-amber-400/80">
          失联 Agent：{{ stalledAgents.map((id) => id.slice(0, 8)).join("、") }}
        </p>
      </div>
    </div>

    <Separator />

    <!-- 主体：两栏 -->
    <div class="grid min-h-0 flex-1 grid-cols-1 gap-6 px-6 py-4 lg:grid-cols-[1fr_360px]">
      <!-- 左：阵营对照 + 渲染占位 -->
      <section class="flex min-h-0 flex-col gap-4">
        <!-- 阵营对照 -->
        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-2">
            <div class="text-muted-foreground text-xs">Order · 蓝方</div>
            <div class="space-y-1.5">
              <div
                v-for="a in orderAgents"
                :key="a.id"
                class="flex items-center justify-between rounded-md border px-3 py-2 text-xs"
              >
                <div class="min-w-0">
                  <p class="truncate font-medium">{{ a.name }}</p>
                  <p class="text-muted-foreground truncate">{{ a.champion }}</p>
                </div>
                <CircleDotIcon
                  class="size-3.5 shrink-0"
                  :class="stalledAgents.includes(a.id) ? 'text-amber-500' : 'text-emerald-500'"
                />
              </div>
              <div v-if="orderAgents.length === 0" class="text-muted-foreground py-4 text-center text-xs">
                等待数据…
              </div>
            </div>
          </div>
          <div class="space-y-2">
            <div class="text-muted-foreground text-xs">Chaos · 红方</div>
            <div class="space-y-1.5">
              <div
                v-for="a in chaosAgents"
                :key="a.id"
                class="flex items-center justify-between rounded-md border px-3 py-2 text-xs"
              >
                <div class="min-w-0">
                  <p class="truncate font-medium">{{ a.name }}</p>
                  <p class="text-muted-foreground truncate">{{ a.champion }}</p>
                </div>
                <CircleDotIcon
                  class="size-3.5 shrink-0"
                  :class="stalledAgents.includes(a.id) ? 'text-amber-500' : 'text-emerald-500'"
                />
              </div>
              <div v-if="chaosAgents.length === 0" class="text-muted-foreground py-4 text-center text-xs">
                等待数据…
              </div>
            </div>
          </div>
        </div>

        <!-- 渲染占位（未来 WASM/WebGPU 渲染） -->
        <div
          class="bg-muted/30 text-muted-foreground flex min-h-0 flex-1 items-center justify-center rounded-lg border border-dashed text-xs"
        >
          操作流渲染区域 · WASM/WebGPU 同步重放（开发中）
        </div>
      </section>

      <!-- 右：事件时间线 -->
      <aside class="flex min-h-0 flex-col rounded-lg border">
        <div class="border-b px-4 py-2.5">
          <Tabs default-value="events">
            <TabsList class="h-8">
              <TabsTrigger value="events" class="text-xs">事件流</TabsTrigger>
              <TabsTrigger value="chat" class="text-xs">Agent 对话</TabsTrigger>
            </TabsList>
            <TabsContent value="events" class="mt-0">
              <ScrollArea class="h-[calc(100vh-280px)] pr-3">
                <ol class="space-y-2 py-3">
                  <li
                    v-for="e in events.slice().reverse()"
                    :key="e.id"
                    class="flex gap-3 text-xs leading-snug"
                  >
                    <span class="text-muted-foreground font-mono tabular-nums">#{{ e.seq.toString().padStart(4, "0") }}</span>
                    <span :class="eventTone(e)">{{ eventLabel(e) }}</span>
                  </li>
                  <li v-if="events.length === 0" class="text-muted-foreground py-8 text-center text-xs">
                    等待事件…
                  </li>
                </ol>
              </ScrollArea>
            </TabsContent>
            <TabsContent value="chat" class="mt-0">
              <div class="text-muted-foreground py-12 text-center text-xs">
                LLM Agent 思考链将以对话气泡形式展示。
                <br />
                房主关闭 Prompt 可见性时此区域隐藏。
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </aside>
    </div>
  </div>
</template>
