<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter } from "vue-router";
import { roomsApi, type Room, type RoomConstraints } from "@/services/cloudApi";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Users2Icon, KeyRoundIcon, PlusIcon, Loader2Icon } from "@lucide/vue";

// 房间大厅与我的房间。
// 数据特点：每个房间是一组结构化属性（名称 / 人数 / 模式 / 状态），适合卡片网格而非长列表。
// 上方提供两个核心入口：加入邀请码、创建房间。

const router = useRouter();

const myRooms = ref<Room[]>([]);
const lobbyRooms = ref<Room[]>([]);
const loading = ref(true);
const activeTab = ref<"lobby" | "mine">("lobby");

// 加入码
const joinCode = ref("");
const joinError = ref("");
const joining = ref(false);

// 创建房间
const showCreate = ref(false);
const creating = ref(false);
const createError = ref("");
const draft = ref<{ name: string; constraints: RoomConstraints }>({
  name: "",
  constraints: {
    max_members: 10,
    max_agents_per_member: 3,
    team_strategy: "free",
    lobby_visible: true,
    reveal_prompts: false,
  },
});

async function refresh() {
  loading.value = true;
  try {
    const [mine, lobby] = await Promise.all([
      roomsApi.listMine().catch(() => [] as Room[]),
      roomsApi.listLobby().catch(() => [] as Room[]),
    ]);
    myRooms.value = mine;
    lobbyRooms.value = lobby;
  } finally {
    loading.value = false;
  }
}

async function handleJoinByCode() {
  joinError.value = "";
  const c = joinCode.value.trim().toUpperCase();
  if (!c) {
    joinError.value = "请输入邀请码";
    return;
  }
  joining.value = true;
  try {
    const room = await roomsApi.joinByCode(c);
    router.push(`/rooms/${room.id}`);
  } catch (e: any) {
    joinError.value = e.message || "加入失败";
  } finally {
    joining.value = false;
  }
}

async function handleCreate() {
  createError.value = "";
  if (!draft.value.name.trim()) {
    createError.value = "请填写房间名称";
    return;
  }
  creating.value = true;
  try {
    const room = await roomsApi.create(draft.value.name.trim(), draft.value.constraints);
    showCreate.value = false;
    router.push(`/rooms/${room.id}`);
  } catch (e: any) {
    createError.value = e.message || "创建失败";
  } finally {
    creating.value = false;
  }
}

async function handleJoin(room: Room) {
  try {
    await roomsApi.join(room.id);
    router.push(`/rooms/${room.id}`);
  } catch (e: any) {
    console.error(e);
  }
}

const statusLabel = (s: Room["status"]) =>
  ({ pending: "待开始", running: "对局中", finished: "已结束" }[s] || s);

const visibleLobby = computed(() => lobbyRooms.value);

onMounted(refresh);
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-6xl flex-col gap-8 px-8 py-8">
    <!-- 标题与主操作 -->
    <header class="flex items-center justify-between">
      <div class="space-y-1">
        <h1 class="text-2xl font-semibold tracking-tight">房间</h1>
        <p class="text-muted-foreground text-sm">与好友组局对战、社交观战 AI 对打</p>
      </div>
      <Button @click="showCreate = true">
        <PlusIcon class="size-4" />
        创建房间
      </Button>
    </header>

    <!-- 双入口：加入邀请码 + Tabs -->
    <section class="space-y-3">
      <div class="flex items-end gap-3">
        <div class="flex-1 max-w-sm space-y-1.5">
          <Label class="text-muted-foreground flex items-center gap-1.5 text-xs">
            <KeyRoundIcon class="size-3.5" />
            邀请码加入
          </Label>
          <div class="flex gap-2">
            <Input
              v-model="joinCode"
              placeholder="ABCD1234"
              class="font-mono uppercase"
              @keydown.enter="handleJoinByCode"
            />
            <Button :disabled="joining" variant="secondary" @click="handleJoinByCode">
              <Loader2Icon v-if="joining" class="size-4 animate-spin" />
              <span v-else>加入</span>
            </Button>
          </div>
          <p v-if="joinError" class="text-destructive text-xs">{{ joinError }}</p>
        </div>
      </div>
    </section>

    <Separator />

    <!-- Tabs: 大厅 / 我的 -->
    <Tabs v-model="activeTab" class="flex min-h-0 flex-1 flex-col gap-6">
      <TabsList>
        <TabsTrigger value="lobby">
          公开大厅
          <Badge variant="secondary" class="ml-1">{{ visibleLobby.length }}</Badge>
        </TabsTrigger>
        <TabsTrigger value="mine">
          我的房间
          <Badge variant="secondary" class="ml-1">{{ myRooms.length }}</Badge>
        </TabsTrigger>
      </TabsList>

      <TabsContent value="lobby" class="flex-1">
        <div v-if="loading" class="text-muted-foreground py-12 text-center text-sm">加载中…</div>
        <div v-else-if="visibleLobby.length === 0" class="text-muted-foreground py-16 text-center text-sm">
          大厅当前没有公开房间。
        </div>
        <div v-else class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          <button
            v-for="r in visibleLobby"
            :key="r.id"
            class="hover:bg-muted/50 group flex flex-col gap-3 rounded-lg border p-5 text-left transition-colors"
            @click="handleJoin(r)"
          >
            <div class="flex items-start justify-between gap-2">
              <h3 class="truncate text-sm font-semibold leading-tight">{{ r.name }}</h3>
              <Badge variant="outline" class="shrink-0">{{ statusLabel(r.status) }}</Badge>
            </div>
            <div class="text-muted-foreground flex items-center gap-4 text-xs">
              <span class="flex items-center gap-1">
                <Users2Icon class="size-3.5" />
                {{ r.member_count }} / {{ r.constraints.max_members }}
              </span>
              <span>{{ r.constraints.team_strategy === "single" ? "单阵营" : "自由阵营" }}</span>
            </div>
          </button>
        </div>
      </TabsContent>

      <TabsContent value="mine" class="flex-1">
        <div v-if="myRooms.length === 0" class="text-muted-foreground py-16 text-center text-sm">
          你还没有加入任何房间。
        </div>
        <div v-else class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          <button
            v-for="r in myRooms"
            :key="r.id"
            class="hover:bg-muted/50 group flex flex-col gap-3 rounded-lg border p-5 text-left transition-colors"
            @click="router.push(`/rooms/${r.id}`)"
          >
            <div class="flex items-start justify-between gap-2">
              <h3 class="truncate text-sm font-semibold leading-tight">{{ r.name }}</h3>
              <Badge variant="outline" class="shrink-0">{{ statusLabel(r.status) }}</Badge>
            </div>
            <div class="text-muted-foreground flex items-center gap-4 text-xs">
              <span class="flex items-center gap-1">
                <Users2Icon class="size-3.5" />
                {{ r.member_count }} / {{ r.constraints.max_members }}
              </span>
              <span class="font-mono">{{ r.invite_code }}</span>
            </div>
            <div v-if="r.status === 'running'" class="flex items-center gap-1.5 text-xs">
              <span class="bg-emerald-500 size-1.5 animate-pulse rounded-full" />
              <span class="text-foreground">对局进行中</span>
            </div>
          </button>
        </div>
      </TabsContent>
    </Tabs>

    <!-- 创建房间对话框 -->
    <Dialog :open="showCreate" @update:open="(v) => (showCreate = v)">
      <DialogContent class="max-w-md">
        <DialogHeader>
          <DialogTitle>创建房间</DialogTitle>
          <DialogDescription>设置房间约束，所有成员将在房间内自行编排 Agent。</DialogDescription>
        </DialogHeader>

        <div class="space-y-4 py-2">
          <div class="space-y-1.5">
            <Label>房间名称</Label>
            <Input v-model="draft.name" placeholder="周末野队挑战" />
          </div>

          <div class="grid grid-cols-2 gap-3">
            <div class="space-y-1.5">
              <Label>最大人数</Label>
              <Input v-model.number="draft.constraints.max_members" type="number" min="2" max="20" />
            </div>
            <div class="space-y-1.5">
              <Label>每人 Agent 上限</Label>
              <Input v-model.number="draft.constraints.max_agents_per_member" type="number" min="1" max="10" />
            </div>
          </div>

          <div class="space-y-1.5">
            <Label>阵营策略</Label>
            <Select v-model="draft.constraints.team_strategy">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="free">自由（红蓝皆可）</SelectItem>
                <SelectItem value="single">单阵营（每人只能在一方）</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-3 pt-1">
            <label class="flex items-center gap-2 text-sm">
              <Checkbox v-model="draft.constraints.lobby_visible" />
              <span>公开到大厅</span>
            </label>
            <label class="flex items-center gap-2 text-sm">
              <Checkbox v-model="draft.constraints.reveal_prompts" />
              <span class="space-y-0.5">
                <span class="block">公开 Prompt 与模型配置</span>
                <span class="text-muted-foreground block text-xs">关闭则成员只能看到 Agent 名称</span>
              </span>
            </label>
          </div>

          <p v-if="createError" class="text-destructive text-xs">{{ createError }}</p>
        </div>

        <DialogFooter>
          <Button variant="ghost" @click="showCreate = false">取消</Button>
          <Button :disabled="creating" @click="handleCreate">
            <Loader2Icon v-if="creating" class="size-4 animate-spin" />
            创建
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
