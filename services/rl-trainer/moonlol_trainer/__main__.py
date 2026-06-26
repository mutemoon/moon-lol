"""守护进程入口：组合 REST + WebSocket 遥测，共享同一控制中心。

用法：
    python3 -m moonlol_trainer [--host H] [--rest-port P] [--ws-port P] [--checkpoint-dir D]
"""

from __future__ import annotations

import argparse
import signal
import threading

from .checkpoints import CheckpointStore
from .control_center import TrainingControlCenter
from .rest_server import create_rest_server
from .ws_server import TelemetryWsServer


def main() -> None:
    parser = argparse.ArgumentParser(description="MoonLoL RL 训练守护进程")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--rest-port", type=int, default=8770)
    parser.add_argument("--ws-port", type=int, default=8771)
    parser.add_argument("--checkpoint-dir", default="./checkpoints")
    args = parser.parse_args()

    store = CheckpointStore(args.checkpoint_dir)
    center = TrainingControlCenter(store)
    ws = TelemetryWsServer(center, args.host, args.ws_port)
    ws.start()
    rest = create_rest_server(center, args.host, args.rest_port)

    print(f"[moonlol-trainer] REST  http://{args.host}:{args.rest_port}")
    print(f"[moonlol-trainer] WS    ws://{args.host}:{args.ws_port}")
    print(f"[moonlol-trainer] ckpt  {args.checkpoint_dir}")

    stop_event = threading.Event()

    def _shutdown(*_a):
        stop_event.set()

    signal.signal(signal.SIGINT, _shutdown)
    signal.signal(signal.SIGTERM, _shutdown)

    rest_thread = threading.Thread(target=rest.serve_forever, daemon=True)
    rest_thread.start()
    try:
        stop_event.wait()
    finally:
        center.stop()
        ws.stop()
        rest.shutdown()


if __name__ == "__main__":
    main()
