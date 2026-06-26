"""最小 RFC6455 WebSocket 服务（仅标准库）。

向所有连接的客户端广播遥测帧（status / metrics / checkpoint），并接收控制帧
（control start/pause/resume/stop、save_checkpoint、apply_checkpoint），转交控制中心。
协议与 `apps/client/src/composables/useRlTelemetry.ts` 一致。

仅实现演示/遥测所需的文本帧与关闭/ping 处理，不追求完整 RFC 覆盖。
"""

from __future__ import annotations

import base64
import hashlib
import json
import socket
import struct
import threading
from typing import Optional, Set

from .control_center import TrainingControlCenter

_WS_GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"


def _accept_key(key: str) -> str:
    digest = hashlib.sha1((key + _WS_GUID).encode("utf-8")).digest()
    return base64.b64encode(digest).decode("ascii")


def _encode_text_frame(payload: str) -> bytes:
    data = payload.encode("utf-8")
    header = bytearray([0x81])  # FIN + text opcode
    n = len(data)
    if n < 126:
        header.append(n)
    elif n < 65536:
        header.append(126)
        header += struct.pack(">H", n)
    else:
        header.append(127)
        header += struct.pack(">Q", n)
    return bytes(header) + data


class _Conn:
    def __init__(self, sock: socket.socket) -> None:
        self.sock = sock
        self._buf = b""

    def _recv_exact(self, n: int) -> Optional[bytes]:
        while len(self._buf) < n:
            try:
                chunk = self.sock.recv(4096)
            except OSError:
                return None
            if not chunk:
                return None
            self._buf += chunk
        out, self._buf = self._buf[:n], self._buf[n:]
        return out

    def read_message(self) -> Optional[str]:
        """读取一个文本消息；遇到关闭返回 None。"""
        first = self._recv_exact(2)
        if not first:
            return None
        opcode = first[0] & 0x0F
        masked = (first[1] & 0x80) != 0
        length = first[1] & 0x7F
        if length == 126:
            ext = self._recv_exact(2)
            if not ext:
                return None
            length = struct.unpack(">H", ext)[0]
        elif length == 127:
            ext = self._recv_exact(8)
            if not ext:
                return None
            length = struct.unpack(">Q", ext)[0]
        mask = self._recv_exact(4) if masked else b"\x00\x00\x00\x00"
        if mask is None:
            return None
        payload = self._recv_exact(length) if length else b""
        if payload is None:
            return None
        if masked:
            payload = bytes(b ^ mask[i % 4] for i, b in enumerate(payload))
        if opcode == 0x8:  # close
            return None
        if opcode == 0x9:  # ping → pong
            try:
                self.sock.sendall(bytes([0x8A, len(payload)]) + payload)
            except OSError:
                return None
            return ""
        if opcode in (0x1, 0x0):  # text / continuation（简化为文本）
            try:
                return payload.decode("utf-8", errors="replace")
            except Exception:
                return ""
        return ""  # 其他帧忽略


class TelemetryWsServer:
    def __init__(self, center: TrainingControlCenter, host: str, port: int) -> None:
        self.center = center
        self.host = host
        self.port = port
        self._server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self._server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self._server.bind((host, port))
        self._server.listen(16)
        self._clients: Set[socket.socket] = set()
        self._clients_lock = threading.Lock()
        self._running = False
        self._accept_thread: Optional[threading.Thread] = None
        center.set_telemetry(self.broadcast)

    @property
    def actual_port(self) -> int:
        return self._server.getsockname()[1]

    def broadcast(self, frame: dict) -> None:
        data = _encode_text_frame(json.dumps(frame, ensure_ascii=False))
        with self._clients_lock:
            dead = []
            for c in self._clients:
                try:
                    c.sendall(data)
                except OSError:
                    dead.append(c)
            for c in dead:
                self._clients.discard(c)

    def start(self) -> None:
        self._running = True
        self._accept_thread = threading.Thread(target=self._accept_loop, daemon=True)
        self._accept_thread.start()

    def _accept_loop(self) -> None:
        while self._running:
            try:
                client, _addr = self._server.accept()
            except OSError:
                break
            threading.Thread(target=self._handle, args=(client,), daemon=True).start()

    def _handshake(self, client: socket.socket) -> bool:
        try:
            data = b""
            while b"\r\n\r\n" not in data:
                chunk = client.recv(4096)
                if not chunk:
                    return False
                data += chunk
            headers = {}
            for line in data.decode("latin1").split("\r\n")[1:]:
                if ":" in line:
                    k, v = line.split(":", 1)
                    headers[k.strip().lower()] = v.strip()
            key = headers.get("sec-websocket-key")
            if not key:
                return False
            resp = (
                "HTTP/1.1 101 Switching Protocols\r\n"
                "Upgrade: websocket\r\n"
                "Connection: Upgrade\r\n"
                f"Sec-WebSocket-Accept: {_accept_key(key)}\r\n\r\n"
            )
            client.sendall(resp.encode("ascii"))
            return True
        except OSError:
            return False

    def _handle(self, client: socket.socket) -> None:
        if not self._handshake(client):
            client.close()
            return
        with self._clients_lock:
            self._clients.add(client)
        # 连接即推送当前状态。
        try:
            client.sendall(
                _encode_text_frame(json.dumps({"type": "status", **self.center.status()}))
            )
        except OSError:
            pass
        conn = _Conn(client)
        try:
            while self._running:
                msg = conn.read_message()
                if msg is None:
                    break
                if msg:
                    self._dispatch(msg)
        finally:
            with self._clients_lock:
                self._clients.discard(client)
            client.close()

    def _dispatch(self, raw: str) -> None:
        try:
            frame = json.loads(raw)
        except Exception:
            return
        ftype = frame.get("type")
        if ftype == "control":
            cmd = frame.get("command")
            if cmd == "start":
                self.center.start(frame.get("config") or {})
            elif cmd == "pause":
                self.center.pause()
            elif cmd == "resume":
                self.center.resume()
            elif cmd == "stop":
                self.center.stop()
        elif ftype == "save_checkpoint":
            self.center.save_checkpoint()
        elif ftype == "apply_checkpoint":
            # 通知层面：真实实现可在此加载权重到推理进程。
            self.broadcast({"type": "status", **self.center.status()})

    def stop(self) -> None:
        self._running = False
        try:
            self._server.close()
        except OSError:
            pass
        with self._clients_lock:
            for c in list(self._clients):
                try:
                    c.close()
                except OSError:
                    pass
            self._clients.clear()
