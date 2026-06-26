"""标准库 HTTP REST 接口。

暴露训练控制与 checkpoint 存取，供运维/脚本直接调用（与 WS 遥测互补）。
"""

from __future__ import annotations

import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Optional, Tuple

from .control_center import TrainingControlCenter


def _make_handler(center: TrainingControlCenter):
    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        # 静默默认日志（避免污染 stdout）。
        def log_message(self, *_args) -> None:  # noqa: D401
            pass

        def _send(self, code: int, payload, content_type: str = "application/json") -> None:
            if isinstance(payload, (dict, list)):
                body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
            elif isinstance(payload, bytes):
                body = payload
            else:
                body = str(payload).encode("utf-8")
            self.send_response(code)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Access-Control-Allow-Origin", "*")
            self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            self.send_header("Access-Control-Allow-Headers", "Content-Type")
            self.end_headers()
            self.wfile.write(body)

        def _read_json(self) -> dict:
            length = int(self.headers.get("Content-Length", 0) or 0)
            if length <= 0:
                return {}
            try:
                return json.loads(self.rfile.read(length).decode("utf-8") or "{}")
            except Exception:
                return {}

        def do_OPTIONS(self) -> None:  # noqa: N802 - CORS 预检
            self._send(204, b"")

        def do_GET(self) -> None:  # noqa: N802
            path = self.path.split("?", 1)[0].rstrip("/")
            if path == "/api/status":
                self._send(200, center.status())
                return
            if path == "/api/checkpoints":
                self._send(200, [c.public() for c in center._store.list()])
                return
            ckpt_id, sub = _match_checkpoint(path)
            if ckpt_id is not None:
                ckpt = center._store.get(ckpt_id)
                if not ckpt:
                    self._send(404, {"error": "checkpoint not found"})
                    return
                if sub == "download":
                    data = center._store.read_weights(ckpt_id) or b""
                    self._send(200, data, content_type="application/octet-stream")
                else:
                    self._send(200, ckpt.public())
                return
            self._send(404, {"error": "not found"})

        def do_POST(self) -> None:  # noqa: N802
            path = self.path.split("?", 1)[0].rstrip("/")
            if path == "/api/train/start":
                self._send(200, center.start(self._read_json()))
                return
            if path == "/api/train/pause":
                self._send(200, center.pause())
                return
            if path == "/api/train/resume":
                self._send(200, center.resume())
                return
            if path == "/api/train/stop":
                self._send(200, center.stop())
                return
            if path == "/api/checkpoints":
                self._send(200, center.save_checkpoint().public())
                return
            self._send(404, {"error": "not found"})

    return Handler


def _match_checkpoint(path: str) -> Tuple[Optional[str], Optional[str]]:
    """解析 /api/checkpoints/{id} 或 /api/checkpoints/{id}/download。"""
    prefix = "/api/checkpoints/"
    if not path.startswith(prefix):
        return None, None
    rest = path[len(prefix):]
    if not rest:
        return None, None
    parts = rest.split("/")
    if len(parts) == 1:
        return parts[0], None
    if len(parts) == 2 and parts[1] == "download":
        return parts[0], "download"
    return None, None


def create_rest_server(center: TrainingControlCenter, host: str, port: int) -> ThreadingHTTPServer:
    return ThreadingHTTPServer((host, port), _make_handler(center))
