"""控制中心与 checkpoint 存储的单元测试。"""

import os
import sys
import tempfile
import time
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from moonlol_trainer.checkpoints import CheckpointStore
from moonlol_trainer.control_center import TrainStatus, TrainingControlCenter
from moonlol_trainer.env import SimulatedEnv


class CheckpointStoreTests(unittest.TestCase):
    def test_save_list_get_download(self):
        with tempfile.TemporaryDirectory() as d:
            store = CheckpointStore(d)
            ckpt = store.save(step=100, ep_return=12.5, config={"algorithm": "ppo"}, weights=b"abc")
            self.assertEqual(ckpt.step, 100)
            self.assertTrue(os.path.exists(ckpt.path))
            listed = store.list()
            self.assertEqual(len(listed), 1)
            self.assertEqual(listed[0].id, ckpt.id)
            got = store.get(ckpt.id)
            self.assertIsNotNone(got)
            self.assertEqual(got.ep_return, 12.5)
            self.assertEqual(store.read_weights(ckpt.id), b"abc")
            self.assertIsNone(store.get("missing"))


class ControlCenterTests(unittest.TestCase):
    def _center(self, store):
        return TrainingControlCenter(
            store,
            env_factory=lambda cfg: SimulatedEnv(episode_len=8, seed=1),
            tick_sleep=0.0,
            emit_pause=0.0,
        )

    def test_train_runs_and_finishes_with_telemetry(self):
        with tempfile.TemporaryDirectory() as d:
            store = CheckpointStore(d)
            center = self._center(store)
            frames = []
            center.set_telemetry(frames.append)
            center.start({"steps": {"max_timesteps": 40, "rollout": 5}})

            deadline = time.time() + 5.0
            while time.time() < deadline and center.status()["status"] != TrainStatus.FINISHED:
                time.sleep(0.02)

            self.assertEqual(center.status()["status"], TrainStatus.FINISHED)
            types = [f.get("type") for f in frames]
            self.assertIn("status", types)
            self.assertIn("metrics", types)
            # 校验 metrics 帧含遥测所需字段。
            metric = next(f for f in frames if f.get("type") == "metrics")
            for key in ("step", "ep_return", "loss", "kl", "entropy", "value", "policy"):
                self.assertIn(key, metric)
            self.assertTrue(abs(sum(p["prob"] for p in metric["policy"]) - 1.0) < 1e-3)

    def test_save_checkpoint_emits_frame(self):
        with tempfile.TemporaryDirectory() as d:
            store = CheckpointStore(d)
            center = self._center(store)
            frames = []
            center.set_telemetry(frames.append)
            ckpt = center.save_checkpoint()
            self.assertEqual(len(store.list()), 1)
            ckpt_frames = [f for f in frames if f.get("type") == "checkpoint"]
            self.assertEqual(len(ckpt_frames), 1)
            self.assertEqual(ckpt_frames[0]["checkpoint"]["id"], ckpt.id)

    def test_pause_resume_stop(self):
        with tempfile.TemporaryDirectory() as d:
            store = CheckpointStore(d)
            center = self._center(store)
            center.start({"steps": {"max_timesteps": 10_000_000, "rollout": 5}})
            center.pause()
            self.assertEqual(center.status()["status"], TrainStatus.PAUSED)
            center.resume()
            self.assertEqual(center.status()["status"], TrainStatus.RUNNING)
            center.stop()
            self.assertEqual(center.status()["status"], TrainStatus.FINISHED)


if __name__ == "__main__":
    unittest.main()
