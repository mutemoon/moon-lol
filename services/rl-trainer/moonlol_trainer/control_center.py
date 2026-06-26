"""训练控制中心：线程化训练循环 + 遥测广播 + checkpoint。

对外提供 start / pause / resume / stop / save_checkpoint / status，由 REST 与 WS 两个
前端共享。训练循环消费 `Env`（默认 `SimulatedEnv`），把 reward 汇成 episodic return，
并产出 PPO 风格的 loss / kl / entropy / value 与策略动作概率分布，经回调广播为遥测帧。
"""

from __future__ import annotations

import random
import threading
import time
from typing import Callable, Dict, List, Optional

from .checkpoints import Checkpoint, CheckpointStore
from .env import Env, SimulatedEnv, StepOutcome

TelemetryCb = Callable[[dict], None]


class TrainStatus:
    IDLE = "idle"
    RUNNING = "running"
    PAUSED = "paused"
    FINISHED = "finished"


_POLICY_ACTIONS = ["move", "attack", "Q", "W", "E", "recall"]


class TrainingControlCenter:
    def __init__(
        self,
        store: CheckpointStore,
        env_factory: Optional[Callable[[dict], Env]] = None,
        telemetry: Optional[TelemetryCb] = None,
        tick_sleep: float = 0.0,
        emit_pause: float = 0.3,
    ) -> None:
        self._store = store
        self._env_factory = env_factory or (lambda cfg: SimulatedEnv(episode_len=256))
        self._telemetry = telemetry
        self._tick_sleep = tick_sleep
        self._emit_pause = emit_pause

        self._lock = threading.Lock()
        self._status = TrainStatus.IDLE
        self._step = 0
        self._max_timesteps = 1_000_000
        self._ep_return = 0.0
        self._last_ep_return = 0.0
        self._config: dict = {}
        self._policy = [1.0 / len(_POLICY_ACTIONS)] * len(_POLICY_ACTIONS)
        self._rng = random.Random(0)
        self._last_breakdown: List[Dict[str, float]] = []

        self._thread: Optional[threading.Thread] = None
        self._stop = threading.Event()
        self._resume = threading.Event()
        self._resume.set()

    # ── 配置/对外查询 ──
    def set_telemetry(self, cb: Optional[TelemetryCb]) -> None:
        self._telemetry = cb

    def status(self) -> dict:
        with self._lock:
            return {
                "status": self._status,
                "step": self._step,
                "max_timesteps": self._max_timesteps,
                "ep_return": round(self._last_ep_return, 4),
            }

    # ── 控制 ──
    def start(self, config: Optional[dict] = None) -> dict:
        self.stop()  # 若在运行先终止旧线程
        config = config or {}
        with self._lock:
            self._config = config
            self._max_timesteps = int(
                (config.get("steps") or {}).get("max_timesteps", 1_000_000)
            )
            self._step = 0
            self._ep_return = 0.0
            self._last_ep_return = 0.0
            self._status = TrainStatus.RUNNING
        self._stop.clear()
        self._resume.set()
        self._emit({"type": "status", "status": TrainStatus.RUNNING})
        self._thread = threading.Thread(target=self._run, name="rl-train", daemon=True)
        self._thread.start()
        return self.status()

    def pause(self) -> dict:
        with self._lock:
            if self._status == TrainStatus.RUNNING:
                self._status = TrainStatus.PAUSED
                self._resume.clear()
        self._emit({"type": "status", "status": self._status})
        return self.status()

    def resume(self) -> dict:
        with self._lock:
            if self._status == TrainStatus.PAUSED:
                self._status = TrainStatus.RUNNING
                self._resume.set()
        self._emit({"type": "status", "status": self._status})
        return self.status()

    def stop(self) -> dict:
        self._stop.set()
        self._resume.set()  # 解除暂停以便线程退出
        thread = self._thread
        if thread and thread.is_alive():
            thread.join(timeout=5.0)
        self._thread = None
        with self._lock:
            if self._status != TrainStatus.IDLE:
                self._status = TrainStatus.FINISHED
        self._emit({"type": "status", "status": self._status})
        return self.status()

    def save_checkpoint(self) -> Checkpoint:
        with self._lock:
            step = self._step
            ep_return = self._last_ep_return
            config = dict(self._config)
        ckpt = self._store.save(step=step, ep_return=ep_return, config=config)
        self._emit({"type": "checkpoint", "checkpoint": ckpt.public()})
        return ckpt

    # ── 内部 ──
    def _emit(self, frame: dict) -> None:
        cb = self._telemetry
        if cb is not None:
            try:
                cb(frame)
            except Exception:
                pass

    def _emit_metrics(self) -> None:
        progress = self._step / self._max_timesteps if self._max_timesteps else 0.0
        loss = max(0.01, 2.0 - progress * 1.5 + (self._rng.random() - 0.5) * 0.3)
        kl = abs((self._rng.random() - 0.5) * 0.05)
        entropy = max(0.05, 0.7 - progress * 0.5 + (self._rng.random() - 0.5) * 0.05)
        # 策略分布随机游走并归一化。
        drift = [max(0.01, p + (self._rng.random() - 0.5) * 0.04) for p in self._policy]
        total = sum(drift)
        self._policy = [p / total for p in drift]
        policy = [
            {"action": a, "prob": round(p, 4)}
            for a, p in zip(_POLICY_ACTIONS, self._policy)
        ]
        frame = {
            "type": "metrics",
            "step": self._step,
            "ep_return": round(self._last_ep_return, 4),
            "loss": round(loss, 4),
            "kl": round(kl, 5),
            "entropy": round(entropy, 4),
            "value": round(self._last_ep_return, 4),
            "policy": policy,
            "reward_breakdown": self._last_breakdown,
        }
        self._emit(frame)

    def _run(self) -> None:
        env = self._env_factory(self._config)
        env.reset()
        self._ep_return = 0.0
        rollout = int((self._config.get("steps") or {}).get("rollout", 2048))
        rollout = max(1, rollout)

        while not self._stop.is_set():
            self._resume.wait()  # 暂停时阻塞，stop 时被放行
            if self._stop.is_set():
                break
            with self._lock:
                if self._step >= self._max_timesteps:
                    break

            outcome: StepOutcome = env.step()
            with self._lock:
                self._step += 1
                self._ep_return += outcome.reward
            self._last_breakdown = outcome.breakdown

            if outcome.done:
                with self._lock:
                    self._last_ep_return = self._ep_return
                    self._ep_return = 0.0
                self._emit_metrics()
                env.reset()
                if self._emit_pause:
                    time.sleep(self._emit_pause)
            elif self._step % rollout == 0:
                with self._lock:
                    self._last_ep_return = self._ep_return
                self._emit_metrics()
                if self._emit_pause:
                    time.sleep(self._emit_pause)

            if self._tick_sleep:
                time.sleep(self._tick_sleep)

        with self._lock:
            self._status = TrainStatus.FINISHED
        self._emit({"type": "status", "status": TrainStatus.FINISHED})
