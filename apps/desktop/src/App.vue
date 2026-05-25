<script setup lang="ts">
import { computed, provide } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { useRoute, useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { useGameStore } from "./stores/gameStore";
import { LOG_CONTEXT_KEY } from "./composables/useLogPoller";
import { Button } from "./components/ui/button";
import { MinusIcon, SquareIcon, XIcon, TrophyIcon } from "@lucide/vue";
import "./style.css";

const win = getCurrentWindow();
const route = useRoute();
const router = useRouter();

const store = useGameStore();
const { statsResult, showStatsModal } = storeToRefs(store);
const { ws } = store;

// Provide the log poller context to all child pages/components
provide(LOG_CONTEXT_KEY, store.log);

type View = "launcher" | "debug" | "settings";

const currentView = computed<View>(() => {
  if (route.path === "/debug") return "debug";
  if (route.path === "/settings") return "settings";
  return "launcher";
});

interface AgentFinishedPayload {
  minion_kills: number;
  gold: number;
}

listen<AgentFinishedPayload>("agent-finished", (event) => {
  statsResult.value = {
    minionKills: event.payload.minion_kills,
    gold: event.payload.gold,
  };
  showStatsModal.value = true;
});

function minimize() {
  win.minimize();
}
function toggleMaximize() {
  win.toggleMaximize();
}
function closeWindow() {
  win.close();
}

async function onNavMouseDown(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target.closest("button, select, input, a")) return;
  try {
    await win.startDragging();
  } catch {
    /* ignore */
  }
}
</script>

<template>
  <div class="relative flex h-screen flex-col overflow-hidden">
    <!-- Navigation -->
    <nav
      class="border-border-subtle bg-bg-deep/85 relative z-10 flex h-12 shrink-0 items-center gap-2 border-b px-6 backdrop-blur-2xl"
      @mousedown="onNavMouseDown"
    >
      <div class="mr-8 flex shrink-0 items-center gap-2.5">
        <span class="font-display text-gold-bright text-base font-bold tracking-widest">MoonLOL</span>
      </div>

      <div class="ml-auto flex items-center gap-0.5">
        <Button
          variant="ghost"
          class="relative rounded-[3px] px-4 py-2 text-xs font-medium tracking-[0.04em] uppercase transition-colors duration-200 hover:bg-transparent focus-visible:ring-0 focus-visible:ring-offset-0"
          :class="
            currentView === 'launcher'
              ? 'text-gold-bright after:bg-gold-default after:absolute after:bottom-0 after:left-1/2 after:h-[2px] after:w-6 after:-translate-x-1/2 after:rounded-[1px] after:content-[\'\']'
              : 'text-text-muted hover:text-text-bright'
          "
          @click="router.push('/')"
        >
          Home
        </Button>
        <Button
          variant="ghost"
          class="relative rounded-[3px] px-4 py-2 text-xs font-medium tracking-[0.04em] uppercase transition-colors duration-200 hover:bg-transparent focus-visible:ring-0 focus-visible:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-35"
          :class="
            currentView === 'debug'
              ? 'text-gold-bright after:bg-gold-default after:absolute after:bottom-0 after:left-1/2 after:h-[2px] after:w-6 after:-translate-x-1/2 after:rounded-[1px] after:content-[\'\']'
              : 'text-text-muted hover:text-text-bright'
          "
          :disabled="!ws.connected"
          @click="router.push('/debug')"
        >
          Debug
        </Button>
        <Button
          variant="ghost"
          class="text-text-muted relative rounded-[3px] px-4 py-2 text-xs font-medium tracking-[0.04em] uppercase transition-colors duration-200 hover:bg-transparent focus-visible:ring-0 focus-visible:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-35"
          disabled
        >
          Stats
        </Button>
        <Button
          variant="ghost"
          class="relative rounded-[3px] px-4 py-2 text-xs font-medium tracking-[0.04em] uppercase transition-colors duration-200 hover:bg-transparent focus-visible:ring-0 focus-visible:ring-offset-0"
          :class="
            currentView === 'settings'
              ? 'text-gold-bright after:bg-gold-default after:absolute after:bottom-0 after:left-1/2 after:h-[2px] after:w-6 after:-translate-x-1/2 after:rounded-[1px] after:content-[\'\']'
              : 'text-text-muted hover:text-text-bright'
          "
          @click="router.push('/settings')"
        >
          Settings
        </Button>
      </div>

      <!-- Window Controls -->
      <div class="-mr-4 flex h-full items-center">
        <Button
          variant="ghost"
          class="text-text-muted hover:bg-bg-elevated hover:text-text-bright flex h-full w-10 items-center justify-center rounded-none p-0 transition-all duration-100 focus-visible:ring-0 focus-visible:ring-offset-0"
          @click="minimize"
          aria-label="Minimize"
        >
          <MinusIcon class="size-3" :stroke-width="1.5" />
        </Button>
        <Button
          variant="ghost"
          class="text-text-muted hover:bg-bg-elevated hover:text-text-bright flex h-full w-10 items-center justify-center rounded-none p-0 transition-all duration-100 focus-visible:ring-0 focus-visible:ring-offset-0"
          @click="toggleMaximize"
          aria-label="Toggle maximize"
        >
          <SquareIcon class="size-3" :stroke-width="1.5" />
        </Button>
        <Button
          variant="ghost"
          class="text-text-muted hover:bg-red flex h-full w-10 items-center justify-center rounded-none p-0 transition-all duration-100 hover:text-white focus-visible:ring-0 focus-visible:ring-offset-0"
          @click="closeWindow"
          aria-label="Close"
        >
          <XIcon class="size-3" :stroke-width="1.5" />
        </Button>
      </div>
    </nav>

    <!-- Content -->
    <main class="relative z-1 min-h-0 flex-1 overflow-y-auto">
      <router-view />
    </main>

    <!-- Premium Agent Stats Modal -->
    <Transition
      enter-active-class="transition-opacity duration-300 ease-out"
      leave-active-class="transition-opacity duration-300 ease-out"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <div
        v-if="showStatsModal"
        class="bg-bg-deep/85 fixed inset-0 z-[1000] flex items-center justify-center p-6 backdrop-blur-md"
      >
        <div
          class="border-gold-muted relative flex w-full max-w-[480px] flex-col gap-6 overflow-hidden rounded-lg border bg-[#110e14] p-[2.2rem] shadow-[0_24px_64px_rgba(0,0,0,0.8),0_0_40px_rgba(185,145,71,0.15)]"
        >
          <div
            class="pointer-events-none absolute top-[-100px] left-1/2 h-[150px] w-[300px] -translate-x-1/2"
            style="background: radial-gradient(circle, rgba(185, 145, 71, 0.25) 0%, transparent 70%)"
          ></div>
          <div class="z-1 flex items-center gap-3">
            <TrophyIcon class="animate-float text-gold-bright size-8 drop-shadow-[0_0_8px_rgba(185,145,71,0.5)]" />
            <h2 class="font-display text-gold-bright m-0 text-[1.4rem] font-bold tracking-wider">
              AI Agent 模拟测试报告
            </h2>
          </div>

          <div
            class="h-px"
            style="
              background: linear-gradient(
                to right,
                transparent,
                var(--color-gold-muted) 20%,
                var(--color-gold-muted) 80%,
                transparent
              );
              filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.4));
            "
          ></div>

          <div class="z-1 flex flex-col gap-[1.2rem]">
            <p class="text-text-muted m-0 text-[0.9rem] leading-normal">
              AI 代理已成功运行并自主决策满 2 分钟，累计运行数据统计如下：
            </p>
            <div class="grid grid-cols-2 gap-4">
              <div
                class="flex flex-col items-center gap-2 rounded-[6px] border border-[rgba(185,145,71,0.12)] bg-white/2 p-[1.2rem]"
              >
                <span class="text-text-muted text-[0.8rem] font-semibold tracking-wider uppercase">
                  总击杀小兵 (补刀)
                </span>
                <span
                  class="text-gold-bright font-mono text-[2.2rem] leading-none font-extrabold"
                  style="text-shadow: 0 0 16px rgba(212, 175, 92, 0.4)"
                >
                  {{ statsResult.minionKills }}
                </span>
              </div>
              <div
                class="flex flex-col items-center gap-2 rounded-[6px] border border-[rgba(185,145,71,0.12)] bg-white/2 p-[1.2rem]"
              >
                <span class="text-text-muted text-[0.8rem] font-semibold tracking-wider uppercase">
                  总累计金币 (Gold)
                </span>
                <span
                  class="text-gold-bright font-mono text-[2.2rem] leading-none font-extrabold"
                  style="text-shadow: 0 0 16px rgba(212, 175, 92, 0.4)"
                >
                  {{ statsResult.gold.toFixed(0) }}
                  <span class="text-gold-muted text-[1.2rem] font-semibold">g</span>
                </span>
              </div>
            </div>
          </div>

          <div class="z-1 flex justify-center">
            <Button
              class="border-gold-bright font-display cursor-pointer rounded-[4px] border px-10 py-3 text-[0.95rem] font-bold text-white shadow-[0_4px_16px_rgba(185,145,71,0.25)] transition-all duration-200 hover:-translate-y-0.5 hover:shadow-[0_6px_24px_rgba(185,145,71,0.45)]"
              style="
                background: linear-gradient(135deg, var(--color-gold-default) 0%, var(--color-gold-dark, #785b28) 100%);
              "
              @click="showStatsModal = false"
            >
              确认并返回
            </Button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
@keyframes float {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-6px);
  }
}
.animate-float {
  animation: float 2.5s ease-in-out infinite;
}
</style>
