<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { useGameStore } from "@/stores/gameStore";
import { useAuthStore } from "@/stores/authStore";
import { services } from "@/services/provider";
import type { Room, RoomAgentSlot, Agent, Team } from "@/services/types";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Label } from "@/components/ui/label";
import TeamSlots from "@/components/TeamSlots.vue";
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
  Trash2Icon,
  Users2Icon,
  LogOutIcon,
  ArrowLeftIcon,
  Loader2Icon,
  RotateCwIcon,
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
const authStore = useAuthStore();



// 添加槽位
const showAdd = ref(false);
const addTeam = ref<Team>("order");
const addAgentId = ref<string>("");
const adding = ref(false);
const addError = ref("");

// 启动对局
const starting = ref(false);
const refreshing = ref(false);
let pollTimer: number | null = null;

async function handleManualRefresh() {
  refreshing.value = true;
  await refresh();
  refreshing.value = false;
}

async function refresh() {
  try {
    const [r, list, agents] = await Promise.all([
      services.cloud.getRoom(roomId.value),
      services.cloud.listRoomSlots(roomId.value).catch(() => [] as RoomAgentSlot[]),
      services.cloud.listAgents().catch(() => [] as Agent[]),
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
  if (!room.value || !authStore.user) return false;
  return room.value.owner_id === authStore.user.id;
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
    await services.cloud.addRoomSlot(roomId.value, addAgentId.value, addTeam.value);
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
    await services.cloud.removeRoomSlot(roomId.value, slot.id);
    await refresh();
  } catch (e) {
    console.error(e);
  }
}

async function handleStart() {
  starting.value = true;
  try {
    const res = await services.cloud.startRoomMatch(roomId.value);
    router.push(`/observe/${res.match_id}`);
  } catch (e: any) {
    console.error(e);
    alert(e.message || "启动失败");
  } finally {
    starting.value = false;
  }
}

const confirmDialog = ref<{
  open: boolean;
  title: string;
  desc: string;
  confirmBtnText: string;
  onConfirm: () => Promise<void>;
  loading: boolean;
  error: string;
}>({
  open: false,
  title: "",
  desc: "",
  confirmBtnText: "确认",
  onConfirm: async () => {},
  loading: false,
  error: "",
});

function showConfirm(params: {
  title: string;
  desc: string;
  confirmBtnText?: string;
  action: () => Promise<void>;
}) {
  confirmDialog.value = {
    open: true,
    title: params.title,
    desc: params.desc,
    confirmBtnText: params.confirmBtnText || "确认",
    onConfirm: async () => {
      confirmDialog.value.loading = true;
      confirmDialog.value.error = "";
      try {
        await params.action();
        confirmDialog.value.open = false;
      } catch (e: any) {
        console.error(e);
        confirmDialog.value.error = e.message || "操作失败";
      } finally {
        confirmDialog.value.loading = false;
      }
    },
    loading: false,
    error: "",
  };
}

function handleLeave() {
  showConfirm({
    title: "确认离开房间？",
    desc: "离开后，你将不再是该房间的成员，你在此房间中分配的 Agent 槽位也将被移除。",
    confirmBtnText: "确认离开",
    action: async () => {
      await services.cloud.leaveRoom(roomId.value);
      router.push("/rooms");
    },
  });
}

function handleDissolve() {
  showConfirm({
    title: "解散房间？",
    desc: "解散后房间将永久关闭，所有成员及槽位信息将被清空且无法恢复，确认继续？",
    confirmBtnText: "确认解散",
    action: async () => {
      await services.cloud.dissolveRoom(roomId.value);
      router.push("/rooms");
    },
  });
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
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <Button variant="ghost" size="icon" @click="router.push('/rooms')">
              <ArrowLeftIcon class="size-4" />
            </Button>
            <h1 class="text-2xl font-semibold tracking-tight">{{ room.name }}</h1>
            <Badge variant="outline">
              {{ room.status === "lobby" ? "待开始" : room.status === "running" ? "对局中" : "已结束" }}
            </Badge>
          </div>
          <Button variant="outline" size="icon" @click="handleManualRefresh" :disabled="refreshing" title="手动刷新">
            <Loader2Icon v-if="refreshing" class="size-4 animate-spin" />
            <RotateCwIcon v-else class="size-4" />
          </Button>
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
        <TeamSlots
          :count="orderSlots.length"
          label="Order · 蓝色方"
          color="blue"
          @add="openAdd('order')"
        >
          <div
            v-for="s in orderSlots"
            :key="s.id"
            class="bg-background/50 hover:bg-muted/40 group flex items-center justify-between gap-3 rounded-md border p-2.5 transition-colors border-border"
          >
            <div class="min-w-0 space-y-0.5">
              <p class="truncate text-sm font-medium text-foreground">{{ agentName(s.agent_id) }}</p>
              <p class="text-muted-foreground truncate text-xs">{{ agentChampion(s.agent_id) }} · 成员 #{{ s.member_user_id }}</p>
            </div>
            <Button
              variant="ghost"
              size="icon"
              class="text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100 size-7"
              @click="handleRemove(s)"
            >
              <Trash2Icon class="size-3.5" />
            </Button>
          </div>
          <div
            v-if="orderSlots.length === 0"
            class="text-muted-foreground py-8 text-center text-xs"
          >
            暂无 Agent
          </div>
        </TeamSlots>

        <!-- Chaos -->
        <TeamSlots
          :count="chaosSlots.length"
          label="Chaos · 红色方"
          color="red"
          @add="openAdd('chaos')"
        >
          <div
            v-for="s in chaosSlots"
            :key="s.id"
            class="bg-background/50 hover:bg-muted/40 group flex items-center justify-between gap-3 rounded-md border p-2.5 transition-colors border-border"
          >
            <div class="min-w-0 space-y-0.5">
              <p class="truncate text-sm font-medium text-foreground">{{ agentName(s.agent_id) }}</p>
              <p class="text-muted-foreground truncate text-xs">{{ agentChampion(s.agent_id) }} · 成员 #{{ s.member_user_id }}</p>
            </div>
            <Button
              variant="ghost"
              size="icon"
              class="text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100 size-7"
              @click="handleRemove(s)"
            >
              <Trash2Icon class="size-3.5" />
            </Button>
          </div>
          <div
            v-if="chaosSlots.length === 0"
            class="text-muted-foreground py-8 text-center text-xs"
          >
            暂无 Agent
          </div>
        </TeamSlots>
      </section>

      <Separator />

      <!-- 底部操作 -->
      <footer class="flex items-center justify-between">
        <div class="flex gap-2">
          <Button variant="ghost" size="sm" @click="handleLeave" data-testid="room-leave-btn">
            <LogOutIcon class="size-3.5" />
            离开房间
          </Button>
          <Button
            v-if="isOwner"
            variant="ghost"
            size="sm"
            class="text-destructive hover:text-destructive"
            @click="handleDissolve"
            data-testid="room-dissolve-btn"
          >
            解散房间
          </Button>
        </div>
        <Button
          :disabled="slots.length === 0 || starting"
          @click="handleStart"
          data-testid="room-start-match-btn"
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
              <SelectTrigger data-testid="room-add-agent-select">
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
          <Button :disabled="adding" @click="handleAdd" data-testid="room-confirm-add-btn">添加</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- 确认操作对话框 -->
    <Dialog :open="confirmDialog.open" @update:open="(v) => (confirmDialog.open = v)">
      <DialogContent class="max-w-sm">
        <DialogHeader>
          <DialogTitle>{{ confirmDialog.title }}</DialogTitle>
        </DialogHeader>
        <div class="space-y-3 py-2 text-sm text-muted-foreground">
          <p>{{ confirmDialog.desc }}</p>
          <p v-if="confirmDialog.error" class="text-destructive text-xs">{{ confirmDialog.error }}</p>
        </div>
        <DialogFooter>
          <Button variant="ghost" :disabled="confirmDialog.loading" @click="confirmDialog.open = false" data-testid="confirm-dialog-cancel-btn">
            取消
          </Button>
          <Button
            :variant="confirmDialog.confirmBtnText.includes('解散') || confirmDialog.confirmBtnText.includes('删除') ? 'destructive' : 'default'"
            :disabled="confirmDialog.loading"
            @click="confirmDialog.onConfirm"
            data-testid="confirm-dialog-submit-btn"
          >
            <Loader2Icon v-if="confirmDialog.loading" class="size-4 animate-spin mr-1.5" />
            {{ confirmDialog.confirmBtnText }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
