import gymnasium as gym
from gymnasium import spaces
import numpy as np
import subprocess
import json
import asyncio
import websockets
import time
import threading
import atexit

class MoonLoLEnv(gym.Env):
    metadata = {"render_modes": []}

    def __init__(self, port=9030, max_steps=400):
        super().__init__()
        self.port = port
        self.max_steps = max_steps
        self.proc = None
        self.websocket = None
        self.req_id = 1
        
        # Register atexit to ensure Bevy is killed when Python exits
        atexit.register(self.close)
        
        # Start a background thread with its own asyncio event loop
        self.loop = asyncio.new_event_loop()
        self.loop_thread = threading.Thread(target=self._start_background_loop, daemon=True)
        self.loop_thread.start()
        
        # 11 continuous features:
        # dx, dy, distance, myself_hp_ratio, fiora_hp_ratio, q_cd, w_cd, e_cd, r_cd, is_windup, is_cooldown
        self.observation_space = spaces.Box(
            low=-np.inf, high=np.inf, shape=(11,), dtype=np.float32
        )
        
        # 7 discrete actions:
        # 0: Move to Fiora
        # 1: Auto Attack Fiora
        # 2: Cast Q (Slot 0)
        # 3: Cast W (Slot 1)
        # 4: Cast E (Slot 2)
        # 5: Cast R (Slot 3)
        # 6: Stop
        self.action_space = spaces.Discrete(7)
        
        # Internal state variables
        self.fiora_id = None
        self.fiora_pos = [0.0, 0.0]
        self.fiora_hp = 620.0
        self.distance = 1800.0
        self.myself_hp = 630.0
        self.myself_pos = [0.0, 0.0]
        self.current_step = 0
        self.controlled_entity_id = None
        self.has_seen_fiora = False

    def _start_background_loop(self):
        asyncio.set_event_loop(self.loop)
        self.loop.run_forever()

    def _run_async(self, coro):
        future = asyncio.run_coroutine_threadsafe(coro, self.loop)
        return future.result()

    async def _close_conn_async(self):
        if self.websocket:
            try:
                await self.websocket.close()
            except Exception:
                pass
            self.websocket = None

    def _close_conn(self):
        self._run_async(self._close_conn_async())

    def close(self):
        self._close_conn()
        if self.proc:
            try:
                self.proc.terminate()
                self.proc.wait(timeout=2.0)
            except Exception:
                try:
                    self.proc.kill()
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
            if resp.get("ok"):
                obs_data = resp.get("data")
                myself = obs_data.get("myself", {})
                level = myself.get("level", 1)
                if level == 18:
                    # We are Riven by default!
                    self.controlled_entity_id = None
                else:
                    # We are Fiora, so Riven is in enemy_heroes
                    enemies = obs_data.get("enemy_heroes", [])
                    if len(enemies) > 0:
                        self.controlled_entity_id = enemies[0].get("entity")
                break
            await asyncio.sleep(0.5)

    def _connect_and_wait(self):
        self._run_async(self._connect_and_wait_async())

    def _level_up_skills(self):
        # Q W E R upgrades
        upgrades = [0]*5 + [1]*5 + [2]*5 + [3]*3
        for idx in upgrades:
            params = {
                "action": {"SkillLevelUp": idx}
            }
            if self.controlled_entity_id is not None:
                params["entity_id"] = self.controlled_entity_id
            req = {
                "id": self.req_id,
                "cmd": "action",
                "params": params
            }
            self.req_id += 1
            self._send_and_recv(req)

    def _get_obs_data(self):
        params = {}
        if self.controlled_entity_id is not None:
            params["entity_id"] = self.controlled_entity_id
        req = {"id": self.req_id, "cmd": "get_observe", "params": params}
        self.req_id += 1
        resp = self._send_and_recv(req)
        if resp.get("ok"):
            return resp.get("data")
        return None

    def _get_obs_vector(self, obs_data):
        if not obs_data:
            return np.zeros((11,), dtype=np.float32)

        myself = obs_data.get("myself", {})
        enemies = obs_data.get("enemy_heroes", [])
        
        myself_pos = myself.get("position", [0.0, 0.0])
        myself_hp = myself.get("health", 630.0)
        myself_max_hp = myself.get("max_health", 630.0)
        myself_hp_ratio = myself_hp / myself_max_hp if myself_max_hp > 0 else 1.0
        
        # Skill cooldowns
        skills = myself.get("skills", [])
        q_cd, w_cd, e_cd, r_cd = 0.0, 0.0, 0.0, 0.0
        for s in skills:
            idx = s.get("index")
            cd = s.get("cooldown_remaining")
            if cd is not None:
                if idx == 0: q_cd = cd
                elif idx == 1: w_cd = cd
                elif idx == 2: e_cd = cd
                elif idx == 3: r_cd = cd
                
        # Attack state
        attack_state = myself.get("attack_state")
        is_windup = 0.0
        is_cooldown = 0.0
        if attack_state is not None:
            status = attack_state.get("status", {})
            if "Windup" in status:
                is_windup = 1.0
            elif "Cooldown" in status:
                is_cooldown = 1.0
                
        # Enemy Fiora
        fiora_id = None
        fiora_pos = self.fiora_pos
        fiora_hp_ratio = self.fiora_hp / 620.0
        distance = self.distance
        
        if len(enemies) > 0:
            fiora = enemies[0]
            fiora_id = fiora.get("entity")
            fiora_pos = fiora.get("position", [0.0, 0.0])
            fiora_hp = fiora.get("health", 0.0)
            fiora_max_hp = fiora.get("max_health", 620.0)
            fiora_hp_ratio = fiora_hp / fiora_max_hp if fiora_max_hp > 0 else 0.0
            distance = fiora.get("distance", 1800.0)
            self.fiora_hp = fiora_hp
            self.has_seen_fiora = True
        else:
            if self.has_seen_fiora:
                self.fiora_hp = 0.0
                fiora_hp_ratio = 0.0
            else:
                self.fiora_hp = 620.0
                fiora_hp_ratio = 1.0
            
        dx = fiora_pos[0] - myself_pos[0]
        dy = fiora_pos[1] - myself_pos[1]
        
        # Save state for step reward
        self.fiora_id = fiora_id
        self.fiora_pos = fiora_pos
        self.distance = distance
        self.myself_hp = myself_hp
        self.myself_pos = myself_pos
        
        return np.array([
            dx, dy, distance,
            myself_hp_ratio, fiora_hp_ratio,
            q_cd, w_cd, e_cd, r_cd,
            is_windup, is_cooldown
        ], dtype=np.float32)

    def reset(self, seed=None, options=None):
        super().reset(seed=seed)
        self.close()
        self.controlled_entity_id = None
        self.has_seen_fiora = False
        
        # Start Bevy
        self.proc = subprocess.Popen([
            "cargo", "run", "--",
            "--headless",
            "--scene", "games/headless-env-kill-fiora.ron",
            "--ws-port", str(self.port)
        ], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        
        # Connect & Level Up
        self._connect_and_wait()
        self._level_up_skills()
        
        self.current_step = 0
        obs_data = self._get_obs_data()
        obs_vec = self._get_obs_vector(obs_data)
        return obs_vec, {}

    def _send_action(self, action_idx):
        if self.fiora_id is None:
            return
            
        if action_idx == 0:
            action_payload = {"Move": self.fiora_pos}
        elif action_idx == 1:
            action_payload = {"Attack": self.fiora_id}
        elif action_idx == 2:
            action_payload = {"Skill": {"index": 0, "point": self.fiora_pos}}
        elif action_idx == 3:
            action_payload = {"Skill": {"index": 1, "point": self.myself_pos}}
        elif action_idx == 4:
            action_payload = {"Skill": {"index": 2, "point": self.fiora_pos}}
        elif action_idx == 5:
            action_payload = {"Skill": {"index": 3, "point": self.fiora_pos}}
        elif action_idx == 6:
            action_payload = "Stop"
        else:
            return
            
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
        self._send_and_recv(req)

    def step(self, action):
        prev_fiora_hp = self.fiora_hp
        prev_distance = self.distance
        
        # Send action
        self._send_action(action)
        
        # Tick the environment (real-time sleep)
        time.sleep(0.1)
        
        # Get new observation
        obs_data = self._get_obs_data()
        obs_vec = self._get_obs_vector(obs_data)
        
        self.current_step += 1
        
        # Reward design
        reward = 0.0
        
        # 1. Damage reward
        damage_dealt = prev_fiora_hp - self.fiora_hp
        if damage_dealt > 0:
            reward += damage_dealt * 2.0  # Encourage doing damage
            
        # 2. Distance reward (shaping)
        dist_diff = prev_distance - self.distance
        reward += dist_diff * 0.5  # Encourage getting closer
        
        # 3. Time penalty
        reward -= 1.0  # Encourage fast completion
        
        # Terminals
        terminated = False
        truncated = False
        
        if self.fiora_hp <= 0:
            terminated = True
            reward += 5000.0  # Large kill bonus!
            print(f"Fiora killed in {self.current_step} steps!")
            
        if self.current_step >= self.max_steps:
            truncated = True
            
        return obs_vec, reward, terminated, truncated, {}
