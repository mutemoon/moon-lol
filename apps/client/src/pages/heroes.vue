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
import { services } from "@/services/provider";

import type { Agent, AgentSnapshot, Visibility } from "@/services/types";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import ScriptEditor from "@/components/agent/ScriptEditor.vue";
import ForkDiffDialog from "@/components/agent/ForkDiffDialog.vue";

import { DEFAULT_SCRIPT } from "@/services/scriptAgentTemplates";
import { useProviders } from "@/stores/providersStore";
import { PLATFORM_PROVIDER_ID } from "@/config/providerPresets";
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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
  DownloadIcon,
  UploadIcon,
  AlertCircleIcon,
} from "@lucide/vue";

// 我的选手 (Agent) 管理页（产品文档 §2.1 / §2.5）。
//
// 这页面的数据有两层语义：
//   1. 配置（本地预设）：名字、英雄、决策类型与策略体（Prompt 或 JSON），随时可改、随时保存；
//   2. 参赛资产（云端 Agent）：可见性与按版本号冻结的「参赛快照」，是 Rank 队列真正取用的对象。
// 这两层用 Tabs 隔开，浏览态用响应式 Card 网格平铺，弱化「左列表 / 右编辑」的 CRUD 套路。

const store = useGameStore();
const { heroPresets, champions } = storeToRefs(store);
const providersStore = useProviders();
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
// Script Agent 的脚本源码单独维护，保存时写入 config_json.script。
const scriptSource = ref("");
// LLM Agent 的思考深度（1-5），保存时写入 config_json.thinking_depth。
const thinkingDepth = ref(2);
// LLM Agent 的模型供应商（写 config_json.provider_id）与模型名（写 draft.model）。
const providerId = ref<string>(PLATFORM_PROVIDER_ID);
const manualModel = ref(false);
// 平台模型：管理员在服务端 env 配置的可选模型名，用户只能选不能手填。
const platformModels = ref<string[]>([]);

const enabledProviders = computed(() => providersStore.providers.filter((p) => p.enabled));
const isPlatform = computed(() => providerId.value === PLATFORM_PROVIDER_ID);
const selectedProvider = computed(() => enabledProviders.value.find((p) => p.id === providerId.value));
// 平台模型用管理员提供的清单；BYO 供应商用其 models 列表。
const modelOptions = computed(() => {
  if (isPlatform.value) {
    return platformModels.value;
  }
  const models = selectedProvider.value?.models ?? [];
  return models.map((m: any) => typeof m === "string" ? m : m.name);
});
// 当前 model 是否不在所选供应商的模型列表里 → 自动切手填（平台模型不允许手填）。
const modelIsManual = computed(
  () =>
    !isPlatform.value &&
    (manualModel.value || (!!draft.value.model && !modelOptions.value.includes(draft.value.model))),
);

async function loadPlatformModels() {
  try {
    platformModels.value = await services.cloud.listPlatformModels();
  } catch {
    platformModels.value = [];
  }
}

function onProviderChange(v: any) {
  providerId.value = String(v);
  manualModel.value = false;
}

function onModelSelect(v: any) {
  draft.value.model = String(v);
  manualModel.value = false;
}
// RL Agent 配置：权重路径 (.pth)、BYO 推理端点、Reward Shaper 权重表。
const RL_REWARD_KEYS = ["last_hit", "kill", "death", "assist", "gold", "level", "health", "time", "proximity"] as const;
const defaultRlRewards = (): Record<string, number> => ({
  last_hit: 1.0,
  kill: 5.0,
  death: -5.0,
  assist: 2.0,
  gold: 0.0,
  level: 1.0,
  health: 1.0,
  time: -0.001,
  proximity: 0.0,
});
const rlModelPath = ref("");
const rlEndpoint = ref("");
const rlRewards = ref<Record<string, number>>(defaultRlRewards());

const errorMsg = ref("");
const successMsg = ref("");
const showDeleteConfirm = ref(false);

// 云端 Agent 与本地预设按名称对齐：本地预设是「可编辑稿」，云端 Agent 承载发布快照与可见性。
const cloudAgents = ref<Agent[]>([]);
const snapshotsByAgentId = ref<Record<string, AgentSnapshot[]>>({});
const publishing = ref(false);
// Fork 上游同步：解析出的上游 Agent、差异对话框开关、拉取中状态。
const upstreamAgent = ref<Agent | null>(null);
const showForkDiff = ref(false);
const pulling = ref(false);

// 显示用预设：直接显示云端/Store中的预设
const displayPresets = computed<HeroPreset[]>(() => heroPresets.value);

function cloudAgentByName(name: string): Agent | undefined {
  return cloudAgents.value.find((a) => a.name === name);
}

const currentCloudAgent = computed(() => (draft.value.name ? cloudAgentByName(draft.value.name) : undefined));

const currentSnapshots = computed<AgentSnapshot[]>(() => {
  const ca = currentCloudAgent.value;
  return ca ? (snapshotsByAgentId.value[ca.id] ?? []) : [];
});

function snapshotsFor(name: string): AgentSnapshot[] {
  const ca = cloudAgentByName(name);
  return ca ? (snapshotsByAgentId.value[ca.id] ?? []) : [];
}

function visibilityFor(name: string): Visibility {
  return cloudAgentByName(name)?.visibility ?? "private";
}

async function loadCloudAgents() {
  try {
    cloudAgents.value = await services.cloud.listAgents();
    await Promise.all(
      cloudAgents.value.map(async (a) => {
        try {
          snapshotsByAgentId.value[a.id] = await services.cloud.listSnapshots(a.id);
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
  const p = displayPresets.value.find((x) => x.name === name);
  if (!p) return;
  editingName.value = name;
  draft.value = { ...p };
  configJsonStr.value = JSON.stringify(p.config_json || {}, null, 2);
  scriptSource.value = typeof p.config_json?.script === "string" ? p.config_json.script : DEFAULT_SCRIPT;
  thinkingDepth.value = typeof p.config_json?.thinking_depth === "number" ? p.config_json.thinking_depth : 2;
  providerId.value = typeof p.config_json?.provider_id === "string" ? p.config_json.provider_id : PLATFORM_PROVIDER_ID;
  manualModel.value = false;
  rlModelPath.value = typeof p.config_json?.model_path === "string" ? p.config_json.model_path : "";
  rlEndpoint.value = typeof p.config_json?.inference_endpoint === "string" ? p.config_json.inference_endpoint : "";
  rlRewards.value = { ...defaultRlRewards(), ...(p.config_json?.reward_shaper || {}) };
  errorMsg.value = "";
  successMsg.value = "";
  mode.value = "edit";
  void loadUpstream();
}

function startNew() {
  editingName.value = null;
  draft.value = emptyDraft();
  configJsonStr.value = "{}";
  scriptSource.value = DEFAULT_SCRIPT;
  thinkingDepth.value = 2;
  providerId.value = PLATFORM_PROVIDER_ID;
  manualModel.value = false;
  rlModelPath.value = "";
  rlEndpoint.value = "";
  rlRewards.value = defaultRlRewards();
  upstreamAgent.value = null;
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
  if (draft.value.agent_type === "script") {
    draft.value.config_json = { script: scriptSource.value };
  } else if (draft.value.agent_type === "rl") {
    draft.value.config_json = {
      model_path: rlModelPath.value.trim(),
      inference_endpoint: rlEndpoint.value.trim(),
      reward_shaper: { ...rlRewards.value },
    };
  } else {
    draft.value.config_json = {
      thinking_depth: thinkingDepth.value,
      ...(providerId.value && providerId.value !== PLATFORM_PROVIDER_ID ? { provider_id: providerId.value } : {}),
    };
  }
  try {
    await store.saveHeroPreset({ ...draft.value, name });
    editingName.value = name;
    await loadCloudAgents();
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
    const snap = await services.cloud.publishSnapshot(ca.id);
    snapshotsByAgentId.value[ca.id] = [snap, ...(snapshotsByAgentId.value[ca.id] || [])];
    successMsg.value = t("heroes.publishSuccess", { version: snap.version });
  } catch (e: any) {
    errorMsg.value = e.message || t("heroes.publishFailed");
  } finally {
    publishing.value = false;
  }
}

// ── 上游同步（Fork 溯源） ──
const upstreamId = computed<string | null>(() => {
  const ca = currentCloudAgent.value;
  return ca ? (ca.upstream_agent_id ?? ca.forked_from ?? null) : null;
});

async function loadUpstream() {
  upstreamAgent.value = null;
  const id = upstreamId.value;
  if (!id) return;
  try {
    upstreamAgent.value = await services.cloud.getAgent(id);
  } catch {
    upstreamAgent.value = null;
  }
}

async function applyPull() {
  const ca = currentCloudAgent.value;
  if (!ca) return;
  pulling.value = true;
  errorMsg.value = "";
  successMsg.value = "";
  try {
    await services.cloud.pullUpstream(ca.id);
    await Promise.all([store.loadHeroPresets(), loadCloudAgents()]);
    if (editingName.value) enterEdit(editingName.value);
    showForkDiff.value = false;
    successMsg.value = t("heroes.pullSuccess");
  } catch (e: any) {
    errorMsg.value = e?.message || t("heroes.pullFailed");
  } finally {
    pulling.value = false;
  }
}

async function handleVisibilityChange(v: Visibility) {
  errorMsg.value = "";
  successMsg.value = "";
  const ca = currentCloudAgent.value;
  if (!ca) return;
  try {
    await services.cloud.updateAgentVisibility(ca.id, v);
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

// 未发布改动指示：云端 Agent 的 updated_at 晚于最新快照，说明改了配置但没重新发布。
// 从未发布过快照的（且已注册云端）也视为「有未发布改动」。
function hasUnpublishedChanges(name: string): boolean {
  const ca = cloudAgentByName(name);
  if (!ca) return false;
  const snaps = snapshotsFor(name);
  const latest = snaps[0];
  if (!latest) return true;
  return new Date(ca.updated_at).getTime() > new Date(latest.created_at).getTime();
}

const currentHasUnpublished = computed(() => (draft.value.name ? hasUnpublishedChanges(draft.value.name) : false));

// ── JSON 导入导出 ──
const importInput = ref<HTMLInputElement | null>(null);

function exportPresets() {
  const data = JSON.stringify(heroPresets.value, null, 2);
  const blob = new Blob([data], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `moonlol-agents-${new Date().toISOString().slice(0, 10)}.json`;
  a.click();
  URL.revokeObjectURL(url);
}

function triggerImport() {
  importInput.value?.click();
}

async function importPresets(ev: Event) {
  const target = ev.target as HTMLInputElement;
  const file = target.files?.[0];
  errorMsg.value = "";
  successMsg.value = "";
  if (!file) return;
  try {
    const parsed = JSON.parse(await file.text());
    const list: HeroPreset[] = Array.isArray(parsed) ? parsed : [parsed];
    let imported = 0;
    for (const p of list) {
      if (p && typeof p.name === "string" && p.name.trim()) {
        await store.saveHeroPreset({ ...emptyDraft(), ...p, name: p.name.trim() });
        imported += 1;
      }
    }
    await Promise.all([store.loadHeroPresets(), loadCloudAgents()]);
    successMsg.value = t("heroes.importSuccess", { count: imported });
  } catch {
    errorMsg.value = t("heroes.importFailed");
  } finally {
    target.value = "";
  }
}

onMounted(async () => {
  await Promise.all([
    store.loadHeroPresets(),
    loadCloudAgents(),
    providersStore.load(),
    loadPlatformModels(),
  ]);
  // 深链编辑：编排页「编辑」按钮跳转 /heroes?edit=<name>，自动进入编辑态。
  const editName = route.query.edit;
  if (typeof editName === "string" && editName && displayPresets.value.some((p) => p.name === editName)) {
    enterEdit(editName);
  }
});
</script>

<template>
  <div class="bg-background flex h-full w-full flex-col overflow-hidden">
    <!-- 顶栏：单行紧凑，靠留白与字号建立层级，不依赖边框 -->
    <header class="flex shrink-0 items-center justify-between px-6 py-4">
      <div class="flex items-center gap-3">
        <Button variant="ghost" size="icon" @click="mode === 'edit' ? backToBrowse() : router.push('/')">
          <ArrowLeftIcon class="size-4" />
        </Button>
        <h1 class="text-lg font-semibold tracking-tight">{{ t("heroes.title") }}</h1>
        <span class="text-muted-foreground text-sm tabular-nums">{{ displayPresets.length }}</span>
      </div>
      <div v-if="mode === 'browse'" class="flex items-center gap-2">
        <input ref="importInput" type="file" accept="application/json,.json" class="hidden" @change="importPresets" />
        <Button variant="ghost" size="sm" @click="triggerImport" data-testid="import-presets-btn">
          <UploadIcon class="size-4" />
          {{ t("heroes.importBtn") }}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          :disabled="displayPresets.length === 0"
          @click="exportPresets"
          data-testid="export-presets-btn"
        >
          <DownloadIcon class="size-4" />
          {{ t("heroes.exportBtn") }}
        </Button>

        <Button size="sm" @click="startNew" data-testid="new-preset-btn">
          <PlusIcon class="size-4" />
          {{ t("heroes.newPreset") }}
        </Button>
      </div>
    </header>
    <Separator />

    <div class="min-h-0 flex-1 overflow-y-auto">
      <!-- Browse：响应式 Card 网格，把"选手"当作可平铺的对象 -->
      <div v-if="mode === 'browse'" class="px-6 py-6">
        <div v-if="displayPresets.length === 0" class="text-muted-foreground flex flex-col items-center gap-4 py-24">
          <span class="text-sm">{{ t("heroes.emptyList") }}</span>
          <Button variant="outline" @click="startNew">
            <PlusIcon class="size-4" />
            {{ t("heroes.newPreset") }}
          </Button>
        </div>
        <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <Card
            v-for="p in displayPresets"
            :key="p.name"
            size="sm"
            class="hover:bg-accent/40 cursor-pointer transition"
            data-testid="preset-card"
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
            <CardContent class="text-muted-foreground flex items-center justify-between text-xs">
              <Badge variant="secondary" class="font-normal">
                {{ t("heroes.visibility." + visibilityFor(p.name)) }}
              </Badge>
              <div class="flex items-center gap-2">

                <Badge
                  v-if="hasUnpublishedChanges(p.name)"
                  variant="outline"
                  class="gap-1 text-[10px] text-amber-500"
                  data-testid="preset-dirty-badge"
                >
                  <AlertCircleIcon class="size-3" />
                  {{ t("heroes.unpublished") }}
                </Badge>
                <span class="font-mono tabular-nums">{{ latestSnapshotLabel(p.name) }}</span>
              </div>
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
            data-testid="preset-delete-btn"
          >
            <Trash2Icon class="size-4" />
            {{ t("heroes.deleteBtn") }}
          </Button>
        </div>

        <Tabs default-value="config">
          <TabsList>
            <TabsTrigger value="config">{{ t("heroes.tabConfig") }}</TabsTrigger>
            <TabsTrigger value="publish" data-testid="preset-tab-publish">
              {{ t("heroes.tabPublish") }}
              <Badge v-if="currentSnapshots[0]" variant="secondary" class="ml-2 font-mono text-[10px]">
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
                data-testid="preset-name-input"
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
                  data-testid="preset-prompt-input"
                />
              </div>
              <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div class="space-y-3">
                  <div class="space-y-2">
                    <Label>{{ t("heroes.providerLabel") }}</Label>
                    <Select :model-value="providerId" @update:model-value="onProviderChange">
                      <SelectTrigger class="w-full">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem :value="PLATFORM_PROVIDER_ID">{{ t("heroes.providerPlatform") }}</SelectItem>
                        <SelectItem v-for="p in enabledProviders" :key="p.id" :value="p.id">
                          {{ p.name }}
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div class="space-y-2">
                    <div class="flex items-center justify-between">
                      <Label>{{ t("heroes.modelSelectLabel") }}</Label>
                      <Button
                        v-if="!isPlatform"
                        variant="ghost"
                        size="sm"
                        class="h-6 text-[11px]"
                        @click="manualModel = !manualModel"
                      >
                        {{ t("heroes.modelManual") }}
                      </Button>
                    </div>
                    <Input
                      v-if="!isPlatform && (modelIsManual || modelOptions.length === 0)"
                      v-model="draft.model"
                      :placeholder="t('heroes.modelPlaceholder')"
                    />
                    <Select v-else :model-value="draft.model" @update:model-value="onModelSelect">
                      <SelectTrigger class="w-full">
                        <SelectValue :placeholder="t('heroes.modelPlaceholder')" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem v-for="m in modelOptions" :key="m" :value="m">{{ m }}</SelectItem>
                      </SelectContent>
                    </Select>
                    <p v-if="isPlatform && platformModels.length === 0" class="text-muted-foreground text-[11px]">
                      {{ t("heroes.platformModelsEmpty") }}
                    </p>
                  </div>
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
              <div class="space-y-2">
                <div class="flex items-center justify-between">
                  <Label>{{ t("heroes.thinkingDepthLabel") }}</Label>
                  <span class="text-muted-foreground font-mono text-xs tabular-nums">{{ thinkingDepth }} / 5</span>
                </div>
                <input
                  v-model.number="thinkingDepth"
                  type="range"
                  min="1"
                  max="5"
                  step="1"
                  class="accent-primary h-1.5 w-full cursor-pointer"
                  data-testid="thinking-depth-slider"
                />
                <p class="text-muted-foreground text-[11px] leading-relaxed">
                  {{ t("heroes.thinkingDepthDesc") }}
                </p>
              </div>
            </template>

            <template v-else-if="draft.agent_type === 'script'">
              <div class="space-y-2">
                <Label>{{ t("heroes.scriptLabel") }}</Label>
                <ScriptEditor v-model="scriptSource" />
              </div>
            </template>

            <template v-else>
              <div class="space-y-2">
                <Label>{{ t("heroes.rlModelPathLabel") }}</Label>
                <Input
                  v-model="rlModelPath"
                  :placeholder="t('heroes.rlModelPathPlaceholder')"
                  class="font-mono text-xs"
                  data-testid="rl-model-path-input"
                />
              </div>
              <div class="space-y-2">
                <Label>{{ t("heroes.rlEndpointLabel") }}</Label>
                <Input
                  v-model="rlEndpoint"
                  :placeholder="t('heroes.rlEndpointPlaceholder')"
                  class="font-mono text-xs"
                  data-testid="rl-endpoint-input"
                />
              </div>
              <div class="space-y-2">
                <Label>{{ t("heroes.rlRewardShaperLabel") }}</Label>
                <div class="grid grid-cols-2 gap-2 sm:grid-cols-3">
                  <div v-for="k in RL_REWARD_KEYS" :key="k" class="space-y-1">
                    <Label class="text-muted-foreground text-[11px]">{{ k }}</Label>
                    <Input v-model.number="rlRewards[k]" type="number" step="0.1" class="h-8 font-mono text-xs" />
                  </div>
                </div>
              </div>
            </template>

            <div class="flex items-center justify-between pt-2">
              <div class="text-xs">
                <span v-if="errorMsg" class="text-destructive">{{ errorMsg }}</span>
                <span v-else-if="successMsg" class="text-foreground" data-testid="preset-save-success">
                  {{ successMsg }}
                </span>
              </div>
              <div class="flex items-center gap-2">

                <Button :disabled="!draft.name.trim()" @click="handleSave" data-testid="preset-save-btn">
                  {{ t("heroes.saveBtn") }}
                </Button>
              </div>
            </div>
          </TabsContent>

          <!-- 发布与快照 -->
          <TabsContent value="publish" class="space-y-8 pt-6">
            <!-- 上游同步（Fork 溯源） -->
            <section v-if="upstreamId" class="space-y-3">
              <div>
                <h3 class="text-sm font-semibold">{{ t("heroes.upstreamSyncTitle") }}</h3>
                <p class="text-muted-foreground text-xs leading-relaxed">
                  {{ t("heroes.upstreamSyncDesc") }}
                </p>
              </div>
              <div class="flex items-center justify-between gap-4 rounded-md border px-4 py-3">
                <div class="min-w-0 text-sm">
                  <span class="text-muted-foreground">{{ t("heroes.forkFromLabel") }}</span>
                  <span class="font-medium" data-testid="fork-from-name">「{{ upstreamAgent?.name ?? "…" }}」</span>
                  <span class="text-muted-foreground text-xs" v-if="upstreamAgent">
                    · {{ t("heroes.forkAuthor") }} #{{ upstreamAgent.owner_id }}
                  </span>
                </div>
                <Button
                  size="sm"
                  variant="outline"
                  :disabled="!upstreamAgent"
                  @click="showForkDiff = true"
                  data-testid="pull-upstream-btn"
                >
                  {{ t("heroes.pullUpstreamBtn") }}
                </Button>
              </div>
            </section>

            <Separator v-if="upstreamId" />

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
                  <p
                    v-if="currentHasUnpublished"
                    class="mt-1 flex items-center gap-1 text-[11px] text-amber-500"
                    data-testid="publish-dirty-hint"
                  >
                    <AlertCircleIcon class="size-3" />
                    {{ t("heroes.unpublishedHint") }}
                  </p>
                </div>
                <Button
                  :disabled="publishing || !editingName"
                  @click="handlePublishSnapshot"
                  data-testid="preset-publish-btn"
                >
                  <RocketIcon class="size-4" />
                  {{ publishing ? t("heroes.publishing") : t("heroes.publishBtn") }}
                </Button>
              </div>

              <p v-if="errorMsg" class="text-destructive text-xs">{{ errorMsg }}</p>
              <p v-else-if="successMsg" class="text-foreground text-xs">{{ successMsg }}</p>
              <p v-else-if="editingName && !currentCloudAgent" class="text-muted-foreground text-xs leading-relaxed">
                {{ t("heroes.publishNeedsSync") }}
              </p>

              <!-- 历史快照：append-only 的时间线，用列表 + 顶部「最新」徽章表达即可，
                   不必额外的颜色或框选噪声 -->
              <div class="space-y-2 pt-2">
                <div class="text-muted-foreground text-[11px] font-medium tracking-wider uppercase">
                  {{ t("heroes.historyTitle") }}
                </div>
                <ul v-if="currentSnapshots.length" class="divide-border divide-y rounded-md border">
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
                        data-testid="snapshot-latest-badge"
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



    <!-- 拉取上游更新：差异对比与合并预览 -->
    <ForkDiffDialog
      :open="showForkDiff"
      :upstream-name="upstreamAgent?.name ?? ''"
      :upstream-author="upstreamAgent?.owner_id ?? null"
      :current-prompt="currentCloudAgent?.prompt ?? ''"
      :upstream-prompt="upstreamAgent?.prompt ?? ''"
      :current-config="JSON.stringify(currentCloudAgent?.config_json ?? {}, null, 2)"
      :upstream-config="JSON.stringify(upstreamAgent?.config_json ?? {}, null, 2)"
      :applying="pulling"
      @update:open="(v) => (showForkDiff = v)"
      @apply="applyPull"
    />

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
          <Button variant="destructive" @click="confirmDelete" data-testid="preset-delete-confirm-btn">
            {{ t("heroes.confirmDeleteBtn") }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
