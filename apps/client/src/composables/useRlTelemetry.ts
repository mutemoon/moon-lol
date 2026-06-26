import { ref, shallowRef } from "vue";

// RL 训练遥测：对接外部训练守护进程（Python）的 WebSocket，消费 reward/loss/KL 等指标，
// 并下发控制指令（启动/暂停/终止 / 保存 checkpoint / 应用 checkpoint）。
//
// 守护进程未配置时回落到本地模拟器，保证面板可离线演示。
//
// 守护进程 → 客户端 帧（JSON）：
//   { type: "status", status: "idle|running|paused|finished" }
//   { type: "metrics", step, ep_return, loss, kl, entropy, value,
//     policy: [{action, prob}], reward_breakdown: [{name, value}] }
//   { type: "checkpoint", checkpoint: { id, step, path, ep_return, created_at } }
// 客户端 → 守护进程 帧：
//   { type: "control", command: "start|pause|resume|stop", config }
//   { type: "save_checkpoint" }
//   { type: "apply_checkpoint", id }

export type TrainStatus = "idle" | "running" | "paused" | "finished";

export interface PolicyAction {
  action: string;
  prob: number;
}

export interface RewardItem {
  name: string;
  value: number;
}

export interface Checkpoint {
  id: string;
  step: number;
  path: string;
  ep_return: number;
  created_at: string;
}

const MAX_POINTS = 200;

export function useRlTelemetry() {
  const endpoint = ref("");
  const connected = ref(false);
  const usingSimulator = ref(false);
  const lastError = ref("");
  const status = ref<TrainStatus>("idle");

  const trainedSteps = ref(0);
  const maxTimesteps = ref(1_000_000);
  const seriesEpReturn = ref<number[]>([]);
  const seriesLoss = ref<number[]>([]);
  const seriesKL = ref<number[]>([]);
  const seriesEntropy = ref<number[]>([]);
  const valueEstimate = ref(0);
  const policyDist = ref<PolicyAction[]>([
    { action: "move", prob: 0.35 },
    { action: "attack", prob: 0.3 },
    { action: "Q", prob: 0.12 },
    { action: "W", prob: 0.08 },
    { action: "E", prob: 0.1 },
    { action: "recall", prob: 0.05 },
  ]);
  const rewardBreakdown = ref<RewardItem[]>([]);
  const checkpoints = ref<Checkpoint[]>([]);

  const ws = shallowRef<WebSocket | null>(null);
  let simTimer: number | null = null;

  function resetSeries() {
    trainedSteps.value = 0;
    seriesEpReturn.value = [];
    seriesLoss.value = [];
    seriesKL.value = [];
    seriesEntropy.value = [];
  }

  function pushPoint(series: number[], v: number) {
    series.push(v);
    if (series.length > MAX_POINTS) series.shift();
  }

  function applyMetrics(m: any) {
    if (typeof m.step === "number") trainedSteps.value = m.step;
    if (typeof m.ep_return === "number") pushPoint(seriesEpReturn.value, m.ep_return);
    if (typeof m.loss === "number") pushPoint(seriesLoss.value, m.loss);
    if (typeof m.kl === "number") pushPoint(seriesKL.value, m.kl);
    if (typeof m.entropy === "number") pushPoint(seriesEntropy.value, m.entropy);
    if (typeof m.value === "number") valueEstimate.value = m.value;
    if (Array.isArray(m.policy)) policyDist.value = m.policy;
    if (Array.isArray(m.reward_breakdown)) rewardBreakdown.value = m.reward_breakdown;
  }

  function handleFrame(data: string) {
    let frame: any;
    try {
      frame = JSON.parse(data);
    } catch {
      return;
    }
    switch (frame.type) {
      case "status":
        if (frame.status) status.value = frame.status;
        break;
      case "metrics":
        applyMetrics(frame);
        break;
      case "checkpoint":
        if (frame.checkpoint) {
          checkpoints.value = [frame.checkpoint, ...checkpoints.value];
        }
        break;
    }
  }

  /** 连接训练守护进程 WS。url 为空则不连接。 */
  function connect(url: string) {
    lastError.value = "";
    endpoint.value = url.trim();
    if (!endpoint.value) {
      lastError.value = "请填写训练守护进程 WS 地址";
      return;
    }
    disconnect();
    try {
      const sock = new WebSocket(endpoint.value);
      ws.value = sock;
      sock.onopen = () => {
        connected.value = true;
        usingSimulator.value = false;
      };
      sock.onmessage = (ev) => handleFrame(typeof ev.data === "string" ? ev.data : "");
      sock.onerror = () => {
        lastError.value = "连接训练守护进程失败";
      };
      sock.onclose = () => {
        connected.value = false;
        if (ws.value === sock) ws.value = null;
      };
    } catch (e: any) {
      lastError.value = e?.message || "无法建立 WS 连接";
    }
  }

  function disconnect() {
    if (ws.value) {
      ws.value.onclose = null;
      ws.value.close();
      ws.value = null;
    }
    connected.value = false;
  }

  function send(payload: Record<string, unknown>): boolean {
    if (ws.value && ws.value.readyState === WebSocket.OPEN) {
      ws.value.send(JSON.stringify(payload));
      return true;
    }
    return false;
  }

  // ── 本地模拟器（无守护进程时） ──
  function simTick() {
    trainedSteps.value = Math.min(maxTimesteps.value, trainedSteps.value + 2048);
    const last = seriesEpReturn.value.at(-1) ?? 0;
    pushPoint(seriesEpReturn.value, last + (Math.random() - 0.4) * 3);
    pushPoint(
      seriesLoss.value,
      Math.max(0.01, 2 - (trainedSteps.value / maxTimesteps.value) * 1.5 + (Math.random() - 0.5) * 0.3),
    );
    pushPoint(seriesKL.value, Math.abs((Math.random() - 0.5) * 0.05));
    pushPoint(
      seriesEntropy.value,
      0.7 - (trainedSteps.value / maxTimesteps.value) * 0.5 + (Math.random() - 0.5) * 0.05,
    );
    valueEstimate.value = last;
    // 模拟策略分布抖动并归一化
    const raw = policyDist.value.map((a) => ({
      action: a.action,
      prob: Math.max(0.01, a.prob + (Math.random() - 0.5) * 0.04),
    }));
    const sum = raw.reduce((s, a) => s + a.prob, 0);
    policyDist.value = raw.map((a) => ({ action: a.action, prob: a.prob / sum }));
    if (trainedSteps.value >= maxTimesteps.value) stop();
  }

  function startSim() {
    usingSimulator.value = true;
    if (simTimer) clearInterval(simTimer);
    simTimer = window.setInterval(simTick, 600);
  }

  function stopSim() {
    if (simTimer) {
      clearInterval(simTimer);
      simTimer = null;
    }
  }

  // ── 训练控制 ──
  function start(config: Record<string, unknown>) {
    resetSeries();
    status.value = "running";
    if (!send({ type: "control", command: "start", config })) {
      startSim();
    }
  }

  function pause() {
    status.value = "paused";
    if (!send({ type: "control", command: "pause" })) stopSim();
  }

  function resume() {
    status.value = "running";
    if (!send({ type: "control", command: "resume" })) startSim();
  }

  function stop() {
    status.value = "finished";
    stopSim();
    send({ type: "control", command: "stop" });
  }

  /** 保存当前训练状态为 checkpoint。无守护进程时本地生成条目。 */
  function saveCheckpoint() {
    if (!send({ type: "save_checkpoint" })) {
      const ckpt: Checkpoint = {
        id: `local-${Date.now()}`,
        step: trainedSteps.value,
        path: `checkpoints/ckpt_${trainedSteps.value}.pth`,
        ep_return: seriesEpReturn.value.at(-1) ?? 0,
        created_at: new Date().toISOString(),
      };
      checkpoints.value = [ckpt, ...checkpoints.value];
    }
  }

  /** 通知守护进程应用某 checkpoint（前端写回 Agent 策略由调用方负责）。 */
  function notifyApply(id: string) {
    send({ type: "apply_checkpoint", id });
  }

  function dispose() {
    stopSim();
    disconnect();
  }

  return {
    // state
    endpoint,
    connected,
    usingSimulator,
    lastError,
    status,
    trainedSteps,
    maxTimesteps,
    seriesEpReturn,
    seriesLoss,
    seriesKL,
    seriesEntropy,
    valueEstimate,
    policyDist,
    rewardBreakdown,
    checkpoints,
    // actions
    connect,
    disconnect,
    start,
    pause,
    resume,
    stop,
    saveCheckpoint,
    notifyApply,
    dispose,
  };
}
