<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, computed, onUnmounted } from "vue";
import { storeToRefs } from "pinia";
import { useGameStore } from "@/stores/gameStore";
import { useRlTelemetry, type Checkpoint } from "@/composables/useRlTelemetry";
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
  PlugIcon,
  SaveIcon,
  CheckIcon,
} from "@lucide/vue";

// RL Agent 训练管理面板（仅桌面模式）。
// 配置（超参/步骤/奖励权重）在本页维护；指标与控制经 useRlTelemetry 对接训练守护进程 WS，
// 未配置守护进程地址时回落到本地模拟器，保证面板可离线演示。

const store = useGameStore();
const { heroPresets } = storeToRefs(store);

const tm = useRlTelemetry();
const {
  connected,
  usingSimulator,
  lastError,
  status,
  trainedSteps,
  seriesEpReturn,
  seriesLoss,
  seriesKL,
  seriesEntropy,
  valueEstimate,
  policyDist,
  rewardBreakdown,
  checkpoints,
} = tm;

const algorithm = ref("ppo");
const endpointInput = ref("");

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

const rewards = ref<Record<string, number>>({
  cs: 1.0,
  kill: 5.0,
  death: -3.0,
  distance_penalty: -0.01,
  turret: 10.0,
});

// 应用 checkpoint 的目标 Agent（仅 RL 类型选手）。
const rlAgents = computed(() => heroPresets.value.filter((p) => p.agent_type === "rl"));
const applyTarget = ref<string>("");
const applyMsg = ref("");

function connect() {
  tm.connect(endpointInput.value);
}

function buildConfig() {
  return {
    algorithm: algorithm.value,
    hyperparams: hp.value,
    steps: steps.value,
    reward_shaper: rewards.value,
  };
}

function startTraining() {
  tm.maxTimesteps.value = steps.value.max_timesteps;
  tm.start(buildConfig());
}

async function applyCheckpoint(ckpt: Checkpoint) {
  applyMsg.value = "";
  const preset = heroPresets.value.find((p) => p.name === applyTarget.value);
  if (!preset) {
    applyMsg.value = "请先选择要应用的 RL 选手";
    return;
  }
  try {
    const config_json = { ...(preset.config_json || {}), model_path: ckpt.path };
    await store.saveHeroPreset({ ...preset, config_json });
    tm.notifyApply(ckpt.id);
    applyMsg.value = `已将 ${ckpt.path} 应用为「${preset.name}」的策略权重`;
  } catch (e: any) {
    applyMsg.value = e?.message || "应用失败";
  }
}

const progressPct = computed(() =>
  Math.round((trainedSteps.value / steps.value.max_timesteps) * 100),
);

function spark(series: number[]): string {
  if (series.length < 2) return "";
  const max = Math.max(...series);
  const min = Math.min(...series);
  const range = max - min || 1;
  const w = 200;
  const h = 50;
  return series
    .map((v, i) => {
      const x = (i / (series.length - 1)) * w;
      const y = h - ((v - min) / range) * h;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
}

function shortTime(iso: string): string {
  return new Date(iso).toLocaleTimeString();
}

onUnmounted(() => tm.dispose());
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
          对接训练守护进程 · 训练完成后保存 .pth 并一键应用为 Agent 策略
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

    <!-- 守护进程连接 -->
    <section class="flex flex-wrap items-center gap-2 rounded-lg border p-3">
      <PlugIcon class="text-muted-foreground size-4" />
      <Input
        v-model="endpointInput"
        placeholder="训练守护进程 WS 地址，如 ws://127.0.0.1:8765"
        class="h-8 max-w-md font-mono text-xs"
        data-testid="rl-endpoint-input"
      />
      <Button v-if="!connected" size="sm" class="h-8" @click="connect" data-testid="rl-connect-btn">
        连接
      </Button>
      <Button v-else size="sm" variant="outline" class="h-8" @click="tm.disconnect()">
        断开
      </Button>
      <Badge v-if="connected" variant="default" class="gap-1 text-[10px]">
        <CheckIcon class="size-3" /> 已连接守护进程
      </Badge>
      <Badge v-else-if="usingSimulator" variant="secondary" class="text-[10px]">本地模拟</Badge>
      <span v-if="lastError" class="text-destructive text-xs">{{ lastError }}</span>
    </section>

    <div class="grid min-h-0 flex-1 grid-cols-1 gap-6 lg:grid-cols-[340px_1fr]">
      <!-- 左：配置 -->
      <aside class="space-y-6 overflow-y-auto pr-2">
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

        <section class="space-y-2">
          <div class="flex gap-2">
            <Button v-if="status === 'idle' || status === 'finished'" class="flex-1" @click="startTraining" data-testid="rl-start-btn">
              <PlayIcon class="size-4" />
              启动训练
            </Button>
            <Button v-else-if="status === 'running'" variant="outline" class="flex-1" @click="tm.pause()">
              <PauseIcon class="size-4" />
              暂停
            </Button>
            <Button v-else-if="status === 'paused'" class="flex-1" @click="tm.resume()">
              <PlayIcon class="size-4" />
              恢复
            </Button>
            <Button
              v-if="status === 'running' || status === 'paused'"
              variant="ghost"
              class="text-destructive"
              @click="tm.stop()"
            >
              <SquareIcon class="size-4" />
              终止
            </Button>
          </div>
          <Button
            variant="outline"
            class="w-full"
            :disabled="status === 'idle'"
            @click="tm.saveCheckpoint()"
            data-testid="rl-save-checkpoint-btn"
          >
            <SaveIcon class="size-4" />
            保存 Checkpoint
          </Button>
        </section>
      </aside>

      <!-- 右：指标 -->
      <main class="min-w-0 space-y-6 overflow-y-auto">
        <section class="space-y-2">
          <div class="flex items-center justify-between text-xs">
            <span class="text-muted-foreground">训练进度</span>
            <span class="font-mono">{{ trainedSteps.toLocaleString() }} / {{ steps.max_timesteps.toLocaleString() }} ({{ progressPct }}%)</span>
          </div>
          <Progress :model-value="progressPct" />
        </section>

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

        <section class="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <div class="space-y-3 rounded-lg border p-4">
            <div class="flex items-center justify-between text-xs">
              <span class="text-muted-foreground flex items-center gap-1">
                <GaugeIcon class="size-3" />
                策略分布（动作概率）
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
            <div class="text-muted-foreground text-xs">Reward 分项（最近 step）</div>
            <ul v-if="rewardBreakdown.length" class="space-y-1.5">
              <li v-for="r in rewardBreakdown" :key="r.name" class="flex items-center justify-between text-xs">
                <span class="font-mono">{{ r.name }}</span>
                <span class="tabular-nums" :class="r.value >= 0 ? 'text-emerald-600' : 'text-destructive'">
                  {{ r.value >= 0 ? "+" : "" }}{{ r.value.toFixed(2) }}
                </span>
              </li>
            </ul>
            <p v-else class="text-muted-foreground text-xs">等待守护进程上报…</p>
            <Separator />
            <div class="flex items-center justify-between text-xs">
              <span class="text-muted-foreground">Value 估计</span>
              <span class="font-mono tabular-nums">{{ valueEstimate.toFixed(3) }}</span>
            </div>
          </div>
        </section>

        <!-- Checkpoints 与权重切换 -->
        <section class="space-y-3 rounded-lg border p-4">
          <div class="flex items-center justify-between">
            <span class="text-muted-foreground flex items-center gap-1 text-xs">
              <ZapIcon class="size-3" />
              模型快照（Checkpoints）
            </span>
            <div class="flex items-center gap-2">
              <Select v-model="applyTarget">
                <SelectTrigger class="h-8 w-48 text-xs" data-testid="rl-apply-target">
                  <SelectValue placeholder="应用到 RL 选手…" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="p in rlAgents" :key="p.name" :value="p.name">{{ p.name }}</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <p v-if="rlAgents.length === 0" class="text-muted-foreground text-xs">
            暂无 RL 类型选手，请先在「我的选手」中创建 RL 选手。
          </p>

          <ul v-if="checkpoints.length" class="divide-border divide-y rounded-md border" data-testid="rl-checkpoint-list">
            <li v-for="c in checkpoints" :key="c.id" class="flex items-center justify-between px-3 py-2">
              <div class="min-w-0 text-xs">
                <div class="font-mono truncate">{{ c.path }}</div>
                <div class="text-muted-foreground">
                  step {{ c.step.toLocaleString() }} · return {{ c.ep_return.toFixed(2) }} · {{ shortTime(c.created_at) }}
                </div>
              </div>
              <Button
                size="sm"
                variant="outline"
                class="h-7 shrink-0"
                :disabled="!applyTarget"
                @click="applyCheckpoint(c)"
                data-testid="rl-apply-checkpoint-btn"
              >
                一键应用
              </Button>
            </li>
          </ul>
          <p v-else class="text-muted-foreground text-xs">尚无 checkpoint，训练中可点击「保存 Checkpoint」。</p>
          <p v-if="applyMsg" class="text-foreground text-xs">{{ applyMsg }}</p>
        </section>
      </main>
    </div>
  </div>
</template>
