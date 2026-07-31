<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
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
  SquareIcon,
  Loader2Icon,
  RefreshCwIcon,
} from "@lucide/vue";

// 粒子播放：连接独立的粒子渲染 server（lol_particle，默认 9002），
// 左栏选英雄触发 load_hero，右栏列出该英雄的粒子系统并逐个 play_particle。
// 渲染画面在 server 自己的窗口里，本页仅负责列表浏览与播放控制。

const {
  connected,
  connecting,
  lastError,
  heroes,
  heroSystems,
  loadErrors,
  connect,
  disconnect,
  fetchHeroes,
  loadHero,
  playParticle,
  stopParticle,
} = useParticleWs();

const url = ref(DEFAULT_PARTICLE_WS_URL);
const selectedHero = ref<string | null>(null);
const loadingHero = ref<string | null>(null);
const playingHash = ref<number | null>(null);
const actionError = ref("");

const currentSystems = computed<ParticleSystem[]>(() =>
  selectedHero.value ? heroSystems[selectedHero.value] ?? [] : [],
);

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
  selectedHero.value = null;
  loadingHero.value = null;
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

async function selectHero(name: string) {
  selectedHero.value = name;
  actionError.value = "";
  // 已缓存则无需重新加载
  if (heroSystems[name]) return;
  loadingHero.value = name;
  try {
    await loadHero(name);
  } catch (e: any) {
    actionError.value = e?.message || `加载 ${name} 失败`;
  } finally {
    loadingHero.value = null;
  }
}

async function onPlay(system: ParticleSystem) {
  actionError.value = "";
  try {
    await playParticle(system.defRon);
    playingHash.value = system.hash;
  } catch (e: any) {
    actionError.value = e?.message || "播放失败";
  }
}

async function onStop() {
  actionError.value = "";
  try {
    await stopParticle();
    playingHash.value = null;
  } catch (e: any) {
    actionError.value = e?.message || "停止失败";
  }
}

function isHeroLoaded(name: string): boolean {
  return !!heroSystems[name];
}

function hashHex(hash: number): string {
  return "0x" + (hash >>> 0).toString(16).padStart(8, "0");
}

onBeforeUnmount(() => disconnect());
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-5xl flex-col gap-5 px-8 py-8">
    <header class="space-y-1">
      <h1 class="flex items-center gap-2 text-2xl font-semibold tracking-tight">
        <SparklesIcon class="size-6" />
        粒子播放
      </h1>
      <p class="text-muted-foreground text-sm">
        连接粒子渲染 server，选择英雄并逐个播放其粒子系统；画面显示在 server 窗口中。
      </p>
    </header>

    <!-- 连接栏 -->
    <div class="flex items-center gap-2">
      <Input v-model="url" placeholder="ws://127.0.0.1:9002" class="max-w-xs" :disabled="connected" />
      <Button v-if="!connected" :disabled="connecting" @click="onConnect">
        <Loader2Icon v-if="connecting" class="size-4 animate-spin" />
        <PlugIcon v-else class="size-4" />
        {{ connecting ? "连接中…" : "连接" }}
      </Button>
      <template v-else>
        <Button variant="outline" @click="refreshHeroes">
          <RefreshCwIcon class="size-4" />
          刷新
        </Button>
        <Button variant="destructive" @click="onDisconnect">
          <PlugZapIcon class="size-4" />
          断开
        </Button>
      </template>
      <Badge v-if="connected" variant="outline" class="text-emerald-600">已连接</Badge>
    </div>

    <p v-if="lastError || actionError" class="text-destructive text-sm">
      {{ actionError || lastError }}
    </p>

    <!-- 未连接占位 -->
    <div
      v-if="!connected"
      class="text-muted-foreground flex flex-1 items-center justify-center rounded-lg border border-dashed text-sm"
    >
      先启动 lol_particle server，再点击「连接」
    </div>

    <!-- 英雄 × 粒子 双栏 -->
    <div v-else class="grid flex-1 grid-cols-[240px_1fr] gap-4 overflow-hidden">
      <!-- 英雄列表 -->
      <div class="flex flex-col overflow-hidden rounded-lg border">
        <div class="border-b px-3 py-2 text-xs font-semibold tracking-wider uppercase">
          英雄 ({{ heroes.length }})
        </div>
        <ScrollArea class="flex-1">
          <div class="flex flex-col p-1">
            <button
              v-for="h in heroes"
              :key="h"
              class="hover:bg-muted flex items-center justify-between rounded-md px-2.5 py-1.5 text-left text-sm"
              :class="{ 'bg-selected font-semibold': selectedHero === h }"
              @click="selectHero(h)"
            >
              <span class="truncate">{{ h }}</span>
              <Loader2Icon v-if="loadingHero === h" class="size-3.5 animate-spin" />
              <span
                v-else-if="isHeroLoaded(h)"
                class="text-muted-foreground text-[10px] tabular-nums"
              >
                {{ heroSystems[h]?.length ?? 0 }}
              </span>
            </button>
          </div>
        </ScrollArea>
      </div>

      <!-- 粒子系统列表 -->
      <div class="flex flex-col overflow-hidden rounded-lg border">
        <div class="flex items-center justify-between border-b px-3 py-2">
          <span class="text-xs font-semibold tracking-wider uppercase">
            {{ selectedHero ? `${selectedHero} · 粒子系统` : "粒子系统" }}
          </span>
          <Button v-if="playingHash !== null" variant="destructive" size="sm" @click="onStop">
            <SquareIcon class="size-3.5" />
            停止
          </Button>
        </div>

        <div
          v-if="!selectedHero"
          class="text-muted-foreground flex flex-1 items-center justify-center text-sm"
        >
          从左侧选择一个英雄
        </div>
        <div
          v-else-if="loadErrors[selectedHero]"
          class="text-destructive flex flex-1 items-center justify-center px-4 text-center text-sm"
        >
          {{ loadErrors[selectedHero] }}
        </div>
        <div
          v-else-if="loadingHero === selectedHero"
          class="text-muted-foreground flex flex-1 items-center justify-center gap-2 text-sm"
        >
          <Loader2Icon class="size-4 animate-spin" />
          加载中…
        </div>
        <div
          v-else-if="currentSystems.length === 0"
          class="text-muted-foreground flex flex-1 items-center justify-center text-sm"
        >
          该英雄没有粒子系统
        </div>
        <ScrollArea v-else class="flex-1">
          <div class="flex flex-col gap-1 p-2">
            <div
              v-for="sys in currentSystems"
              :key="sys.hash"
              class="hover:bg-muted flex items-center justify-between gap-2 rounded-md px-2.5 py-1.5"
              :class="{ 'bg-selected': playingHash === sys.hash }"
            >
              <div class="min-w-0">
                <div class="truncate text-sm">{{ sys.name }}</div>
                <div class="text-muted-foreground font-mono text-[10px]">{{ hashHex(sys.hash) }}</div>
              </div>
              <Button
                :variant="playingHash === sys.hash ? 'secondary' : 'outline'"
                size="sm"
                @click="onPlay(sys)"
              >
                <PlayIcon class="size-3.5" />
                {{ playingHash === sys.hash ? "播放中" : "播放" }}
              </Button>
            </div>
          </div>
        </ScrollArea>
      </div>
    </div>
  </div>
</template>
