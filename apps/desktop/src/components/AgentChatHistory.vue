<script setup lang="ts">
import { ref, computed } from "vue";
import { ScrollArea } from "./ui/scroll-area";
import { Badge } from "./ui/badge";
import { Input } from "./ui/input";
import { Button } from "./ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { MarkdownRender } from "markstream-vue";
import { SearchIcon, HelpCircleIcon, SettingsIcon, RefreshCwIcon, ChevronDownIcon } from "@lucide/vue";

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

      // 兜底
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
    <div class="border-b border-border bg-card/90 sticky top-0 z-30 flex shrink-0 flex-col gap-3 pb-4 pt-1 px-1 backdrop-blur-md">
      <!-- 第一排：搜索、轮次筛选与全局折叠 -->
      <div class="flex flex-wrap items-center justify-between gap-3">
        <!-- 搜索输入框 -->
        <div class="relative min-w-[200px] flex-1">
          <SearchIcon class="text-muted-foreground/50 absolute left-3 top-2.5 h-4 w-4" />
          <Input
            v-model="searchQuery"
            placeholder="搜寻思维、工具指令、返回数据..."
            class="bg-muted/40 border-border/40 focus-visible:border-primary/50 focus-visible:ring-primary/10 h-9 w-full pl-9 pr-8 text-xs placeholder:text-muted-foreground/40 text-foreground"
          />
          <button
            v-if="searchQuery"
            class="text-muted-foreground hover:text-foreground absolute right-3 top-2.5 cursor-pointer text-xs"
            @click="searchQuery = ''"
          >
            ✕
          </button>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <!-- 轮次过滤器 -->
          <div class="flex items-center gap-1.5">
            <span class="text-muted-foreground text-[10px] font-medium whitespace-nowrap">轮次范围:</span>
            <Select v-model="roundFilter">
              <SelectTrigger class="bg-muted/40 border-border/40 h-8 w-[100px] text-[11px] font-semibold text-foreground">
                <SelectValue placeholder="所有轮次" />
              </SelectTrigger>
              <SelectContent class="border-border bg-popover text-xs text-popover-foreground">
                <SelectItem value="all" class="cursor-pointer text-[11px]">所有轮次</SelectItem>
                <SelectItem value="3" class="cursor-pointer text-[11px]">最近 3 轮</SelectItem>
                <SelectItem value="5" class="cursor-pointer text-[11px]">最近 5 轮</SelectItem>
                <SelectItem value="10" class="cursor-pointer text-[11px]">最近 10 轮</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="bg-border h-4 w-px"></div>

          <!-- 默认折叠快捷操作 -->
          <div class="flex items-center gap-1.5">
            <Button
              variant="outline"
              size="xs"
              class="border-primary/20 hover:bg-primary/10 hover:text-primary text-muted-foreground h-8 rounded px-2.5 text-[11px] cursor-pointer gap-1 transition-all"
              @click="defaultCollapseThoughts = !defaultCollapseThoughts"
            >
              <HelpCircleIcon class="h-3 w-3 text-primary" />
              <span>思维: {{ defaultCollapseThoughts ? '全折叠' : '全展开' }}</span>
            </Button>

            <Button
              variant="outline"
              size="xs"
              class="border-blue-500/20 hover:bg-blue-500/10 hover:text-blue-400 text-muted-foreground h-8 rounded px-2.5 text-[11px] cursor-pointer gap-1 transition-all"
              @click="defaultCollapseTools = !defaultCollapseTools"
            >
              <SettingsIcon class="h-3 w-3 text-blue-400" />
              <span>工具: {{ defaultCollapseTools ? '全折叠' : '全展开' }}</span>
            </Button>
          </div>
        </div>
      </div>

      <!-- 第二排：类型筛选开关按钮 -->
      <div class="flex flex-wrap items-center gap-2 text-xs">
        <span class="text-muted-foreground text-[10px] font-medium mr-1 select-none">过滤内容:</span>
        
        <!-- 思维过程 Toggle Button -->
        <button
          @click="showThoughts = !showThoughts"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-semibold cursor-pointer transition-all shadow-xs duration-200 outline-none select-none',
            showThoughts
              ? 'bg-primary/15 border-primary/65 text-primary shadow-inner'
              : 'bg-muted/20 border-border text-muted-foreground hover:text-foreground'
          ]"
        >
          <span class="h-1.5 w-1.5 rounded-full" :class="showThoughts ? 'bg-primary animate-pulse' : 'bg-muted-foreground/60'"></span>
          思维链 (Thoughts)
        </button>

        <!-- 工具指令 Toggle Button -->
        <button
          @click="showTools = !showTools"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-semibold cursor-pointer transition-all shadow-xs duration-200 outline-none select-none',
            showTools
              ? 'bg-blue-500/10 border-blue-500/40 text-blue-400 shadow-inner'
              : 'bg-muted/20 border-border text-muted-foreground hover:text-foreground'
          ]"
        >
          <span class="h-1.5 w-1.5 rounded-full" :class="showTools ? 'bg-blue-400 animate-pulse' : 'bg-muted-foreground/60'"></span>
          工具与输出 (Tools)
        </button>

        <!-- 公开决策 Toggle Button -->
        <button
          @click="showDecisions = !showDecisions"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-semibold cursor-pointer transition-all shadow-xs duration-200 outline-none select-none',
            showDecisions
              ? 'bg-purple-500/10 border-purple-500/40 text-purple-400 shadow-inner'
              : 'bg-muted/20 border-border text-muted-foreground hover:text-foreground'
          ]"
        >
          <span class="h-1.5 w-1.5 rounded-full" :class="showDecisions ? 'bg-purple-400 animate-pulse' : 'bg-muted-foreground/60'"></span>
          公开决策 (Replies)
        </button>

        <!-- 环境观测 Toggle Button -->
        <button
          @click="showObservations = !showObservations"
          :class="[
            'flex items-center gap-1.5 px-3 py-1 rounded-full border text-[11px] font-semibold cursor-pointer transition-all shadow-xs duration-200 outline-none select-none',
            showObservations
              ? 'bg-zinc-500/10 border-zinc-500/40 text-zinc-300 shadow-inner'
              : 'bg-muted/20 border-border text-muted-foreground hover:text-foreground'
          ]"
        >
          <span class="h-1.5 w-1.5 rounded-full" :class="showObservations ? 'bg-zinc-400 animate-pulse' : 'bg-muted-foreground/60'"></span>
          环境观测 (Observ.)
        </button>

        <!-- 数量指示器 -->
        <span class="text-muted-foreground text-[10px] ml-auto font-mono select-none">
          显示 {{ filteredMessages.length }} / {{ props.history.length }} 条记录
        </span>
      </div>
    </div>

    <!-- ==================== 历史流滚动区域 ==================== -->
    <ScrollArea class="flex-1 w-full min-h-0 pr-2">
      <!-- 空数据或 Loading 状态 -->
      <div
        v-if="loading || !history || history.length === 0"
        class="text-muted-foreground flex h-full flex-col items-center justify-center gap-3 py-24 text-center text-xs italic"
      >
        <RefreshCwIcon class="h-6 w-6 animate-spin text-primary" />
        <span class="max-w-md px-6 leading-relaxed">{{ placeholderText }}</span>
      </div>

      <!-- 无匹配过滤结果状态 -->
      <div
        v-else-if="filteredMessages.length === 0"
        class="text-muted-foreground flex h-full flex-col items-center justify-center gap-3 py-20 text-center text-xs"
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
          class="border-border hover:bg-muted mt-1.5 text-[10px]"
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
              <div class="h-px bg-border flex-1"></div>
              <div
                class="border-primary/25 text-primary flex items-center gap-1.5 rounded-full border bg-muted px-4 py-0.5 font-mono text-[9px] tracking-wider uppercase shadow-sm select-none"
              >
                <span class="h-1.5 w-1.5 rounded-full bg-primary animate-pulse"></span>
                决策轮次 #{{ msg.roundNumber }} · 环境观测与指令
              </div>
              <div class="h-px bg-border flex-1"></div>
            </div>

            <!-- User 消息的各子项渲染 -->
            <div class="flex flex-col gap-2.5 px-1">
              <template v-for="(item, itemIdx) in msg.items" :key="itemIdx">
                <!-- 普通文本指令 -->
                <div
                  v-if="item.type === 'text'"
                  class="text-foreground border-border/30 leading-relaxed max-w-4xl rounded-r-lg rounded-bl-lg border bg-card p-3 text-xs shadow-sm"
                >
                  <MarkdownRender :content="item.text || ''" :smooth-streaming="false" :fade="true" />
                </div>

                <!-- 工具执行结果 (终端输出效果) -->
                <div
                  v-else-if="item.type === 'toolresult'"
                  class="border-border/20 bg-muted/30 overflow-hidden rounded border font-mono shadow-sm transition-all duration-300"
                >
                  <!-- 终端标题栏 -->
                  <div class="border-border/30 bg-muted flex items-center justify-between border-b px-3 py-1.5 text-[9px]">
                    <div class="text-muted-foreground flex items-center gap-1.5 select-none">
                      <span class="h-2 w-2 rounded-full" :class="item.isError ? 'bg-destructive/60' : 'bg-green-500/60'"></span>
                      <span>Tool Output (ID: {{ item.id }})</span>
                    </div>
                    <div class="flex items-center gap-2 select-none">
                      <span class="text-muted-foreground font-mono">bash</span>
                      <!-- 折叠动作图标 -->
                      <button
                        @click="toggleItemCollapse(msg.originalIndex, itemIdx, 'toolresult')"
                        class="text-muted-foreground hover:text-foreground cursor-pointer flex items-center gap-0.5 rounded px-1.5 py-0.5 bg-muted/80 transition-colors"
                      >
                        <span class="text-[8px] font-bold">{{ isItemCollapsed(msg.originalIndex, itemIdx, 'toolresult') ? '展开' : '折叠' }}</span>
                        <ChevronDownIcon
                          class="h-3 w-3 transform transition-transform duration-200"
                          :class="isItemCollapsed(msg.originalIndex, itemIdx, 'toolresult') ? '' : 'rotate-180'"
                        />
                      </button>
                    </div>
                  </div>
                  <!-- 终端内容 -->
                  <div
                    v-show="!isItemCollapsed(msg.originalIndex, itemIdx, 'toolresult')"
                    class="whitespace-pre-wrap p-3 text-[11px] leading-relaxed select-text"
                    :class="item.isError ? 'text-destructive bg-destructive/5' : 'text-foreground/90 bg-muted/10'"
                  >
                    {{ item.text }}
                  </div>
                  <!-- 折叠简略提示 -->
                  <div
                    v-show="isItemCollapsed(msg.originalIndex, itemIdx, 'toolresult')"
                    @click="toggleItemCollapse(msg.originalIndex, itemIdx, 'toolresult')"
                    class="cursor-pointer p-2.5 text-[10px] text-muted-foreground bg-muted/20 italic flex justify-between items-center transition-colors select-none"
                  >
                    <span>终端输出已折叠 ({{ item.text ? item.text.split('\n').length : 0 }} 行内容 · 点击展开)</span>
                    <span class="font-semibold text-primary/80 group-hover:underline">展开终端结果</span>
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
                  class="border-primary/20 bg-primary/5 relative overflow-hidden rounded border p-3.5 shadow-sm transition-all duration-300"
                >
                  <!-- 顶部小徽标 -->
                  <div class="mb-2 flex items-center justify-between select-none">
                    <div class="text-primary flex items-center gap-1.5 text-[11px] font-semibold tracking-wide">
                      <BrainIcon class="h-3.5 w-3.5 animate-pulse text-primary" />
                      <span>{{ champion }} 的内部思维链 (Thought reasoning)</span>
                      <span v-if="isItemCollapsed(msg.originalIndex, itemIdx, 'thought')" class="text-muted-foreground text-[10px] font-normal italic">
                        (共 {{ item.text ? item.text.length : 0 }} 字 · 已折叠)
                      </span>
                    </div>
                    <div class="flex items-center gap-2">
                      <span class="text-muted-foreground font-mono text-[9px] opacity-60">Thought Mind</span>
                      <!-- 折叠触发器 -->
                      <button
                        @click="toggleItemCollapse(msg.originalIndex, itemIdx, 'thought')"
                        class="text-primary hover:text-primary/80 cursor-pointer flex items-center gap-0.5 rounded px-1.5 py-0.5 bg-primary/10 hover:bg-primary/20 transition-colors"
                      >
                        <span class="text-[9px] font-bold">{{ isItemCollapsed(msg.originalIndex, itemIdx, 'thought') ? '展开' : '折叠' }}</span>
                        <ChevronDownIcon
                          class="h-3.5 w-3.5 transform transition-transform duration-200"
                          :class="isItemCollapsed(msg.originalIndex, itemIdx, 'thought') ? '' : 'rotate-180'"
                        />
                      </button>
                    </div>
                  </div>
                  <!-- 思维文本 -->
                  <div
                    v-show="!isItemCollapsed(msg.originalIndex, itemIdx, 'thought')"
                    class="text-muted-foreground leading-relaxed select-text font-sans text-xs"
                  >
                    <MarkdownRender :content="item.text || ''" :smooth-streaming="false" :fade="true" />
                  </div>
                  <!-- 折叠简略预览提示 -->
                  <div
                    v-show="isItemCollapsed(msg.originalIndex, itemIdx, 'thought')"
                    @click="toggleItemCollapse(msg.originalIndex, itemIdx, 'thought')"
                    class="cursor-pointer py-2 px-2.5 text-[11px] text-primary/80 hover:text-primary bg-primary/5 border border-primary/10 rounded transition-colors italic flex items-center justify-between"
                  >
                    <span class="truncate pr-4 select-none text-muted-foreground">{{ item.text ? item.text.replace(/[\n\s#*`>]/g, ' ').slice(0, 75) : '' }}...</span>
                    <span class="font-semibold text-primary shrink-0 flex items-center gap-1 select-none">
                      展开思维链
                    </span>
                  </div>
                </div>

                <!-- 2. 工具调用指令 (Tool Call) -->
                <div
                  v-else-if="item.type === 'toolcall'"
                  class="border-blue-500/20 bg-blue-500/5 flex flex-col overflow-hidden rounded border font-mono transition-all duration-300"
                >
                  <!-- 工具调用标题 -->
                  <div class="border-blue-500/10 bg-blue-500/10 text-blue-500 flex items-center justify-between border-b px-3.5 py-1.5 text-[10px] font-semibold select-none">
                    <div class="flex items-center gap-1.5">
                      <SettingsIcon class="h-3.5 w-3.5" />
                      <span>发起工具指令: {{ item.name }}</span>
                    </div>
                    <Badge variant="outline" class="border-blue-400/30 text-blue-500 px-1 py-0 text-[8px] bg-blue-500/10">Active Call</Badge>
                  </div>
                  <!-- 调用的命令行内容 -->
                  <div class="bg-muted p-3 text-xs leading-relaxed text-foreground select-text flex items-center gap-2">
                    <span class="text-blue-500 font-bold select-none">$</span>
                    <span class="font-mono flex-1">{{ item.command }}</span>
                  </div>
                </div>

                <!-- 3. 普通文本消息 (Text Reply) -->
                <div
                  v-else-if="item.type === 'text'"
                  class="border-border/40 rounded border bg-card p-4 font-sans text-xs leading-relaxed shadow-sm relative overflow-hidden transition-all duration-300"
                >
                  <div class="absolute right-0 top-0 h-24 w-24 bg-gradient-to-br from-primary/5 to-transparent rounded-full blur-xl pointer-events-none"></div>

                  <div class="mb-2.5 flex items-center justify-between border-b border-border/10 pb-1.5 select-none">
                    <span class="text-primary text-[11px] font-bold tracking-wide">
                      {{ champion }} 的公开对线决策
                    </span>
                    <span class="text-muted-foreground font-mono text-[9px] opacity-75">Assistant Reply</span>
                  </div>

                  <div class="text-foreground pr-1.5 leading-relaxed select-text font-medium">
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

<style scoped>
/* Scrollbar styles inside history view */
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
