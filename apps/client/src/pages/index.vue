<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter } from "vue-router";
import { useGameStore } from "@/stores/gameStore";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  TerminalIcon,
  PlusIcon,
  HistoryIcon,
  FileTextIcon,
} from "@lucide/vue";

const router = useRouter();
const store = useGameStore();

onMounted(async () => {
  await store.refreshRunningGames();
  store.loadScenariosList();
  store.loadHistoriesList();
});

function handleNewMatch() {
  store.selectedScenario = "";
  router.push("/launcher");
}

function handleSelectScenario(s: string) {
  store.selectedScenario = s;
  router.push("/launcher");
}
</script>

<template>
  <div class="bg-background flex h-full w-full flex-col overflow-hidden p-6 gap-6">
    <!-- Header -->
    <div class="flex items-center justify-between shrink-0">
      <div class="space-y-1">
        <h1 class="text-2xl font-bold tracking-tight text-foreground">仿真工作台</h1>
        <p class="text-sm text-muted-foreground">
          欢迎使用 Moon-LOL 本地仿真控制台。在这里管理和监控仿真环境，配置选手并启动对局。
        </p>
      </div>
    </div>

    <!-- Quick Actions / Welcome -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 shrink-0">
      <!-- Card: Launch New Game -->
      <div
        class="border border-border bg-card/40 rounded-lg p-5 flex flex-col justify-between h-40 hover:border-primary/40 transition-colors group cursor-pointer"
        @click="handleNewMatch"
      >
        <div class="flex items-start justify-between">
          <div class="space-y-1.5">
            <h3 class="font-semibold text-foreground group-hover:text-primary transition-colors text-base">新建本地对局</h3>
            <p class="text-xs text-muted-foreground pr-4">自主选择选手配置与出生点，自定义 Bevy 本地仿真场景。</p>
          </div>
          <div class="bg-primary/10 text-primary p-2 rounded-lg">
            <PlusIcon class="size-5" />
          </div>
        </div>
        <div class="text-[11px] text-muted-foreground font-mono">快捷键: ⌘N</div>
      </div>

      <!-- Card: Running Games Count -->
      <div
        class="border border-border bg-card/40 rounded-lg p-5 flex flex-col justify-between h-40 hover:border-primary/40 transition-colors group cursor-pointer"
        @click="router.push('/games')"
      >
        <div class="flex items-start justify-between">
          <div class="space-y-1.5">
            <h3 class="font-semibold text-foreground group-hover:text-primary transition-colors text-base">对局进程管理</h3>
            <p class="text-xs text-muted-foreground pr-4">监控在本地运行的 Bevy 子进程，查看分配端口与状态。</p>
          </div>
          <div class="bg-primary/10 text-primary p-2 rounded-lg">
            <TerminalIcon class="size-5" />
          </div>
        </div>
        <div class="flex items-center gap-2">
          <span class="bg-emerald-500 size-2 animate-pulse rounded-full" />
          <span class="text-xs text-foreground font-medium">{{ store.runningGames.length }} 个对局运行中</span>
        </div>
      </div>

      <!-- Card: Match History -->
      <div
        class="border border-border bg-card/40 rounded-lg p-5 flex flex-col justify-between h-40 hover:border-primary/40 transition-colors group cursor-pointer"
        @click="router.push('/history')"
      >
        <div class="flex items-start justify-between">
          <div class="space-y-1.5">
            <h3 class="font-semibold text-foreground group-hover:text-primary transition-colors text-base">对局历史记录</h3>
            <p class="text-xs text-muted-foreground pr-4">查看历史对局决策对话与补刀等核心统计，重放游戏场景。</p>
          </div>
          <div class="bg-primary/10 text-primary p-2 rounded-lg">
            <HistoryIcon class="size-5" />
          </div>
        </div>
        <div class="text-xs text-muted-foreground">{{ store.histories.length }} 条对局记录</div>
      </div>
    </div>

    <!-- Main Lists Section (Running games & Scenarios) -->
    <div class="flex-1 grid grid-cols-1 md:grid-cols-2 gap-4 min-h-0">
      <!-- Left: Running Games List -->
      <div class="border border-border rounded-lg bg-card/20 flex flex-col min-h-0">
        <div class="border-b border-border px-4 py-3 flex items-center justify-between shrink-0">
          <span class="text-sm font-semibold text-foreground">活跃运行中对局</span>
          <Badge variant="secondary" class="bg-primary/10 text-primary font-mono text-[10px]">
            {{ store.runningGames.length }}
          </Badge>
        </div>
        <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-2">
          <div
            v-for="game in store.runningGames"
            :key="game.id"
            class="border border-border bg-card flex items-center justify-between gap-4 rounded-lg px-4 py-2 text-xs transition-colors hover:border-primary/20"
          >
            <div class="flex items-center gap-2">
              <span class="bg-emerald-500 size-2 animate-pulse rounded-full" />
              <span class="font-mono text-foreground font-semibold">端口: {{ game.port }}</span>
            </div>
            <div class="flex items-center gap-1.5">
              <Button
                variant="outline"
                size="xs"
                class="h-7 border-primary/20 text-primary hover:border-primary hover:bg-primary/10 bg-primary/5 px-3"
                @click="router.push(`/debug/${game.id}`)"
              >
                调试
              </Button>
              <Button
                variant="outline"
                size="xs"
                class="h-7 text-destructive hover:bg-destructive hover:text-destructive-foreground border-destructive/20 bg-destructive/5 px-3"
                @click="store.stopGame(game.id)"
              >
                停止
              </Button>
            </div>
          </div>
          <div
            v-if="store.runningGames.length === 0"
            class="flex-1 flex flex-col items-center justify-center text-muted-foreground text-xs italic py-12"
          >
            暂无活跃运行中对局
          </div>
        </div>
      </div>

      <!-- Right: Scenario Templates List -->
      <div class="border border-border rounded-lg bg-card/20 flex flex-col min-h-0">
        <div class="border-b border-border px-4 py-3 flex items-center justify-between shrink-0">
          <span class="text-sm font-semibold text-foreground">场景配置模板</span>
          <Badge variant="secondary" class="bg-tag text-foreground-subtle font-mono text-[10px]">
            {{ store.scenariosList.length }}
          </Badge>
        </div>
        <div class="flex-1 overflow-y-auto p-4 flex flex-col gap-2">
          <div
            v-for="s in store.scenariosList"
            :key="s"
            class="border border-border bg-card flex items-center justify-between gap-4 rounded-lg px-4 py-2 text-xs transition-colors hover:border-primary/20 cursor-pointer"
            @click="handleSelectScenario(s)"
          >
            <div class="flex items-center gap-2 truncate">
              <FileTextIcon class="size-4 text-muted-foreground" />
              <span class="font-medium text-foreground truncate">{{ s }}</span>
            </div>
            <Button
              variant="ghost"
              size="xs"
              class="h-7 text-[11px] text-primary hover:bg-primary/5"
            >
              载入编排
            </Button>
          </div>
          <div
            v-if="store.scenariosList.length === 0"
            class="flex-1 flex flex-col items-center justify-center text-muted-foreground text-xs italic py-12"
          >
            暂无自定义场景配置模板
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
::-webkit-scrollbar {
  width: 4px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--muted-foreground);
}
</style>
