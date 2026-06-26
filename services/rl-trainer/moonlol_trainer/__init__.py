"""MoonLoL RL 训练守护进程（纯标准库实现）。

模块：
  - env：训练环境抽象（SimulatedEnv 默认；BevyEnv 接 Bevy MoonLoLEnv）。
  - checkpoints：checkpoint 磁盘存取。
  - control_center：训练控制中心（线程化训练循环 + 遥测回调 + checkpoint）。
  - rest_server：标准库 HTTP REST 接口。
  - ws_server：最小 RFC6455 WebSocket 遥测/控制服务。
"""

from .checkpoints import Checkpoint, CheckpointStore
from .control_center import TrainingControlCenter, TrainStatus
from .env import BevyEnv, Env, SimulatedEnv, StepOutcome

__all__ = [
    "Checkpoint",
    "CheckpointStore",
    "TrainingControlCenter",
    "TrainStatus",
    "Env",
    "SimulatedEnv",
    "BevyEnv",
    "StepOutcome",
]
