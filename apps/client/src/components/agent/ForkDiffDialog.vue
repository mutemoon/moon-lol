<script setup lang="ts">
// 拉取上游更新的差异对比与合并预览（产品文档 §三 Fork 溯源与上游同步）。
//
// 用 Monaco DiffEditor 并排展示「当前」与「上游」的 Prompt / config_json 差异，
// 用户确认后由父组件调用 pullUpstream API 覆盖编辑态并标记待发布。
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import "@/lib/monaco";
import { VueMonacoDiffEditor } from "@guolao/vue-monaco-editor";
import { useLocale } from "@/composables/useLocale";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { GitForkIcon } from "@lucide/vue";

const props = defineProps<{
  open: boolean;
  upstreamName: string;
  upstreamAuthor: number | null;
  currentPrompt: string;
  upstreamPrompt: string;
  currentConfig: string;
  upstreamConfig: string;
  applying: boolean;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "apply"): void;
}>();

const { t } = useLocale();

const isDark = ref(true);
let themeObserver: MutationObserver | null = null;
function syncTheme() {
  isDark.value = document.documentElement.classList.contains("dark");
}

const diffOptions = computed(() => ({
  readOnly: true,
  renderSideBySide: true,
  minimap: { enabled: false },
  fontSize: 12,
  automaticLayout: true,
  scrollBeyondLastLine: false,
}));

const promptChanged = computed(() => props.currentPrompt !== props.upstreamPrompt);
const configChanged = computed(() => props.currentConfig !== props.upstreamConfig);

onMounted(() => {
  syncTheme();
  themeObserver = new MutationObserver(syncTheme);
  themeObserver.observe(document.documentElement, {
    attributes: true,
    attributeFilter: ["class"],
  });
});
onBeforeUnmount(() => themeObserver?.disconnect());
</script>

<template>
  <Dialog :open="open" @update:open="(v) => emit('update:open', v)">
    <DialogContent class="max-w-4xl">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <GitForkIcon class="size-4" />
          {{ t("heroes.diffTitle") }}
        </DialogTitle>
        <DialogDescription>
          {{ t("heroes.forkFromLabel") }}「{{ upstreamName }}」·
          {{ t("heroes.forkAuthor") }} #{{ upstreamAuthor ?? "—" }}
        </DialogDescription>
      </DialogHeader>

      <Tabs default-value="prompt">
        <TabsList>
          <TabsTrigger value="prompt">
            {{ t("heroes.diffPromptTab") }}
            <span v-if="promptChanged" class="ml-1 text-amber-500">•</span>
          </TabsTrigger>
          <TabsTrigger value="config">
            {{ t("heroes.diffConfigTab") }}
            <span v-if="configChanged" class="ml-1 text-amber-500">•</span>
          </TabsTrigger>
        </TabsList>

        <div class="text-muted-foreground mt-2 flex justify-between px-1 text-[11px]">
          <span>← {{ t("heroes.diffCurrent") }}</span>
          <span>{{ t("heroes.diffUpstream") }} →</span>
        </div>

        <TabsContent value="prompt">
          <div class="overflow-hidden rounded-md border" data-testid="fork-diff-prompt">
            <VueMonacoDiffEditor
              :original="currentPrompt"
              :modified="upstreamPrompt"
              language="plaintext"
              :theme="isDark ? 'vs-dark' : 'vs'"
              height="360px"
              :options="diffOptions"
            />
          </div>
        </TabsContent>
        <TabsContent value="config">
          <div class="overflow-hidden rounded-md border" data-testid="fork-diff-config">
            <VueMonacoDiffEditor
              :original="currentConfig"
              :modified="upstreamConfig"
              language="json"
              :theme="isDark ? 'vs-dark' : 'vs'"
              height="360px"
              :options="diffOptions"
            />
          </div>
        </TabsContent>
      </Tabs>

      <DialogFooter>
        <Button variant="outline" :disabled="applying" @click="emit('update:open', false)">
          {{ t("heroes.diffCancel") }}
        </Button>
        <Button :disabled="applying" @click="emit('apply')" data-testid="fork-diff-apply-btn">
          {{ applying ? t("heroes.pulling") : t("heroes.diffApply") }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
