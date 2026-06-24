<script setup lang="ts">
import { computed, provide, onMounted } from "vue";
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
  SwordsIcon,
  CpuIcon,
  CodeIcon,
  DatabaseIcon,
  BarChart4Icon,
  HammerIcon,
  RocketIcon,
  ArrowLeftIcon,
  UserIcon,
  Users2Icon,
  GemIcon,
  GlobeIcon,
  ShieldIcon,
  ZapIcon,
  CrownIcon,
} from "@lucide/vue";

const { currentTab: settingsTab } = useSettingsTab();

const { t } = useLocale();

const isTauri = (window as any).__TAURI__ !== undefined;
const isDesktop = computed(() => {
  return (
    typeof window !== "undefined" &&
    ((window as any).IS_DESKTOP ??
      ((window as any).__TAURI__ !== undefined || (window as any).__TAURI_INTERNALS__ !== undefined))
  );
});
const win = isTauri
  ? getCurrentWindow()
  : {
      minimize: () => {},
      toggleMaximize: () => {},
      close: () => {},
    };
const route = useRoute();
const router = useRouter();

const store = useGameStore();
const { statsResult, showStatsModal, scenariosList, histories, selectedScenario } = storeToRefs(store);
const { ws } = store;

provide(LOG_CONTEXT_KEY, store.log);

type View =
  | "launcher"
  | "debug"
  | "settings"
  | "history"
  | "agents"
  | "spawnPresets"
  | "heroes"
  | "rooms"
  | "rank"
  | "leaderboard"
  | "community"
  | "billing"
  | "admin"
  | "rlTraining"
  | "logsArchive";

const currentView = computed<View>(() => {
  if (route.path === "/debug") return "debug";
  if (route.path === "/settings") return "settings";
  if (route.path === "/history") return "history";
  if (route.path === "/agents") return "agents";
  if (route.path === "/spawn-presets") return "spawnPresets";
  if (route.path === "/heroes") return "heroes";
  if (route.path.startsWith("/rooms")) return "rooms";
  if (route.path.startsWith("/observe")) return "rooms";
  if (route.path === "/rank") return "rank";
  if (route.path === "/leaderboard") return "leaderboard";
  if (route.path === "/community") return "community";
  if (route.path === "/billing") return "billing";
  if (route.path === "/admin") return "admin";
  if (route.path === "/rl-training") return "rlTraining";
  if (route.path === "/logs-archive") return "logsArchive";
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
  rooms: "房间",
  rank: "Rank 竞技",
  leaderboard: "排行榜",
  community: "社区",
  billing: "精粹与订阅",
  admin: "对局池监控",
  rlTraining: "RL 训练面板",
  logsArchive: "日志归档",
}));

if (isTauri) {
  backendClient.onAgentFinished((data) => {
    statsResult.value = {
      minionKills: data.minion_kills,
      gold: data.gold,
    };
    showStatsModal.value = true;
  });
}

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

// Stub for account settings/profile navigation or details modal
function handleAccountClick() {
  console.log("Account profile clicked");
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
  <div class="bg-background-alt text-foreground relative flex h-screen w-screen flex-row overflow-hidden font-sans">
    <!-- 1. Left Sidebar -->
    <aside class="bg-background-alt flex w-[264px] shrink-0 flex-col select-none">
      <!-- Top Window Control & Navigation -->
      <div class="flex h-12 shrink-0 items-center justify-between px-4" data-tauri-drag-region>
        <!-- macOS Style Window Buttons -->
        <div class="flex items-center gap-2">
          <button
            class="size-3 rounded-full bg-red-500/90 transition-colors hover:bg-red-500"
            @click="closeWindow"
            aria-label="Close"
          />
          <button
            class="size-3 rounded-full bg-yellow-500/90 transition-colors hover:bg-yellow-500"
            @click="minimize"
            aria-label="Minimize"
          />
          <button
            class="size-3 rounded-full bg-green-500/90 transition-colors hover:bg-green-500"
            @click="toggleMaximize"
            aria-label="Maximize"
          />
        </div>

        <!-- Navigation Buttons -->
        <div class="flex items-center gap-1">
          <Button variant="ghost" size="icon" class="size-7 rounded-sm" @click="router.back()" aria-label="Go back">
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

      <!-- Navigation Content -->
      <div class="flex min-h-0 flex-1 flex-col">
        <!-- Settings Category Nav (shown when on settings page) -->
        <template v-if="currentView === 'settings'">
          <div class="border-border/10 flex flex-col gap-1 border-b px-2 py-3">
            <div
              role="button"
              class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0"
              @click="router.push('/')"
            >
              <div class="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
                <ArrowLeftIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
                <span class="truncate">{{ t("settings.backToWorkspace") }}</span>
              </div>
            </div>
          </div>

          <div class="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-2 py-3">
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
              :class="settingsTab === item.key ? 'bg-selected text-foreground font-semibold' : 'text-foreground-subtle'"
              @click="settingsTab = item.key as any"
            >
              <component :is="item.icon" class="size-4 shrink-0" />
              <span class="truncate">{{ item.label }}</span>
            </button>

            <div class="border-border mt-auto flex flex-col gap-1 border-t pt-3">
              <button
                class="text-foreground-subtle hover:text-foreground inline-flex h-8 w-full items-center gap-2 rounded-lg px-2.5 text-[13px]"
              >
                <RocketIcon class="size-4 shrink-0" />
                <span class="truncate">{{ t("settings.auxiliary.tutorial") }}</span>
              </button>
            </div>
          </div>
        </template>

        <!-- Normal Sidebar (non-settings views) -->
        <template v-else>
          <!-- Core Shortcuts -->
          <div class="border-border/10 flex flex-col gap-1 border-b px-2 py-3">
            <div
              v-if="isDesktop"
              role="group"
              class="group hover:bg-surface-hover hover:text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0"
              @click="handleNewMatch"
            >
              <div class="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
                <PlusIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
                <span class="truncate">{{ t("app.sidebar.newMatch") }}</span>
                <span class="text-foreground-subtlest ml-auto shrink-0 font-mono text-[11px] font-normal">
                  {{ t("app.sidebar.newMatchShortcut") }}
                </span>
              </div>
            </div>

            <div
              role="button"
              class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0"
              :class="{ 'bg-selected font-semibold': currentView === 'heroes' }"
              @click="router.push('/heroes')"
            >
              <div class="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
                <SwordsIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
                <span class="truncate">{{ t("app.sidebar.heroPresets") }}</span>
              </div>
            </div>
          </div>

          <!-- Online: Rooms / Rank / Community -->
          <div class="border-border/10 flex flex-col gap-1 border-b px-2 py-3">
            <span class="text-foreground-subtlest mb-1 px-2.5 text-[10px] font-semibold tracking-wider uppercase">
              在线
            </span>
            <div
              role="button"
              class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full cursor-pointer items-center gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5"
              :class="{ 'bg-selected font-semibold': currentView === 'rooms' }"
              @click="router.push('/rooms')"
            >
              <Users2Icon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
              <span class="truncate text-[13px]">房间</span>
            </div>
            <div
              role="button"
              class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full cursor-pointer items-center gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5"
              :class="{ 'bg-selected font-semibold': currentView === 'rank' }"
              @click="router.push('/rank')"
            >
              <ZapIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
              <span class="truncate text-[13px]">Rank 竞技</span>
            </div>
            <div
              role="button"
              class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full cursor-pointer items-center gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5"
              :class="{ 'bg-selected font-semibold': currentView === 'leaderboard' }"
              @click="router.push('/leaderboard')"
            >
              <CrownIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
              <span class="truncate text-[13px]">排行榜</span>
            </div>
            <div
              role="button"
              class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full cursor-pointer items-center gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5"
              :class="{ 'bg-selected font-semibold': currentView === 'community' }"
              @click="router.push('/community')"
            >
              <GlobeIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
              <span class="truncate text-[13px]">社区</span>
            </div>
            <div
              role="button"
              class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full cursor-pointer items-center gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5"
              :class="{ 'bg-selected font-semibold': currentView === 'billing' }"
              @click="router.push('/billing')"
            >
              <GemIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
              <span class="truncate text-[13px]">精粹与订阅</span>
            </div>
          </div>

          <!-- Tools: RL Training / Admin / Logs (desktop-leaning) -->
          <div class="border-border/10 flex flex-col gap-1 border-b px-2 py-3">
            <span class="text-foreground-subtlest mb-1 px-2.5 text-[10px] font-semibold tracking-wider uppercase">
              工具
            </span>
            <div
              v-if="isDesktop"
              role="button"
              class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full cursor-pointer items-center gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5"
              :class="{ 'bg-selected font-semibold': currentView === 'rlTraining' }"
              @click="router.push('/rl-training')"
            >
              <RocketIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
              <span class="truncate text-[13px]">RL 训练面板</span>
            </div>
            <div
              role="button"
              class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full cursor-pointer items-center gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5"
              :class="{ 'bg-selected font-semibold': currentView === 'logsArchive' }"
              @click="router.push('/logs-archive')"
            >
              <DatabaseIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
              <span class="truncate text-[13px]">日志归档</span>
            </div>
            <div
              role="button"
              class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full cursor-pointer items-center gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5"
              :class="{ 'bg-selected font-semibold': currentView === 'admin' }"
              @click="router.push('/admin')"
            >
              <ShieldIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
              <span class="truncate text-[13px]">对局池监控</span>
            </div>
          </div>

          <!-- Workspace Section -->
          <div v-if="isDesktop" class="flex min-h-0 flex-1 flex-col gap-3 p-2">
            <!-- Active Debug Session -->
            <div class="flex flex-col gap-1">
              <span class="text-foreground-subtlest px-2.5 text-[11px] font-semibold tracking-wider uppercase">
                {{ t("app.sidebar.debugSessions") }}
              </span>
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
                  <TerminalIcon class="text-foreground-subtle size-4 shrink-0" />
                  <span class="truncate font-medium">{{ t("app.sidebar.currentDebug") }}</span>
                  <span v-if="ws.connected" class="bg-green ml-auto size-2 animate-pulse rounded-full" />
                  <span
                    v-else-if="store?.isStarting"
                    class="text-foreground-subtlest ml-auto animate-pulse text-[10px]"
                  >
                    {{ t("app.sidebar.starting") }}
                  </span>
                  <span v-else class="bg-foreground-subtlest/30 ml-auto size-2 rounded-full" />
                </button>
              </div>
            </div>

            <!-- Scrollable Lists -->
            <ScrollArea class="w-full flex-1">
              <div class="flex flex-col gap-4 py-1 pr-3">
                <!-- Scenario Templates List -->
                <div v-if="scenariosList.length > 0" class="flex flex-col gap-1">
                  <div class="flex items-center justify-between pr-1">
                    <span class="text-foreground-subtlest px-2.5 text-[11px] font-semibold tracking-wider uppercase">
                      {{ t("app.sidebar.scenarioTemplates") }}
                    </span>
                    <Badge
                      variant="outline"
                      class="border-border text-foreground-subtlest scale-90 px-1.5 py-0 text-[9px] font-normal"
                    >
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
                      <span class="text-foreground-subtle truncate pr-2">{{ s }}</span>
                      <Badge
                        variant="secondary"
                        class="bg-tag/40 text-foreground-subtle scale-95 border-transparent px-1 py-0 text-[9px]"
                      >
                        {{ t("app.sidebar.config") }}
                      </Badge>
                    </button>
                  </div>
                </div>

                <!-- History Archives List -->
                <div v-if="histories.length > 0" class="flex flex-col gap-1">
                  <div class="flex items-center justify-between pr-1">
                    <span class="text-foreground-subtlest px-2.5 text-[11px] font-semibold tracking-wider uppercase">
                      {{ t("app.sidebar.completedMatches") }}
                    </span>
                    <Badge
                      variant="outline"
                      class="border-border text-foreground-subtlest scale-90 px-1.5 py-0 text-[9px] font-normal"
                    >
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
                      <span class="text-foreground-subtle truncate pr-1 font-mono text-[11.5px]">
                        {{ h.datetime.replace("_", " ").substring(5) }}
                      </span>
                      <Badge
                        variant="secondary"
                        class="bg-tag/40 text-foreground-subtle scale-95 border-transparent px-1 py-0 text-[9px]"
                      >
                        {{ t("app.sidebar.archived") }}
                      </Badge>
                    </button>
                  </div>
                </div>
              </div>
            </ScrollArea>
          </div>
        </template>
      </div>

      <!-- Shared Bottom Section (Account & Settings) -->
      <div class="border-border/10 bg-background-alt/50 mt-auto flex shrink-0 flex-col gap-1 border-t px-2 py-3">
        <!-- Settings Shortcut (only shown when not on settings page) -->
        <div
          v-if="currentView !== 'settings'"
          role="button"
          class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-8 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0"
          @click="router.push('/settings')"
        >
          <div class="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
            <SettingsIcon class="text-foreground-subtle group-hover:text-foreground size-4 shrink-0" />
            <span class="truncate">{{ t("app.sidebar.settings") }}</span>
          </div>
        </div>

        <!-- Account Info -->
        <div
          role="button"
          class="group hover:bg-surface-hover hover:text-foreground text-foreground inline-flex h-9 w-full shrink-0 cursor-pointer items-center justify-stretch gap-2 overflow-hidden rounded-lg pr-2.5 pl-2.5 active:translate-y-0"
          @click="handleAccountClick"
        >
          <div class="flex min-w-0 flex-1 items-center gap-2">
            <div class="bg-primary/10 flex size-6 shrink-0 items-center justify-center rounded-full">
              <UserIcon class="text-primary size-3.5" />
            </div>
            <div class="flex min-w-0 flex-col text-left">
              <span class="text-foreground truncate text-[12px] font-semibold">电竞经理</span>
              <span class="text-foreground-subtle truncate text-[10px]">已登录</span>
            </div>
          </div>
        </div>
      </div>
    </aside>

    <!-- 2. Main Workspace -->
    <div class="bg-background-alt flex flex-1 flex-col overflow-hidden p-2 pt-0 pl-0">
      <div class="h-2 w-full select-none" data-tauri-drag-region></div>
      <section
        class="border-border bg-background relative flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border"
      >
        <!-- Header -->
        <header data-testid="workspace-header" class="border-border flex h-12 w-full shrink-0 border-b select-none">
          <div
            class="flex h-12 min-w-0 flex-1 items-center justify-between gap-2 overflow-hidden px-3 py-2 pl-4"
            data-tauri-drag-region
          >
            <!-- Header Left: View Title -->
            <div class="flex min-w-0 items-center gap-2 overflow-hidden">
              <h1
                class="text-foreground flex max-w-100 min-w-12 shrink items-center gap-2 truncate text-sm font-semibold"
              >
                <span class="min-w-0 truncate">{{ viewTitles[currentView] }}</span>
              </h1>

              <!-- Connection Status Indicator -->
              <div v-if="ws.connected" class="bg-green/10 flex items-center gap-1.5 rounded-full px-2 py-0.5">
                <span class="bg-green size-1.5 animate-pulse rounded-full" />
                <span class="text-green text-[11px] font-medium">{{ t("app.header.connected") }}</span>
              </div>
              <div
                v-else-if="store?.isStarting"
                class="flex items-center gap-1.5 rounded-full bg-yellow-500/10 px-2 py-0.5"
              >
                <span class="size-1.5 animate-pulse rounded-full bg-yellow-500" />
                <span class="text-[11px] font-medium text-yellow-500">{{ t("app.header.starting") }}</span>
              </div>
            </div>
          </div>
        </header>

        <!-- Main Content Viewport -->
        <main class="relative min-h-0 flex-1 overflow-y-auto">
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
        class="fixed inset-0 z-[1000] flex items-center justify-center bg-black/80 p-6 backdrop-blur-md"
      >
        <div
          class="border-border bg-card relative flex w-full max-w-[480px] flex-col gap-6 overflow-hidden rounded-lg border p-[2.2rem] shadow-2xl"
        >
          <div
            class="pointer-events-none absolute top-[-100px] left-1/2 h-[150px] w-[300px] -translate-x-1/2"
            style="background: radial-gradient(circle, var(--primary) 0%, transparent 70%); opacity: 0.15"
          ></div>
          <div class="z-1 flex items-center gap-3">
            <TrophyIcon class="animate-float text-primary size-8" />
            <h2 class="text-foreground m-0 text-[1.4rem] font-bold tracking-wider">
              {{ t("app.statsModal.title") }}
            </h2>
          </div>

          <div class="bg-border h-px" />

          <div class="z-1 flex flex-col gap-[1.2rem]">
            <p class="text-muted-foreground m-0 text-[0.9rem] leading-normal">
              {{ t("app.statsModal.description") }}
            </p>
            <div class="grid grid-cols-2 gap-4">
              <div class="border-border bg-muted/30 flex flex-col items-center gap-2 rounded-[6px] border p-[1.2rem]">
                <span class="text-muted-foreground text-[0.8rem] font-semibold tracking-wider uppercase">
                  {{ t("app.statsModal.minionKills") }}
                </span>
                <span class="text-foreground font-mono text-[2.2rem] leading-none font-extrabold">
                  {{ statsResult.minionKills }}
                </span>
              </div>
              <div class="border-border bg-muted/30 flex flex-col items-center gap-2 rounded-[6px] border p-[1.2rem]">
                <span class="text-muted-foreground text-[0.8rem] font-semibold tracking-wider uppercase">
                  {{ t("app.statsModal.gold") }}
                </span>
                <span class="text-foreground font-mono text-[2.2rem] leading-none font-extrabold">
                  {{ statsResult.gold.toFixed(0) }}
                  <span class="text-muted-foreground text-[1.2rem] font-semibold">g</span>
                </span>
              </div>
            </div>
          </div>

          <div class="z-1 flex justify-center">
            <Button class="w-full py-3" @click="showStatsModal = false">
              {{ t("app.statsModal.confirm") }}
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
