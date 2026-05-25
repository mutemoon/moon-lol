<script setup lang="ts">
import { ref, computed } from "vue";
import { ScrollArea } from "./ui/scroll-area";
import { Badge } from "./ui/badge";
import { Input } from "./ui/input";
import { Button } from "./ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { MarkdownRender } from "markstream-vue";

interface ChatMessage {
  role: string;
  content: string | any[];
}

const props = withDefaults(
  defineProps<{
    history?: ChatMessage[];
    champion?: string;
    loading?: boolean;
    placeholderText?: string;
  }>(),
  {
    history: () => [],
    champion: "AI Agent",
    loading: false,
    placeholderText: "正在等待 AI 的第一次决策分析与行动（>= 40秒时开始）...",
  }
);

// 辅助函数：判断是否为数组
function isArray(val: any): val is any[] {
  return Array.isArray(val);
}

// 辅助函数：将任何类型的 content 转换为可显示格式
// 如果是 User / Environment 消息
function getParsedUserContent(content: any): ParsedSubItem[] {
  if (typeof content === "string") {
    return [{ type: "text" as const, text: content }];
  }
  if (isArray(content)) {
    return content.map((item: any) => {
      if (item.type === "toolresult") {
        let resultText = "";
        if (isArray(item.content)) {
          resultText = item.content.map((c: any) => c.text || JSON.stringify(c)).join("\n");
        } else {
          resultText = typeof item.content === "string" ? item.content : JSON.stringify(item.content);
        }
        return {
          type: "toolresult" as const,
          id: item.id,
          text: resultText,
          isError: resultText.includes("失败") || resultText.includes("错误") || resultText.includes("error") || resultText.includes("限制"),
        };
      }
      return {
        type: "text" as const,
        text: item.text || item.content || JSON.stringify(item),
      };
    });
  }
  return [{ type: "text" as const, text: JSON.stringify(content) }];
}

// 辅助函数：将 Assistant 消息进行结构化分类
function getParsedAssistantContent(content: any): ParsedSubItem[] {
  if (typeof content === "string") {
    return [{ type: "text" as const, text: content }];
  }
  if (isArray(content)) {
    const parsed: ParsedSubItem[] = [];
    content.forEach((item: any) => {
      // 1. 判断是否为 Thought (内部思维)
      if (item.content && isArray(item.content)) {
        const firstSub = item.content[0];
        if (firstSub && firstSub.type === "text" && firstSub.content && firstSub.content.text) {
          parsed.push({
            type: "thought" as const,
            text: firstSub.content.text,
            signature: firstSub.content.signature,
          });
          return;
        }
      }

      // 2. 判断是否为 Text Reply (公开发言)
      if (item.text && !item.function) {
        parsed.push({
          type: "text" as const,
          text: item.text,
        });
        return;
      }

      // 3. 判断是否为 Tool Call (工具调用指令)
      if (item.function) {
        const func = item.function;
        let cmd = "";
        if (func.arguments && func.arguments.cmd) {
          cmd = func.arguments.cmd;
        } else {
          cmd = JSON.stringify(func.arguments);
        }
        parsed.push({
          type: "toolcall" as const,
          id: item.id,
          name: func.name,
          command: cmd,
        });
        return;
      }

      // 兜底：如果完全不匹配，当成 json 或普通文本
      parsed.push({
        type: "unknown" as const,
        text: JSON.stringify(item),
      });
    });
    return parsed;
  }
  return [{ type: "text" as const, text: JSON.stringify(content) }];
}

// ==================== 筛选与过滤状态 ====================
const searchQuery = ref("");
const showThoughts = ref(true);
const showTools = ref(true);
const showDecisions = ref(true);
const showObservations = ref(true);
const roundFilter = ref("all");

// ==================== 折叠管理 ====================
const defaultCollapseThoughts = ref(false);
const defaultCollapseTools = ref(false);
const localCollapsed = ref<Record<string, boolean>>({});

function isItemCollapsed(msgIndex: number, itemIdx: number, type: 'thought' | 'toolresult') {
  const key = `${msgIndex}-${itemIdx}`;
  if (localCollapsed.value[key] !== undefined) {
    return localCollapsed.value[key];
  }
  if (type === 'thought') {
    return defaultCollapseThoughts.value;
  } else {
    return defaultCollapseTools.value;
  }
}

function toggleItemCollapse(msgIndex: number, itemIdx: number, type: 'thought' | 'toolresult') {
  const key = `${msgIndex}-${itemIdx}`;
  const current = isItemCollapsed(msgIndex, itemIdx, type);
  localCollapsed.value[key] = !current;
}

// ==================== 结构化消息解析与计算 ====================
interface ParsedSubItem {
  type: 'text' | 'toolresult' | 'thought' | 'toolcall' | 'unknown';
  id?: string;
  text?: string;
  signature?: string;
  name?: string;
  command?: string;
  isError?: boolean;
}

interface ParsedMessage {
  originalIndex: number;
  role: 'user' | 'assistant';
  roundNumber: number;
  items: ParsedSubItem[];
}

const parsedMessages = computed<ParsedMessage[]>(() => {
  return props.history.map((msg, index) => {
    const role = msg.role as 'user' | 'assistant';
    const roundNumber = Math.floor(index / 2) + 1;
    let items: ParsedSubItem[] = [];
    if (role === 'user') {
      items = getParsedUserContent(msg.content);
    } else {
      items = getParsedAssistantContent(msg.content);
    }
    return {
      originalIndex: index,
      role,
      roundNumber,
      items,
    };
  });
});

const maxRound = computed(() => {
  if (props.history.length === 0) return 0;
  return Math.floor((props.history.length - 1) / 2) + 1;
});

const filteredMessages = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  const limitRounds = roundFilter.value !== 'all' ? parseInt(roundFilter.value, 10) : null;
  const minRoundToKeep = limitRounds ? maxRound.value - limitRounds + 1 : 0;

  return parsedMessages.value
    .map((msg) => {
      // 1. 过滤轮次
      if (limitRounds && msg.roundNumber < minRoundToKeep) {
        return null;
      }

      // 2. 过滤子项类型
      const filteredItems = msg.items.filter((item) => {
        // 类型过滤
        if (item.type === "thought" && !showThoughts.value) return false;
        if ((item.type === "toolcall" || item.type === "toolresult") && !showTools.value) return false;
        if (msg.role === "assistant" && item.type === "text" && !showDecisions.value) return false;
        if (msg.role === "user" && item.type === "text" && !showObservations.value) return false;

        // 搜索过滤
        if (query) {
          const textMatch = item.text?.toLowerCase().includes(query) || false;
          const nameMatch = item.name?.toLowerCase().includes(query) || false;
          const cmdMatch = item.command?.toLowerCase().includes(query) || false;
          const idMatch = item.id?.toLowerCase().includes(query) || false;
          return textMatch || nameMatch || cmdMatch || idMatch;
        }

        return true;
      });

      if (filteredItems.length === 0) return null;

      return {
        ...msg,
        items: filteredItems,
      };
    })
    .filter((msg): msg is NonNullable<typeof msg> => msg !== null);
});
</script>

<template>
  <div class="flex h-full w-full flex-col overflow-hidden">
    <!-- ==================== 顶部悬浮筛选过滤控制面板 ==================== -->
    <div class="border-border-subtle/30 bg-bg-surface/90 sticky top-0 z-30 flex shrink-0 flex-col gap-3 border-b pb-4 pt-1 px-1 backdrop-blur-md">
      <!-- 第一排：搜索、轮次筛选与全局折叠 -->
      <div class="flex flex-wrap items-center justify-between gap-3">
        <!-- 搜索输入框 -->
        <div class="relative min-w-[200px] flex-1">
          <svg
            class="text-text-muted/50 absolute left-3 top-2.5 h-4 w-4"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <Input
            v-model="searchQuery"
            placeholder="搜寻思维、工具指令、返回数据..."
            class="bg-bg-deep/40 border-border-subtle/40 focus-visible:border-gold-muted/50 focus-visible:ring-gold-muted/10 h-9 w-full pl-9 pr-8 text-xs placeholder:text-text-muted/40 text-text-bright"
          />
          <button
            v-if="searchQuery"
            class="text-text-muted hover:text-text-bright absolute right-3 top-2.5 cursor-pointer text-xs"
            @click="searchQuery = ''"
          >
            ✕
          </button>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <!-- 轮次过滤器 -->
          <div class="flex items-center gap-1.5">
            <span class="text-text-muted text-[10px] font-medium whitespace-nowrap">轮次范围:</span>
            <Select v-model="roundFilter">
              <SelectTrigger class="bg-bg-deep/40 border-border-subtle/40 h-8 w-[100px] text-[11px] font-semibold text-text-bright">
                <SelectValue placeholder="所有轮次" />
              </SelectTrigger>
              <SelectContent class="border-border-subtle bg-bg-surface/95 text-xs text-text-bright">
                <SelectItem value="all" class="cursor-pointer text-[11px]">所有轮次</SelectItem>
                <SelectItem value="3" class="cursor-pointer text-[11px]">最近 3 轮</SelectItem>
                <SelectItem value="5" class="cursor-pointer text-[11px]">最近 5 轮</SelectItem>
                <SelectItem value="10" class="cursor-pointer text-[11px]">最近 10 轮</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="bg-border-subtle/30 h-4 w-px"></div>

          <!-- 默认折叠快捷操作 -->
          <div class="flex items-center gap-1.5">
            <Button
              variant="outline"
              size="xs"
              class="border-gold-dimmer/20 hover:bg-gold-dimmer/10 hover:text-gold-bright text-text-muted h-8 rounded px-2.5 text-[11px] cursor-pointer gap-1 transition-all"
              @click="defaultCollapseThoughts = !defaultCollapseThoughts"
            >
              <svg class="h-3 w-3 text-gold-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
              </svg>
              <span>思维: {{ defaultCollapseThoughts ? '全折叠' : '全展开' }}</span>
            </Button>

            <Button
              variant="outline"
              size="xs"
              class="border-blue-500/20 hover:bg-blue-500/10 hover:text-blue-400 text-text-muted h-8 rounded px-2.5 text-[11px] cursor-pointer gap-1 transition-all"
              @click="defaultCollapseTools = !defaultCollapseTools"
            >
              <svg class="h-3 w-3 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              </svg>
              <span>工具: {{ defaultCollapseTools ? '全折叠' : '全展开' }}</span>
            </Button>
          </div>
        </div>
      </div>

      <!-- 第二排：类型筛选开关按钮 -->
      <div class="flex flex-wrap items-center gap-2 text-xs">
        <span class="text-text-muted text-[10px] font-medium mr-1 select-none">过滤内容:</span>
        
        <!-- 思维过程 Toggle Button -->
        <button
          @click="showThoughts = !showThoughts"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-semibold cursor-pointer transition-all shadow-xs duration-200 outline-none select-none',
            showThoughts
              ? 'bg-gold-dimmer/15 border-gold-dimmer/65 text-gold-bright shadow-inner'
              : 'bg-zinc-950/20 border-zinc-800/40 text-zinc-500 hover:text-zinc-400'
          ]"
        >
          <span class="h-1.5 w-1.5 rounded-full" :class="showThoughts ? 'bg-gold-muted animate-pulse' : 'bg-zinc-600'"></span>
          思维链 (Thoughts)
        </button>

        <!-- 工具指令 Toggle Button -->
        <button
          @click="showTools = !showTools"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-semibold cursor-pointer transition-all shadow-xs duration-200 outline-none select-none',
            showTools
              ? 'bg-blue-500/10 border-blue-500/40 text-blue-400 shadow-inner'
              : 'bg-zinc-950/20 border-zinc-800/40 text-zinc-500 hover:text-zinc-400'
          ]"
        >
          <span class="h-1.5 w-1.5 rounded-full" :class="showTools ? 'bg-blue-400 animate-pulse' : 'bg-zinc-600'"></span>
          工具与输出 (Tools)
        </button>

        <!-- 公开决策 Toggle Button -->
        <button
          @click="showDecisions = !showDecisions"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-semibold cursor-pointer transition-all shadow-xs duration-200 outline-none select-none',
            showDecisions
              ? 'bg-purple-950/15 border-purple-500/35 text-purple-400 shadow-inner'
              : 'bg-zinc-950/20 border-zinc-800/40 text-zinc-500 hover:text-zinc-400'
          ]"
        >
          <span class="h-1.5 w-1.5 rounded-full" :class="showDecisions ? 'bg-purple-400 animate-pulse' : 'bg-zinc-600'"></span>
          公开决策 (Replies)
        </button>

        <!-- 环境观测 Toggle Button -->
        <button
          @click="showObservations = !showObservations"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-semibold cursor-pointer transition-all shadow-xs duration-200 outline-none select-none',
            showObservations
              ? 'bg-zinc-800/30 border-zinc-600/30 text-zinc-300 shadow-inner'
              : 'bg-zinc-950/20 border-zinc-800/40 text-zinc-500 hover:text-zinc-400'
          ]"
        >
          <span class="h-1.5 w-1.5 rounded-full" :class="showObservations ? 'bg-zinc-400 animate-pulse' : 'bg-zinc-600'"></span>
          环境观测 (Observ.)
        </button>

        <!-- 数量指示器 -->
        <span class="text-text-muted text-[10px] ml-auto font-mono select-none">
          显示 {{ filteredMessages.length }} / {{ props.history.length }} 条记录
        </span>
      </div>
    </div>

    <!-- ==================== 历史流滚动区域 ==================== -->
    <ScrollArea class="flex-1 w-full min-h-0 pr-2">
      <!-- 空数据或 Loading 状态 -->
      <div
        v-if="loading || !history || history.length === 0"
        class="text-text-muted flex h-full flex-col items-center justify-center gap-3 py-24 text-center text-xs italic"
      >
        <svg
          class="text-gold-muted h-6 w-6 animate-spin"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          ></path>
        </svg>
        <span class="max-w-md px-6 leading-relaxed">{{ placeholderText }}</span>
      </div>

      <!-- 无匹配过滤结果状态 -->
      <div
        v-else-if="filteredMessages.length === 0"
        class="text-text-muted flex h-full flex-col items-center justify-center gap-3 py-20 text-center text-xs"
      >
        <svg class="h-8 w-8 text-zinc-600 opacity-60 animate-pulse" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <span class="max-w-xs px-6 text-zinc-400 italic">
          没有找到符合当前过滤条件的决策记录。请尝试在上方调整过滤内容或搜索词。
        </span>
        <Button
          variant="outline"
          size="xs"
          class="border-zinc-800 hover:bg-zinc-900 mt-1.5 text-[10px]"
          @click="searchQuery = ''; showThoughts = true; showTools = true; showDecisions = true; showObservations = true; roundFilter = 'all'"
        >
          重置所有过滤条件
        </Button>
      </div>

      <!-- 消息历史流 -->
      <div v-else class="flex flex-col gap-5 pb-6 pt-2">
        <div
          v-for="msg in filteredMessages"
          :key="msg.originalIndex"
          class="flex flex-col gap-2"
        >
          <!-- ==================== USER ROLE ==================== -->
          <template v-if="msg.role === 'user'">
            <div class="mt-4 flex items-center gap-3">
              <div class="bg-border-subtle/30 h-px flex-1"></div>
              <div
                class="border-gold-dimmer/25 text-gold-bright flex items-center gap-1.5 rounded-full border bg-[#16121a] px-4 py-0.5 font-mono text-[9px] tracking-wider uppercase shadow-[0_2px_4px_rgba(0,0,0,0.2)] select-none"
              >
                <span class="h-1.5 w-1.5 rounded-full bg-gold-muted animate-pulse"></span>
                决策轮次 #{{ msg.roundNumber }} · 环境观测与指令
              </div>
              <div class="bg-border-subtle/30 h-px flex-1"></div>
            </div>

            <!-- User 消息的各子项渲染 -->
            <div class="flex flex-col gap-2.5 px-1">
              <template v-for="(item, itemIdx) in msg.items" :key="itemIdx">
                <!-- 普通文本指令 -->
                <div
                  v-if="item.type === 'text'"
                  class="text-text-bright border-border-subtle/30 leading-relaxed max-w-4xl rounded-r-lg rounded-bl-lg border bg-white/[0.01] p-3 text-xs shadow-inner"
                >
                  <MarkdownRender :content="item.text || ''" :smooth-streaming="false" :fade="true" />
                </div>

                <!-- 工具执行结果 (终端输出效果) -->
                <div
                  v-else-if="item.type === 'toolresult'"
                  class="border-border-subtle/20 bg-bg-deep/80 overflow-hidden rounded border font-mono shadow-[0_4px_12px_rgba(0,0,0,0.5)] transition-all duration-300"
                >
                  <!-- 终端标题栏 -->
                  <div class="border-border-subtle/30 bg-bg-surface flex items-center justify-between border-b px-3 py-1.5 text-[9px]">
                    <div class="text-text-muted flex items-center gap-1.5 select-none">
                      <span class="h-2 w-2 rounded-full" :class="item.isError ? 'bg-red/60' : 'bg-green/60'"></span>
                      <span>Tool Output (ID: {{ item.id }})</span>
                    </div>
                    <div class="flex items-center gap-2 select-none">
                      <span class="text-text-muted font-mono">bash</span>
                      <!-- 折叠动作图标 -->
                      <button
                        @click="toggleItemCollapse(msg.originalIndex, itemIdx, 'toolresult')"
                        class="text-text-muted hover:text-text-bright cursor-pointer flex items-center gap-0.5 rounded px-1.5 py-0.5 bg-zinc-950/20 hover:bg-zinc-800 transition-colors"
                      >
                        <span class="text-[8px] font-bold">{{ isItemCollapsed(msg.originalIndex, itemIdx, 'toolresult') ? '展开' : '折叠' }}</span>
                        <svg
                          class="h-3 w-3 transform transition-transform duration-200"
                          :class="isItemCollapsed(msg.originalIndex, itemIdx, 'toolresult') ? '' : 'rotate-180'"
                          fill="none"
                          viewBox="0 0 24 24"
                          stroke="currentColor"
                        >
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                      </button>
                    </div>
                  </div>
                  <!-- 终端内容 -->
                  <div
                    v-show="!isItemCollapsed(msg.originalIndex, itemIdx, 'toolresult')"
                    class="whitespace-pre-wrap p-3 text-[11px] leading-relaxed select-text"
                    :class="item.isError ? 'text-red bg-red/4' : 'text-green/90 bg-green/2'"
                  >
                    {{ item.text }}
                  </div>
                  <!-- 折叠简略提示 -->
                  <div
                    v-show="isItemCollapsed(msg.originalIndex, itemIdx, 'toolresult')"
                    @click="toggleItemCollapse(msg.originalIndex, itemIdx, 'toolresult')"
                    class="cursor-pointer p-2.5 text-[10px] text-zinc-500 hover:text-zinc-400 bg-zinc-950/20 italic flex justify-between items-center transition-colors select-none"
                  >
                    <span>终端输出已折叠 ({{ item.text ? item.text.split('\n').length : 0 }} 行内容 · 点击展开)</span>
                    <span class="font-semibold text-green/60 group-hover:underline">展开终端结果</span>
                  </div>
                </div>
              </template>
            </div>
          </template>

          <!-- ==================== ASSISTANT ROLE ==================== -->
          <template v-else>
            <!-- Assistant 消息的各子项渲染 -->
            <div class="flex flex-col gap-3">
              <template v-for="(item, itemIdx) in msg.items" :key="itemIdx">
                <!-- 1. AI 内部思维 (Thought Box) -->
                <div
                  v-if="item.type === 'thought'"
                  class="border-gold-dimmer/20 bg-gold-dimmer/4 relative overflow-hidden rounded border p-3.5 shadow-sm transition-all duration-300"
                >
                  <!-- 顶部小徽标 -->
                  <div class="mb-2 flex items-center justify-between select-none">
                    <div class="text-gold-bright flex items-center gap-1.5 text-[11px] font-semibold tracking-wide">
                      <svg class="h-3.5 w-3.5 animate-pulse text-gold-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                      </svg>
                      <span>{{ champion }} 的内部思维链 (Thought reasoning)</span>
                      <span v-if="isItemCollapsed(msg.originalIndex, itemIdx, 'thought')" class="text-zinc-500 text-[10px] font-normal italic">
                        (共 {{ item.text ? item.text.length : 0 }} 字 · 已折叠)
                      </span>
                    </div>
                    <div class="flex items-center gap-2">
                      <span class="text-text-muted font-mono text-[9px] opacity-60">Thought Mind</span>
                      <!-- 折叠触发器 -->
                      <button
                        @click="toggleItemCollapse(msg.originalIndex, itemIdx, 'thought')"
                        class="text-gold-bright hover:text-gold-default cursor-pointer flex items-center gap-0.5 rounded px-1.5 py-0.5 bg-gold-dimmer/10 hover:bg-gold-dimmer/20 transition-colors"
                      >
                        <span class="text-[9px] font-bold">{{ isItemCollapsed(msg.originalIndex, itemIdx, 'thought') ? '展开' : '折叠' }}</span>
                        <svg
                          class="h-3.5 w-3.5 transform transition-transform duration-200"
                          :class="isItemCollapsed(msg.originalIndex, itemIdx, 'thought') ? '' : 'rotate-180'"
                          fill="none"
                          viewBox="0 0 24 24"
                          stroke="currentColor"
                        >
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                        </svg>
                      </button>
                    </div>
                  </div>
                  <!-- 思维文本 -->
                  <div
                    v-show="!isItemCollapsed(msg.originalIndex, itemIdx, 'thought')"
                    class="text-text-muted leading-relaxed select-text font-sans text-xs"
                  >
                    <MarkdownRender :content="item.text || ''" :smooth-streaming="false" :fade="true" />
                  </div>
                  <!-- 折叠简略预览提示 -->
                  <div
                    v-show="isItemCollapsed(msg.originalIndex, itemIdx, 'thought')"
                    @click="toggleItemCollapse(msg.originalIndex, itemIdx, 'thought')"
                    class="cursor-pointer py-2 px-2.5 text-[11px] text-gold-muted/80 hover:text-gold-bright/90 bg-gold-dimmer/5 border border-gold-dimmer/10 rounded transition-colors italic flex items-center justify-between"
                  >
                    <span class="truncate pr-4 select-none">{{ item.text ? item.text.replace(/[\n\s#*`>]/g, ' ').slice(0, 75) : '' }}...</span>
                    <span class="font-semibold text-gold-muted shrink-0 flex items-center gap-1 select-none">
                      展开思维链
                    </span>
                  </div>
                </div>

                <!-- 2. 工具调用指令 (Tool Call) -->
                <div
                  v-else-if="item.type === 'toolcall'"
                  class="border-blue-500/20 bg-blue-950/8 flex flex-col overflow-hidden rounded border font-mono transition-all duration-300"
                >
                  <!-- 工具调用标题 -->
                  <div class="border-blue-500/10 bg-blue-950/20 text-blue-300 flex items-center justify-between border-b px-3.5 py-1.5 text-[10px] font-semibold select-none">
                    <div class="flex items-center gap-1.5">
                      <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      </svg>
                      <span>发起工具指令: {{ item.name }}</span>
                    </div>
                    <Badge variant="outline" class="border-blue-400/30 text-blue-300 px-1 py-0 text-[8px] bg-blue-900/10">Active Call</Badge>
                  </div>
                  <!-- 调用的命令行内容 -->
                  <div class="bg-[#0b0c10] p-3 text-xs leading-relaxed text-blue-100 select-text flex items-center gap-2">
                    <span class="text-blue-500 font-bold select-none">$</span>
                    <span class="font-mono flex-1">{{ item.command }}</span>
                  </div>
                </div>

                <!-- 3. 普通文本消息 (Text Reply) -->
                <div
                  v-else-if="item.type === 'text'"
                  class="border-border-subtle/40 rounded border bg-[#0d0b0f] p-4 font-sans text-xs leading-relaxed shadow-md relative overflow-hidden transition-all duration-300"
                >
                  <!-- 背景微弱渐变光环，增加高级感 -->
                  <div class="absolute right-0 top-0 h-24 w-24 bg-gradient-to-br from-gold-muted/5 to-transparent rounded-full blur-xl pointer-events-none"></div>

                  <div class="mb-2.5 flex items-center justify-between border-b border-border-subtle/10 pb-1.5 select-none">
                    <span class="text-gold-bright text-[11px] font-bold tracking-wide">
                      {{ champion }} 的公开对线决策
                    </span>
                    <span class="text-text-muted font-mono text-[9px] opacity-75">Assistant Reply</span>
                  </div>

                  <div class="text-text-bright pr-1.5 leading-relaxed select-text font-medium">
                    <MarkdownRender :content="item.text || ''" :smooth-streaming="false" :fade="true" />
                  </div>
                </div>
              </template>
            </div>
          </template>
        </div>
      </div>
    </ScrollArea>
  </div>
</template>

