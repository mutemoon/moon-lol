<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { useGameStore } from "@/stores/gameStore";
import {
  roomsApi,
  agentsApi,
  type Room,
  type RoomAgentSlot,
  type Agent,
  type Team,
} from "@/services/cloudApi";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui/select";
import {
  CopyIcon,
  PlayIcon,
  PlusIcon,
  Trash2Icon,
  Users2Icon,
  LogOutIcon,
  ArrowLeftIcon,
} from "@lucide/vue";

// 房间详情：成员、约束、双阵营 Agent 槽位。
// 数据特点：两个对称的容器（Order/Chaos），需突出"谁加了什么 Agent"。
// 用并排两列布局而非长列表表现红蓝对称；约束以一行 chip 显示。

const route = useRoute();
const router = useRouter();
const roomId = computed(() => String((route.params as any).id));

const room = ref<Room | null>(null);
const slots = ref<RoomAgentSlot[]>([]);
const myAgents = ref<Agent[]>([]);
const loading = ref(true);

const store = useGameStore();
const { heroPresets } = storeToRefs(store);

// 添加槽位
const showAdd = ref(false);
const addTeam = ref<Team>("order");
const addAgentId = ref<string>("");
const adding = ref(false);
const addError = ref("");

// 启动对局
const starting = ref(false);
let pollTimer: number | null = null;

async function refresh() {
  try {
    const [r, list, agents] = await Promise.all([
      roomsApi.get(roomId.value),
      roomsApi.listSlots(roomId.value).catch(() => [] as RoomAgentSlot[]),
      agentsApi.list().catch(() => [] as Agent[]),
    ]);
    room.value = r;
    slots.value = list;
    myAgents.value = agents;
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

function agentName(id: string): string {
  const a = myAgents.value.find((x) => x.id === id);
  if (a) return a.name;
  const h = heroPresets.value.find((x: any) => x.id === id);
  return h?.name || id.slice(0, 8);
}

function agentChampion(id: string): string {
  const a = myAgents.value.find((x) => x.id === id);
  return a?.champion || "—";
}

const orderSlots = computed(() => slots.value.filter((s) => s.team === "order"));
const chaosSlots = computed(() => slots.value.filter((s) => s.team === "chaos"));

const isOwner = computed(() => {
  // 简化：第一个成员视为房主（实际由 owner_user_id 比对 me）
  return room.value !== null;
});

async function copyInvite() {
  if (!room.value) return;
  try {
    await navigator.clipboard.writeText(room.value.invite_code);
  } catch {
    /* ignore */
  }
}

function openAdd(team: Team) {
  addTeam.value = team;
  addAgentId.value = myAgents.value[0]?.id || "";
  addError.value = "";
  showAdd.value = true;
}

async function handleAdd() {
  if (!addAgentId.value) {
    addError.value = "请选择 Agent";
    return;
  }
  adding.value = true;
  addError.value = "";
  try {
    await roomsApi.addSlot(roomId.value, addAgentId.value, addTeam.value);
    showAdd.value = false;
    await refresh();
  } catch (e: any) {
    addError.value = e.message || "添加失败";
  } finally {
    adding.value = false;
  }
}

async function handleRemove(slot: RoomAgentSlot) {
  try {
    await roomsApi.removeSlot(roomId.value, slot.id);
    await refresh();
  } catch (e) {
    console.error(e);
  }
}

async function handleStart() {
  starting.value = true;
  try {
    const res = await roomsApi.start(roomId.value);
    router.push(`/observe/${res.match_id}`);
  } catch (e: any) {
    console.error(e);
    alert(e.message || "启动失败");
  } finally {
    starting.value = false;
  }
}

async function handleLeave() {
  if (!confirm("确认离开房间？")) return;
  try {
    await roomsApi.leave(roomId.value);
    router.push("/rooms");
  } catch (e) {
    console.error(e);
  }
}

async function handleDissolve() {
  if (!confirm("解散后房间将永久关闭，确认继续？")) return;
  try {
    await roomsApi.dissolve(roomId.value);
    router.push("/rooms");
  } catch (e) {
    console.error(e);
  }
}

onMounted(async () => {
  await refresh();
  // 轻量轮询以反映其他成员加入
  pollTimer = window.setInterval(refresh, 5000);
});
onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-6xl flex-col gap-6 px-8 py-6">
    <div v-if="loading" class="text-muted-foreground py-12 text-center text-sm">加载中…</div>

    <template v-else-if="room">
      <!-- 顶部：标题 + 操作 -->
      <header class="space-y-3">
        <div class="flex items-center gap-2">
          <Button variant="ghost" size="icon" @click="router.push('/rooms')">
            <ArrowLeftIcon class="size-4" />
          </Button>
          <h1 class="text-2xl font-semibold tracking-tight">{{ room.name }}</h1>
          <Badge variant="outline">
            {{ room.status === "lobby" ? "待开始" : room.status === "running" ? "对局中" : "已结束" }}
          </Badge>
        </div>

        <!-- 一行 chip 表达约束 -->
        <div class="text-muted-foreground flex flex-wrap items-center gap-x-5 gap-y-1 text-xs">
          <span class="flex items-center gap-1.5">
            <Users2Icon class="size-3.5" />
            {{ room.member_count }} / {{ room.constraints.max_members }} 成员
          </span>
          <span>每人最多 {{ room.constraints.max_agents_per_member }} 个 Agent</span>
          <span>{{ room.constraints.team_policy === "single_team" ? "单阵营策略" : "自由阵营" }}</span>
          <span>{{ room.constraints.lobby_visible ? "大厅公开" : "邀请码加入" }}</span>
          <span>{{ room.constraints.prompt_visible ? "Prompt 公开" : "Prompt 隐藏" }}</span>
          <span class="flex items-center gap-1.5">
            <span class="font-mono uppercase">{{ room.invite_code }}</span>
            <button
              class="hover:bg-muted inline-flex size-5 items-center justify-center rounded"
              @click="copyInvite"
              title="复制邀请码"
            >
              <CopyIcon class="size-3" />
            </button>
          </span>
        </div>
      </header>

      <Separator />

      <!-- 阵营编排：两栏对称 -->
      <section class="grid flex-1 grid-cols-1 gap-6 lg:grid-cols-2">
        <!-- Order -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <h2 class="text-sm font-semibold tracking-tight">Order · 蓝色方</h2>
              <p class="text-muted-foreground text-xs">{{ orderSlots.length }} 个 Agent</p>
            </div>
            <Button variant="outline" size="sm" @click="openAdd('order')">
              <PlusIcon class="size-3.5" />
              添加
            </Button>
          </div>
          <div class="space-y-2">
            <div
              v-for="s in orderSlots"
              :key="s.id"
              class="hover:bg-muted/40 group flex items-center justify-between gap-3 rounded-lg border px-4 py-3"
            >
              <div class="min-w-0 space-y-0.5">
                <p class="truncate text-sm font-medium">{{ agentName(s.agent_id) }}</p>
                <p class="text-muted-foreground truncate text-xs">{{ agentChampion(s.agent_id) }} · 成员 #{{ s.member_user_id }}</p>
              </div>
              <Button
                variant="ghost"
                size="icon"
                class="text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100"
                @click="handleRemove(s)"
              >
                <Trash2Icon class="size-3.5" />
              </Button>
            </div>
            <div
              v-if="orderSlots.length === 0"
              class="text-muted-foreground rounded-lg border border-dashed py-8 text-center text-xs"
            >
              暂无 Agent
            </div>
          </div>
        </div>

        <!-- Chaos -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <div class="space-y-0.5">
              <h2 class="text-sm font-semibold tracking-tight">Chaos · 红色方</h2>
              <p class="text-muted-foreground text-xs">{{ chaosSlots.length }} 个 Agent</p>
            </div>
            <Button variant="outline" size="sm" @click="openAdd('chaos')">
              <PlusIcon class="size-3.5" />
              添加
            </Button>
          </div>
          <div class="space-y-2">
            <div
              v-for="s in chaosSlots"
              :key="s.id"
              class="hover:bg-muted/40 group flex items-center justify-between gap-3 rounded-lg border px-4 py-3"
            >
              <div class="min-w-0 space-y-0.5">
                <p class="truncate text-sm font-medium">{{ agentName(s.agent_id) }}</p>
                <p class="text-muted-foreground truncate text-xs">{{ agentChampion(s.agent_id) }} · 成员 #{{ s.member_user_id }}</p>
              </div>
              <Button
                variant="ghost"
                size="icon"
                class="text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100"
                @click="handleRemove(s)"
              >
                <Trash2Icon class="size-3.5" />
              </Button>
            </div>
            <div
              v-if="chaosSlots.length === 0"
              class="text-muted-foreground rounded-lg border border-dashed py-8 text-center text-xs"
            >
              暂无 Agent
            </div>
          </div>
        </div>
      </section>

      <Separator />

      <!-- 底部操作 -->
      <footer class="flex items-center justify-between">
        <div class="flex gap-2">
          <Button variant="ghost" size="sm" @click="handleLeave">
            <LogOutIcon class="size-3.5" />
            离开房间
          </Button>
          <Button
            v-if="isOwner"
            variant="ghost"
            size="sm"
            class="text-destructive hover:text-destructive"
            @click="handleDissolve"
          >
            解散房间
          </Button>
        </div>
        <Button
          :disabled="slots.length === 0 || starting"
          @click="handleStart"
        >
          <PlayIcon class="size-4" />
          {{ starting ? "启动中…" : "开始对局" }}
        </Button>
      </footer>
    </template>

    <!-- 添加 Agent 对话框 -->
    <Dialog :open="showAdd" @update:open="(v) => (showAdd = v)">
      <DialogContent class="max-w-sm">
        <DialogHeader>
          <DialogTitle>
            添加到 {{ addTeam === "order" ? "Order（蓝方）" : "Chaos（红方）" }}
          </DialogTitle>
        </DialogHeader>
        <div class="space-y-3 py-2">
          <div class="space-y-1.5">
            <Label>选择 Agent</Label>
            <Select v-model="addAgentId">
              <SelectTrigger>
                <SelectValue placeholder="选择 Agent…" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="a in myAgents" :key="a.id" :value="a.id">
                  {{ a.name }} · {{ a.champion }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <p v-if="addError" class="text-destructive text-xs">{{ addError }}</p>
        </div>
        <DialogFooter>
          <Button variant="ghost" @click="showAdd = false">取消</Button>
          <Button :disabled="adding" @click="handleAdd">添加</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
