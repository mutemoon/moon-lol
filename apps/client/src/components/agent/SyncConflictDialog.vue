<script setup lang="ts">
// 本地 ↔ 云端 选手冲突解决对话框（产品文档：桌面端云优先 + 冲突解决）。
//
// 离线编辑后回到在线，本地缓存可能与云端不一致。本对话框逐项列出差异选手，
// 左右两列并排展示「云端」与「本地」内容，用户为每一项选择保留哪边：
//   保留本地 → 推送本地覆盖云端；保留云端 → 拉取云端覆盖本地。
// 缺失的一侧显示「（云端无）」「（本地无）」，仍可选择保留另一侧（即推送 / 拉取）。
import { ref, watch, computed } from "vue";
import { useLocale } from "@/composables/useLocale";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { CloudIcon, HardDriveIcon, AlertCircleIcon } from "@lucide/vue";
import type { Divergence, SyncChoice } from "@/composables/useAgentSyncMachine";
import type { HeroPreset } from "@/services/backend";

const props = defineProps<{
  open: boolean;
  divergences: Divergence[];
  applying: boolean;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "apply", choices: SyncChoice[]): void;
}>();

const { t } = useLocale();

// 每个差异项的保留选择：name -> 'local' | 'cloud'。
const choices = ref<Record<string, "local" | "cloud">>({});

// 对话框打开 / 差异集变化时重置为默认选择：
// 冲突=保留本地（优先本地数据）、仅本地=保留本地、仅云端=保留云端。
watch(
  () => [props.open, props.divergences] as const,
  ([open, divs]) => {
    if (!open) return;
    const next: Record<string, "local" | "cloud"> = {};
    for (const d of divs) {
      next[d.name] = d.kind === "cloud_only" ? "cloud" : "local";
    }
    choices.value = next;
  },
  { immediate: true, deep: true },
);

function pick(name: string, side: "local" | "cloud") {
  choices.value = { ...choices.value, [name]: side };
}

function apply() {
  const result: SyncChoice[] = props.divergences.map((d) => ({
    name: d.name,
    keep: choices.value[d.name] ?? "local",
  }));
  emit("apply", result);
}

function summary(p: HeroPreset | null): string {
  if (!p) return "";
  const cfg = p.config_json ? JSON.stringify(p.config_json) : "";
  return cfg;
}

const hasAny = computed(() => props.divergences.length > 0);
</script>

<template>
  <Dialog :open="open" @update:open="(v) => emit('update:open', v)">
    <DialogContent class="max-h-[85vh] max-w-4xl overflow-hidden">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AlertCircleIcon class="size-4 text-amber-500" />
          {{ t("heroes.syncDialogTitle") }}
        </DialogTitle>
        <DialogDescription>{{ t("heroes.syncDialogDesc") }}</DialogDescription>
      </DialogHeader>

      <div v-if="hasAny" class="flex flex-col gap-3 overflow-y-auto pr-1" style="max-height: 60vh">
        <div
          v-for="d in divergences"
          :key="d.name"
          class="rounded-md border p-3"
          data-testid="sync-conflict-item"
        >
          <div class="mb-2 flex items-center justify-between gap-2">
            <span class="truncate text-sm font-semibold">{{ d.name }}</span>
            <Badge
              variant="outline"
              class="text-[10px] uppercase tracking-wider"
              :class="{
                'text-amber-500': d.kind === 'conflict',
                'text-blue-500': d.kind === 'local_only',
                'text-muted-foreground': d.kind === 'cloud_only',
              }"
            >
              {{ t(`heroes.divBadge.${d.kind}`) }}
            </Badge>
          </div>

          <!-- 左右两列：云端 | 本地 -->
          <div class="grid grid-cols-2 gap-3">
            <!-- 云端 -->
            <button
              type="button"
              class="rounded-md border p-3 text-left transition"
              :class="
                choices[d.name] === 'cloud'
                  ? 'border-primary bg-primary/5'
                  : 'border-border hover:bg-accent/40'
              "
              :disabled="applying"
              data-testid="sync-keep-cloud"
              @click="pick(d.name, 'cloud')"
            >
              <div class="text-muted-foreground mb-1 flex items-center gap-1 text-[11px] font-medium">
                <CloudIcon class="size-3" />
                {{ t("heroes.syncDialogCloud") }}
              </div>
              <template v-if="d.cloud">
                <div class="text-xs">{{ t("champions." + d.cloud.champion) }}</div>
                <div class="text-muted-foreground font-mono text-[10px]">
                  {{ d.cloud.agent_type }} · {{ d.cloud.model || "—" }}
                </div>
                <p class="text-muted-foreground mt-1 line-clamp-2 text-[11px]">
                  {{ d.cloud.prompt || "—" }}
                </p>
                <p class="text-muted-foreground mt-1 truncate font-mono text-[10px]">
                  {{ summary(d.cloud) || "—" }}
                </p>
              </template>
              <p v-else class="text-muted-foreground text-[11px] italic">
                {{ t("heroes.syncDialogNoCloud") }}
              </p>
            </button>

            <!-- 本地 -->
            <button
              type="button"
              class="rounded-md border p-3 text-left transition"
              :class="
                choices[d.name] === 'local'
                  ? 'border-primary bg-primary/5'
                  : 'border-border hover:bg-accent/40'
              "
              :disabled="applying"
              data-testid="sync-keep-local"
              @click="pick(d.name, 'local')"
            >
              <div class="text-muted-foreground mb-1 flex items-center gap-1 text-[11px] font-medium">
                <HardDriveIcon class="size-3" />
                {{ t("heroes.syncDialogLocal") }}
              </div>
              <template v-if="d.local">
                <div class="text-xs">{{ t("champions." + d.local.champion) }}</div>
                <div class="text-muted-foreground font-mono text-[10px]">
                  {{ d.local.agent_type }} · {{ d.local.model || "—" }}
                </div>
                <p class="text-muted-foreground mt-1 line-clamp-2 text-[11px]">
                  {{ d.local.prompt || "—" }}
                </p>
                <p class="text-muted-foreground mt-1 truncate font-mono text-[10px]">
                  {{ summary(d.local) || "—" }}
                </p>
              </template>
              <p v-else class="text-muted-foreground text-[11px] italic">
                {{ t("heroes.syncDialogNoLocal") }}
              </p>
            </button>
          </div>
        </div>
      </div>

      <p v-else class="text-muted-foreground py-8 text-center text-sm">
        {{ t("heroes.syncDialogEmpty") }}
      </p>

      <DialogFooter>
        <Button variant="outline" :disabled="applying" @click="emit('update:open', false)">
          {{ t("heroes.syncDialogCancel") }}
        </Button>
        <Button :disabled="applying || !hasAny" @click="apply" data-testid="sync-conflict-apply">
          {{ applying ? t("heroes.syncDialogApplying") : t("heroes.syncDialogApply") }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
