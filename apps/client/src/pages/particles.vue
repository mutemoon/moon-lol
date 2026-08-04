<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, watch, onBeforeUnmount } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import EmitterEditor from "@/components/particle/EmitterEditor.vue";
import {
  useParticleWs,
  DEFAULT_PARTICLE_WS_URL,
  type ParticleSystem,
} from "@/composables/useParticleWs";
import {
  SparklesIcon,
  PlugIcon,
  PlugZapIcon,
  PlayIcon,
  Loader2Icon,
  RefreshCwIcon,
  ChevronRightIcon,
  ChevronDownIcon,
  FolderIcon,
  FileCodeIcon,
  SearchIcon,
  RotateCcwIcon,
  XIcon,
} from "lucide-vue-next";

const {
  connected,
  connecting,
  lastError,
  heroes,
  heroSystems,
  connect,
  disconnect,
  fetchHeroes,
  loadHero,
  playParticle,
  serializeVfxSystem,
} = useParticleWs();

const url = ref(DEFAULT_PARTICLE_WS_URL);
const actionError = ref("");
const searchQuery = ref("");
const loadingHero = ref<string | null>(null);

// 展开状态映射 (hero -> boolean)
const expandedHeroes = ref<Record<string, boolean>>({});

// 选中的粒子系统与原始未修改的备份定义
const selectedHeroName = ref<string | null>(null);
const selectedSystemHash = ref<number | null>(null);
const activeSystem = ref<ParticleSystem | null>(null);
const workingDef = ref<any | null>(null);
const initialDefBackup = ref<any | null>(null);

// 播放状态与自动播放开关
const playingHash = ref<number | null>(null);
const autoPlayOnChange = ref(true);
const activeTab = ref("0");

// 防抖定时器
let autoPlayTimer: ReturnType<typeof setTimeout> | null = null;

// 检索过滤后的英雄与粒子列表 (树结构)
const filteredTree = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) {
    return heroes.value.map((h) => ({
      hero: h,
      systems: heroSystems[h] ?? [],
    }));
  }

  return heroes.value
    .map((h) => {
      const matchHero = h.toLowerCase().includes(q);
      const systems = heroSystems[h] ?? [];
      const matchedSystems = systems.filter(
        (s) => s.name.toLowerCase().includes(q) || hashHex(s.hash).toLowerCase().includes(q)
      );
      if (matchHero || matchedSystems.length > 0) {
        return {
          hero: h,
          systems: matchHero ? systems : matchedSystems,
        };
      }
      return null;
    })
    .filter((item): item is { hero: string; systems: ParticleSystem[] } => item !== null);
});

// 计算所有匹配的粒子总数
const totalMatchedSystems = computed(() => {
  return filteredTree.value.reduce((sum, item) => sum + item.systems.length, 0);
});

// 监听搜索词变化，若有搜索词则自动展开匹配的英雄节点
watch(searchQuery, (newQ) => {
  const q = newQ.trim().toLowerCase();
  if (q) {
    filteredTree.value.forEach((item) => {
      if (item.systems.length > 0) {
        expandedHeroes.value[item.hero] = true;
      }
    });
  }
});

// 当前选中系统的发射器列表 (complex_emitter_definition_data 或 simple_emitter_definition_data)
const emitterList = computed(() => {
  if (!workingDef.value) return [];
  return (
    workingDef.value.complex_emitter_definition_data ??
    workingDef.value.simple_emitter_definition_data ??
    []
  );
});

async function onConnect() {
  actionError.value = "";
  try {
    await connect(url.value);
    await fetchHeroes();
  } catch (e: any) {
    actionError.value = e?.message || "连接失败";
  }
}

function onDisconnect() {
  disconnect();
  selectedHeroName.value = null;
  selectedSystemHash.value = null;
  activeSystem.value = null;
  workingDef.value = null;
  initialDefBackup.value = null;
  playingHash.value = null;
}

async function refreshHeroes() {
  actionError.value = "";
  try {
    await fetchHeroes();
  } catch (e: any) {
    actionError.value = e?.message || "刷新英雄列表失败";
  }
}

async function toggleHeroExpand(heroName: string) {
  const next = !expandedHeroes.value[heroName];
  expandedHeroes.value[heroName] = next;

  if (next && !heroSystems[heroName]) {
    try {
      await loadHero(heroName);
    } catch (e: any) {
      actionError.value = e?.message || `加载 ${heroName} 失败`;
    }
  }
}

async function selectParticleSystem(heroName: string, system: ParticleSystem) {
  selectedHeroName.value = heroName;
  selectedSystemHash.value = system.hash;
  activeSystem.value = system;

  // 深拷贝防修改原对象
  workingDef.value = JSON.parse(JSON.stringify(system.def ?? {}));
  initialDefBackup.value = JSON.parse(JSON.stringify(system.def ?? {}));
  activeTab.value = "0";

  await playCurrentSystem();
}

async function playCurrentSystem() {
  if (!workingDef.value) return;
  actionError.value = "";
  try {
    const ronStr = await serializeVfxSystem(workingDef.value);
    await playParticle(ronStr);
    if (activeSystem.value) {
      playingHash.value = activeSystem.value.hash;
    }
  } catch (e: any) {
    actionError.value = e?.message || "播放失败";
  }
}

// 参数修改后的回调：防抖 200ms 重新播放
function onEmitterChanged() {
  if (!autoPlayOnChange.value) return;
  if (autoPlayTimer) clearTimeout(autoPlayTimer);
  autoPlayTimer = setTimeout(() => {
    playCurrentSystem();
  }, 200);
}

// 重置单个发射器
function onResetSingleEmitter(idx: number) {
  if (!initialDefBackup.value || !workingDef.value) return;
  const initialEmitters =
    initialDefBackup.value.complex_emitter_definition_data ??
    initialDefBackup.value.simple_emitter_definition_data ??
    [];
  if (initialEmitters[idx]) {
    emitterList.value[idx] = JSON.parse(JSON.stringify(initialEmitters[idx]));
    onEmitterChanged();
  }
}

// 仅播放当前选中的单个发射器/粒子
async function playSingleEmitter(idx: number) {
  if (!workingDef.value) return;
  actionError.value = "";
  try {
    const singleDef = JSON.parse(JSON.stringify(workingDef.value));
    const targetEmitter = singleDef.complex_emitter_definition_data?.[idx];
    if (targetEmitter) {
      singleDef.complex_emitter_definition_data = [targetEmitter];
    }
    const ronStr = await serializeVfxSystem(singleDef);
    await playParticle(ronStr);
  } catch (e: any) {
    actionError.value = e?.message || "播放单个粒子失败";
  }
}

// 重置整个系统
function resetEntireSystem() {
  if (!initialDefBackup.value) return;
  workingDef.value = JSON.parse(JSON.stringify(initialDefBackup.value));
  playCurrentSystem();
}

function hashHex(hash: number): string {
  return "0x" + (hash >>> 0).toString(16).padStart(8, "0");
}

onBeforeUnmount(() => {
  if (autoPlayTimer) clearTimeout(autoPlayTimer);
  disconnect();
});
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-7xl flex-col gap-4 px-6 py-6 min-h-0 overflow-hidden">
    <!-- 页头 -->
    <header class="flex flex-col gap-2.5 shrink-0 border-b pb-3">
      <!-- 第一行：标题与服务器连接状态栏 -->
      <div class="flex items-center justify-between">
        <div class="space-y-0.5">
          <h1 class="flex items-center gap-2 text-xl font-semibold tracking-tight">
            <SparklesIcon class="size-5 text-primary" />
            {{ $t('particles.title') }}
          </h1>
          <p class="text-muted-foreground text-xs">
            {{ $t('particles.subtitle') }}
          </p>
        </div>

        <!-- 服务器连接控制 -->
        <div class="flex items-center gap-2">
          <Input
            v-model="url"
            placeholder="ws://127.0.0.1:9002"
            class="h-8 max-w-[200px] text-xs font-mono"
            :disabled="connected"
          />
          <Button v-if="!connected" size="sm" class="h-8 text-xs" :disabled="connecting" @click="onConnect">
            <Loader2Icon v-if="connecting" class="size-3.5 animate-spin" />
            <PlugIcon v-else class="size-3.5" />
            {{ connecting ? $t('particles.connecting') : $t('particles.connectServer') }}
          </Button>
          <template v-else>
            <Button variant="outline" size="sm" class="h-8 text-xs" @click="refreshHeroes">
              <RefreshCwIcon class="size-3.5" />
              {{ $t('particles.refreshList') }}
            </Button>
            <Button variant="destructive" size="sm" class="h-8 text-xs" @click="onDisconnect">
              <PlugZapIcon class="size-3.5" />
              {{ $t('particles.disconnect') }}
            </Button>
          </template>
          <Badge v-if="connected" variant="outline" class="text-emerald-600 border-emerald-600/30 text-xs">
            {{ $t('particles.connected') }}
          </Badge>
        </div>
      </div>

      <!-- 第二行：独立全局控制栏 (单独占一行，宽敞整洁) -->
      <div class="flex items-center justify-between pt-1">
        <div class="flex items-center gap-2">
          <div class="flex items-center gap-2 px-3 py-1.5 rounded-md bg-muted/40 border border-border/60 text-xs shadow-xs">
            <Checkbox
              id="header_auto_play_on_change"
              :model-value="autoPlayOnChange"
              @update:model-value="autoPlayOnChange = Boolean($event)"
            />
            <Label for="header_auto_play_on_change" class="text-xs text-foreground cursor-pointer font-medium select-none">
              {{ $t('particles.autoPlayOnChange') }}
            </Label>
          </div>
        </div>
      </div>
    </header>

    <p v-if="lastError || actionError" class="text-destructive text-xs shrink-0 bg-destructive/10 p-2 rounded border border-destructive/20">
      {{ actionError || lastError }}
    </p>

    <!-- 未连接占位 -->
    <div
      v-if="!connected"
      class="text-muted-foreground flex flex-1 items-center justify-center rounded-lg border border-dashed text-sm min-h-0 bg-muted/10"
    >
      {{ $t('particles.connectPlaceholder') }}
    </div>

    <!-- 主框架：左侧二级树状菜单 × 右侧发射器 Tabs 编辑器 -->
    <div v-else class="grid min-h-0 flex-1 grid-cols-[280px_1fr] gap-4 overflow-hidden">
      <!-- 左侧：英雄及粒子系统树 -->
      <div class="flex min-h-0 flex-col overflow-hidden rounded-lg border bg-card">
        <!-- 搜索框与过滤信息 -->
        <div class="p-2 border-b shrink-0 bg-muted/20 space-y-1.5">
          <div class="relative flex items-center">
            <SearchIcon class="absolute left-2.5 top-2.5 size-3.5 text-muted-foreground" />
            <Input
              v-model="searchQuery"
              :placeholder="$t('particles.searchPlaceholder')"
              class="h-8 text-xs pl-8 pr-7 bg-background"
            />
            <button
              v-if="searchQuery"
              class="absolute right-2 text-muted-foreground hover:text-foreground p-0.5"
              :title="$t('particles.clearSearch')"
              @click="searchQuery = ''"
            >
              <XIcon class="size-3.5" />
            </button>
          </div>

          <div class="flex items-center justify-between text-[10px] text-muted-foreground px-0.5 font-mono">
            <span>
              {{ searchQuery ? $t('particles.matchedParticles', { count: totalMatchedSystems }) : $t('particles.totalHeroes', { count: heroes.length }) }}
            </span>
          </div>
        </div>

        <ScrollArea class="min-h-0 flex-1">
          <div class="flex flex-col p-1.5 space-y-0.5">
            <div v-for="item in filteredTree" :key="item.hero" class="flex flex-col">
              <!-- 一级节点：英雄 -->
              <button
                class="hover:bg-muted/60 flex items-center justify-between rounded-md px-2 py-1.5 text-left text-xs font-medium transition-colors"
                :class="{ 'bg-muted/80 text-foreground': selectedHeroName === item.hero }"
                @click="toggleHeroExpand(item.hero)"
              >
                <div class="flex items-center gap-1.5 truncate">
                  <component
                    :is="expandedHeroes[item.hero] ? ChevronDownIcon : ChevronRightIcon"
                    class="size-3.5 text-muted-foreground shrink-0"
                  />
                  <FolderIcon class="size-3.5 text-amber-500/80 shrink-0" />
                  <span class="truncate font-semibold">{{ item.hero }}</span>
                </div>
                <Loader2Icon v-if="loadingHero === item.hero" class="size-3 animate-spin text-primary shrink-0" />
                <Badge v-else-if="heroSystems[item.hero]" variant="secondary" class="text-[10px] h-4 px-1 font-mono shrink-0">
                  {{ item.systems.length }}
                </Badge>
              </button>

              <!-- 二级节点：英雄的粒子列表 -->
              <div v-if="expandedHeroes[item.hero]" class="ml-4 pl-2 border-l border-border/50 my-0.5 space-y-0.5">
                <div
                  v-for="sys in item.systems"
                  :key="sys.hash"
                  class="hover:bg-muted flex items-center justify-between rounded px-2 py-1 text-xs cursor-pointer group transition-colors"
                  :class="{
                    'bg-primary/10 text-primary font-medium': selectedSystemHash === sys.hash,
                    'text-muted-foreground': selectedSystemHash !== sys.hash
                  }"
                  @click="selectParticleSystem(item.hero, sys)"
                >
                  <div class="flex items-center gap-1.5 min-w-0">
                    <FileCodeIcon class="size-3 shrink-0" :class="selectedSystemHash === sys.hash ? 'text-primary' : 'text-muted-foreground'" />
                    <span class="truncate text-[11px]">{{ sys.name }}</span>
                  </div>
                  <span class="text-[9px] font-mono opacity-60 group-hover:opacity-100 shrink-0">
                    {{ hashHex(sys.hash).slice(0, 6) }}
                  </span>
                </div>

                <div v-if="item.systems.length === 0" class="text-[11px] text-muted-foreground/60 px-2 py-1 italic">
                  {{ $t('particles.noMatch') }}
                </div>
              </div>
            </div>

            <div v-if="filteredTree.length === 0" class="text-xs text-muted-foreground p-4 text-center">
              {{ $t('particles.noMatch') }}
            </div>
          </div>
        </ScrollArea>
      </div>

      <!-- 右侧：粒子系统 Tabs 与编辑区域 -->
      <div class="flex min-h-0 flex-col overflow-hidden rounded-lg border bg-card">
        <!-- 未选择粒子系统提示 -->
        <div
          v-if="!activeSystem"
          class="text-muted-foreground flex flex-1 flex-col items-center justify-center gap-2 text-sm min-h-0 bg-muted/5"
        >
          <SparklesIcon class="size-8 text-muted-foreground/40" />
          <span>{{ $t('particles.selectPrompt') }}</span>
        </div>

        <template v-else>
          <!-- 面板顶栏 -->
          <div class="flex shrink-0 items-center justify-between border-b px-4 py-2 bg-muted/20">
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-xs font-bold truncate">
                {{ selectedHeroName }} / {{ activeSystem.name }}
              </span>
              <Badge variant="outline" class="font-mono text-[10px]">
                {{ hashHex(activeSystem.hash) }}
              </Badge>
            </div>

            <!-- 控制按钮组 -->
            <div class="flex items-center gap-2">
              <Button variant="outline" size="sm" class="h-7 text-xs gap-1" @click="resetEntireSystem">
                <RotateCcwIcon class="size-3" />
                {{ $t('particles.resetSystem') }}
              </Button>

              <Button
                variant="default"
                size="sm"
                class="h-7 text-xs gap-1"
                @click="playCurrentSystem"
              >
                <PlayIcon class="size-3.5" />
                {{ $t('particles.play') }}
              </Button>
            </div>
          </div>

          <!-- 发射器 Tabs 编辑区域 -->
          <div v-if="emitterList.length === 0" class="flex flex-1 items-center justify-center text-xs text-muted-foreground">
            {{ $t('particles.noParticlesInSystem') }}
          </div>

          <div v-else class="flex flex-1 flex-col min-h-0 overflow-hidden">
            <Tabs v-model="activeTab" class="flex flex-col h-full overflow-hidden">
              <div class="border-b px-3 bg-muted/10 shrink-0">
                <TabsList class="h-9 bg-transparent p-0 gap-1">
                  <TabsTrigger
                    v-for="(em, idx) in emitterList"
                    :key="idx"
                    :value="String(idx)"
                    class="h-8 text-xs px-3 data-[state=active]:bg-background data-[state=active]:shadow-sm rounded-t-md rounded-b-none"
                  >
                    {{ em.emitter_name || `发射器 #${idx + 1}` }}
                  </TabsTrigger>
                </TabsList>
              </div>

              <div class="flex-1 min-h-0 overflow-hidden">
                <TabsContent
                  v-for="(em, idx) in emitterList"
                  :key="idx"
                  :value="String(idx)"
                  class="h-full m-0 p-0 overflow-hidden border-none"
                >
                  <EmitterEditor
                    :emitter="em"
                    :initial-emitter="initialDefBackup?.complex_emitter_definition_data?.[idx]"
                    @change="onEmitterChanged"
                    @reset="onResetSingleEmitter(idx)"
                    @play-single="playSingleEmitter(idx)"
                  />
                </TabsContent>
              </div>
            </Tabs>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>
