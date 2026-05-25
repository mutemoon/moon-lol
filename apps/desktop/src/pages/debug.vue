<script setup lang="ts">
import { ref, computed } from "vue";
import { useGameStore } from "../stores/gameStore";
import GameConsoleLogs from "../components/GameConsoleLogs.vue";
import { Button } from "../components/ui/button";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "../components/ui/select";

const store = useGameStore();
const { ws } = store;
const { stopGame } = store;

// Access unwrapped values reactively via computed properties
const connected = computed(() => ws.connected);
const gameState = computed(() => ws.gameState);

const switchTarget = ref("Riven");

function toggleGodMode() {
  store.ws.send("god_mode", { enabled: !gameState.value.godMode });
}

function toggleCooldown() {
  store.ws.send("toggle_cooldown", { enabled: !gameState.value.cooldownDisabled });
}

function togglePause() {
  store.ws.send("toggle_pause", {});
}

function resetPosition() {
  store.ws.send("reset_position", {});
}

function switchChampion() {
  store.ws.send("switch_champion", { name: switchTarget.value });
}

const champions = ["Riven", "Fiora"];
</script>

<template>
  <div class="bg-bg-deep flex h-full flex-col gap-3 overflow-hidden p-4">
    <!-- Status Bar -->
    <div
      class="bg-bg-surface border-border-subtle flex shrink-0 items-center justify-between rounded border px-3.5 py-2 shadow-[0_1px_2px_rgba(0,0,0,0.4)]"
    >
      <div class="flex items-center gap-3">
        <span
          class="inline-flex items-center gap-1.5 rounded border px-2 py-0.5 text-[11px] font-semibold tracking-wider uppercase transition-colors"
          :class="connected ? 'text-green border-green/15 bg-green/8' : 'text-red border-red/15 bg-red/8'"
        >
          <span
            class="h-1.5 w-1.5 rounded-full transition-shadow"
            :class="
              connected
                ? 'bg-green shadow-[0_0_6px_rgba(74,158,90,0.6)]'
                : 'bg-red shadow-[0_0_6px_rgba(200,74,74,0.4)]'
            "
          ></span>
          {{ connected ? "Connected" : "Disconnected" }}
        </span>
        <span class="bg-border-subtle h-3.5 w-px"></span>
        <span class="flex items-center gap-1.5">
          <span class="text-text-muted text-[11px] uppercase">Champion</span>
          <span class="text-text-bright text-xs font-semibold">{{ gameState.champion || "—" }}</span>
        </span>
      </div>
      <Button
        variant="outline"
        size="sm"
        class="text-red hover:text-red hover:bg-red/12 hover:border-red/45 border-red/25 bg-red/4 h-7 cursor-pointer rounded px-3 py-1 text-xs font-medium transition-all duration-200"
        @click="stopGame"
      >
        Stop Game
      </Button>
    </div>

    <!-- Main Workspace Layout -->
    <div class="flex min-h-0 flex-1 gap-3.5 overflow-hidden">
      <!-- LEFT COLUMN: Global Control Sidebar -->
      <div class="flex min-h-0 w-44 flex-col gap-3 overflow-hidden">
        <!-- Toggles Group -->
        <div class="bg-bg-surface border-border-subtle flex flex-col gap-1.5 rounded border p-2.5">
          <span class="text-text-muted text-[11px] font-semibold uppercase">Toggles</span>
          <div class="flex flex-col gap-1">
            <Button
              variant="outline"
              size="sm"
              class="text-text-muted bg-bg-deep border-border-subtle hover:text-text-default hover:border-gold-muted flex h-8 w-full cursor-pointer items-center justify-start gap-1.5 rounded px-2.5 py-1 text-xs whitespace-nowrap transition-all duration-200"
              :class="{ 'text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.06)]': gameState.godMode }"
              @click="toggleGodMode"
            >
              <span
                class="h-1.5 w-1.5 rounded-full transition-all"
                :class="
                  gameState.godMode ? 'bg-gold-default shadow-[0_0_6px_rgba(185,145,71,0.5)]' : 'bg-border-default'
                "
              ></span>
              God Mode
            </Button>
            <Button
              variant="outline"
              size="sm"
              class="text-text-muted bg-bg-deep border-border-subtle hover:text-text-default hover:border-gold-muted flex h-8 w-full cursor-pointer items-center justify-start gap-1.5 rounded px-2.5 py-1 text-xs whitespace-nowrap transition-all duration-200"
              :class="{ 'text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.06)]': gameState.cooldownDisabled }"
              @click="toggleCooldown"
            >
              <span
                class="h-1.5 w-1.5 rounded-full transition-all"
                :class="
                  gameState.cooldownDisabled
                    ? 'bg-gold-default shadow-[0_0_6px_rgba(185,145,71,0.5)]'
                    : 'bg-border-default'
                "
              ></span>
              No Cooldown
            </Button>
            <Button
              variant="outline"
              size="sm"
              class="text-text-muted bg-bg-deep border-border-subtle hover:text-text-default hover:border-gold-muted flex h-8 w-full cursor-pointer items-center justify-start gap-1.5 rounded px-2.5 py-1 text-xs whitespace-nowrap transition-all duration-200"
              :class="{ 'text-gold-bright border-gold-dimmer bg-[rgba(185,145,71,0.06)]': gameState.paused }"
              @click="togglePause"
            >
              <span
                class="h-1.5 w-1.5 rounded-full transition-all"
                :class="
                  gameState.paused ? 'bg-gold-default shadow-[0_0_6px_rgba(185,145,71,0.5)]' : 'bg-border-default'
                "
              ></span>
              {{ gameState.paused ? "Resume" : "Pause" }}
            </Button>
          </div>
        </div>

        <!-- Champion Group -->
        <div class="bg-bg-surface border-border-subtle flex flex-col gap-1.5 rounded border p-2.5">
          <span class="text-text-muted text-[11px] font-semibold uppercase">Champion</span>
          <div class="flex w-full flex-col gap-1.5">
            <Select v-model="switchTarget">
              <SelectTrigger
                class="bg-bg-deep border-gold-dimmer text-text-bright hover:border-gold-muted focus:border-gold-default focus-visible:ring-gold-default/30 h-8 w-full cursor-pointer px-2 text-xs focus-visible:ring-1"
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent class="border-border-subtle text-text-bright border bg-[#110e14]">
                <SelectGroup>
                  <SelectItem
                    v-for="c in champions"
                    :key="c"
                    :value="c"
                    class="cursor-pointer text-xs hover:bg-white/[0.04]"
                  >
                    {{ c }}
                  </SelectItem>
                </SelectGroup>
              </SelectContent>
            </Select>
            <Button
              variant="outline"
              size="xs"
              class="text-text-muted border-border-subtle hover:text-gold-bright hover:border-gold-muted h-8 w-full cursor-pointer rounded bg-transparent px-2.5 py-1 text-xs transition-all duration-200"
              @click="switchChampion"
            >
              Switch Champion
            </Button>
          </div>
        </div>

        <!-- Actions Group -->
        <div class="bg-bg-surface border-border-subtle flex flex-col gap-1.5 rounded border p-2.5">
          <span class="text-text-muted text-[11px] font-semibold uppercase">Actions</span>
          <Button
            variant="outline"
            size="xs"
            class="text-text-muted border-border-subtle hover:text-gold-bright hover:border-gold-muted h-8 w-full cursor-pointer rounded bg-transparent px-2.5 py-1 text-xs transition-all duration-200"
            @click="resetPosition"
          >
            Reset Position
          </Button>
        </div>
      </div>

      <!-- RIGHT COLUMN: Game Console Logs Workspace -->
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
        <div class="flex min-h-0 flex-1 flex-col overflow-hidden">
          <GameConsoleLogs />
        </div>
      </div>
    </div>
  </div>
</template>
