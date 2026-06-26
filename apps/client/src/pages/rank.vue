<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import {
  rankApi,
  agentsApi,
  type RankQueueEntry,
  type Season,
  type Agent,
  type AgentSnapshot,
} from "@/services/cloudApi";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui/select";
import { TrophyIcon, ZapIcon, RocketIcon, XIcon, ClockIcon } from "@lucide/vue";

// Rank 排队页：Agent 即选手，发布快照后进入匹配池。
// 数据特点：一个核心动作（入队）+ 当前队列状态摘要。
// 用单列卡片堆叠，左侧报名，右侧队列实时态。

const router = useRouter();

const mode = ref("top_solo");
const agents = ref<Agent[]>([]);
const selectedAgent = ref<string>("");
const snapshots = ref<AgentSnapshot[]>([]);
const selectedSnapshot = ref<string>("");

const queue = ref<RankQueueEntry[]>([]);
const season = ref<Season | null>(null);

const enqueueing = ref(false);
const errorMsg = ref("");

const MODES = [
  { value: "top_solo", label: "上单 SOLO", desc: "1v1 · 上路一塔前出生" },
];

async function loadAgents() {
  try {
    agents.value = await agentsApi.list();
    if (!selectedAgent.value && agents.value[0]) {
      selectedAgent.value = agents.value[0].id;
      await loadSnapshots();
    }
  } catch (e) {
    console.error(e);
  }
}

async function loadSnapshots() {
  if (!selectedAgent.value) {
    snapshots.value = [];
    return;
  }
  try {
    snapshots.value = await agentsApi.listSnapshots(selectedAgent.value);
    selectedSnapshot.value = snapshots.value[0]?.id || "";
  } catch (e) {
    snapshots.value = [];
  }
}

async function loadQueue() {
  try {
    queue.value = await rankApi.status();
  } catch (e) {
    queue.value = [];
  }
}

async function loadSeason() {
  try {
    season.value = await rankApi.currentSeason();
  } catch (e) {
    season.value = null;
  }
}

async function handleEnqueue() {
  errorMsg.value = "";
  if (!selectedAgent.value) {
    errorMsg.value = "请选择 Agent";
    return;
  }
  if (!selectedSnapshot.value) {
    errorMsg.value = "请先发布参赛快照";
    return;
  }
  enqueueing.value = true;
  try {
    await rankApi.enqueue(selectedAgent.value, selectedSnapshot.value, mode.value);
    await loadQueue();
  } catch (e: any) {
    errorMsg.value = e.message || "入队失败";
  } finally {
    enqueueing.value = false;
  }
}

function ago(iso: string) {
  const diff = (Date.now() - new Date(iso).getTime()) / 1000;
  if (diff < 60) return `${Math.floor(diff)}s`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m`;
  return `${Math.floor(diff / 3600)}h`;
}

const currentMode = computed(() => MODES.find((m) => m.value === mode.value)!);

onMounted(async () => {
  await loadSeason();
  await Promise.all([loadAgents(), loadQueue()]);
});
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-5xl flex-col gap-8 px-8 py-8">
    <!-- 顶部：Season + Leaderboard 入口 -->
    <header class="flex items-end justify-between">
      <div class="space-y-1">
        <h1 class="text-2xl font-semibold tracking-tight">Rank 竞技</h1>
        <p class="text-muted-foreground text-sm">
          Agent 7×24 全自动排队上分，离线时仍持续匹配。
        </p>
      </div>
      <Button variant="outline" @click="router.push('/leaderboard')">
        <TrophyIcon class="size-4" />
        查看排行榜
      </Button>
    </header>

    <!-- 报名区 -->
    <section class="space-y-4 rounded-lg border p-6">
      <div class="space-y-1">
        <h2 class="text-sm font-semibold">报名参赛</h2>
        <p class="text-muted-foreground text-xs">
          队列始终使用 Agent 最新发布的快照参赛；修改 Agent 后需在 Agent 页重新发布快照。
        </p>
      </div>

      <Separator />

      <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
        <div class="space-y-1.5">
          <Label>模式</Label>
          <Select v-model="mode">
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="m in MODES" :key="m.value" :value="m.value">{{ m.label }}</SelectItem>
            </SelectContent>
          </Select>
          <p class="text-muted-foreground text-xs">{{ currentMode.desc }}</p>
        </div>

        <div class="space-y-1.5">
          <Label>Agent</Label>
          <Select v-model="selectedAgent" @update:modelValue="loadSnapshots">
            <SelectTrigger data-testid="rank-agent-select">
              <SelectValue placeholder="选择 Agent" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="a in agents" :key="a.id" :value="a.id">
                {{ a.name }} · {{ a.champion }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="space-y-1.5">
          <Label>参赛快照</Label>
          <Select v-model="selectedSnapshot" :disabled="snapshots.length === 0">
            <SelectTrigger data-testid="rank-snapshot-select">
              <SelectValue :placeholder="snapshots.length ? '选择快照' : '尚未发布快照'" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="s in snapshots" :key="s.id" :value="s.id">v{{ s.version }} · {{ ago(s.created_at) }} 前</SelectItem>
            </SelectContent>
          </Select>
          <button
            v-if="selectedAgent && snapshots.length === 0"
            class="text-muted-foreground hover:text-foreground text-xs underline-offset-2 hover:underline"
            @click="router.push('/heroes')"
          >
            去「我的选手」发布首个快照 →
          </button>
        </div>
      </div>

      <div class="flex items-center justify-between pt-2">
        <p v-if="errorMsg" class="text-destructive text-xs">{{ errorMsg }}</p>
        <span v-else class="text-muted-foreground text-xs">
          赛季：{{ season ? new Date(season.starts_at).toLocaleDateString() : "—" }}
        </span>
        <Button :disabled="enqueueing" @click="handleEnqueue" data-testid="rank-enqueue-btn">
          <RocketIcon class="size-4" />
          {{ enqueueing ? "入队中…" : "加入匹配池" }}
        </Button>
      </div>
    </section>

    <!-- 当前队列 -->
    <section class="space-y-3">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold">我的排队状态</h2>
        <Badge variant="outline">{{ queue.length }}</Badge>
      </div>

      <div v-if="queue.length === 0" class="text-muted-foreground rounded-lg border border-dashed py-12 text-center text-sm">
        当前未在任何队列中。
      </div>
      <ul v-else class="space-y-2">
        <li
          v-for="q in queue"
          :key="`${q.agent_id}-${q.mode}`"
          class="flex items-center justify-between rounded-lg border px-4 py-3"
        >
          <div class="space-y-0.5">
            <div class="flex items-center gap-2 text-sm font-medium">
              <ZapIcon class="size-3.5" />
              {{ q.mode }}
              <Badge variant="secondary" class="font-mono">{{ q.rating }} ELO</Badge>
            </div>
            <div class="text-muted-foreground flex items-center gap-1 text-xs">
              <ClockIcon class="size-3" />
              入队 {{ ago(q.enqueued_at) }} 前 · Agent {{ q.agent_id.slice(0, 8) }}
            </div>
          </div>
          <Button variant="ghost" size="sm" data-testid="rank-dequeue-btn">
            <XIcon class="size-3.5" />
            退出
          </Button>
        </li>
      </ul>
    </section>
  </div>
</template>
