<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { storeToRefs } from "pinia";
import { useGameStore, type HeroPreset } from "@/stores/gameStore";
import { useRouter, useRoute } from "vue-router";
import { useLocale } from "@/composables/useLocale";
import {
  agentsApi,
  type Agent,
  type AgentSnapshot,
  type Visibility,
} from "@/services/cloudApi";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/ui/card";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  PlusIcon,
  Trash2Icon,
  ArrowLeftIcon,
  RocketIcon,
  CheckIcon,
} from "@lucide/vue";

// 我的选手 (Agent) 管理页（产品文档 §2.1 / §2.5）。
//
// 这页面的数据有两层语义：
//   1. 配置（本地预设）：名字、英雄、决策类型与策略体（Prompt 或 JSON），随时可改、随时保存；
//   2. 参赛资产（云端 Agent）：可见性与按版本号冻结的「参赛快照」，是 Rank 队列真正取用的对象。
// 这两层用 Tabs 隔开，浏览态用响应式 Card 网格平铺，弱化「左列表 / 右编辑」的 CRUD 套路。

const store = useGameStore();
const { heroPresets, champions } = storeToRefs(store);
const router = useRouter();
const route = useRoute();
const { t } = useLocale();

type Mode = "browse" | "edit";
const mode = ref<Mode>("browse");
const editingName = ref<string | null>(null);

const emptyDraft = (): HeroPreset => ({
  name: "",
  champion: "Riven",
  agent_type: "llm",
  prompt: "",
  preamble: "",
  model: "",
  config_json: {},
});
const draft = ref<HeroPreset>(emptyDraft());
const configJsonStr = ref("{}");

const errorMsg = ref("");
const successMsg = ref("");
const showDeleteConfirm = ref(false);

// 云端 Agent 与本地预设按名称对齐：本地预设是「可编辑稿」，云端 Agent 承载发布快照与可见性。
const cloudAgents = ref<Agent[]>([]);
const snapshotsByAgentId = ref<Record<string, AgentSnapshot[]>>({});
const publishing = ref(false);

function cloudAgentByName(name: string): Agent | undefined {
  return cloudAgents.value.find((a) => a.name === name);
}

const currentCloudAgent = computed(() =>
  draft.value.name ? cloudAgentByName(draft.value.name) : undefined,
);

const currentSnapshots = computed<AgentSnapshot[]>(() => {
  const ca = currentCloudAgent.value;
  return ca ? snapshotsByAgentId.value[ca.id] ?? [] : [];
});

function snapshotsFor(name: string): AgentSnapshot[] {
  const ca = cloudAgentByName(name);
  return ca ? snapshotsByAgentId.value[ca.id] ?? [] : [];
}

function visibilityFor(name: string): Visibility {
  return cloudAgentByName(name)?.visibility ?? "private";
}

async function loadCloudAgents() {
  try {
    cloudAgents.value = await agentsApi.list();
    await Promise.all(
      cloudAgents.value.map(async (a) => {
        try {
          snapshotsByAgentId.value[a.id] = await agentsApi.listSnapshots(a.id);
        } catch {
          snapshotsByAgentId.value[a.id] = [];
        }
      }),
    );
  } catch {
    cloudAgents.value = [];
  }
}

function enterEdit(name: string) {
  const p = heroPresets.value.find((x) => x.name === name);
  if (!p) return;
  editingName.value = name;
  draft.value = { ...p };
  configJsonStr.value = JSON.stringify(p.config_json || {}, null, 2);
  errorMsg.value = "";
  successMsg.value = "";
  mode.value = "edit";
}

function startNew() {
  editingName.value = null;
  draft.value = emptyDraft();
  configJsonStr.value = "{}";
  errorMsg.value = "";
  successMsg.value = "";
  mode.value = "edit";
}

function backToBrowse() {
  mode.value = "browse";
  editingName.value = null;
  errorMsg.value = "";
  successMsg.value = "";
}

async function handleSave() {
  errorMsg.value = "";
  successMsg.value = "";
  const name = draft.value.name.trim();
  if (!name) {
    errorMsg.value = t("heroes.errorFillName");
    return;
  }
  if (draft.value.agent_type !== "llm") {
    try {
      draft.value.config_json = JSON.parse(configJsonStr.value || "{}");
    } catch {
      errorMsg.value = t("heroes.invalidJson");
      return;
    }
  } else {
    draft.value.config_json = {};
  }
  try {
    await store.saveHeroPreset({ ...draft.value, name });
    editingName.value = name;
    successMsg.value = t("heroes.successSave");
  } catch (e: any) {
    errorMsg.value = e.message || t("heroes.errorSave");
  }
}

async function handlePublishSnapshot() {
  errorMsg.value = "";
  successMsg.value = "";
  const ca = currentCloudAgent.value;
  if (!ca) {
    errorMsg.value = t("heroes.publishNeedsSync");
    return;
  }
  publishing.value = true;
  try {
    const snap = await agentsApi.publishSnapshot(ca.id);
    snapshotsByAgentId.value[ca.id] = [snap, ...(snapshotsByAgentId.value[ca.id] || [])];
    successMsg.value = t("heroes.publishSuccess", { version: snap.version });
  } catch (e: any) {
    errorMsg.value = e.message || t("heroes.publishFailed");
  } finally {
    publishing.value = false;
  }
}

async function handleVisibilityChange(v: Visibility) {
  errorMsg.value = "";
  successMsg.value = "";
  const ca = currentCloudAgent.value;
  if (!ca) return;
  try {
    await agentsApi.updateVisibility(ca.id, v);
    ca.visibility = v;
    successMsg.value = t("heroes.visibilityUpdated");
  } catch (e: any) {
    errorMsg.value = e.message || t("heroes.visibilityFailed");
  }
}

async function confirmDelete() {
  if (!editingName.value) return;
  try {
    await store.deleteHeroPreset(editingName.value);
    showDeleteConfirm.value = false;
    backToBrowse();
  } catch (e: any) {
    errorMsg.value = e.message || t("heroes.errorDelete");
  }
}

function ago(iso: string): string {
  const diff = (Date.now() - new Date(iso).getTime()) / 1000;
  if (diff < 60) return `${Math.floor(diff)}s`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
  return `${Math.floor(diff / 86400)}d`;
}

function latestSnapshotLabel(name: string): string {
  const list = snapshotsFor(name);
  const first = list[0];
  if (!first) return t("heroes.noPublish");
  return `v${first.version}`;
}

onMounted(async () => {
  await Promise.all([store.loadHeroPresets(), loadCloudAgents()]);
  // 深链编辑：编排页「编辑」按钮跳转 /heroes?edit=<name>，自动进入编辑态。
  const editName = route.query.edit;
  if (
    typeof editName === "string" &&
    editName &&
    heroPresets.value.some((p) => p.name === editName)
  ) {
    enterEdit(editName);
  }
});
</script>

<template>
  <div class="bg-background flex h-full w-full flex-col overflow-hidden">
    <!-- 顶栏：单行紧凑，靠留白与字号建立层级，不依赖边框 -->
    <header class="flex shrink-0 items-center justify-between px-6 py-4">
      <div class="flex items-center gap-3">
        <Button
          variant="ghost"
          size="icon"
          @click="mode === 'edit' ? backToBrowse() : router.push('/')"
        >
          <ArrowLeftIcon class="size-4" />
        </Button>
        <h1 class="text-lg font-semibold tracking-tight">{{ t("heroes.title") }}</h1>
        <span class="text-muted-foreground text-sm tabular-nums">{{ heroPresets.length }}</span>
      </div>
      <Button v-if="mode === 'browse'" size="sm" @click="startNew">
        <PlusIcon class="size-4" />
        {{ t("heroes.newPreset") }}
      </Button>
    </header>
    <Separator />

    <div class="min-h-0 flex-1 overflow-y-auto">
      <!-- Browse：响应式 Card 网格，把"选手"当作可平铺的对象 -->
      <div v-if="mode === 'browse'" class="px-6 py-6">
        <div
          v-if="heroPresets.length === 0"
          class="text-muted-foreground flex flex-col items-center gap-4 py-24"
        >
          <span class="text-sm">{{ t("heroes.emptyList") }}</span>
          <Button variant="outline" @click="startNew">
            <PlusIcon class="size-4" />
            {{ t("heroes.newPreset") }}
          </Button>
        </div>
        <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <Card
            v-for="p in heroPresets"
            :key="p.name"
            size="sm"
            class="hover:bg-accent/40 cursor-pointer transition"
            @click="enterEdit(p.name)"
          >
            <CardHeader>
              <div class="flex items-start justify-between gap-2">
                <CardTitle class="truncate text-base">{{ p.name }}</CardTitle>
                <Badge variant="outline" class="font-mono text-[10px] tracking-wider uppercase">
                  {{ p.agent_type }}
                </Badge>
              </div>
              <CardDescription>{{ t("champions." + p.champion) }}</CardDescription>
            </CardHeader>
            <CardContent
              class="text-muted-foreground flex items-center justify-between text-xs"
            >
              <Badge variant="secondary" class="font-normal">
                {{ t("heroes.visibility." + visibilityFor(p.name)) }}
              </Badge>
              <span class="font-mono tabular-nums">{{ latestSnapshotLabel(p.name) }}</span>
            </CardContent>
          </Card>
        </div>
      </div>

      <!-- Edit：单栏聚焦视图，配置与发布拆 Tabs，避免长 Prompt 与历史快照互相挤占 -->
      <div v-else class="mx-auto max-w-3xl px-6 py-8">
        <div class="mb-6 flex items-end justify-between gap-4">
          <div class="min-w-0">
            <p class="text-muted-foreground text-xs">
              {{ editingName ? t("heroes.editing") : t("heroes.creating") }}
            </p>
            <h2 class="truncate text-2xl font-semibold tracking-tight">
              {{ draft.name || t("heroes.untitled") }}
            </h2>
          </div>
          <Button
            v-if="editingName"
            variant="ghost"
            size="sm"
            class="text-destructive hover:text-destructive"
            @click="showDeleteConfirm = true"
          >
            <Trash2Icon class="size-4" />
            {{ t("heroes.deleteBtn") }}
          </Button>
        </div>

        <Tabs default-value="config">
          <TabsList>
            <TabsTrigger value="config">{{ t("heroes.tabConfig") }}</TabsTrigger>
            <TabsTrigger value="publish">
              {{ t("heroes.tabPublish") }}
              <Badge
                v-if="currentSnapshots[0]"
                variant="secondary"
                class="ml-2 font-mono text-[10px]"
              >
                v{{ currentSnapshots[0].version }}
              </Badge>
            </TabsTrigger>
          </TabsList>

          <!-- 配置 -->
          <TabsContent value="config" class="space-y-6 pt-6">
            <div class="space-y-2">
              <Label>{{ t("heroes.presetName") }}</Label>
              <Input
                v-model="draft.name"
                :placeholder="t('heroes.presetNamePlaceholder')"
              />
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-2">
                <Label>{{ t("heroes.heroLabel") }}</Label>
                <Select v-model="draft.champion">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem v-for="c in champions" :key="c" :value="c">
                      {{ t("champions." + c) }}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div class="space-y-2">
                <Label>{{ t("heroes.agentType") }}</Label>
                <Select v-model="draft.agent_type">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="llm">{{ t("agents.types.llm") }}</SelectItem>
                    <SelectItem value="rl">{{ t("agents.types.rl") }}</SelectItem>
                    <SelectItem value="script">{{ t("agents.types.script") }}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <p
              v-if="draft.agent_type !== 'llm'"
              class="text-muted-foreground text-xs leading-relaxed"
              v-html="t('agents.typeWarning', { type: draft.agent_type.toUpperCase() })"
            ></p>

            <template v-if="draft.agent_type === 'llm'">
              <div class="space-y-2">
                <Label>{{ t("heroes.promptLabel") }}</Label>
                <Textarea
                  v-model="draft.prompt"
                  :placeholder="t('heroes.promptPlaceholder')"
                  class="min-h-[140px] font-mono text-xs leading-relaxed"
                />
              </div>
              <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div class="space-y-2">
                  <Label>{{ t("heroes.modelLabel") }}</Label>
                  <Input
                    v-model="draft.model"
                    :placeholder="t('heroes.modelPlaceholder')"
                  />
                </div>
                <div class="space-y-2">
                  <Label>{{ t("heroes.preambleLabel") }}</Label>
                  <Textarea
                    v-model="draft.preamble"
                    :placeholder="t('heroes.preamblePlaceholder')"
                    class="min-h-[40px] font-mono text-xs leading-relaxed"
                  />
                </div>
              </div>
            </template>

            <template v-else>
              <div class="space-y-2">
                <Label>{{ t("heroes.configJsonLabel") }}</Label>
                <Textarea
                  v-model="configJsonStr"
                  :placeholder="t('heroes.configJsonPlaceholder')"
                  class="min-h-[200px] font-mono text-xs leading-relaxed"
                />
              </div>
            </template>

            <div class="flex items-center justify-between pt-2">
              <div class="text-xs">
                <span v-if="errorMsg" class="text-destructive">{{ errorMsg }}</span>
                <span v-else-if="successMsg" class="text-foreground">{{ successMsg }}</span>
              </div>
              <Button :disabled="!draft.name.trim()" @click="handleSave">
                {{ t("heroes.saveBtn") }}
              </Button>
            </div>
          </TabsContent>

          <!-- 发布与快照 -->
          <TabsContent value="publish" class="space-y-8 pt-6">
            <!-- 可见性 -->
            <section class="space-y-3">
              <div>
                <h3 class="text-sm font-semibold">{{ t("heroes.visibilityTitle") }}</h3>
                <p class="text-muted-foreground text-xs leading-relaxed">
                  {{ t("heroes.visibilityDesc") }}
                </p>
              </div>
              <Select
                :model-value="visibilityFor(draft.name)"
                :disabled="!currentCloudAgent"
                @update:model-value="(v) => handleVisibilityChange(v as Visibility)"
              >
                <SelectTrigger class="w-60">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="private">{{ t("heroes.visibility.private") }}</SelectItem>
                  <SelectItem value="friends">{{ t("heroes.visibility.friends") }}</SelectItem>
                  <SelectItem value="public">{{ t("heroes.visibility.public") }}</SelectItem>
                </SelectContent>
              </Select>
            </section>

            <Separator />

            <!-- 发布快照 -->
            <section class="space-y-4">
              <div class="flex items-end justify-between gap-4">
                <div class="min-w-0">
                  <h3 class="text-sm font-semibold">{{ t("heroes.publishTitle") }}</h3>
                  <p class="text-muted-foreground text-xs leading-relaxed">
                    {{ t("heroes.publishDesc") }}
                  </p>
                </div>
                <Button
                  :disabled="publishing || !editingName"
                  @click="handlePublishSnapshot"
                >
                  <RocketIcon class="size-4" />
                  {{ publishing ? t("heroes.publishing") : t("heroes.publishBtn") }}
                </Button>
              </div>

              <p v-if="errorMsg" class="text-destructive text-xs">{{ errorMsg }}</p>
              <p v-else-if="successMsg" class="text-foreground text-xs">{{ successMsg }}</p>
              <p
                v-else-if="editingName && !currentCloudAgent"
                class="text-muted-foreground text-xs leading-relaxed"
              >
                {{ t("heroes.publishNeedsSync") }}
              </p>

              <!-- 历史快照：append-only 的时间线，用列表 + 顶部「最新」徽章表达即可，
                   不必额外的颜色或框选噪声 -->
              <div class="space-y-2 pt-2">
                <div class="text-muted-foreground text-[11px] font-medium tracking-wider uppercase">
                  {{ t("heroes.historyTitle") }}
                </div>
                <ul
                  v-if="currentSnapshots.length"
                  class="divide-border divide-y rounded-md border"
                >
                  <li
                    v-for="(s, idx) in currentSnapshots"
                    :key="s.id"
                    class="flex items-center justify-between px-4 py-2.5"
                  >
                    <div class="flex items-center gap-3 text-sm">
                      <span class="font-mono tabular-nums">v{{ s.version }}</span>
                      <Badge
                        v-if="idx === 0"
                        variant="secondary"
                        class="gap-1 text-[10px]"
                      >
                        <CheckIcon class="size-3" />
                        {{ t("heroes.currentLatest") }}
                      </Badge>
                    </div>
                    <span class="text-muted-foreground text-xs tabular-nums">
                      {{ ago(s.created_at) }} {{ t("heroes.agoSuffix") }}
                    </span>
                  </li>
                </ul>
                <p v-else class="text-muted-foreground text-xs">
                  {{ t("heroes.noSnapshots") }}
                </p>
              </div>
            </section>
          </TabsContent>
        </Tabs>
      </div>
    </div>

    <!-- 删除确认 -->
    <Dialog :open="showDeleteConfirm" @update:open="(v) => (showDeleteConfirm = v)">
      <DialogContent class="max-w-sm">
        <DialogHeader>
          <DialogTitle>
            {{ t("heroes.deleteConfirmTitle", { name: editingName }) }}
          </DialogTitle>
          <DialogDescription>{{ t("heroes.deleteConfirmDesc") }}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="showDeleteConfirm = false">
            {{ t("heroes.cancelBtn") }}
          </Button>
          <Button variant="destructive" @click="confirmDelete">
            {{ t("heroes.confirmDeleteBtn") }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
