// 桌面端「我的选手」同步生命周期（数据驱动，非状态机）。
//
// 设计原则：数据是唯一真相来源，状态由数据派生，而非人为显式区分。
//
// 过去用 xstate 状态机管理「离线 / 已同步 / 有差异 / 同步中」四个态，
// 但其中 offline / synced / divergent 三个都是数据（online、divergences）
// 的纯投影——把它们塞进机器，就得为每个投影造事件、造路由，事件互相
// 区分，态越多事件越多，最终状态爆炸。
//
// 真正有时序依赖的只有「打开对话框 → 落盘中 → 完成」这条交互子流：
// dialog 打开才能 apply，apply 完才能 RESOLVED。故只保留 dialogOpen /
// applying 两个交互 ref；mode（含 offline/synced/divergent）与 divergences
// 由数据纯派生，数据一变自动重算，无需事件驱动。
//
// 外部数据（localPresets、cloudPresets、online）仍由 heroes.vue 持有；
// heroes.vue 在加载 / 保存 / 聚焦后调用 recheck() 刷新派生数据即可。
// 冲突解决子流的时序仍由 send() 显式驱动（OPEN_DIALOG / APPLY /
// RESOLVED / CLOSE / ERROR），但只翻转交互 ref，不再迁移状态。
//
// Web 端不启用本机（无本地源），调用方传入 enabled=false 时返回恒 synced 的 stub。

import { computed, ref, type ComputedRef, type Ref } from "vue";
import type { HeroPreset } from "@/services/backend";

export type DivergenceKind = "conflict" | "local_only" | "cloud_only";

export interface Divergence {
  name: string;
  kind: DivergenceKind;
  local: HeroPreset | null;
  cloud: HeroPreset | null;
}

export type SyncMode = "offline" | "synced" | "divergent" | "applying";

export interface SyncChoice {
  name: string;
  keep: "local" | "cloud";
}

// 交互子流事件：只翻转 dialogOpen / applying / error，不迁移状态。
type SyncEvent =
  | { type: "OPEN_DIALOG" }
  | { type: "CLOSE" }
  | { type: "APPLY" }
  | { type: "RESOLVED" }
  | { type: "ERROR"; message?: string };

export interface AgentSyncMachine {
  mode: ComputedRef<SyncMode>;
  dialogOpen: ComputedRef<boolean>;
  applying: ComputedRef<boolean>;
  error: Ref<string | undefined>;
  divergences: ComputedRef<Divergence[]>;
  /** 用最新算出的 (online, divergences) 刷新派生态。数据变即重算 mode。 */
  recheck: (online: boolean, divergences: Divergence[]) => void;
  send: (event: SyncEvent) => void;
}

export function useAgentSyncMachine(enabled: boolean): AgentSyncMachine {
  if (!enabled) {
    // Web 端 / 非桌面：无本地源，恒为 synced，吞掉所有事件。
    const noop = (() => {}) as (e: SyncEvent) => void;
    return {
      mode: computed(() => "synced" as SyncMode),
      dialogOpen: computed(() => false),
      applying: computed(() => false),
      error: ref<string | undefined>(undefined),
      divergences: computed(() => []),
      recheck: () => {},
      send: noop,
    };
  }

  // 数据真相：在线状态 + 差异集。由 heroes.vue 经 recheck() 更新。
  const online = ref(false);
  const divergences = ref<Divergence[]>([]);
  // 交互态：对话框开关与落盘中标志。时序由 send() 显式驱动。
  const dialogOpen = ref(false);
  const applying = ref(false);
  const error = ref<string | undefined>(undefined);

  // mode 是数据的纯投影：离线压过一切；否则按 divergences 是否为空分 synced/divergent；
  // applying 期间无论差异如何都报 applying（落盘中不可写）。
  const mode = computed<SyncMode>(() => {
    if (!online.value) return "offline";
    if (applying.value) return "applying";
    return divergences.value.length > 0 ? "divergent" : "synced";
  });

  function recheck(nextOnline: boolean, nextDivergences: Divergence[]) {
    online.value = nextOnline;
    divergences.value = nextDivergences;
  }

  function send(event: SyncEvent) {
    switch (event.type) {
      case "OPEN_DIALOG":
        dialogOpen.value = true;
        break;
      case "CLOSE":
        dialogOpen.value = false;
        break;
      case "APPLY":
        applying.value = true;
        error.value = undefined;
        break;
      case "RESOLVED":
        applying.value = false;
        dialogOpen.value = false;
        error.value = undefined;
        break;
      case "ERROR":
        applying.value = false;
        error.value = event.message;
        break;
    }
  }

  return {
    mode,
    dialogOpen: computed(() => dialogOpen.value),
    applying: computed(() => applying.value),
    error,
    divergences: computed(() => divergences.value),
    recheck,
    send,
  };
}
