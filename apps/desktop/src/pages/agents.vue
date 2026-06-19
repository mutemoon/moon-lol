<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { storeToRefs } from "pinia";
import { useGameStore, type AgentPreset } from "../stores/gameStore";
import { useRouter } from "vue-router";
import { Button } from "../components/ui/button";
import { Badge } from "../components/ui/badge";
import { Input } from "../components/ui/input";
import { Textarea } from "../components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "../components/ui/dialog";
import {
  BotIcon,
  PlusIcon,
  Trash2Icon,
  SaveIcon,
  ArrowLeftIcon,
  InfoIcon,
} from "@lucide/vue";

// Agent 预设管理页（产品文档 §2.3 / §3.0）。
// Agent 预设是"策略大脑"（类型 + Prompt + 模型/前言），英雄无关。
// 英雄绑定由"英雄预设"管理页负责；编排页只选英雄预设。

const store = useGameStore();
const { agentPresets } = storeToRefs(store);
const router = useRouter();

const selectedName = ref<string | null>(null);
const errorMsg = ref("");
const showDeleteConfirm = ref(false);

const AGENT_TYPES = [
  { value: "llm", label: "LLM（语言模型）", soon: false },
  { value: "rl", label: "RL（强化学习）", soon: true },
  { value: "script", label: "Script（脚本）", soon: true },
];

// 正在编辑的草稿（独立于 store，编辑后显式保存）
const emptyDraft = (): AgentPreset => ({
  name: "",
  agent_type: "llm",
  prompt: "",
  preamble: "",
  model: "",
});
const draft = ref<AgentPreset>(emptyDraft());

function selectPreset(name: string) {
  selectedName.value = name;
  const p = agentPresets.value.find((x) => x.name === name);
  if (p) draft.value = { ...p };
}

function startNew() {
  selectedName.value = null;
  draft.value = emptyDraft();
}

async function handleSave() {
  errorMsg.value = "";
  const name = draft.value.name.trim();
  if (!name) {
    errorMsg.value = "请填写预设名称";
    return;
  }
  try {
    await store.saveAgentPreset({ ...draft.value, name });
    selectedName.value = name;
    errorMsg.value = "保存成功";
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

const currentTypeMeta = computed(() => AGENT_TYPES.find((t) => t.value === draft.value.agent_type));

onMounted(() => {
  store.loadAgentPresets();
});
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden bg-background p-4">
    <!-- 顶部 Header -->
    <header
      class="border-border bg-card flex shrink-0 items-center justify-between rounded-lg border px-4 py-2.5 shadow-sm"
    >
      <div class="flex items-center gap-2.5">
        <Button variant="ghost" size="icon" class="size-7" @click="router.push('/')">
          <ArrowLeftIcon class="size-4" />
        </Button>
        <div class="flex size-8 items-center justify-center rounded-lg bg-primary/10">
          <BotIcon class="text-primary size-4" />
        </div>
        <div class="flex items-baseline gap-2">
          <h1 class="text-foreground text-sm font-bold tracking-tight">Agent 预设管理</h1>
          <Badge variant="secondary" class="text-[10px]">{{ agentPresets.length }} 个</Badge>
        </div>
      </div>
      <Button variant="outline" size="xs" class="h-7 gap-1 text-[11px]" @click="startNew">
        <PlusIcon class="size-3" />
        新建预设
      </Button>
    </header>

    <div class="mt-3 flex min-h-0 flex-1 gap-3">
      <!-- 左：预设列表 -->
      <aside class="border-border bg-card w-60 shrink-0 overflow-hidden rounded-lg border shadow-sm">
        <div class="border-border shrink-0 border-b px-3 py-2 text-[11px] font-bold uppercase tracking-wide">
          预设列表
        </div>
        <div class="min-h-0 flex-1 overflow-y-auto p-2">
          <div v-if="agentPresets.length === 0" class="text-muted-foreground py-6 text-center text-xs italic">
            暂无预设
          </div>
          <div v-else class="flex flex-col gap-1">
            <button
              v-for="p in agentPresets"
              :key="p.name"
              class="hover:bg-muted border-border flex items-center justify-between rounded border px-2 py-1.5 text-left transition-colors"
              :class="p.name === selectedName ? 'border-primary/40 bg-primary/10' : 'bg-muted/30'"
              @click="selectPreset(p.name)"
            >
              <span class="flex min-w-0 flex-col">
                <span class="text-foreground truncate text-xs font-medium">{{ p.name }}</span>
                <span class="text-muted-foreground truncate text-[10px]">
                  {{ p.agent_type.toUpperCase() }}
                </span>
              </span>
            </button>
          </div>
        </div>
      </aside>

      <!-- 右：编辑表单 -->
      <section class="border-border bg-card min-w-0 flex-1 overflow-y-auto rounded-lg border p-5 shadow-sm">
        <div v-if="errorMsg" class="border-border mb-3 rounded border-l-2 px-3 py-1.5 text-xs"
          :class="errorMsg === '保存成功' ? 'border-green-500 text-green-500 bg-green-500/5' : 'border-destructive text-destructive bg-destructive/5'">
          {{ errorMsg }}
        </div>

        <div class="mx-auto flex max-w-xl flex-col gap-4">
          <!-- 名称 -->
          <div>
            <label class="text-muted-foreground mb-1 block text-[10px] font-semibold uppercase tracking-wider">
              预设名称
            </label>
            <Input
              v-model="draft.name"
              placeholder="如：激进压制 / 稳健发育 / 游走支援"
              class="border-border bg-muted/40 h-9 text-sm"
            />
          </div>

          <!-- Agent 类型 -->
          <div>
            <label class="text-muted-foreground mb-1 block text-[10px] font-semibold uppercase tracking-wider">
              Agent 类型
            </label>
            <Select v-model="draft.agent_type">
              <SelectTrigger class="bg-muted/40 border-border h-9 text-sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent class="border-border bg-popover text-foreground">
                <SelectItem v-for="t in AGENT_TYPES" :key="t.value" :value="t.value" class="text-sm">
                  {{ t.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- 说明：Agent 预设英雄无关 -->
          <div class="border-primary/20 bg-primary/5 text-foreground/70 flex items-start gap-1.5 rounded-md border px-3 py-2 text-[11px] leading-relaxed">
            <InfoIcon class="mt-0.5 size-3.5 shrink-0 text-primary" />
            <span>Agent 预设是<strong>英雄无关</strong>的"策略大脑"，可被任意英雄预设复用。英雄绑定在「英雄预设」管理页完成。</span>
          </div>

          <!-- 类型说明 -->
          <div v-if="currentTypeMeta?.soon" class="border-amber-500/30 bg-amber-500/5 text-amber-500 flex items-start gap-1.5 rounded-md border px-3 py-2 text-[11px] leading-relaxed">
            <InfoIcon class="mt-0.5 size-3.5 shrink-0" />
            <span>
              <strong>{{ currentTypeMeta.label }}</strong> 类型尚未接入运行时。该预设可保存，但在对局中会回退为 LLM 决策。
            </span>
          </div>

          <!-- Prompt -->
          <div>
            <label class="text-muted-foreground mb-1 block text-[10px] font-semibold uppercase tracking-wider">
              系统提示词 (Prompt)
            </label>
            <Textarea
              v-model="draft.prompt"
              rows="5"
              class="border-border bg-muted/20 text-foreground min-h-28 w-full resize-y text-xs"
              placeholder="描述该 Agent 的行为策略、对线风格、连招意图…"
            />
          </div>

          <!-- LLM 专属：模型 + 前言 -->
          <template v-if="draft.agent_type === 'llm'">
            <div>
              <label class="text-muted-foreground mb-1 block text-[10px] font-semibold uppercase tracking-wider">
                模型（留空用全局默认）
              </label>
              <Input
                v-model="draft.model"
                placeholder="如 claude-sonnet-4-5"
                class="border-border bg-muted/40 h-9 font-mono text-xs"
              />
            </div>
            <div>
              <label class="text-muted-foreground mb-1 block text-[10px] font-semibold uppercase tracking-wider">
                英雄级前言 Preamble（可选）
              </label>
              <Textarea
                v-model="draft.preamble"
                rows="3"
                class="border-border bg-muted/20 text-foreground min-h-16 w-full resize-y text-xs"
                placeholder="覆盖全局 preamble 的英雄级补充说明…"
              />
            </div>
          </template>

          <!-- 操作 -->
          <div class="border-border mt-2 flex items-center gap-2 border-t pt-4">
            <Button class="gap-1.5" :disabled="!draft.name.trim()" @click="handleSave">
              <SaveIcon class="size-3.5" />
              保存预设
            </Button>
            <Button
              v-if="selectedName"
              variant="outline"
              class="border-destructive/20 bg-destructive/5 text-destructive hover:bg-destructive hover:text-destructive-foreground gap-1.5"
              @click="showDeleteConfirm = true"
            >
              <Trash2Icon class="size-3.5" />
              删除
            </Button>
          </div>
        </div>
      </section>
    </div>

    <!-- 删除确认 -->
    <Dialog :open="showDeleteConfirm" @update:open="(v) => (showDeleteConfirm = v)">
      <DialogContent class="border-border bg-card text-foreground max-w-sm p-6">
        <DialogHeader>
          <DialogTitle class="text-foreground text-sm">删除预设「{{ selectedName }}」？</DialogTitle>
          <DialogDescription class="text-muted-foreground text-[11px]">
            该操作不可撤销。引用此预设的场景槽位需手动重新选择。
          </DialogDescription>
        </DialogHeader>
        <DialogFooter class="gap-2">
          <Button variant="outline" size="sm" @click="showDeleteConfirm = false">取消</Button>
          <Button variant="destructive" size="sm" @click="confirmDelete">删除</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
