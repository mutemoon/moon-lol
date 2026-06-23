<script setup lang="ts">
import { ref, computed, provide, onMounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useRoute, useRouter } from "vue-router";
import { backendClient } from "../services/backend";
import { storeToRefs } from "pinia";
import { useGameStore } from "../stores/gameStore";
import { LOG_CONTEXT_KEY } from "../composables/useLogPoller";
import { useSettingsTab } from "../composables/useSettingsTab";
import { Button } from "../components/ui/button";
import { ScrollArea } from "../components/ui/scroll-area";
import { Badge } from "../components/ui/badge";
import { useEventListener } from "@vueuse/core";
import { useLocale } from "../composables/useLocale";
import {
  TrophyIcon,
  PlusIcon,
  SettingsIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  TerminalIcon,
  BotIcon,
  MapPinIcon,
  SwordsIcon,
  CpuIcon,
  CodeIcon,
  DatabaseIcon,
  BarChart4Icon,
  HammerIcon,
  RocketIcon,
  ArrowLeftIcon,
} from "@lucide/vue";

const { currentTab: settingsTab } = useSettingsTab();

const { t } = useLocale();

const isTauri = (window as any).__TAURI__ !== undefined;
const win = isTauri ? getCurrentWindow() : {
  minimize: () => {},
  toggleMaximize: () => {},
  close: () => {}
};
const route = useRoute();
const router = useRouter();

const store = useGameStore();
const { statsResult, showStatsModal, scenariosList, histories, selectedScenario } = storeToRefs(store);
const { ws } = store;

provide(LOG_CONTEXT_KEY, store.log);

type View = "launcher" | "debug" | "settings" | "history" | "agents" | "spawnPresets" | "heroes";

const currentView = computed<View>(() => {
  if (route.path === "/debug") return "debug";
  if (route.path === "/settings") return "settings";
  if (route.path === "/history") return "history";
  if (route.path === "/agents") return "agents";
  if (route.path === "/spawn-presets") return "spawnPresets";
  if (route.path === "/heroes") return "heroes";
  return "launcher";
});

const viewTitles = computed<Record<View, string>>(() => ({
  launcher: t("app.viewTitles.launcher"),
  debug: t("app.viewTitles.debug"),
  history: t("app.viewTitles.history"),
  settings: t("app.viewTitles.settings"),
  agents: t("app.viewTitles.agents"),
  spawnPresets: t("app.viewTitles.spawnPresets"),
  heroes: t("app.viewTitles.heroes"),
}));

backendClient.onAgentFinished((data) => {
  statsResult.value = {
    minionKills: data.minion_kills,
    gold: data.gold,
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

useEventListener("keydown", (e) => {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "n") {
    e.preventDefault();
    handleNewMatch();
  }
});

function handleNewMatch() {
  selectedScenario.value = "";
  router.push("/");
}

function handleSelectScenario(s: string) {
  selectedScenario.value = s;
  router.push("/");
}

function handleSelectHistory(datetime: string) {
  router.push(`/history?datetime=${datetime}`);
}

onMounted(() => {
  store.loadScenariosList();
  store.loadHistoriesList();
});
</script>

<template>
  <div class="relative flex h-screen w-screen flex-row overflow-hidden bg-background-alt text-foreground font-sans">
    <!-- 1. Left Sidebar -->
    <aside class="flex w-[264px] shrink-0 flex-col border-r border-border bg-background-alt select-none">
      <!-- Top Window Control & Navigation -->
      <div class="flex h-12 shrink-0 items-center justify-between px-4" data-tauri-drag-region>
        <!-- macOS Style Window Buttons -->
        <div class="flex items-center gap-2">
          <button
            class="size-3 rounded-full bg-red-500/90 hover:bg-red-500 transition-colors"
            @click="closeWindow"
            aria-label="Close"
          />
          <button
            class="size-3 rounded-full bg-yellow-500/90 hover:bg-yellow-500 transition-colors"
            @click="minimize"
            aria-label="Minimize"
          />
          <button
            class="size-3 rounded-full bg-green-500/90 hover:bg-green-500 transition-colors"
            @click="toggleMaximize"
            aria-label="Maximize"
          />
        </div>

        <!-- Navigation Buttons -->
        <div class="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            class="size-7 rounded-sm"
            @click="router.back()"
            aria-label="Go back"
          >
            <ChevronLeftIcon class="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="size-7 rounded-sm"
            @click="router.forward()"
            aria-label="Go forward"
          >
            <ChevronRightIcon class="size-4" />
          </Button>
        </div>
      </div>

      <!-- Settings Category Nav (shown when on settings page) -->
      <template v-if="currentView === 'settings'">
        <div class="flex flex-col gap-1 px-2 py-3 border-b border-border/10">
          <div
            role="button"
            class="group hover:bg-surface-hover hover:text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0 text-foreground"
            @click="router.push('/')"
          >
            <div class="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
              <ArrowLeftIcon class="size-4 shrink-0 text-foreground-subtle group-hover:text-foreground" />
              <span class="truncate">{{ t('settings.backToWorkspace') }}</span>
            </div>
          </div>
        </div>

        <div class="flex-1 min-h-0 flex flex-col px-2 py-3 gap-0.5">
          <button
            v-for="item in [
              { key: 'general', icon: SettingsIcon, label: t('settings.nav.general') },
              { key: 'model_settings', icon: CpuIcon, label: t('settings.nav.modelSettings') },
              { key: 'code_preview', icon: CodeIcon, label: t('settings.nav.codePreview') },
              { key: 'skills', icon: HammerIcon, label: t('settings.nav.skills') },
              { key: 'mcp', icon: CpuIcon, label: t('settings.nav.mcp') },
              { key: 'plugins', icon: CpuIcon, label: t('settings.nav.plugins') },
              { key: 'commands', icon: TerminalIcon, label: t('settings.nav.commands') },
              { key: 'indexes', icon: DatabaseIcon, label: t('settings.nav.indexes') },
              { key: 'usage', icon: BarChart4Icon, label: t('settings.nav.usage') },
            ]"
            :key="item.key"
            class="group hover:bg-surface-hover hover:text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 text-[13px] transition-colors"
            :class="settingsTab === item.key ? 'bg-selected font-semibold text-foreground' : 'text-foreground-subtle'"
            @click="settingsTab = item.key as any"
          >
            <component :is="item.icon" class="size-4 shrink-0" />
            <span class="truncate">{{ item.label }}</span>
          </button>

          <div class="border-border mt-auto flex flex-col gap-1 border-t pt-3">
            <button class="text-foreground-subtle hover:text-foreground inline-flex h-8 w-full items-center gap-2 rounded-lg px-2.5 text-[13px]">
              <RocketIcon class="size-4 shrink-0" />
              <span class="truncate">{{ t('settings.auxiliary.tutorial') }}</span>
            </button>
          </div>
        </div>
      </template>

      <!-- Normal Sidebar (non-settings views) -->
      <template v-else>
        <!-- Core Shortcuts -->
        <div class="flex flex-col gap-1 px-2 py-3 border-b border-border/10">
          <div
            role="group"
            class="group hover:bg-surface-hover hover:text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0"
            @click="handleNewMatch"
          >
            <div class="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
              <PlusIcon class="size-4 shrink-0 text-foreground-subtle group-hover:text-foreground" />
              <span class="truncate">{{ t('app.sidebar.newMatch') }}</span>
              <span class="text-foreground-subtlest ml-auto shrink-0 text-[11px] font-normal font-mono">{{ t('app.sidebar.newMatchShortcut') }}</span>
            </div>
          </div>

          <div
            role="button"
            class="group hover:bg-surface-hover hover:text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0 text-foreground"
            :class="{ 'bg-selected font-semibold': currentView === 'settings' }"
            @click="router.push('/settings')"
          >
            <div class="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
              <SettingsIcon class="size-4 shrink-0 text-foreground-subtle group-hover:text-foreground" />
              <span class="truncate">{{ t('app.sidebar.settings') }}</span>
            </div>
          </div>

          <div
            role="button"
            class="group hover:bg-surface-hover hover:text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0 text-foreground"
            :class="{ 'bg-selected font-semibold': currentView === 'heroes' }"
            @click="router.push('/heroes')"
          >
            <div class="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
              <SwordsIcon class="size-4 shrink-0 text-foreground-subtle group-hover:text-foreground" />
              <span class="truncate">{{ t('app.sidebar.heroPresets') }}</span>
            </div>
          </div>

          <div
            role="button"
            class="group hover:bg-surface-hover hover:text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0 text-foreground"
            :class="{ 'bg-selected font-semibold': currentView === 'agents' }"
            @click="router.push('/agents')"
          >
            <div class="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
              <BotIcon class="size-4 shrink-0 text-foreground-subtle group-hover:text-foreground" />
              <span class="truncate">{{ t('app.sidebar.agentPresets') }}</span>
            </div>
          </div>

          <div
            role="button"
            class="group hover:bg-surface-hover hover:text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0 text-foreground"
            :class="{ 'bg-selected font-semibold': currentView === 'spawnPresets' }"
            @click="router.push('/spawn-presets')"
          >
            <div class="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
              <MapPinIcon class="size-4 shrink-0 text-foreground-subtle group-hover:text-foreground" />
              <span class="truncate">{{ t('app.sidebar.spawnPresets') }}</span>
            </div>
          </div>
        </div>

        <!-- Workspace Section -->
        <div class="flex-1 min-h-0 flex flex-col p-2 gap-3">
          <!-- Active Debug Session -->
          <div class="flex flex-col gap-1">
            <span class="text-[11px] text-foreground-subtlest font-semibold px-2.5 uppercase tracking-wider">{{ t('app.sidebar.debugSessions') }}</span>
            <div class="flex flex-col gap-0.5 pl-1">
              <button
                class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-[13px] transition-colors"
                :class="
                  currentView === 'debug'
                    ? 'bg-selected text-foreground font-semibold'
                    : 'text-foreground hover:bg-surface-hover'
                "
                :disabled="!ws.connected && !store?.isStarting"
                @click="router.push('/debug')"
              >
                <TerminalIcon class="size-4 shrink-0 text-foreground-subtle" />
                <span class="truncate font-medium">{{ t('app.sidebar.currentDebug') }}</span>
                <span v-if="ws.connected" class="ml-auto size-2 rounded-full bg-green animate-pulse" />
                <span v-else-if="store?.isStarting" class="ml-auto text-[10px] text-foreground-subtlest animate-pulse">{{ t('app.sidebar.starting') }}</span>
                <span v-else class="ml-auto size-2 rounded-full bg-foreground-subtlest/30" />
              </button>
            </div>
          </div>

          <!-- Scrollable Lists -->
          <ScrollArea class="flex-1 w-full">
            <div class="flex flex-col gap-4 pr-3 py-1">
              <!-- Scenario Templates List -->
              <div v-if="scenariosList.length > 0" class="flex flex-col gap-1">
                <div class="flex items-center justify-between pr-1">
                  <span class="text-[11px] text-foreground-subtlest font-semibold px-2.5 uppercase tracking-wider">{{ t('app.sidebar.scenarioTemplates') }}</span>
                  <Badge variant="outline" class="text-[9px] px-1.5 py-0 border-border text-foreground-subtlest font-normal scale-90">
                    {{ scenariosList.length }}
                  </Badge>
                </div>
                <div class="flex flex-col gap-0.5 pl-1">
                  <button
                    v-for="s in scenariosList"
                    :key="s"
                    class="flex w-full items-center justify-between rounded-lg px-2.5 py-1 text-left text-[13px] transition-colors"
                    :class="
                      currentView === 'launcher' && selectedScenario === s
                        ? 'bg-selected text-foreground font-semibold'
                        : 'text-foreground hover:bg-surface-hover'
                    "
                    @click="handleSelectScenario(s)"
                  >
                    <span class="truncate pr-2 text-foreground-subtle">{{ s }}</span>
                    <Badge variant="secondary" class="text-[9px] px-1 py-0 scale-95 border-transparent bg-tag/40 text-foreground-subtle">{{ t('app.sidebar.config') }}</Badge>
                  </button>
                </div>
              </div>

              <!-- History Archives List -->
              <div v-if="histories.length > 0" class="flex flex-col gap-1">
                <div class="flex items-center justify-between pr-1">
                  <span class="text-[11px] text-foreground-subtlest font-semibold px-2.5 uppercase tracking-wider">{{ t('app.sidebar.completedMatches') }}</span>
                  <Badge variant="outline" class="text-[9px] px-1.5 py-0 border-border text-foreground-subtlest font-normal scale-90">
                    {{ histories.length }}
                  </Badge>
                </div>
                <div class="flex flex-col gap-0.5 pl-1">
                  <button
                    v-for="h in histories"
                    :key="h.datetime"
                    class="flex w-full items-center justify-between rounded-lg px-2.5 py-1 text-left text-[13px] transition-colors"
                    :class="
                      currentView === 'history' && route.query.datetime === h.datetime
                        ? 'bg-selected text-foreground font-semibold'
                        : 'text-foreground hover:bg-surface-hover'
                    "
                    @click="handleSelectHistory(h.datetime)"
                  >
                    <span class="truncate pr-1 text-[11.5px] font-mono text-foreground-subtle">{{ h.datetime.replace('_', ' ').substring(5) }}</span>
                    <Badge variant="secondary" class="text-[9px] px-1 py-0 scale-95 border-transparent bg-tag/40 text-foreground-subtle">{{ t('app.sidebar.archived') }}</Badge>
                  </button>
                </div>
              </div>
            </div>
          </ScrollArea>
        </div>
      </template>
    </aside>

    <!-- 2. Main Workspace -->
    <div class="flex flex-1 flex-col overflow-hidden bg-background-alt p-2 pt-0 pl-0">
      <div class="h-2 w-full select-none" data-tauri-drag-region></div>
      <section class="border-border bg-background relative flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border">
        <!-- Header -->
        <header
          data-testid="workspace-header"
          class="border-border flex h-12 w-full shrink-0 border-b select-none"
        >
          <div class="flex h-12 min-w-0 flex-1 items-center justify-between gap-2 overflow-hidden px-3 py-2 pl-4" data-tauri-drag-region>
            <!-- Header Left: View Title -->
            <div class="flex min-w-0 items-center gap-2 overflow-hidden">
              <h1 class="text-foreground flex max-w-100 min-w-12 shrink items-center gap-2 truncate text-sm font-semibold">
                <span class="min-w-0 truncate">{{ viewTitles[currentView] }}</span>
              </h1>

              <!-- Connection Status Indicator -->
              <div v-if="ws.connected" class="flex items-center gap-1.5 rounded-full bg-green/10 px-2 py-0.5">
                <span class="size-1.5 rounded-full bg-green animate-pulse" />
                <span class="text-[11px] text-green font-medium">{{ t('app.header.connected') }}</span>
              </div>
              <div v-else-if="store?.isStarting" class="flex items-center gap-1.5 rounded-full bg-yellow-500/10 px-2 py-0.5">
                <span class="size-1.5 rounded-full bg-yellow-500 animate-pulse" />
                <span class="text-[11px] text-yellow-500 font-medium">{{ t('app.header.starting') }}</span>
              </div>
            </div>
          </div>
        </header>

        <!-- Main Content Viewport -->
        <main class="flex-1 overflow-y-auto min-h-0 relative">
          <router-view />
        </main>
      </section>
    </div>

    <!-- Stats Modal Popup -->
    <Transition
      enter-active-class="transition-opacity duration-300 ease-out"
      leave-active-class="transition-opacity duration-300 ease-out"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <div
        v-if="showStatsModal"
        class="fixed inset-0 z-[1000] flex items-center justify-center p-6 bg-black/80 backdrop-blur-md"
      >
        <div
          class="relative flex w-full max-w-[480px] flex-col gap-6 overflow-hidden rounded-lg border border-border bg-card p-[2.2rem] shadow-2xl"
        >
          <div
            class="pointer-events-none absolute top-[-100px] left-1/2 h-[150px] w-[300px] -translate-x-1/2"
            style="background: radial-gradient(circle, var(--primary) 0%, transparent 70%); opacity: 0.15"
          ></div>
          <div class="z-1 flex items-center gap-3">
            <TrophyIcon class="animate-float text-primary size-8" />
            <h2 class="text-foreground m-0 text-[1.4rem] font-bold tracking-wider">
              {{ t('app.statsModal.title') }}
            </h2>
          </div>

          <div class="h-px bg-border" />

          <div class="z-1 flex flex-col gap-[1.2rem]">
            <p class="text-muted-foreground m-0 text-[0.9rem] leading-normal">
              {{ t('app.statsModal.description') }}
            </p>
            <div class="grid grid-cols-2 gap-4">
              <div
                class="flex flex-col items-center gap-2 rounded-[6px] border border-border bg-muted/30 p-[1.2rem]"
              >
                <span class="text-muted-foreground text-[0.8rem] font-semibold tracking-wider uppercase">
                  {{ t('app.statsModal.minionKills') }}
                </span>
                <span
                  class="text-foreground font-mono text-[2.2rem] leading-none font-extrabold"
                >
                  {{ statsResult.minionKills }}
                </span>
              </div>
              <div
                class="flex flex-col items-center gap-2 rounded-[6px] border border-border bg-muted/30 p-[1.2rem]"
              >
                <span class="text-muted-foreground text-[0.8rem] font-semibold tracking-wider uppercase">
                  {{ t('app.statsModal.gold') }}
                </span>
                <span
                  class="text-foreground font-mono text-[2.2rem] leading-none font-extrabold"
                >
                  {{ statsResult.gold.toFixed(0) }}
                  <span class="text-muted-foreground text-[1.2rem] font-semibold">g</span>
                </span>
              </div>
            </div>
          </div>

          <div class="z-1 flex justify-center">
            <Button
              class="w-full py-3"
              @click="showStatsModal = false"
            >
              {{ t('app.statsModal.confirm') }}
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
