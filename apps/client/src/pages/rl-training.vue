<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onUnmounted } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Progress } from "@/components/ui/progress";
import {
  PlayIcon,
  PauseIcon,
  SquareIcon,
  RocketIcon,
  GaugeIcon,
  ZapIcon,
  TrendingUpIcon,
} from "@lucide/vue";

// RL Agent 训练管理面板（仅桌面模式）。
// 数据特点：分三块 — 配置（超参 / 步骤 / 奖励权重）/ 控制（启动/暂停/终止）/ 指标（曲线 + 分布）。
// 用 grid 左右切分；指标用 sparkline-like 折线图占位（dev 环境用 div 高度模拟）。

const algorithm = ref("ppo");
const status = ref<"idle" | "running" | "paused" | "finished">("idle");

const hp = ref({
  learning_rate: 3e-4,
  gamma: 0.99,
  entropy_coeff: 0.01,
  clip_range: 0.2,
  net_layers: 2,
  net_width: 256,
});

const steps = ref({
  max_timesteps: 1_000_000,
  rollout: 2048,
  batch_size: 64,
  epochs: 10,
});

const rewards = ref({
  cs: 1.0,
  kill: 5.0,
  death: -3.0,
  distance_penalty: -0.01,
  turret: 10.0,
});

// 实时指标（模拟）
const trainedSteps = ref(0);
const seriesEpReturn = ref<number[]>([]);
const seriesLoss = ref<number[]>([]);
const seriesKL = ref<number[]>([]);
const seriesEntropy = ref<number[]>([]);
const policyDist = ref<{ action: string; prob: number }[]>([
  { action: "move", prob: 0.35 },
  { action: "attack", prob: 0.30 },
  { action: "Q", prob: 0.12 },
  { action: "W", prob: 0.08 },
  { action: "E", prob: 0.10 },
  { action: "recall", prob: 0.05 },
]);
const valueEstimate = ref<number>(0);
const rewardBreakdown = ref<{ name: string; value: number }[]>([
  { name: "cs", value: 0 },
  { name: "kill", value: 0 },
  { name: "death", value: 0 },
  { name: "distance", value: 0 },
  { name: "turret", value: 0 },
]);

let tickTimer: number | null = null;

function startTraining() {
  status.value = "running";
  trainedSteps.value = 0;
  seriesEpReturn.value = [];
  seriesLoss.value = [];
  seriesKL.value = [];
  seriesEntropy.value = [];
  tickTimer = window.setInterval(tick, 600);
}

function pauseTraining() {
  status.value = "paused";
  if (tickTimer) {
    clearInterval(tickTimer);
    tickTimer = null;
  }
}

function resumeTraining() {
  status.value = "running";
  tickTimer = window.setInterval(tick, 600);
}

function stopTraining() {
  status.value = "finished";
  if (tickTimer) {
    clearInterval(tickTimer);
    tickTimer = null;
  }
}

function tick() {
  trainedSteps.value = Math.min(steps.value.max_timesteps, trainedSteps.value + steps.value.rollout);
  // 模拟曲线
  const last = seriesEpReturn.value.at(-1) ?? 0;
  seriesEpReturn.value.push(last + (Math.random() - 0.4) * 3);
  seriesLoss.value.push(Math.max(0.01, 2 - trainedSteps.value / steps.value.max_timesteps * 1.5 + (Math.random() - 0.5) * 0.3));
  seriesKL.value.push(Math.abs((Math.random() - 0.5) * 0.05));
  seriesEntropy.value.push(0.7 - trainedSteps.value / steps.value.max_timesteps * 0.5 + (Math.random() - 0.5) * 0.05);
  valueEstimate.value = last;
  rewardBreakdown.value = rewardBreakdown.value.map((r) => ({
    ...r,
    value: r.value + (Math.random() - 0.45) * 2,
  }));
  // 上限 200 点
  if (seriesEpReturn.value.length > 200) {
    seriesEpReturn.value.shift();
    seriesLoss.value.shift();
    seriesKL.value.shift();
    seriesEntropy.value.shift();
  }
  if (trainedSteps.value >= steps.value.max_timesteps) stopTraining();
}

const progressPct = computed(() =>
  Math.round((trainedSteps.value / steps.value.max_timesteps) * 100)
);

function spark(series: number[]): string {
  if (series.length < 2) return "";
  const max = Math.max(...series);
  const min = Math.min(...series);
  const range = max - min || 1;
  const w = 200;
  const h = 50;
  const pts = series.map((v, i) => {
    const x = (i / (series.length - 1)) * w;
    const y = h - ((v - min) / range) * h;
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  });
  return pts.join(" ");
}

onUnmounted(() => {
  if (tickTimer) clearInterval(tickTimer);
});
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-7xl flex-col gap-6 px-8 py-6">
    <header class="flex items-center justify-between">
      <div class="space-y-1">
        <h1 class="flex items-center gap-2 text-2xl font-semibold tracking-tight">
          <RocketIcon class="size-5" />
          RL 训练面板
        </h1>
        <p class="text-muted-foreground text-sm">
          仅桌面端可用 · 训练完成后一键发布 .pth 权重应用于 Agent
        </p>
      </div>
      <Badge :variant="status === 'running' ? 'default' : 'outline'" class="gap-1.5">
        <span
          v-if="status === 'running'"
          class="bg-emerald-500 size-1.5 animate-pulse rounded-full"
        />
        {{
          status === "running"
            ? "训练中"
            : status === "paused"
              ? "已暂停"
              : status === "finished"
                ? "已完成"
                : "未启动"
        }}
      </Badge>
    </header>

    <div class="grid min-h-0 flex-1 grid-cols-1 gap-6 lg:grid-cols-[340px_1fr]">
      <!-- 左：配置 -->
      <aside class="space-y-6 overflow-y-auto pr-2">
        <!-- 算法 -->
        <section class="space-y-3">
          <h2 class="text-sm font-semibold">算法</h2>
          <Select v-model="algorithm">
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="ppo">PPO (Proximal Policy Optimization)</SelectItem>
              <SelectItem value="sac">SAC (Soft Actor-Critic)</SelectItem>
              <SelectItem value="dqn">DQN (Deep Q-Network)</SelectItem>
            </SelectContent>
          </Select>
        </section>

        <Separator />

        <!-- 超参 -->
        <section class="space-y-3">
          <h2 class="text-sm font-semibold">超参数</h2>
          <div class="grid grid-cols-2 gap-3">
            <div class="space-y-1">
              <Label class="text-xs">Learning Rate</Label>
              <Input v-model.number="hp.learning_rate" type="number" step="0.0001" class="h-8 font-mono text-xs" />
            </div>
            <div class="space-y-1">
              <Label class="text-xs">Gamma</Label>
              <Input v-model.number="hp.gamma" type="number" step="0.001" class="h-8 font-mono text-xs" />
            </div>
            <div class="space-y-1">
              <Label class="text-xs">Entropy Coef</Label>
              <Input v-model.number="hp.entropy_coeff" type="number" step="0.001" class="h-8 font-mono text-xs" />
            </div>
            <div class="space-y-1">
              <Label class="text-xs">Clip Range</Label>
              <Input v-model.number="hp.clip_range" type="number" step="0.01" class="h-8 font-mono text-xs" />
            </div>
            <div class="space-y-1">
              <Label class="text-xs">网络层数</Label>
              <Input v-model.number="hp.net_layers" type="number" class="h-8 font-mono text-xs" />
            </div>
            <div class="space-y-1">
              <Label class="text-xs">隐层维度</Label>
              <Input v-model.number="hp.net_width" type="number" class="h-8 font-mono text-xs" />
            </div>
          </div>
        </section>

        <Separator />

        <!-- 步骤 -->
        <section class="space-y-3">
          <h2 class="text-sm font-semibold">训练步骤</h2>
          <div class="grid grid-cols-2 gap-3">
            <div class="space-y-1">
              <Label class="text-xs">Max Timesteps</Label>
              <Input v-model.number="steps.max_timesteps" type="number" class="h-8 font-mono text-xs" />
            </div>
            <div class="space-y-1">
              <Label class="text-xs">Rollout Steps</Label>
              <Input v-model.number="steps.rollout" type="number" class="h-8 font-mono text-xs" />
            </div>
            <div class="space-y-1">
              <Label class="text-xs">Batch Size</Label>
              <Input v-model.number="steps.batch_size" type="number" class="h-8 font-mono text-xs" />
            </div>
            <div class="space-y-1">
              <Label class="text-xs">Epochs</Label>
              <Input v-model.number="steps.epochs" type="number" class="h-8 font-mono text-xs" />
            </div>
          </div>
        </section>

        <Separator />

        <!-- Reward Shaper -->
        <section class="space-y-3">
          <h2 class="text-sm font-semibold">Reward Shaper</h2>
          <div class="space-y-2">
            <div v-for="(_, k) in rewards" :key="k" class="flex items-center gap-2">
              <Label class="w-24 text-xs">{{ k }}</Label>
              <Input v-model.number="rewards[k]" type="number" step="0.1" class="h-8 flex-1 font-mono text-xs" />
            </div>
          </div>
        </section>

        <Separator />

        <!-- 控制 -->
        <section class="space-y-2">
          <div class="flex gap-2">
            <Button v-if="status === 'idle'" class="flex-1" @click="startTraining">
              <PlayIcon class="size-4" />
              启动训练
            </Button>
            <Button v-else-if="status === 'running'" variant="outline" class="flex-1" @click="pauseTraining">
              <PauseIcon class="size-4" />
              暂停
            </Button>
            <Button v-else-if="status === 'paused'" class="flex-1" @click="resumeTraining">
              <PlayIcon class="size-4" />
              恢复
            </Button>
            <Button
              v-if="status === 'running' || status === 'paused'"
              variant="ghost"
              class="text-destructive"
              @click="stopTraining"
            >
              <SquareIcon class="size-4" />
              终止
            </Button>
          </div>
          <Button v-if="status === 'finished'" class="w-full" variant="outline">
            <ZapIcon class="size-4" />
            发布权重为 Agent
          </Button>
        </section>
      </aside>

      <!-- 右：指标 -->
      <main class="min-w-0 space-y-6 overflow-y-auto">
        <!-- 进度 -->
        <section class="space-y-2">
          <div class="flex items-center justify-between text-xs">
            <span class="text-muted-foreground">训练进度</span>
            <span class="font-mono">{{ trainedSteps.toLocaleString() }} / {{ steps.max_timesteps.toLocaleString() }} ({{ progressPct }}%)</span>
          </div>
          <Progress :model-value="progressPct" />
        </section>

        <!-- 曲线 4 联 -->
        <section class="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div v-for="m in [
            { key: 'epreturn', label: 'Episodic Return', series: seriesEpReturn },
            { key: 'loss', label: 'Loss', series: seriesLoss },
            { key: 'kl', label: 'KL Divergence', series: seriesKL },
            { key: 'entropy', label: 'Entropy', series: seriesEntropy },
          ]" :key="m.key" class="space-y-2 rounded-lg border p-4">
            <div class="flex items-center justify-between text-xs">
              <span class="text-muted-foreground flex items-center gap-1">
                <TrendingUpIcon class="size-3" />
                {{ m.label }}
              </span>
              <span class="font-mono tabular-nums">{{ m.series.at(-1)?.toFixed(3) ?? "—" }}</span>
            </div>
            <svg viewBox="0 0 200 50" preserveAspectRatio="none" class="h-12 w-full">
              <polyline
                v-if="m.series.length > 1"
                :points="spark(m.series)"
                fill="none"
                stroke="currentColor"
                stroke-width="1.2"
                vector-effect="non-scaling-stroke"
                class="text-foreground"
              />
            </svg>
          </div>
        </section>

        <!-- 策略分布 + Value -->
        <section class="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <div class="space-y-3 rounded-lg border p-4">
            <div class="flex items-center justify-between text-xs">
              <span class="text-muted-foreground flex items-center gap-1">
                <GaugeIcon class="size-3" />
                策略分布
              </span>
            </div>
            <div class="space-y-1.5">
              <div v-for="a in policyDist" :key="a.action" class="space-y-1">
                <div class="flex items-center justify-between text-xs">
                  <span class="font-mono">{{ a.action }}</span>
                  <span class="text-muted-foreground tabular-nums">{{ (a.prob * 100).toFixed(1) }}%</span>
                </div>
                <Progress :model-value="a.prob * 100" class="h-1" />
              </div>
            </div>
          </div>

          <div class="space-y-3 rounded-lg border p-4">
            <div class="text-muted-foreground text-xs">Reward 分项（最近 episode）</div>
            <ul class="space-y-1.5">
              <li v-for="r in rewardBreakdown" :key="r.name" class="flex items-center justify-between text-xs">
                <span class="font-mono">{{ r.name }}</span>
                <span class="tabular-nums" :class="r.value >= 0 ? 'text-emerald-600' : 'text-destructive'">
                  {{ r.value >= 0 ? "+" : "" }}{{ r.value.toFixed(2) }}
                </span>
              </li>
            </ul>
            <Separator />
            <div class="flex items-center justify-between text-xs">
              <span class="text-muted-foreground">Value 估计</span>
              <span class="font-mono tabular-nums">{{ valueEstimate.toFixed(3) }}</span>
            </div>
          </div>
        </section>
      </main>
    </div>
  </div>
</template>
