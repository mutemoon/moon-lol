import gymnasium as gym
from gymnasium import spaces
import numpy as np
import time

from sdk import MoonLoLClient

class MoonLoLEnv(gym.Env):
    metadata = {"render_modes": []}

    def __init__(self, port=9030, max_steps=400):
        super().__init__()
        self.port = port
        self.max_steps = max_steps
        self.client = MoonLoLClient(port=port)
        
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
        self.has_seen_fiora = False

    def close(self):
        self.client.close()

    def _level_up_skills(self):
        upgrades = [0]*5 + [1]*5 + [2]*5 + [3]*3
        for idx in upgrades:
            self.client.level_up_skill(idx)

    def _get_obs_data(self):
        return self.client.get_observe()

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
            if cd is None:
                continue
            if idx == 0:
                q_cd = cd
            elif idx == 1:
                w_cd = cd
            elif idx == 2:
                e_cd = cd
            elif idx == 3:
                r_cd = cd
                
        # Attack state
        attack_state = myself.get("attack_state") or {}
        status = attack_state.get("status", {})
        is_windup = 1.0 if "Windup" in status else 0.0
        is_cooldown = 1.0 if "Cooldown" in status else 0.0
                
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
        self.client.start()
        self._level_up_skills()
        
        self.current_step = 0
        reset_res = self.client.rl_reset()
        obs_data = reset_res.get("observation") if reset_res else None
        if obs_data is None:
            obs_data = self._get_obs_data()
        obs_vec = self._get_obs_vector(obs_data)
        return obs_vec, {}

    def _send_action(self, action_idx):
        if self.fiora_id is None:
            return
            
        if action_idx == 0:
            self.client.move(self.fiora_pos)
        elif action_idx == 1:
            self.client.attack(self.fiora_id)
        elif action_idx == 2:
            self.client.cast_skill(0, self.fiora_pos)
        elif action_idx == 3:
            self.client.cast_skill(1, self.myself_pos)
        elif action_idx == 4:
            self.client.cast_skill(2, self.fiora_pos)
        elif action_idx == 5:
            self.client.cast_skill(3, self.fiora_pos)
        elif action_idx == 6:
            self.client.stop()

    def step(self, action):
        prev_fiora_hp = self.fiora_hp
        prev_distance = self.distance
        
        # Send action
        self._send_action(action)
        
        # Precise step: advance Bevy by 6 frames (0.1s at 60Hz)
        step_res = self.client.rl_step(frames=6)
        obs_data = step_res.get("observation") if step_res else None
        if obs_data is None:
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
