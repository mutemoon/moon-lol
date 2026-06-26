"""REST 接口的功能测试（启动真实 HTTP 服务，用 urllib 调用）。"""

import json
import os
import sys
import tempfile
import threading
import time
import unittest
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from moonlol_trainer.checkpoints import CheckpointStore
from moonlol_trainer.control_center import TrainingControlCenter
from moonlol_trainer.env import SimulatedEnv
from moonlol_trainer.rest_server import create_rest_server


def _req(method, url, body=None):
    data = json.dumps(body).encode("utf-8") if body is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    if data is not None:
        req.add_header("Content-Type", "application/json")
    with urllib.request.urlopen(req, timeout=5) as resp:
        raw = resp.read()
        ctype = resp.headers.get("Content-Type", "")
        if "application/json" in ctype:
            return resp.status, json.loads(raw.decode("utf-8"))
        return resp.status, raw


class RestApiTests(unittest.TestCase):
    def setUp(self):
        self._dir = tempfile.TemporaryDirectory()
        store = CheckpointStore(self._dir.name)
        self.center = TrainingControlCenter(
            store,
            env_factory=lambda cfg: SimulatedEnv(episode_len=8, seed=2),
            tick_sleep=0.0,
            emit_pause=0.0,
        )
        self.server = create_rest_server(self.center, "127.0.0.1", 0)
        self.port = self.server.server_address[1]
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.base = f"http://127.0.0.1:{self.port}"

    def tearDown(self):
        self.center.stop()
        self.server.shutdown()
        self.server.server_close()
        self._dir.cleanup()

    def test_status_and_training_lifecycle(self):
        status, body = _req("GET", f"{self.base}/api/status")
        self.assertEqual(status, 200)
        self.assertEqual(body["status"], "idle")

        status, body = _req(
            "POST", f"{self.base}/api/train/start", {"steps": {"max_timesteps": 40, "rollout": 5}}
        )
        self.assertEqual(status, 200)
        self.assertIn(body["status"], ("running", "finished"))

        # 等待训练结束。
        deadline = time.time() + 5.0
        while time.time() < deadline:
            _, st = _req("GET", f"{self.base}/api/status")
            if st["status"] == "finished":
                break
            time.sleep(0.02)
        self.assertEqual(st["status"], "finished")
        self.assertGreater(st["step"], 0)

    def test_checkpoint_endpoints(self):
        # 保存 checkpoint。
        status, ckpt = _req("POST", f"{self.base}/api/checkpoints")
        self.assertEqual(status, 200)
        ckpt_id = ckpt["id"]

        # 列表。
        status, items = _req("GET", f"{self.base}/api/checkpoints")
        self.assertEqual(status, 200)
        self.assertEqual(len(items), 1)
        self.assertEqual(items[0]["id"], ckpt_id)

        # 单个元数据。
        status, one = _req("GET", f"{self.base}/api/checkpoints/{ckpt_id}")
        self.assertEqual(status, 200)
        self.assertEqual(one["id"], ckpt_id)

        # 下载权重（占位字节）。
        status, raw = _req("GET", f"{self.base}/api/checkpoints/{ckpt_id}/download")
        self.assertEqual(status, 200)
        self.assertIsInstance(raw, (bytes, bytearray))

    def test_missing_checkpoint_404(self):
        try:
            _req("GET", f"{self.base}/api/checkpoints/nope")
            self.fail("expected 404")
        except urllib.error.HTTPError as e:
            self.assertEqual(e.code, 404)


if __name__ == "__main__":
    unittest.main()
