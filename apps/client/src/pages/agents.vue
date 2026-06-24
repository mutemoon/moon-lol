<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onMounted, watch, shallowRef } from "vue";
import { storeToRefs } from "pinia";
import { useGameStore, type AgentPreset } from "@/stores/gameStore";
import { useRouter, useRoute } from "vue-router";
import { useLocale } from "@/composables/useLocale";
import {
  agentsApi,
  type AgentSnapshot,
  type Visibility,
} from "@/services/cloudApi";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Separator } from "@/components/ui/separator";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  BotIcon,
  PlusIcon,
  Trash2Icon,
  SaveIcon,
  ArrowLeftIcon,
  RocketIcon,
  GlobeIcon,
  LockIcon,
  UsersIcon,
  PlayIcon,
  ZapIcon,
  CodeIcon,
  HistoryIcon,
} from "@lucide/vue";

// Agent 配置管理页（产品文档 §2 / §4.3）。
// 数据特点：左侧是预设清单，右侧 detail 基于 agent_type 动态切换：
//   - LLM    → Prompt / 模型 / 前言（表单）
//   - Script → JS 脚本编辑器（Monaco）
//   - RL     → 模型权重上传 + 训练入口
// 顶部加入"发布参赛快照"与可见性切换（Phase 4）。

const store = useGameStore();
const { agentPresets } = storeToRefs(store);
const router = useRouter();
const route = useRoute();
const { t } = useLocale();

const selectedName = ref<string | null>(null);
const errorMsg = ref("");
const showDeleteConfirm = ref(false);

const AGENT_TYPES = [
  { value: "llm", label: "LLM（语言模型）" },
  { value: "rl", label: "RL（强化学习）" },
  { value: "script", label: "Script（脚本）" },
];

const emptyDraft = (): AgentPreset => ({
  name: "",
  agent_type: "llm",
  prompt: "",
  preamble: "",
  model: "",
});
const draft = ref<AgentPreset>(emptyDraft());

// Phase 4: 快照与可见性
const visibility = ref<Visibility>("private");
const snapshots = shallowRef<AgentSnapshot[]>([]);
const publishing = ref(false);

// Monaco（Script Agent）
const MonacoEditor = shallowRef<any>(null);
async function ensureMonaco() {
  if (MonacoEditor.value) return;
  const mod = await import("@guolao/vue-monaco-editor");
  MonacoEditor.value = mod.VueMonacoEditor;
}

const scriptCode = ref<string>(
  `// MoonLOL Script Agent — JavaScript runtime
// 在每个 tick 调用 onTick(observation) 并返回 action。
//
// 提供的 API：
//   observe() — 获取当前观测
//   action(cmd, params) — 下发指令
//   log(...args)        — 调试日志

function onTick(obs) {
  // 例：移动到补刀线
  if (obs.last_hit_minion) {
    return { type: "attack", target: obs.last_hit_minion.id };
  }
  return { type: "move", target: [7500, 7500] };
}
`
);
const hotReload = ref(true);

function selectPreset(name: string) {
  selectedName.value = name;
  const p = agentPresets.value.find((x) => x.name === name);
  if (p) draft.value = { ...p };
  // 加载快照（伪 id）
  loadSnapshots();
}

function startNew() {
  selectedName.value = null;
  draft.value = emptyDraft();
  snapshots.value = [];
  visibility.value = "private";
}

async function handleSave() {
  errorMsg.value = "";
  const name = draft.value.name.trim();
  if (!name) {
    errorMsg.value = t("agents.errorFillName");
    return;
  }
  try {
    await store.saveAgentPreset({ ...draft.value, name });
    selectedName.value = name;
    errorMsg.value = "✓ 已保存";
  } catch (e: any) {
    errorMsg.value = e.message || "保存失败";
  }
}

async function confirmDelete() {
  if (!selectedName.value) return;
  try {
    await store.deleteAgentPreset(selectedName.value);
    selectedName.value = null;
    draft.value = emptyDraft();
    showDeleteConfirm.value = false;
  } catch (e: any) {
    errorMsg.value = e.message || "删除失败";
  }
}

// 通过 name 反查 agent id（store 还没暴露，简化使用 name 标识）
function currentAgentId(): string | null {
  // 实际实现需要在 store 内通过 name → id 映射；此处用 name 作为伪 id 触发 API
  return selectedName.value;
}

async function loadSnapshots() {
  const id = currentAgentId();
  if (!id) {
    snapshots.value = [];
    return;
  }
  try {
    snapshots.value = await agentsApi.listSnapshots(id);
  } catch (e) {
    snapshots.value = [];
  }
}

async function publishSnapshot() {
  const id = currentAgentId();
  if (!id) return;
  publishing.value = true;
  try {
    await agentsApi.publishSnapshot(id);
    await loadSnapshots();
    errorMsg.value = "✓ 快照已发布";
  } catch (e: any) {
    errorMsg.value = e.message || "发布失败";
  } finally {
    publishing.value = false;
  }
}

async function updateVisibility(v: Visibility) {
  visibility.value = v;
  const id = currentAgentId();
  if (!id) return;
  try {
    await agentsApi.updateVisibility(id, v);
  } catch (e: any) {
    errorMsg.value = e.message || "更新可见性失败";
  }
}

const latestSnapshot = computed(() => snapshots.value[0]);

watch(
  () => draft.value.agent_type,
  (t) => {
    if (t === "script") ensureMonaco();
  }
);

onMounted(async () => {
  await store.loadAgentPresets();
  const focus = route.query.focus as string | undefined;
  if (focus && agentPresets.value.find((a) => a.name === focus)) {
    selectPreset(focus);
  }
});
</script>

<template>
  <div class="flex h-full w-full overflow-hidden">
    <!-- 左：预设清单 -->
    <aside class="bg-background flex w-64 shrink-0 flex-col overflow-hidden border-r">
      <div class="flex h-12 shrink-0 items-center justify-between border-b px-4">
        <div class="flex items-center gap-2">
          <BotIcon class="size-4" />
          <h2 class="text-sm font-semibold">Agent 配置</h2>
        </div>
        <Button variant="ghost" size="icon" class="size-7" @click="startNew">
          <PlusIcon class="size-4" />
        </Button>
      </div>

      <div class="flex-1 overflow-y-auto p-2">
        <div v-if="agentPresets.length === 0" class="text-muted-foreground p-6 text-center text-xs">
          还没有 Agent 配置。
        </div>
        <ul v-else class="space-y-0.5">
          <li v-for="p in agentPresets" :key="p.name">
            <button
              class="w-full rounded-md px-2.5 py-2 text-left text-sm transition-colors hover:bg-muted/60"
              :class="p.name === selectedName ? 'bg-muted font-medium' : ''"
              @click="selectPreset(p.name)"
            >
              <div class="flex items-center justify-between gap-2">
                <span class="truncate">{{ p.name }}</span>
                <Badge variant="outline" class="shrink-0 text-[10px] uppercase">{{ p.agent_type }}</Badge>
              </div>
            </button>
          </li>
        </ul>
      </div>
    </aside>

    <!-- 右：详情 -->
    <section class="flex min-w-0 flex-1 flex-col overflow-hidden">
      <!-- 头：标题 + 主操作 -->
      <header class="flex shrink-0 items-center justify-between gap-3 border-b px-6 py-3">
        <div class="flex items-center gap-2">
          <Button v-if="!selectedName" variant="ghost" size="icon" @click="router.push('/')">
            <ArrowLeftIcon class="size-4" />
          </Button>
          <h1 class="text-base font-semibold tracking-tight">
            {{ selectedName ? draft.name : "新建 Agent 配置" }}
          </h1>
          <Badge v-if="latestSnapshot" variant="secondary" class="font-mono text-[10px]">
            v{{ latestSnapshot.version }}
          </Badge>
        </div>
        <div class="flex items-center gap-2">
          <!-- 可见性 -->
          <Select v-if="selectedName" :model-value="visibility" @update:model-value="(v) => updateVisibility(v as Visibility)">
            <SelectTrigger class="h-8 w-32 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="private">
                <span class="flex items-center gap-2"><LockIcon class="size-3" />私有</span>
              </SelectItem>
              <SelectItem value="friends">
                <span class="flex items-center gap-2"><UsersIcon class="size-3" />好友可见</span>
              </SelectItem>
              <SelectItem value="public">
                <span class="flex items-center gap-2"><GlobeIcon class="size-3" />公开</span>
              </SelectItem>
            </SelectContent>
          </Select>

          <Button
            v-if="selectedName"
            variant="outline"
            size="sm"
            :disabled="publishing"
            @click="publishSnapshot"
          >
            <RocketIcon class="size-3.5" />
            {{ publishing ? "发布中…" : "发布参赛快照" }}
          </Button>
          <Button size="sm" :disabled="!draft.name.trim()" @click="handleSave">
            <SaveIcon class="size-3.5" />
            保存
          </Button>
        </div>
      </header>

      <!-- 错误/成功消息 -->
      <p
        v-if="errorMsg"
        class="text-muted-foreground border-b px-6 py-1.5 text-xs"
        :class="errorMsg.startsWith('✓') ? 'text-emerald-600' : 'text-destructive'"
      >
        {{ errorMsg }}
      </p>

      <!-- 内容：基于类型动态切换 -->
      <div class="min-h-0 flex-1 overflow-y-auto">
        <div class="mx-auto max-w-3xl space-y-8 px-8 py-8">
          <!-- 基础信息 -->
          <section class="space-y-4">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div class="space-y-1.5">
                <Label>配置名称</Label>
                <Input v-model="draft.name" placeholder="激进压制 / 稳健补刀…" />
              </div>
              <div class="space-y-1.5">
                <Label>类型</Label>
                <Select v-model="draft.agent_type">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem v-for="ty in AGENT_TYPES" :key="ty.value" :value="ty.value">
                      {{ ty.label }}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <p class="text-muted-foreground text-xs leading-relaxed">
              Agent 配置即"策略大脑"，英雄无关。同一配置可被多个 Agent（英雄 + 大脑 + 出生点）复用。
            </p>
          </section>

          <Separator />

          <!-- LLM 表单 -->
          <section v-if="draft.agent_type === 'llm'" class="space-y-5">
            <h2 class="flex items-center gap-2 text-sm font-semibold">
              <ZapIcon class="size-4" />
              LLM 策略
            </h2>
            <div class="space-y-1.5">
              <Label>System Prompt</Label>
              <Textarea v-model="draft.prompt" rows="6" class="font-mono text-xs" placeholder="你是一名激进的上单选手…" />
            </div>
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div class="space-y-1.5">
                <Label>模型</Label>
                <Input v-model="draft.model" placeholder="deepseek/deepseek-chat" />
              </div>
              <div class="space-y-1.5">
                <Label>Preamble（全局前言）</Label>
                <Input v-model="draft.preamble" placeholder="覆盖全局 preamble" />
              </div>
            </div>
          </section>

          <!-- Script Agent: Monaco -->
          <section v-else-if="draft.agent_type === 'script'" class="space-y-4">
            <div class="flex items-center justify-between">
              <h2 class="flex items-center gap-2 text-sm font-semibold">
                <CodeIcon class="size-4" />
                Script 编辑器
              </h2>
              <label class="text-muted-foreground flex items-center gap-2 text-xs">
                <input type="checkbox" v-model="hotReload" />
                启用热重载
              </label>
            </div>

            <div class="overflow-hidden rounded-lg border">
              <component
                v-if="MonacoEditor"
                :is="MonacoEditor"
                v-model:value="scriptCode"
                language="javascript"
                theme="vs-dark"
                :height="420"
                :options="{ minimap: { enabled: false }, fontSize: 12, fontFamily: 'ui-monospace, SFMono-Regular' }"
              />
              <div v-else class="bg-muted/40 flex h-[420px] items-center justify-center text-xs">
                正在加载 Monaco 编辑器…
              </div>
            </div>

            <div class="text-muted-foreground flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
              <span class="flex items-center gap-1.5">
                <PlayIcon class="size-3" />
                支持断点 / 单步执行 / 变量观察
              </span>
              <span>API: observe() · action() · log()</span>
              <span>常用模板：补刀脚本 · 走 A · 技能连招</span>
            </div>
          </section>

          <!-- RL Agent: 权重 + 训练入口 -->
          <section v-else-if="draft.agent_type === 'rl'" class="space-y-5">
            <h2 class="flex items-center gap-2 text-sm font-semibold">
              <ZapIcon class="size-4" />
              RL 模型权重
            </h2>

            <div class="space-y-2">
              <Label>权重文件 (.pth)</Label>
              <div class="text-muted-foreground rounded-lg border border-dashed px-6 py-10 text-center text-xs">
                拖入或点击上传 PPO/SAC 训练后的 .pth 权重<br />
                或在桌面端"训练面板"中训练完成后一键应用。
              </div>
            </div>

            <Separator />

            <div class="flex items-center justify-between">
              <div class="space-y-0.5">
                <p class="text-sm font-medium">桌面端训练面板</p>
                <p class="text-muted-foreground text-xs">
                  超参 / Reward Shaper / 实时 Reward / Loss / KL 曲线
                </p>
              </div>
              <Button variant="outline" size="sm" @click="router.push('/rl-training')">
                <RocketIcon class="size-3.5" />
                打开训练面板
              </Button>
            </div>
          </section>

          <Separator />

          <!-- 参赛快照历史 -->
          <section v-if="selectedName" class="space-y-3">
            <div class="flex items-center justify-between">
              <h2 class="flex items-center gap-2 text-sm font-semibold">
                <HistoryIcon class="size-4" />
                参赛快照
              </h2>
              <span class="text-muted-foreground text-xs">
                Rank 队列始终使用最新发布的快照
              </span>
            </div>

            <div v-if="snapshots.length === 0" class="text-muted-foreground rounded-lg border border-dashed py-8 text-center text-xs">
              尚无快照。配置稳定后点击"发布参赛快照"冻结当前版本。
            </div>
            <ol v-else class="space-y-1.5">
              <li
                v-for="s in snapshots"
                :key="s.id"
                class="flex items-center justify-between rounded-md border px-4 py-2.5 text-sm"
              >
                <div class="flex items-center gap-3">
                  <Badge variant="secondary" class="font-mono">v{{ s.version }}</Badge>
                  <span class="text-muted-foreground text-xs">
                    {{ new Date(s.created_at).toLocaleString() }}
                  </span>
                </div>
                <Badge v-if="s === latestSnapshot" variant="outline">最新</Badge>
              </li>
            </ol>
          </section>

          <Separator v-if="selectedName" />

          <!-- 危险操作 -->
          <section v-if="selectedName" class="flex justify-end">
            <Button variant="ghost" size="sm" class="text-destructive hover:text-destructive" @click="showDeleteConfirm = true">
              <Trash2Icon class="size-3.5" />
              删除此配置
            </Button>
          </section>
        </div>
      </div>
    </section>

    <!-- 删除确认 -->
    <Dialog :open="showDeleteConfirm" @update:open="(v) => (showDeleteConfirm = v)">
      <DialogContent class="max-w-sm">
        <DialogHeader>
          <DialogTitle>删除 "{{ selectedName }}"</DialogTitle>
          <DialogDescription>此操作不可恢复，引用此配置的 Agent 将解绑。</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="ghost" @click="showDeleteConfirm = false">取消</Button>
          <Button variant="destructive" @click="confirmDelete">确认删除</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
