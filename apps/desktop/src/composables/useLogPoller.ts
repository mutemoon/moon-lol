import { ref, computed, onUnmounted, provide, inject, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";

export const LOG_CONTEXT_KEY = Symbol("log");

export interface LogEntry {
  id: number;
  timestamp: number;
  level: "info" | "warn" | "error" | "debug";
  file?: string;
  line?: number;
  entity_id?: number;
  entity_name?: string;
  category?: string;
  message: string;
  count: number;
}

interface LogRow {
  id: number;
  timestamp: number;
  level: string;
  file: string | null;
  line: number | null;
  entity_id: number | null;
  entity_name: string | null;
  category: string | null;
  message: string;
}

function processLogRows(rows: LogRow[]): LogEntry[] {
  const newLogs: LogEntry[] = [];

  for (const row of rows) {
    const entry: LogEntry = {
      id: row.id,
      timestamp: row.timestamp,
      level: (row.level || "info") as LogEntry["level"],
      file: row.file ?? undefined,
      line: row.line ?? undefined,
      entity_id: row.entity_id ?? undefined,
      entity_name: row.entity_name ?? undefined,
      category: row.category ?? undefined,
      message: row.message || "",
      count: 1,
    };

    const last = newLogs[newLogs.length - 1];
    const isDuplicate =
      last &&
      last.message === entry.message &&
      last.file === entry.file &&
      last.level === entry.level &&
      last.entity_id === entry.entity_id;

    if (isDuplicate) {
      last.count++;
      last.id = entry.id;
      continue;
    }

    newLogs.push(entry);
  }

  return newLogs;
}

export function createLogContext() {
  const logs = ref<LogEntry[]>([]);
  const totalLogsCount = ref(0);
  const currentPage = ref(1);
  const pageSize = ref(100);
  const autoScroll = ref(true);

  const selectedLevels = ref<string[]>(["info", "warn", "error", "debug"]);
  const filterEntityId = ref<number | null>(null);
  const filterCategory = ref<string | null>(null);
  const searchText = ref("");
  const regexEnabled = ref(false);
  const autoRefresh = ref(false);
  const autoRefreshInterval = ref(1000);

  const logEntities = ref<{ entity_id: number; entity_name: string }[]>([]);
  const logCategories = ref<string[]>([]);
  let intervalId: any = null;
  let queryTimeout: any = null;
  let isAutoUpdatingPage = false;

  const totalPages = computed(() => Math.ceil(totalLogsCount.value / pageSize.value) || 1);

  interface QueryLogsResult {
    rows: LogRow[];
    total_count: number;
  }

  async function refreshLogMetadata() {
    try {
      const [ents, cats] = await Promise.all([
        invoke<{ entity_id: number | null; entity_name: string | null }[]>("query_log_entities"),
        invoke<{ category: string | null }[]>("query_log_categories"),
      ]);
      logEntities.value = ents
        .filter((e) => e.entity_id !== null)
        .map((e) => ({ entity_id: e.entity_id as number, entity_name: e.entity_name ?? "" }));
      logCategories.value = cats.map((c) => c.category as string).filter(Boolean) as string[];
    } catch {
      // ignore
    }
  }

  async function clearLogsDb() {
    try {
      await invoke("clear_logs");
    } catch {
      // ignore
    }
  }

  async function query(isTimerRefresh = false) {
    try {
      const backendSearchText = !regexEnabled.value && searchText.value ? searchText.value : null;

      // 如果开启了自动追踪最新，且当前是定时刷新，我们传入 -1 让后端定位到最后一页
      // 否则我们根据前端的 currentPage 和 pageSize 算出物理偏移
      let offset = (currentPage.value - 1) * pageSize.value;
      if (isTimerRefresh && autoScroll.value) {
        offset = -1;
      }

      const res = await invoke<QueryLogsResult>("query_logs", {
        offset,
        limit: pageSize.value,
        levels: selectedLevels.value.length < 4 ? selectedLevels.value : null,
        entityId: filterEntityId.value,
        category: filterCategory.value,
        searchText: backendSearchText || null,
      });

      if (!res || !res.rows || res.rows.length === 0) {
        logs.value = [];
        totalLogsCount.value = 0;
        return;
      }

      totalLogsCount.value = res.total_count;

      if (isTimerRefresh && autoScroll.value) {
        isAutoUpdatingPage = true;
        currentPage.value = totalPages.value;
        setTimeout(() => {
          isAutoUpdatingPage = false;
        }, 0);
      }

      const processed = processLogRows(res.rows);

      // 前端正则筛选仅针对这当前页的一页日志数据
      if (regexEnabled.value && searchText.value.trim()) {
        try {
          const regex = new RegExp(searchText.value, "i");
          logs.value = processed.filter((e) => regex.test(e.message));
        } catch {
          logs.value = processed.filter((e) => e.message.toLowerCase().includes(searchText.value.toLowerCase()));
        }
      } else {
        logs.value = processed;
      }
    } catch {
      // ignore
    }
  }

  function debouncedQuery() {
    if (queryTimeout) clearTimeout(queryTimeout);
    queryTimeout = setTimeout(() => {
      query(false);
    }, 150);
  }

  async function refresh() {
    await Promise.all([refreshLogMetadata(), query(false)]);
  }

  async function start() {
    await clearLogsDb();
    logs.value = [];
    logEntities.value = [];
    logCategories.value = [];
    totalLogsCount.value = 0;
    currentPage.value = 1;
    await refresh();

    if (intervalId) clearInterval(intervalId);
    let metaCounter = 0;
    intervalId = setInterval(() => {
      if (autoRefresh.value) {
        query(true);
        metaCounter++;
        if (metaCounter >= 5) {
          refreshLogMetadata();
          metaCounter = 0;
        }
      }
    }, autoRefreshInterval.value);
  }

  function stop() {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
  }

  // 1. 过滤条件或每页条数变化时，重置页码为第一页并防抖刷新
  watch(
    [selectedLevels, filterEntityId, filterCategory, searchText, regexEnabled, pageSize],
    () => {
      currentPage.value = 1;
      debouncedQuery();
    },
    { deep: true },
  );

  // 2. 当前页码改变时，主动执行无延时的加载，如果是手动翻离最后一页，自动解除 autoScroll 锁定
  watch(currentPage, (newVal) => {
    if (isAutoUpdatingPage) return;

    if (newVal !== totalPages.value) {
      autoScroll.value = false;
    }
    query(false);
  });

  // 3. 自动刷新频率发生变化时，即时重建定时器以应用新周期
  watch(autoRefreshInterval, (newVal) => {
    if (intervalId) {
      clearInterval(intervalId);
      let metaCounter = 0;
      intervalId = setInterval(() => {
        if (autoRefresh.value) {
          query(true);
          metaCounter++;
          if (metaCounter >= 5) {
            refreshLogMetadata();
            metaCounter = 0;
          }
        }
      }, newVal);
    }
  });

  onUnmounted(() => {
    stop();
  });

  function addLog(level: string, message: string) {
    const entry: LogEntry = {
      id: Date.now(),
      timestamp: Date.now(),
      level: level as LogEntry["level"],
      message: message || "",
      count: 1,
    };
    logs.value.push(entry);
    if (logs.value.length > pageSize.value) {
      logs.value.shift();
    }
  }

  function clearLogs() {
    logs.value = [];
    totalLogsCount.value = 0;
  }

  function setEntityFilter(id: number | null) {
    filterEntityId.value = id;
  }

  function exportLogs(format: "txt" | "json") {
    let content = "";
    if (format === "json") {
      content = JSON.stringify(logs.value, null, 2);
    } else {
      content = logs.value
        .map((l) => {
          const time = new Date(l.timestamp).toLocaleTimeString();
          const cat = l.category ? `[${l.category}]` : "";
          const ent = l.entity_name ? `(${l.entity_name})` : "";
          return `[${time}] [${l.level.toUpperCase()}]${cat}${ent}: ${l.message}`;
        })
        .join("\n");
    }

    const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = `moon_lol_logs_${Date.now()}.${format}`;
    link.click();
    URL.revokeObjectURL(url);
  }

  const res = {
    logs,
    totalLogsCount,
    currentPage,
    pageSize,
    autoScroll,
    selectedLevels,
    filterEntityId,
    filterCategory,
    searchText,
    regexEnabled,
    autoRefresh,
    autoRefreshInterval,
    logEntities,
    logCategories,
    start,
    stop,
    query,
    refresh,
    addLog,
    clearLogs,
    setEntityFilter,
    exportLogs,
  };

  provide(LOG_CONTEXT_KEY, res);

  return res;
}

export type LogPoller = ReturnType<typeof createLogContext>;

export function useLog() {
  const poller = inject(LOG_CONTEXT_KEY) as LogPoller;
  if (!poller) {
    throw new Error("useLog() must be used inside a component descended from createLogContext()");
  }
  return poller;
}
