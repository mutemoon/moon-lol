import subprocess
import json
import asyncio
import websockets
import threading
import atexit
import os
import signal
import sys

class MoonLoLClient:
    def __init__(self, port=9030, scene="games/headless-env-kill-fiora.ron"):
        self.port = port
        self.scene = scene
        self.proc = None
        self.websocket = None
        self.req_id = 1
        self.controlled_entity_id = None
        
        atexit.register(self.close)
        
        try:
            signal.signal(signal.SIGTERM, lambda s, f: sys.exit(1))
        except ValueError:
            pass
            
        self.loop = asyncio.new_event_loop()
        self.loop_thread = threading.Thread(target=self._start_background_loop, daemon=True)
        self.loop_thread.start()

    def _start_background_loop(self):
        asyncio.set_event_loop(self.loop)
        self.loop.run_forever()

    def _run_async(self, coro):
        future = asyncio.run_coroutine_threadsafe(coro, self.loop)
        return future.result()

    async def _close_conn_async(self):
        if not self.websocket:
            return
        try:
            await self.websocket.close()
        except Exception:
            pass
        self.websocket = None

    def _close_conn(self):
        self._run_async(self._close_conn_async())

    def close(self):
        self._close_conn()
        if not self.proc:
            return
        try:
            pgid = os.getpgid(self.proc.pid)
            os.killpg(pgid, signal.SIGTERM)
            self.proc.wait(timeout=2.0)
        except Exception:
            try:
                pgid = os.getpgid(self.proc.pid)
                os.killpg(pgid, signal.SIGKILL)
            except Exception:
                pass
        self.proc = None

    async def _send_and_recv_async(self, req):
        if not self.websocket:
            raise ConnectionError("Not connected")
        await self.websocket.send(json.dumps(req))
        resp = await self.websocket.recv()
        return json.loads(resp)

    def _send_and_recv(self, req):
        return self._run_async(self._send_and_recv_async(req))

    def _resolve_controlled_entity(self, obs_data):
        myself = obs_data.get("myself", {})
        level = myself.get("level", 1)
        if level == 18:
            return None
        enemies = obs_data.get("enemy_heroes", [])
        if not enemies:
            return None
        return enemies[0].get("entity")

    async def _connect_and_wait_async(self):
        uri = f"ws://127.0.0.1:{self.port}"
        websocket = None
        for attempt in range(1, 30):
            try:
                websocket = await websockets.connect(uri)
                break
            except Exception:
                await asyncio.sleep(0.5)
        if not websocket:
            raise ConnectionError(f"Failed to connect to Bevy server on port {self.port}")
        
        self.websocket = websocket
        
        # Wait for champion to spawn
        for attempt in range(1, 30):
            req = {"id": self.req_id, "cmd": "get_observe", "params": {}}
            self.req_id += 1
            await websocket.send(json.dumps(req))
            resp = json.loads(await websocket.recv())
            if not resp.get("ok"):
                await asyncio.sleep(0.5)
                continue
            
            obs_data = resp.get("data")
            self.controlled_entity_id = self._resolve_controlled_entity(obs_data)
            break

    def start(self):
        self.close()
        self.controlled_entity_id = None
        
        self.proc = subprocess.Popen([
            "cargo", "run", "--",
            "--scene", self.scene,
            "--ws-port", str(self.port),

        ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        
        self._run_async(self._connect_and_wait_async())

    def get_observe(self):
        params = {}
        if self.controlled_entity_id is not None:
            params["entity_id"] = self.controlled_entity_id
        req = {"id": self.req_id, "cmd": "get_observe", "params": params}
        self.req_id += 1
        resp = self._send_and_recv(req)
        if resp.get("ok"):
            return resp.get("data")
        return None

    def send_action(self, action_payload):
        params = {
            "action": action_payload
        }
        if self.controlled_entity_id is not None:
            params["entity_id"] = self.controlled_entity_id
            
        req = {
            "id": self.req_id,
            "cmd": "action",
            "params": params
        }
        self.req_id += 1
        return self._send_and_recv(req)

    def move(self, position):
        return self.send_action({"Move": position})

    def attack(self, target_id):
        return self.send_action({"Attack": target_id})

    def cast_skill(self, index, position):
        return self.send_action({"Skill": {"index": index, "point": position}})

    def stop(self):
        return self.send_action("Stop")

    def level_up_skill(self, index):
        return self.send_action({"SkillLevelUp": index})
