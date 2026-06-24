<route lang="yaml">
meta:
  layout: dashboard
</route>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import {
  essenceApi,
  subscriptionsApi,
  type BillingPlan,
  type EssenceTransaction,
} from "@/services/cloudApi";
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
import { GemIcon, CalendarCheckIcon, CheckIcon, SparklesIcon } from "@lucide/vue";

// 精粹钱包 + 订阅计费。
// 数据特点：(1) 余额数字（巨大字号突出）(2) 套餐对比表 (3) 流水时间线
// 用三段式自上而下：突出 hero stat → 套餐卡 → 明细。

const balance = ref<number>(0);
const transactions = ref<EssenceTransaction[]>([]);
const plans = ref<BillingPlan[]>([]);
const currentPlan = ref<BillingPlan | null>(null);
const loading = ref(true);

const checkInState = ref<{ already: boolean; granted: number } | null>(null);
const subscribing = ref<string | null>(null);

async function refresh() {
  try {
    const [b, tx, p, cur] = await Promise.all([
      essenceApi.balance().catch(() => 0),
      essenceApi.transactions(50, 0).catch(() => [] as EssenceTransaction[]),
      subscriptionsApi.listPlans().catch(() => [] as BillingPlan[]),
      subscriptionsApi.current().catch(() => null),
    ]);
    balance.value = b;
    transactions.value = tx;
    plans.value = p;
    currentPlan.value = cur;
  } finally {
    loading.value = false;
  }
}

async function handleCheckIn() {
  try {
    const res = await essenceApi.checkIn();
    checkInState.value = { already: res.already_checked_in, granted: res.granted };
    balance.value = res.balance;
  } catch (e) {
    console.error(e);
  }
}

async function handleSubscribe(plan: BillingPlan) {
  subscribing.value = plan.id;
  try {
    await subscriptionsApi.subscribe(plan.id);
    await refresh();
  } catch (e: any) {
    alert(e.message || "订阅失败");
  } finally {
    subscribing.value = null;
  }
}

function txKindLabel(reason: string): string {
  return (
    {
      check_in: "每日签到",
      llm_token: "模型 Token 消耗",
      slot_purchase: "购买 Agent 槽位",
      recharge: "充值",
      subscription: "订阅",
    }[reason] || reason
  );
}

function fmtDate(iso: string) {
  return new Date(iso).toLocaleString();
}

function fmtPrice(cents: number) {
  if (cents === 0) return "免费";
  return `¥${(cents / 100).toFixed(0)} / 月`;
}

onMounted(refresh);
</script>

<template>
  <div class="mx-auto flex h-full w-full max-w-5xl flex-col gap-10 px-8 py-8">
    <!-- 余额 Hero -->
    <header class="flex items-center justify-between">
      <div class="space-y-2">
        <div class="text-muted-foreground flex items-center gap-1.5 text-xs">
          <GemIcon class="size-3.5" />
          精粹余额
        </div>
        <div class="flex items-baseline gap-2">
          <span class="text-5xl font-semibold tracking-tight tabular-nums">{{ balance.toLocaleString() }}</span>
          <span class="text-muted-foreground text-sm">BE</span>
        </div>
        <p class="text-muted-foreground text-xs">用于抵扣平台模型 Token 与购买 Agent 槽位</p>
      </div>

      <div class="space-y-2 text-right">
        <Button @click="handleCheckIn">
          <CalendarCheckIcon class="size-4" />
          每日签到
        </Button>
        <p v-if="checkInState" class="text-xs">
          <template v-if="checkInState.already"
            >今天已签到。</template
          >
          <template v-else>
            <span class="text-emerald-600">+{{ checkInState.granted }} BE 已发放</span>
          </template>
        </p>
      </div>
    </header>

    <Separator />

    <!-- 订阅套餐 -->
    <section class="space-y-4">
      <div class="space-y-1">
        <h2 class="text-sm font-semibold">订阅套餐</h2>
        <p class="text-muted-foreground text-xs">订阅可获得每月精粹补贴与更多 Agent 槽位</p>
      </div>

      <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
        <div
          v-for="p in plans"
          :key="p.id"
          class="space-y-4 rounded-lg border p-6"
          :class="currentPlan?.id === p.id ? 'ring-2 ring-foreground/10' : ''"
        >
          <div class="flex items-start justify-between">
            <div class="space-y-0.5">
              <p class="text-sm font-semibold">{{ p.name }}</p>
              <p class="text-muted-foreground text-xs">{{ fmtPrice(p.price_cents) }}</p>
            </div>
            <Badge v-if="currentPlan?.id === p.id" class="gap-1">
              <CheckIcon class="size-3" />
              当前
            </Badge>
          </div>

          <ul class="text-muted-foreground space-y-2 text-xs">
            <li class="flex items-center gap-2">
              <CheckIcon class="size-3" />
              {{ p.agent_limit }} 个 Agent 槽位
            </li>
            <li class="flex items-center gap-2">
              <CheckIcon class="size-3" />
              每月 {{ p.monthly_essence.toLocaleString() }} BE
            </li>
            <li class="flex items-center gap-2">
              <CheckIcon class="size-3" />
              不限对局局数
            </li>
          </ul>

          <Button
            v-if="currentPlan?.id !== p.id"
            class="w-full"
            variant="outline"
            :disabled="subscribing === p.id"
            @click="handleSubscribe(p)"
          >
            <SparklesIcon class="size-4" />
            {{ subscribing === p.id ? "处理中…" : "选择此套餐" }}
          </Button>
        </div>
      </div>
    </section>

    <Separator />

    <!-- 流水明细 -->
    <section class="space-y-3">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold">精粹流水</h2>
        <Badge variant="outline">{{ transactions.length }}</Badge>
      </div>

      <div v-if="transactions.length === 0" class="text-muted-foreground py-8 text-center text-xs">
        暂无流水
      </div>
      <div v-else class="overflow-hidden rounded-lg border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>类型</TableHead>
              <TableHead class="text-right">变动</TableHead>
              <TableHead class="text-right">时间</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="t in transactions" :key="t.id">
              <TableCell class="text-sm">{{ txKindLabel(t.reason) }}</TableCell>
              <TableCell class="text-right font-mono tabular-nums">
                <span :class="t.amount >= 0 ? 'text-emerald-600' : 'text-destructive'">
                  {{ t.amount >= 0 ? "+" : "" }}{{ t.amount }}
                </span>
              </TableCell>
              <TableCell class="text-muted-foreground text-right text-xs">{{ fmtDate(t.created_at) }}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
    </section>
  </div>
</template>
