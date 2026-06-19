<script setup lang="ts">
import { ref } from "vue";

// ── Subcomponents ──
import ConsoleLogToolbar from "./game-console-logs/console-log-toolbar.vue";
import ConsoleLogFilters from "./game-console-logs/console-log-filters.vue";
import ConsoleLogAnalytics from "./game-console-logs/console-log-analytics.vue";
import ConsoleLogTable from "./game-console-logs/console-log-table.vue";
import ConsoleLogGroups from "./game-console-logs/console-log-groups.vue";
import ConsoleLogPagination from "./game-console-logs/console-log-pagination.vue";

const viewMode = ref<"timeline" | "groups">("timeline");
const showAnalytics = ref(false);
const copySuccessMsg = ref("");

const tableRef = ref<any>(null);

// 复制提示防抖
let copyTipTimeout: any = null;
function copyLogMessage(msg: string) {
  navigator.clipboard.writeText(msg).then(() => {
    copySuccessMsg.value = "已复制日志到剪贴板！";
    if (copyTipTimeout) clearTimeout(copyTipTimeout);
    copyTipTimeout = setTimeout(() => {
      copySuccessMsg.value = "";
    }, 2000);
  });
}
</script>

<template>
  <section
    class="bg-bg-surface border-border-subtle flex min-h-0 flex-1 flex-col overflow-hidden rounded border shadow-[0_1px_2px_rgba(0,0,0,0.4)]"
  >
    <!-- 日志工具栏 -->
    <ConsoleLogToolbar
      v-model:view-mode="viewMode"
      v-model:show-analytics="showAnalytics"
      :table="tableRef?.table"
    />

    <!-- 搜索与多维过滤 -->
    <ConsoleLogFilters />

    <!-- 可折叠的日志多维分析面板 -->
    <Transition name="slide-down">
      <ConsoleLogAnalytics
        v-if="showAnalytics"
        @close="showAnalytics = false"
      />
    </Transition>

    <!-- 复制提示气泡 -->
    <Transition name="fade">
      <div
        v-if="copySuccessMsg"
        class="bg-primary text-primary-foreground border-primary pointer-events-none fixed bottom-8 left-1/2 z-50 -translate-x-1/2 rounded-full border px-4 py-2 text-xs font-bold shadow-md shadow-primary/20"
      >
        <span>✨ {{ copySuccessMsg }}</span>
      </div>
    </Transition>

    <!-- 视图 1: Rich TanStack Table Timeline -->
    <ConsoleLogTable
      v-if="viewMode === 'timeline'"
      ref="tableRef"
      @copy="copyLogMessage"
    />

    <!-- 视图 2: 实体分组折叠列表 -->
    <ConsoleLogGroups
      v-else
      @copy="copyLogMessage"
    />

    <!-- 分页控制条 -->
    <ConsoleLogPagination
      v-if="viewMode === 'timeline'"
    />
  </section>
</template>

<style scoped>
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.25s ease-out;
}
.slide-down-enter-from,
.slide-down-leave-to {
  transform: translateY(-10px);
  opacity: 0;
  max-height: 0 !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
  margin-top: 0 !important;
  margin-bottom: 0 !important;
  border-bottom-width: 0 !important;
}
.slide-down-enter-to,
.slide-down-leave-from {
  max-height: 180px !important;
  opacity: 1;
}
</style>
