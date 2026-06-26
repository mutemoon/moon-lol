"""WebSocket 遥测/控制服务的测试：编解码工具 + 真实握手/控制集成。"""

import os
import socket
import struct
import sys
import tempfile
import time
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from moonlol_trainer.checkpoints import CheckpointStore
from moonlol_trainer.control_center import TrainingControlCenter
from moonlol_trainer.env import SimulatedEnv
from moonlol_trainer.ws_server import (
    TelemetryWsServer,
    _Conn,
    _accept_key,
    _encode_text_frame,
)


def _mask_text_frame(payload: str) -> bytes:
    data = payload.encode("utf-8")
    header = bytearray([0x81])
    n = len(data)
    mask = b"\x12\x34\x56\x78"
    if n < 126:
        header.append(0x80 | n)
    elif n < 65536:
        header.append(0x80 | 126)
        header += struct.pack(">H", n)
    else:
        header.append(0x80 | 127)
        header += struct.pack(">Q", n)
    masked = bytes(b ^ mask[i % 4] for i, b in enumerate(data))
    return bytes(header) + mask + masked


class FrameUtilTests(unittest.TestCase):
    def test_accept_key_matches_rfc_example(self):
        # RFC 6455 §1.3 示例。
        self.assertEqual(_accept_key("dGhlIHNhbXBsZSBub25jZQ=="), "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=")

    def test_encode_text_frame_small(self):
        frame = _encode_text_frame("hi")
        self.assertEqual(frame[0], 0x81)
        self.assertEqual(frame[1], 2)
        self.assertEqual(frame[2:], b"hi")


class WsIntegrationTests(unittest.TestCase):
    def setUp(self):
        self._dir = tempfile.TemporaryDirectory()
        store = CheckpointStore(self._dir.name)
        self.store = store
        self.center = TrainingControlCenter(
            store,
            env_factory=lambda cfg: SimulatedEnv(episode_len=8, seed=3),
            tick_sleep=0.0,
            emit_pause=0.0,
        )
        self.ws = TelemetryWsServer(self.center, "127.0.0.1", 0)
        self.ws.start()
        self.port = self.ws.actual_port

    def tearDown(self):
        self.center.stop()
        self.ws.stop()
        self._dir.cleanup()

    def _connect(self):
        sock = socket.create_connection(("127.0.0.1", self.port), timeout=5)
        sock.settimeout(5)
        handshake = (
            "GET / HTTP/1.1\r\n"
            "Host: 127.0.0.1\r\n"
            "Upgrade: websocket\r\n"
            "Connection: Upgrade\r\n"
            "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n"
            "Sec-WebSocket-Version: 13\r\n\r\n"
        )
        sock.sendall(handshake.encode("ascii"))
        # 读取握手响应，保留多读到的字节交给帧解析器。
        buf = b""
        while b"\r\n\r\n" not in buf:
            chunk = sock.recv(4096)
            self.assertTrue(chunk, "handshake closed early")
            buf += chunk
        header_end = buf.index(b"\r\n\r\n") + 4
        self.assertIn(b"101", buf[:header_end])
        leftover = buf[header_end:]
        conn = _Conn(sock)
        conn._buf = leftover
        return sock, conn

    def test_handshake_status_and_control_checkpoint(self):
        sock, conn = self._connect()
        try:
            # 连接后服务端推送当前状态帧。
            first = conn.read_message()
            self.assertTrue(first)
            self.assertIn("status", first)

            # 发送控制帧：保存 checkpoint。
            sock.sendall(_mask_text_frame('{"type":"save_checkpoint"}'))

            # 应收到广播的 checkpoint 帧。
            got_checkpoint = False
            deadline = time.time() + 5.0
            while time.time() < deadline:
                msg = conn.read_message()
                if msg is None:
                    break
                if msg and '"checkpoint"' in msg:
                    got_checkpoint = True
                    break
            self.assertTrue(got_checkpoint, "expected checkpoint broadcast")
            self.assertEqual(len(self.store.list()), 1)
        finally:
            sock.close()

    def test_control_start_triggers_training(self):
        sock, conn = self._connect()
        try:
            conn.read_message()  # 丢弃初始 status
            sock.sendall(
                _mask_text_frame(
                    '{"type":"control","command":"start","config":{"steps":{"max_timesteps":40,"rollout":5}}}'
                )
            )
            deadline = time.time() + 5.0
            saw_metrics = False
            while time.time() < deadline:
                msg = conn.read_message()
                if msg is None:
                    break
                if msg and '"metrics"' in msg:
                    saw_metrics = True
                    break
            self.assertTrue(saw_metrics, "expected metrics frames after start")
        finally:
            sock.close()


if __name__ == "__main__":
    unittest.main()
