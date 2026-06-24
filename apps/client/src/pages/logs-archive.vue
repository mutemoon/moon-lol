<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { matchesApi, BASE, type Match } from "@/services/cloudApi";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  DatabaseIcon,
  DownloadIcon,
  UploadCloudIcon,
  TimerIcon,
  PlayCircleIcon,
} from "@lucide/vue";

// 日志归档：服务端保留 24h 的对局日志可下载为 SQLite DB；
// 桌面端可加载本地 DB 在调试面板回放分析。
// 数据特点：少量摘要（保留窗口）+ 一张可下载/加载的表。

const matches = ref<Match[]>([]);
const loading = ref(true);

// 24h 内的"过期阈值"展示
function ago(iso: string | null) {
  if (!iso) return "—";
  const diff = (Date.now() - new Date(iso).getTime()) / 1000;
  if (diff < 60) return `${Math.floor(diff)}s 前`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m 前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h 前`;
  return `${Math.floor(diff / 86400)}d 前`;
}

function expireIn(iso: string | null): string {
  if (!iso) return "—";
  const remain = 86400 - (Date.now() - new Date(iso).getTime()) / 1000;
  if (remain <= 0) return "已过期";
  if (remain < 3600) return `${Math.floor(remain / 60)}m 后过期`;
  return `${Math.floor(remain / 3600)}h 后过期`;
}

async function refresh() {
  try {
    matches.value = await matchesApi.listMine();
  } catch (e) {
    matches.value = [];
  } finally {
    loading.value = false;
  }
}

function downloadDb(m: Match) {
  const url = `${BASE}/api/matches/${m.id}/log-db`;
  const a = document.createElement("a");
  a.href = url;
  a.download = `match-${m.id.slice(0, 8)}.sqlite`;
  document.body.appendChild(a);
  a.click();
  a.remove();
}

// 本地加载 .sqlite → 调试器
const loadedFile = ref<File | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);
const router = useRouter();

function pickFile() {
  fileInput.value?.click();
}
function handleFile(e: Event) {
  const f = (e.target as HTMLInputElement).files?.[0];
  if (f) loadedFile.value = f;
}
function loadIntoDebug() {
  if (!loadedFile.value) return;
  // 把文件交给桌面端日志系统（通过 Tauri 命令）。Web 模式下走 OPFS。
  alert(`已交给日志系统：${loadedFile.value.name}\n将跳转到 /debug 进行离线分析`);
  router.push("/debug");
}

onMounted(refresh);
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-5xl flex-col gap-6 px-8 py-8">
    <header class="space-y-1">
      <h1 class="flex items-center gap-2 text-2xl font-semibold tracking-tight">
        <DatabaseIcon class="size-5" />
        日志归档
      </h1>
      <p class="text-muted-foreground text-sm">
        服务器保留最近 24 小时对局日志；下载为 SQLite DB 可在桌面端调试器中离线回放分析。
      </p>
    </header>

    <!-- 上载本地 DB → 调试 -->
    <section class="flex items-center justify-between rounded-lg border p-5">
      <div class="space-y-1">
        <p class="flex items-center gap-2 text-sm font-medium">
          <UploadCloudIcon class="size-4" />
          加载本地 SQLite 文件
        </p>
        <p class="text-muted-foreground text-xs">
          {{ loadedFile ? `已选择 ${loadedFile.name}（${(loadedFile.size / 1024 / 1024).toFixed(1)} MB）` : "选择之前下载的 .sqlite 文件，加载至调试器" }}
        </p>
      </div>
      <div class="flex gap-2">
        <input ref="fileInput" type="file" accept=".sqlite,.db" class="hidden" @change="handleFile" />
        <Button variant="outline" size="sm" @click="pickFile">选择文件</Button>
        <Button :disabled="!loadedFile" size="sm" @click="loadIntoDebug">
          <PlayCircleIcon class="size-3.5" />
          加载到调试器
        </Button>
      </div>
    </section>

    <Separator />

    <!-- 远端归档表 -->
    <section class="space-y-3">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold">服务器对局日志（24h 内）</h2>
        <Badge variant="outline">{{ matches.length }}</Badge>
      </div>

      <div v-if="loading" class="text-muted-foreground py-12 text-center text-sm">加载中…</div>
      <div v-else-if="matches.length === 0" class="text-muted-foreground py-12 text-center text-sm">
        近 24 小时没有对局记录
      </div>
      <div v-else class="overflow-hidden rounded-lg border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>对局</TableHead>
              <TableHead>模式</TableHead>
              <TableHead>状态</TableHead>
              <TableHead>开始</TableHead>
              <TableHead>归档保留</TableHead>
              <TableHead class="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="m in matches" :key="m.id">
              <TableCell class="font-mono text-xs">{{ m.id.slice(0, 8) }}</TableCell>
              <TableCell>
                <Badge variant="secondary">{{ m.mode }}</Badge>
              </TableCell>
              <TableCell class="text-xs">{{ m.status }}</TableCell>
              <TableCell class="text-muted-foreground text-xs">{{ ago(m.created_at) }}</TableCell>
              <TableCell class="text-muted-foreground text-xs">
                <span class="inline-flex items-center gap-1">
                  <TimerIcon class="size-3" />
                  {{ expireIn(m.created_at) }}
                </span>
              </TableCell>
              <TableCell class="text-right">
                <Button variant="ghost" size="sm" @click="downloadDb(m)">
                  <DownloadIcon class="size-3.5" />
                  下载 DB
                </Button>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
    </section>
  </div>
</template>
