"""训练环境抽象。

`Env` 是控制中心使用的最小协议：`reset()` 与 `step()`。提供两种实现：
  - `SimulatedEnv`：纯标准库的自洽模拟，便于守护进程独立运行与测试；
  - `BevyEnv`：连接 Bevy 侧 `MoonLoLEnv`（`rl_reset` / `rl_step` WS 指令）。
    需要额外安装 `websockets`，按需惰性导入，不影响标准库运行。
"""

from __future__ import annotations

import random
from dataclasses import dataclass, field
from typing import Dict, List, Protocol


@dataclass
class StepOutcome:
    """一步交互的结果（对齐 Bevy `StepResult`）。"""

    reward: float
    terminated: bool
    truncated: bool
    breakdown: List[Dict[str, float]] = field(default_factory=list)

    @property
    def done(self) -> bool:
        return self.terminated or self.truncated


class Env(Protocol):
    """训练环境协议。"""

    def reset(self) -> None: ...

    def step(self) -> StepOutcome: ...


class SimulatedEnv:
    """无外部依赖的模拟环境：回合内 reward 随机游走，到达 `episode_len` 后截断。

    用于守护进程独立演示/测试；真实训练应注入 `BevyEnv`。
    """

    def __init__(self, episode_len: int = 256, seed: int | None = None) -> None:
        self.episode_len = max(1, episode_len)
        self._rng = random.Random(seed)
        self._t = 0

    def reset(self) -> None:
        self._t = 0

    def step(self) -> StepOutcome:
        self._t += 1
        # 简化的分项 reward，便于客户端遥测展示。
        last_hit = self._rng.choice([0.0, 0.0, 0.0, 1.0])
        kill = self._rng.choice([0.0] * 30 + [5.0])
        death = self._rng.choice([0.0] * 60 + [-5.0])
        health = (self._rng.random() - 0.5) * 0.4
        time_pen = -0.001
        breakdown = [
            {"name": "last_hit", "value": last_hit},
            {"name": "kill", "value": kill},
            {"name": "death", "value": death},
            {"name": "health", "value": round(health, 4)},
            {"name": "time", "value": time_pen},
        ]
        reward = sum(item["value"] for item in breakdown)
        terminated = death < 0.0
        truncated = self._t >= self.episode_len
        return StepOutcome(
            reward=reward,
            terminated=terminated,
            truncated=truncated,
            breakdown=breakdown,
        )


class BevyEnv:
    """连接 Bevy 侧 `MoonLoLEnv` 的环境（`rl_reset` / `rl_step` WS 指令）。

    依赖第三方 `websockets`（惰性导入）；未安装时构造即抛出，调用方应回落到
    `SimulatedEnv`。仅消费 `rl_step` 返回的 JSON（reward / terminated / truncated /
    breakdown）用于训练与遥测；观测的 msgpack 解码留给使用方的 PPO 实现。
    """

    def __init__(self, url: str, entity_id: int | None = None, config_json: dict | None = None) -> None:
        try:
            import websockets  # noqa: F401
        except Exception as exc:  # pragma: no cover - 依赖缺失路径
            raise RuntimeError(
                "BevyEnv 需要 'websockets' 包：pip install websockets"
            ) from exc
        self.url = url
        self.entity_id = entity_id
        self.config_json = config_json or {}
        self._next_id = 0
        self._ws = None  # 由调用方在异步上下文中建立连接

    # 说明：真实连接/收发在使用方的异步训练进程中完成。这里给出协议构造，
    # 保证与 Bevy 指令格式一致，便于上层封装。
    def _cmd(self, cmd: str, params: dict) -> dict:
        self._next_id += 1
        return {"id": self._next_id, "type": "cmd", "cmd": cmd, "params": params}

    def reset_cmd(self) -> dict:
        params: dict = {"config_json": self.config_json}
        if self.entity_id is not None:
            params["entity_id"] = self.entity_id
        return self._cmd("rl_reset", params)

    def step_cmd(self) -> dict:
        params: dict = {}
        if self.entity_id is not None:
            params["entity_id"] = self.entity_id
        return self._cmd("rl_step", params)

    @staticmethod
    def parse_step(data: dict) -> StepOutcome:
        breakdown = []
        bd = (data or {}).get("breakdown") or {}
        for comp in bd.get("components", []) if isinstance(bd, dict) else []:
            breakdown.append({"name": comp.get("name", ""), "value": float(comp.get("value", 0.0))})
        return StepOutcome(
            reward=float((data or {}).get("reward", 0.0)),
            terminated=bool((data or {}).get("terminated", False)),
            truncated=bool((data or {}).get("truncated", False)),
            breakdown=breakdown,
        )

    def reset(self) -> None:  # pragma: no cover - 需运行中的 Bevy
        raise NotImplementedError("BevyEnv 的同步收发需在异步训练进程中驱动")

    def step(self) -> StepOutcome:  # pragma: no cover - 需运行中的 Bevy
        raise NotImplementedError("BevyEnv 的同步收发需在异步训练进程中驱动")
