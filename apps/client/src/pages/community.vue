<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { useRouter } from "vue-router";
import { agentsApi, type Agent } from "@/services/cloudApi";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { GitForkIcon, SearchIcon, GlobeIcon, FlameIcon, ClockIcon, MedalIcon } from "@lucide/vue";

// Agent 社区市场。
// 数据特点：浏览 + 二次操作（fork）。卡片网格突出英雄/作者/排名。
// 排序 tab 在顶部；点击卡片打开 fork 对话框（fork 是核心 CTA）。

const router = useRouter();
const sort = ref<"recent" | "popular" | "elo">("recent");
const search = ref("");
const list = ref<Agent[]>([]);
const loading = ref(true);

const forkTarget = ref<Agent | null>(null);
const forkName = ref("");
const forking = ref(false);

async function refresh() {
  loading.value = true;
  try {
    list.value = await agentsApi.browseCommunity(sort.value, 60);
  } catch (e) {
    list.value = [];
  } finally {
    loading.value = false;
  }
}

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return list.value;
  return list.value.filter(
    (a) => a.name.toLowerCase().includes(q) || a.champion.toLowerCase().includes(q)
  );
});

function openFork(a: Agent) {
  forkTarget.value = a;
  forkName.value = `${a.name} · 副本`;
}

async function confirmFork() {
  if (!forkTarget.value) return;
  forking.value = true;
  try {
    const created = await agentsApi.fork(forkTarget.value.id, forkName.value);
    forkTarget.value = null;
    router.push(`/agents?focus=${created.id}`);
  } catch (e: any) {
    alert(e.message || "Fork 失败");
  } finally {
    forking.value = false;
  }
}

watch(sort, refresh);
onMounted(refresh);
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-6xl flex-col gap-6 px-8 py-8">
    <header class="space-y-1">
      <h1 class="flex items-center gap-2 text-2xl font-semibold tracking-tight">
        <GlobeIcon class="size-6" />
        Agent 社区
      </h1>
      <p class="text-muted-foreground text-sm">浏览其他电竞经理公开的 Agent，Fork 到本地继续训练或参赛。</p>
    </header>

    <div class="flex flex-wrap items-center justify-between gap-3">
      <Tabs v-model="sort">
        <TabsList>
          <TabsTrigger value="recent">
            <ClockIcon class="size-3.5" />
            最新
          </TabsTrigger>
          <TabsTrigger value="popular">
            <FlameIcon class="size-3.5" />
            热门
          </TabsTrigger>
          <TabsTrigger value="elo">
            <MedalIcon class="size-3.5" />
            ELO 高
          </TabsTrigger>
        </TabsList>
      </Tabs>

      <div class="relative w-64">
        <SearchIcon class="text-muted-foreground absolute top-1/2 left-3 size-3.5 -translate-y-1/2" />
        <Input v-model="search" placeholder="搜索 Agent / 英雄" class="pl-9" />
      </div>
    </div>

    <Separator />

    <div v-if="loading" class="text-muted-foreground py-16 text-center text-sm">加载中…</div>
    <div v-else-if="filtered.length === 0" class="text-muted-foreground py-16 text-center text-sm">
      暂无公开 Agent
    </div>
    <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
      <article
        v-for="a in filtered"
        :key="a.id"
        class="flex flex-col gap-4 rounded-lg border p-5 transition-colors hover:bg-muted/40"
      >
        <header class="flex items-start justify-between gap-2">
          <div class="min-w-0 space-y-0.5">
            <h3 class="truncate text-sm font-semibold">{{ a.name }}</h3>
            <p class="text-muted-foreground text-xs">{{ a.champion }} · 经理 #{{ a.owner_id }}</p>
          </div>
          <Badge v-if="a.forked_from" variant="outline" class="shrink-0 text-[10px]">
            <GitForkIcon class="size-3" />
            Fork
          </Badge>
        </header>

        <div class="text-muted-foreground text-xs">
          创建于 {{ new Date(a.created_at).toLocaleDateString() }}
        </div>

        <Button variant="outline" size="sm" class="w-full" @click="openFork(a)">
          <GitForkIcon class="size-3.5" />
          Fork 到我的 Agent
        </Button>
      </article>
    </div>

    <!-- Fork 对话框 -->
    <Dialog :open="!!forkTarget" @update:open="(v) => !v && (forkTarget = null)">
      <DialogContent class="max-w-sm">
        <DialogHeader>
          <DialogTitle>Fork Agent</DialogTitle>
          <DialogDescription>
            将创建一个属于你的副本，可继续编辑配置或上游同步。
          </DialogDescription>
        </DialogHeader>
        <div class="space-y-2 py-2">
          <Input v-model="forkName" placeholder="新 Agent 名称" />
        </div>
        <DialogFooter>
          <Button variant="ghost" @click="forkTarget = null">取消</Button>
          <Button :disabled="forking" @click="confirmFork">{{ forking ? "处理中…" : "确认 Fork" }}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
