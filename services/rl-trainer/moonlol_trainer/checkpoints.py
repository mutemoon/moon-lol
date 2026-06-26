"""Checkpoint 磁盘存取。

每个 checkpoint 由两部分组成：
  - `<id>.json`：元数据（id / step / ep_return / created_at / 训练配置）；
  - `<id>.pth`：权重文件占位（真实训练写入模型 state_dict 字节）。
"""

from __future__ import annotations

import json
import os
import threading
import time
from dataclasses import asdict, dataclass
from typing import List, Optional


@dataclass
class Checkpoint:
    id: str
    step: int
    path: str
    ep_return: float
    created_at: str
    config: dict

    def public(self) -> dict:
        """对外（客户端遥测）暴露的精简视图。"""
        return {
            "id": self.id,
            "step": self.step,
            "path": self.path,
            "ep_return": self.ep_return,
            "created_at": self.created_at,
        }


class CheckpointStore:
    """线程安全的 checkpoint 存储。"""

    def __init__(self, directory: str) -> None:
        self.directory = directory
        os.makedirs(directory, exist_ok=True)
        self._lock = threading.Lock()

    def _meta_path(self, ckpt_id: str) -> str:
        return os.path.join(self.directory, f"{ckpt_id}.json")

    def _weights_path(self, ckpt_id: str) -> str:
        return os.path.join(self.directory, f"{ckpt_id}.pth")

    def save(self, step: int, ep_return: float, config: dict, weights: Optional[bytes] = None) -> Checkpoint:
        with self._lock:
            ckpt_id = f"ckpt_{step}_{int(time.time() * 1000)}"
            weights_path = self._weights_path(ckpt_id)
            with open(weights_path, "wb") as f:
                f.write(weights if weights is not None else b"")
            ckpt = Checkpoint(
                id=ckpt_id,
                step=step,
                path=weights_path,
                ep_return=round(float(ep_return), 4),
                created_at=time.strftime("%Y-%m-%dT%H:%M:%S", time.localtime()),
                config=config,
            )
            with open(self._meta_path(ckpt_id), "w", encoding="utf-8") as f:
                json.dump(asdict(ckpt), f, ensure_ascii=False, indent=2)
            return ckpt

    def list(self) -> List[Checkpoint]:
        with self._lock:
            items: List[Checkpoint] = []
            for name in os.listdir(self.directory):
                if not name.endswith(".json"):
                    continue
                try:
                    with open(os.path.join(self.directory, name), "r", encoding="utf-8") as f:
                        items.append(Checkpoint(**json.load(f)))
                except Exception:
                    continue
            items.sort(key=lambda c: c.created_at, reverse=True)
            return items

    def get(self, ckpt_id: str) -> Optional[Checkpoint]:
        with self._lock:
            meta = self._meta_path(ckpt_id)
            if not os.path.exists(meta):
                return None
            with open(meta, "r", encoding="utf-8") as f:
                return Checkpoint(**json.load(f))

    def read_weights(self, ckpt_id: str) -> Optional[bytes]:
        with self._lock:
            path = self._weights_path(ckpt_id)
            if not os.path.exists(path):
                return None
            with open(path, "rb") as f:
                return f.read()
