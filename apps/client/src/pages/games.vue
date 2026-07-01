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
import { TerminalIcon, StopCircleIcon, PlusIcon } from "@lucide/vue";

const router = useRouter();
const store = useGameStore();

onMounted(async () => {
  await store.refreshRunningGames();
});

async function handleStopGame(id: string) {
  await store.stopGame(id);
}
</script>

<template>
  <div class="bg-background flex h-full w-full flex-col overflow-hidden p-6 gap-6">
    <div class="flex items-center justify-between shrink-0">
      <div class="space-y-1">
        <h1 class="text-2xl font-bold tracking-tight text-foreground">运行中对局</h1>
        <p class="text-sm text-muted-foreground">
          查看并管理当前在本地计算机上运行的 Bevy 游戏仿真对局。
        </p>
      </div>
      <Button
        size="sm"
        class="bg-primary text-primary-foreground hover:bg-primary/90 gap-1.5 font-semibold"
        @click="router.push('/')"
      >
        <PlusIcon class="size-4" />
        新建对局
      </Button>
    </div>

    <!-- Table of Running Games -->
    <div class="flex-1 min-h-0 overflow-y-auto border border-border rounded-lg bg-card/30">
      <div class="min-w-full inline-block align-middle">
        <table class="min-w-full divide-y divide-border">
          <thead class="bg-muted/50">
            <tr>
              <th scope="col" class="px-6 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">对局 ID</th>
              <th scope="col" class="px-6 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">通信端口</th>
              <th scope="col" class="px-6 py-3 text-left text-xs font-semibold text-muted-foreground uppercase tracking-wider">状态</th>
              <th scope="col" class="px-6 py-3 text-right text-xs font-semibold text-muted-foreground uppercase tracking-wider">操作</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-border bg-transparent">
            <tr v-for="game in store.runningGames" :key="game.id" class="hover:bg-muted/20 transition-colors">
              <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-foreground font-medium">
                {{ game.id }}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-muted-foreground">
                <Badge variant="outline" class="border-primary/20 bg-primary/5 text-primary font-mono font-medium">
                  {{ game.port }}
                </Badge>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm">
                <span class="inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium bg-emerald-500/10 text-emerald-500 border border-emerald-500/20">
                  <span class="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                  {{ game.status }}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                <div class="flex justify-end gap-2">
                  <Button
                    variant="outline"
                    size="xs"
                    class="h-7 text-xs border-primary/20 text-primary hover:border-primary hover:bg-primary/10 bg-primary/5 gap-1"
                    @click="router.push(`/debug/${game.id}`)"
                  >
                    <TerminalIcon class="size-3.5" />
                    进入调试
                  </Button>
                  <Button
                    variant="outline"
                    size="xs"
                    class="h-7 text-xs text-destructive hover:bg-destructive hover:text-destructive-foreground border-destructive/20 bg-destructive/5 gap-1"
                    @click="handleStopGame(game.id)"
                  >
                    <StopCircleIcon class="size-3.5" />
                    停止
                  </Button>
                </div>
              </td>
            </tr>
            <tr v-if="store.runningGames.length === 0">
              <td colspan="4" class="px-6 py-12 text-center text-sm text-muted-foreground italic">
                当前没有正在运行的仿真游戏。
              </td>
            </tr>
          </tbody>
        </table>
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
