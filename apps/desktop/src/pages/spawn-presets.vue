<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { storeToRefs } from "pinia";
import { useGameStore, type SpawnPreset } from "../stores/gameStore";
import { useRouter } from "vue-router";
import { Button } from "../components/ui/button";
import { Badge } from "../components/ui/badge";
import { Input } from "../components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "../components/ui/dialog";
import RiftMap from "../components/RiftMap.vue";
import {
  MapPinIcon,
  PlusIcon,
  Trash2Icon,
  SaveIcon,
  ArrowLeftIcon,
} from "@lucide/vue";

// 出生点预设管理页（产品文档 §3.2 / §3.0）。
// 这是出生点坐标的唯一编辑入口：编排页只下拉选预设，新建/编辑都跳到这里用地图点选。

const store = useGameStore();
const { spawnPresets } = storeToRefs(store);
const router = useRouter();

const selectedName = ref<string | null>(null);
const errorMsg = ref("");
const showDeleteConfirm = ref(false);

const emptyDraft = (): SpawnPreset => ({
  name: "",
  x: 1981,
  z: 11441,
  team: "Order",
});
const draft = ref<SpawnPreset>(emptyDraft());

const draftMarkers = computed(() => [
  {
    id: "draft",
    champion: draft.value.team === "Order" ? "蓝" : "红",
    team: draft.value.team,
    spawnPoint: [draft.value.x, draft.value.z] as [number, number],
  },
]);

function selectPreset(name: string) {
  selectedName.value = name;
  const p = spawnPresets.value.find((x) => x.name === name);
  if (p) draft.value = { ...p };
}

function startNew() {
  selectedName.value = null;
  draft.value = emptyDraft();
}

function handleMapPick(x: number, z: number) {
  draft.value.x = x;
  draft.value.z = z;
}

async function handleSave() {
  errorMsg.value = "";
  const name = draft.value.name.trim();
  if (!name) {
    errorMsg.value = "请填写预设名称";
    return;
  }
  try {
    await store.saveSpawnPreset({ ...draft.value, name });
    selectedName.value = name;
    errorMsg.value = "保存成功";
  } catch (e: any) {
    errorMsg.value = e.message || "保存失败";
  }
}

async function confirmDelete() {
  if (!selectedName.value) return;
  try {
    await store.deleteSpawnPreset(selectedName.value);
    selectedName.value = null;
    draft.value = emptyDraft();
    showDeleteConfirm.value = false;
  } catch (e: any) {
    errorMsg.value = e.message || "删除失败";
  }
}

onMounted(() => {
  store.loadSpawnPresets();
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
          <MapPinIcon class="text-primary size-4" />
        </div>
        <div class="flex items-baseline gap-2">
          <h1 class="text-foreground text-sm font-bold tracking-tight">出生点预设管理</h1>
          <Badge variant="secondary" class="text-[10px]">{{ spawnPresets.length }} 个</Badge>
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
          <div v-if="spawnPresets.length === 0" class="text-muted-foreground py-6 text-center text-xs italic">
            暂无预设
          </div>
          <div v-else class="flex flex-col gap-1">
            <button
              v-for="p in spawnPresets"
              :key="p.name"
              class="hover:bg-muted border-border flex items-center justify-between rounded border px-2 py-1.5 text-left transition-colors"
              :class="p.name === selectedName ? 'border-primary/40 bg-primary/10' : 'bg-muted/30'"
              @click="selectPreset(p.name)"
            >
              <span class="flex min-w-0 flex-col">
                <span class="text-foreground truncate text-xs font-medium">{{ p.name }}</span>
                <span class="text-muted-foreground truncate font-mono text-[10px]">
                  {{ Math.round(p.x) }}, {{ Math.round(p.z) }}
                </span>
              </span>
              <span
                class="size-2 shrink-0 rounded-full"
                :class="p.team === 'Order' ? 'bg-blue-500' : 'bg-red-500'"
              />
            </button>
          </div>
        </div>
      </aside>

      <!-- 右：编辑表单 + 地图 -->
      <section class="border-border bg-card min-w-0 flex-1 overflow-y-auto rounded-lg border p-5 shadow-sm">
        <div v-if="errorMsg" class="border-border mb-3 rounded border-l-2 px-3 py-1.5 text-xs"
          :class="errorMsg === '保存成功' ? 'border-green-500 text-green-500 bg-green-500/5' : 'border-destructive text-destructive bg-destructive/5'">
          {{ errorMsg }}
        </div>

        <div class="mx-auto flex max-w-xl flex-col gap-4">
          <!-- 名称 + 阵营 -->
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-muted-foreground mb-1 block text-[10px] font-semibold uppercase tracking-wider">
                预设名称
              </label>
              <Input
                v-model="draft.name"
                placeholder="如：上路一塔前方"
                class="border-border bg-muted/40 h-9 text-sm"
              />
            </div>
            <div>
              <label class="text-muted-foreground mb-1 block text-[10px] font-semibold uppercase tracking-wider">
                默认阵营
              </label>
              <Select v-model="draft.team">
                <SelectTrigger class="bg-muted/40 border-border h-9 text-sm">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent class="border-border bg-popover text-foreground">
                  <SelectItem value="Order" class="text-sm">Order / 秩序</SelectItem>
                  <SelectItem value="Chaos" class="text-sm">Chaos / 混沌</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <!-- 坐标读数 -->
          <div class="bg-muted/40 border-border flex items-center justify-between rounded-md border px-3 py-2">
            <span class="text-muted-foreground text-[10px] font-semibold uppercase tracking-wider">坐标</span>
            <span class="text-foreground font-mono text-sm font-semibold">
              {{ Math.round(draft.x) }}, {{ Math.round(draft.z) }}
            </span>
          </div>

          <!-- 地图点选 -->
          <div>
            <label class="text-muted-foreground mb-1 block text-[10px] font-semibold uppercase tracking-wider">
              点击地图设置坐标（15000 × 15000）
            </label>
            <div class="border-border/50 bg-muted/30 relative mx-auto aspect-square w-full max-w-[340px] overflow-hidden rounded-md border">
              <RiftMap :agents="draftMarkers" selected-id="draft" :view-box="500" @pick="handleMapPick" />
            </div>
          </div>

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
