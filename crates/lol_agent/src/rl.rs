//! Section 2 服务端 RL 接入（Bevy 引擎侧）。
//!
//! 提供三块能力，供外部 Python 训练守护进程（Gymnasium 客户端）经 WebSocket 高频驱动：
//!   1. [`MoonLoLEnv`]：Gym 风格的环境状态机，`reset()` / `step()` 维护回合状态并计算 reward；
//!   2. [`RewardShaper`]：解析 Agent `config_json` 中的 Reward Shaper 权重，按观测增量动态拼装最终 reward；
//!   3. msgpack 编解码 + base64 包装：obs/action 的高速二进制传递（[`pack_observe`] / [`unpack_action`] 等）。
//!
//! 真正的环境交互通过 `systems.rs` 的 WS 指令暴露：`rl_reset` / `rl_step` / `get_observe_packed` /
//! `action_packed`。本模块只承载可单测的纯逻辑与编解码，不直接持有 Bevy 查询。

use std::collections::HashMap;

use base64::Engine;
use bevy::prelude::{Entity, Resource};
use lol_core::action::Action;
use serde::{Deserialize, Serialize};

use crate::models::Observe;

/// 单回合默认最大步数（超出即 `truncated`，对应 Gym 的 time-limit）。
pub const DEFAULT_MAX_STEPS: u32 = 10_000;

// ════════════════════════ Reward Shaper ════════════════════════

/// Reward Shaper 权重表。可由 Agent `config_json` 的 `reward_shaper` / `reward` 节点反序列化，
/// 缺省字段回落到 [`Default`]（典型 LoL PPO 塑形）。
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct RewardWeights {
    /// 每个补刀（minion_kills 增量）。
    pub last_hit: f32,
    /// 每次击杀（kills 增量）。
    pub kill: f32,
    /// 每次死亡（deaths 增量，通常为负）。
    pub death: f32,
    /// 每次助攻（assists 增量）。
    pub assist: f32,
    /// 每点金币增量。
    pub gold: f32,
    /// 每级升级。
    pub level: f32,
    /// 生命值占比净变化（cur - prev 的 HP 比例）。
    pub health: f32,
    /// 每步时间惩罚（常量，通常为负）。
    pub time: f32,
    /// 与最近敌方英雄的接近度（0..1，越近越高），鼓励交战。
    pub proximity: f32,
}

impl Default for RewardWeights {
    fn default() -> Self {
        Self {
            last_hit: 1.0,
            kill: 5.0,
            death: -5.0,
            assist: 2.0,
            gold: 0.0,
            level: 1.0,
            health: 1.0,
            time: -0.001,
            proximity: 0.0,
        }
    }
}

/// 单项 reward 贡献（供客户端遥测「Reward 分项」可视化）。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RewardComponent {
    pub name: String,
    pub value: f32,
}

/// 一次 step 的 reward 拆解。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RewardBreakdown {
    pub components: Vec<RewardComponent>,
    pub total: f32,
}

impl RewardBreakdown {
    fn empty() -> Self {
        Self {
            components: Vec::new(),
            total: 0.0,
        }
    }
}

/// 按权重把观测增量动态拼装为 reward。
#[derive(Clone, Debug, Default)]
pub struct RewardShaper {
    pub weights: RewardWeights,
}

impl RewardShaper {
    pub fn new(weights: RewardWeights) -> Self {
        Self { weights }
    }

    /// 从 Agent 的 `config_json` 解析权重。支持 `reward_shaper` 或 `reward` 节点，二者皆无则用默认值。
    pub fn from_config_json(config: &serde_json::Value) -> Self {
        let node = config.get("reward_shaper").or_else(|| config.get("reward"));
        let weights = node
            .and_then(|v| serde_json::from_value::<RewardWeights>(v.clone()).ok())
            .unwrap_or_default();
        Self { weights }
    }

    /// 计算从 `prev` 到 `cur` 的 reward 分项。
    pub fn compute(&self, prev: &Observe, cur: &Observe) -> RewardBreakdown {
        let w = &self.weights;
        let pm = &prev.myself;
        let cm = &cur.myself;

        let delta_nonneg_u = |a: u32, b: u32| (b as i64 - a as i64).max(0) as f32;
        let prev_hpf = if pm.max_health > 0.0 {
            pm.health / pm.max_health
        } else {
            0.0
        };
        let cur_hpf = if cm.max_health > 0.0 {
            cm.health / cm.max_health
        } else {
            0.0
        };

        let proximity_factor = nearest_enemy_distance(cur)
            .map(|d| 1.0 - (d / 2000.0).clamp(0.0, 1.0))
            .unwrap_or(0.0);

        let raw = [
            (
                "last_hit",
                delta_nonneg_u(pm.minion_kills, cm.minion_kills) * w.last_hit,
            ),
            ("kill", delta_nonneg_u(pm.kills, cm.kills) * w.kill),
            ("death", delta_nonneg_u(pm.deaths, cm.deaths) * w.death),
            ("assist", delta_nonneg_u(pm.assists, cm.assists) * w.assist),
            ("gold", (cm.gold - pm.gold).max(0.0) * w.gold),
            ("level", delta_nonneg_u(pm.level, cm.level) * w.level),
            ("health", (cur_hpf - prev_hpf) * w.health),
            ("time", w.time),
            ("proximity", proximity_factor * w.proximity),
        ];

        let total = raw.iter().map(|(_, v)| *v).sum();
        let components = raw
            .into_iter()
            .map(|(name, value)| RewardComponent {
                name: name.to_string(),
                value,
            })
            .collect();

        RewardBreakdown { components, total }
    }
}

fn nearest_enemy_distance(obs: &Observe) -> Option<f32> {
    obs.enemy_heroes
        .iter()
        .map(|h| h.distance)
        .fold(None, |acc, d| Some(acc.map_or(d, |m: f32| m.min(d))))
}

// ════════════════════════ MoonLoLEnv ════════════════════════

/// 单个 step 的结果（Gym `step` 返回的 obs/reward/terminated/truncated/info 的服务端载体）。
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StepResult {
    pub reward: f32,
    pub breakdown: RewardBreakdown,
    /// 回合终止（如英雄死亡）。
    pub terminated: bool,
    /// 回合被截断（达到最大步数）。
    pub truncated: bool,
    pub step: u32,
}

/// Gym 风格环境：维护回合状态并据 [`RewardShaper`] 计算 reward。
///
/// 动作的施加由既有 action 系统完成（`action_packed` / `action` 指令），本结构聚焦
/// 观测 → reward 的转换与回合记账，对应 `MoonLoLEnv.reset()` / `step()` 语义。
#[derive(Clone)]
pub struct MoonLoLEnv {
    pub shaper: RewardShaper,
    pub last_obs: Option<Observe>,
    pub step_count: u32,
    pub max_steps: u32,
}

impl MoonLoLEnv {
    pub fn new(shaper: RewardShaper, max_steps: u32) -> Self {
        Self {
            shaper,
            last_obs: None,
            step_count: 0,
            max_steps,
        }
    }

    /// 重置回合：记录初始观测，步数清零。
    pub fn reset(&mut self, obs: Observe) {
        self.last_obs = Some(obs);
        self.step_count = 0;
    }

    /// 推进一步：以新观测对上一观测计算 reward，并判定终止/截断。
    pub fn step(&mut self, obs: Observe) -> StepResult {
        self.step_count += 1;
        let breakdown = match &self.last_obs {
            Some(prev) => self.shaper.compute(prev, &obs),
            None => RewardBreakdown::empty(),
        };
        let terminated = obs.myself.health <= 0.0;
        let truncated = self.max_steps > 0 && self.step_count >= self.max_steps;
        self.last_obs = Some(obs);
        StepResult {
            reward: breakdown.total,
            breakdown,
            terminated,
            truncated,
            step: self.step_count,
        }
    }
}

/// 对局内每个 RL 实体的环境状态（普通 `Resource`，`MoonLoLEnv` 为 `Send`）。
#[derive(Resource, Default)]
pub struct RlEnvs(pub HashMap<Entity, MoonLoLEnv>);

// ════════════════════════ msgpack / base64 编解码 ════════════════════════

/// 把观测编码为 msgpack（命名字段，便于 Python 侧按名读取）。
pub fn pack_observe(obs: &Observe) -> Result<Vec<u8>, String> {
    rmp_serde::to_vec_named(obs).map_err(|e| e.to_string())
}

/// 从 msgpack 解码观测。
pub fn unpack_observe(bytes: &[u8]) -> Result<Observe, String> {
    rmp_serde::from_slice(bytes).map_err(|e| e.to_string())
}

/// 把动作编码为 msgpack。
pub fn pack_action(action: &Action) -> Result<Vec<u8>, String> {
    rmp_serde::to_vec(action).map_err(|e| e.to_string())
}

/// 从 msgpack 解码动作。
pub fn unpack_action(bytes: &[u8]) -> Result<Action, String> {
    rmp_serde::from_slice(bytes).map_err(|e| e.to_string())
}

/// base64 编码（用于在文本 WS 通道上承载 msgpack 二进制）。
pub fn b64_encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// base64 解码。
pub fn b64_decode(s: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec2;

    use super::*;
    use crate::models::{ObserveHero, ObserveMyself};

    fn myself(
        minion_kills: u32,
        kills: u32,
        deaths: u32,
        assists: u32,
        health: f32,
    ) -> ObserveMyself {
        ObserveMyself {
            position: Vec2::new(0.0, 0.0),
            attack_state: None,
            run_target: None,
            health,
            max_health: 100.0,
            level: 1,
            ability_resource: None,
            attack_damage: 60.0,
            attack_range: 175.0,
            attack_speed: 0.6,
            armor: 30.0,
            skill_points: 1,
            skills: Vec::new(),
            gold: 0.0,
            kills,
            deaths,
            assists,
            minion_kills,
        }
    }

    fn obs(m: ObserveMyself, enemy_dist: Option<f32>) -> Observe {
        let enemy_heroes = enemy_dist
            .map(|d| {
                vec![ObserveHero {
                    entity: Entity::PLACEHOLDER,
                    position: Vec2::new(d, 0.0),
                    health: 100.0,
                    max_health: 100.0,
                    distance: d,
                }]
            })
            .unwrap_or_default();
        Observe {
            time: 0.0,
            myself: m,
            minions: Vec::new(),
            friendly_heroes: Vec::new(),
            enemy_heroes,
        }
    }

    #[test]
    fn weights_parse_partial_config() {
        let cfg = serde_json::json!({ "reward_shaper": { "kill": 8.0, "death": -3.0 } });
        let shaper = RewardShaper::from_config_json(&cfg);
        assert_eq!(shaper.weights.kill, 8.0);
        assert_eq!(shaper.weights.death, -3.0);
        // 未指定字段回落默认
        assert_eq!(shaper.weights.last_hit, RewardWeights::default().last_hit);
    }

    #[test]
    fn weights_default_when_absent() {
        let shaper = RewardShaper::from_config_json(&serde_json::json!({}));
        assert_eq!(shaper.weights, RewardWeights::default());
    }

    #[test]
    fn reward_sums_components() {
        let shaper = RewardShaper::default();
        let prev = obs(myself(10, 1, 0, 0, 100.0), None);
        // +2 补刀, +1 击杀, -50 血量
        let cur = obs(myself(12, 2, 0, 0, 50.0), None);
        let bd = shaper.compute(&prev, &cur);
        // last_hit 2*1=2, kill 1*5=5, health (0.5-1.0)*1=-0.5, time -0.001
        let by = |n: &str| bd.components.iter().find(|c| c.name == n).unwrap().value;
        assert!((by("last_hit") - 2.0).abs() < 1e-4);
        assert!((by("kill") - 5.0).abs() < 1e-4);
        assert!((by("health") - (-0.5)).abs() < 1e-4);
        let expected: f32 = bd.components.iter().map(|c| c.value).sum();
        assert!((bd.total - expected).abs() < 1e-4);
    }

    #[test]
    fn proximity_uses_nearest_enemy() {
        let shaper = RewardShaper::new(RewardWeights {
            proximity: 10.0,
            time: 0.0,
            ..RewardWeights::default()
        });
        let prev = obs(myself(0, 0, 0, 0, 100.0), Some(2000.0));
        let near = obs(myself(0, 0, 0, 0, 100.0), Some(0.0));
        let far = obs(myself(0, 0, 0, 0, 100.0), Some(2000.0));
        let by = |bd: &RewardBreakdown| {
            bd.components
                .iter()
                .find(|c| c.name == "proximity")
                .unwrap()
                .value
        };
        // 紧贴 → factor=1 → 10；最远 → factor≈0 → 0
        assert!((by(&shaper.compute(&prev, &near)) - 10.0).abs() < 1e-3);
        assert!(by(&shaper.compute(&prev, &far)).abs() < 1e-3);
    }

    #[test]
    fn env_reset_and_step_terminates_on_death() {
        let mut env = MoonLoLEnv::new(RewardShaper::default(), 100);
        env.reset(obs(myself(0, 0, 0, 0, 100.0), None));
        let alive = env.step(obs(myself(1, 0, 0, 0, 100.0), None));
        assert_eq!(alive.step, 1);
        assert!(!alive.terminated);
        let dead = env.step(obs(myself(1, 0, 1, 0, 0.0), None));
        assert!(dead.terminated);
        assert_eq!(dead.step, 2);
    }

    #[test]
    fn env_truncates_at_max_steps() {
        let mut env = MoonLoLEnv::new(RewardShaper::default(), 2);
        env.reset(obs(myself(0, 0, 0, 0, 100.0), None));
        assert!(!env.step(obs(myself(0, 0, 0, 0, 100.0), None)).truncated);
        assert!(env.step(obs(myself(0, 0, 0, 0, 100.0), None)).truncated);
    }

    #[test]
    fn observe_msgpack_round_trip() {
        let o = obs(myself(5, 1, 0, 2, 80.0), Some(300.0));
        let packed = pack_observe(&o).unwrap();
        let back = unpack_observe(&packed).unwrap();
        assert_eq!(back.myself.minion_kills, 5);
        assert_eq!(back.myself.assists, 2);
        assert_eq!(back.enemy_heroes.len(), 1);
        // base64 包装往返
        let b64 = b64_encode(&packed);
        assert_eq!(b64_decode(&b64).unwrap(), packed);
    }

    #[test]
    fn action_msgpack_round_trip() {
        let a = Action::Move(Vec2::new(12.0, -3.0));
        let packed = pack_action(&a).unwrap();
        match unpack_action(&packed).unwrap() {
            Action::Move(p) => {
                assert!((p.x - 12.0).abs() < 1e-4);
                assert!((p.y + 3.0).abs() < 1e-4);
            }
            other => panic!("expected Move, got {other:?}"),
        }
    }

    #[test]
    fn step_result_serializes_for_telemetry() {
        let mut env = MoonLoLEnv::new(RewardShaper::default(), 100);
        env.reset(obs(myself(0, 0, 0, 0, 100.0), None));
        let r = env.step(obs(myself(3, 0, 0, 0, 100.0), None));
        let v = serde_json::to_value(&r).unwrap();
        assert!(v.get("reward").is_some());
        assert!(
            v.get("breakdown")
                .and_then(|b| b.get("components"))
                .is_some()
        );
    }
}
