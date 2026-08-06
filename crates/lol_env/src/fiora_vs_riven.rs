use std::path::PathBuf;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_champions::fiora::passive::Vital;
use lol_champions::fiora::{Fiora, PluginFiora};
use lol_champions::riven::{PluginRiven, Riven};
use lol_core::action::{Action, CommandAction};
use lol_core::base::direction::Direction;
use lol_core::life::Health;
use lol_core::skill::{Skill, Skills};
use lol_core::team::Team;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FioraVsRivenAction {
    MoveEast50 = 0,  // Stand 50u East (+X relative to Riven)
    MoveWest50 = 1,  // Stand 50u West (-X relative to Riven)
    MoveNorth50 = 2, // Stand 50u North (+Z relative to Riven)
    MoveSouth50 = 3, // Stand 50u South (-Z relative to Riven)
    AttackRiven = 4,
    CastQ = 5,
    CastW = 6,
    CastE = 7,
    CastR = 8,
}

impl FioraVsRivenAction {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::MoveEast50,
            1 => Self::MoveWest50,
            2 => Self::MoveNorth50,
            3 => Self::MoveSouth50,
            4 => Self::AttackRiven,
            5 => Self::CastQ,
            6 => Self::CastW,
            7 => Self::CastE,
            8 => Self::CastR,
            _ => Self::MoveEast50,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::MoveEast50 => "MoveEast50 (东侧 50u 站位)",
            Self::MoveWest50 => "MoveWest50 (西侧 50u 站位)",
            Self::MoveNorth50 => "MoveNorth50 (北侧 50u 站位)",
            Self::MoveSouth50 => "MoveSouth50 (南侧 50u 站位)",
            Self::AttackRiven => "AttackRiven (普通攻击 瑞雯)",
            Self::CastQ => "CastQ (Q: 破空斩)",
            Self::CastW => "CastW (W: 劳伦特心眼刀)",
            Self::CastE => "CastE (E: 夺命连刺)",
            Self::CastR => "CastR (R: 无双挑战)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FioraVsRivenObs {
    pub fiora_pos: Vec3,
    pub fiora_hp: f32,
    pub fiora_max_hp: f32,
    pub riven_pos: Vec3,
    pub riven_hp: f32,
    pub riven_max_hp: f32,
    pub distance: f32,
    pub q_ready: bool,
    pub w_ready: bool,
    pub e_ready: bool,
    pub r_ready: bool,
    // Vital observation
    pub has_vital: bool,
    pub vital_is_active: bool,
    pub vital_dir_x: f32,
    pub vital_dir_neg_x: f32,
    pub vital_dir_z: f32,
    pub vital_dir_neg_z: f32,
}

impl FioraVsRivenObs {
    pub fn to_vector(&self) -> Vec<f32> {
        vec![
            self.fiora_pos.x / 1000.0,
            self.fiora_pos.z / 1000.0,
            if self.fiora_max_hp > 0.0 {
                self.fiora_hp / self.fiora_max_hp
            } else {
                0.0
            },
            self.riven_pos.x / 1000.0,
            self.riven_pos.z / 1000.0,
            if self.riven_max_hp > 0.0 {
                self.riven_hp / self.riven_max_hp
            } else {
                0.0
            },
            self.distance / 1000.0,
            if self.q_ready { 1.0 } else { 0.0 },
            if self.w_ready { 1.0 } else { 0.0 },
            if self.e_ready { 1.0 } else { 0.0 },
            if self.r_ready { 1.0 } else { 0.0 },
            if self.has_vital { 1.0 } else { 0.0 },
            if self.vital_is_active { 1.0 } else { 0.0 },
            self.vital_dir_x,
            self.vital_dir_neg_x,
            self.vital_dir_z,
            self.vital_dir_neg_z,
        ]
    }

    pub fn dim() -> usize {
        17
    }
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub obs: FioraVsRivenObs,
    pub reward: f32,
    pub terminated: bool,
    pub truncated: bool,
    pub step: usize,
    pub reward_breakdown: Vec<RewardBreakdownItem>,
}

#[derive(Debug, Clone)]
pub struct RewardBreakdownItem {
    pub name: String,
    pub value: f32,
}

pub struct FioraVsRivenEnv {
    app: App,
    fiora: Entity,
    riven: Entity,
    step_count: usize,
    max_steps: usize,
    initial_fiora_pos: Vec3,
    initial_riven_pos: Vec3,
}

impl FioraVsRivenEnv {
    pub fn new(max_steps: usize) -> Self {
        Self::new_with_render(max_steps, false)
    }

    pub fn new_with_render(max_steps: usize, render: bool) -> Self {
        let mut app = App::new();

        // High CPU throughput configuration per docs/game/facts/bevy.md:
        // FixedTimesteps(1) with app.update() for exact stepping
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
        let workspace_root = PathBuf::from(&manifest_dir)
            .parent()
            .map(|p| p.parent())
            .flatten()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(&manifest_dir));

        let asset_plugin = bevy::asset::AssetPlugin {
            file_path: workspace_root.join("assets").to_string_lossy().to_string(),
            ..Default::default()
        };

        if render {
            app.add_plugins(DefaultPlugins.set(asset_plugin).set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Fiora vs Riven RL Evaluation (Render Mode)".to_string(),
                    resolution: (1280, 720).into(),
                    ..Default::default()
                }),
                ..Default::default()
            }));
            app.add_plugins(lol_render::PluginRender);
            app.add_plugins(lol_core::PluginCore);
            app.add_plugins(lol_particle::PluginParticle);
        } else {
            app.add_plugins((
                MinimalPlugins.set(ScheduleRunnerPlugin::run_once()),
                asset_plugin,
                bevy::world_serialization::WorldSerializationPlugin,
            ));
            app.add_plugins(lol_core::PluginCore);
        }

        app.add_plugins(PluginFiora);
        app.add_plugins(PluginRiven);

        app.insert_resource(lol_base::map::MapPaths::new("test"));

        app.finish();
        app.cleanup();

        let asset_server = app.world().resource::<AssetServer>();
        let fiora_config_handle = asset_server.load::<DynamicWorld>("characters/fiora/config.ron");
        let riven_config_handle = asset_server.load::<DynamicWorld>("characters/Riven/config.ron");

        let fiora_skin_handle = if render {
            Some(asset_server.load::<DynamicWorld>("characters/fiora/skins/skin0.ron"))
        } else {
            None
        };
        let riven_skin_handle = if render {
            Some(asset_server.load::<DynamicWorld>("characters/Riven/skins/skin0.ron"))
        } else {
            None
        };

        let initial_fiora_pos = Vec3::ZERO;
        let initial_riven_pos = Vec3::new(250.0, 0.0, 0.0);

        // Spawn Level 6 Fiora (Order team)
        let mut fiora_builder = app.world_mut().spawn((
            Fiora::default(),
            Transform::from_translation(initial_fiora_pos),
            Team::Order,
            ConfigCharacterRecord {
                character_record: fiora_config_handle.clone(),
            },
            Health::new(500.0),
            lol_core::damage::Armor(35.0),
            lol_core::movement::Movement { speed: 345.0 },
        ));

        if render {
            fiora_builder.insert((
                lol_render::controller::SelfPlayer,
                lol_base_render::camera::Focus,
                ConfigSkin {
                    skin: fiora_skin_handle.clone().unwrap(),
                },
            ));
        }

        let fiora = fiora_builder.id();

        // Spawn Level 6 Riven (Chaos team)
        let mut riven_builder = app.world_mut().spawn((
            Riven::default(),
            Transform::from_translation(initial_riven_pos),
            Team::Chaos,
            ConfigCharacterRecord {
                character_record: riven_config_handle.clone(),
            },
            Health::new(500.0),
            lol_core::damage::Armor(33.0),
            lol_core::movement::Movement { speed: 340.0 },
        ));

        if render {
            riven_builder.insert(ConfigSkin {
                skin: riven_skin_handle.clone().unwrap(),
            });
        }

        let riven = riven_builder.id();

        // Wait for config and skin assets to load completely
        for _ in 0..500 {
            let asset_server = app.world().resource::<AssetServer>();
            let fiora_ready = if render {
                asset_server
                    .get_recursive_dependency_load_state(&fiora_skin_handle.clone().unwrap())
                    .is_some_and(|s| s.is_loaded())
            } else {
                asset_server
                    .get_recursive_dependency_load_state(&fiora_config_handle)
                    .is_some_and(|s| s.is_loaded())
            };
            let riven_ready = if render {
                asset_server
                    .get_recursive_dependency_load_state(&riven_skin_handle.clone().unwrap())
                    .is_some_and(|s| s.is_loaded())
            } else {
                asset_server
                    .get_recursive_dependency_load_state(&riven_config_handle)
                    .is_some_and(|s| s.is_loaded())
            };

            if fiora_ready && riven_ready {
                break;
            }
            app.update();
        }

        let mut env = Self {
            app,
            fiora,
            riven,
            step_count: 0,
            max_steps,
            initial_fiora_pos,
            initial_riven_pos,
        };

        env.setup_champion_skill_levels();
        env
    }

    /// Sets Fiora and Riven to Level 6 with Q3, W1, E1, R1
    fn setup_champion_skill_levels(&mut self) {
        // Fiora skills: Q Level 3, W Level 1, E Level 1, R Level 1
        if let Some(skills) = self.app.world().get::<Skills>(self.fiora) {
            let skill_entities = skills.to_vec();
            if skill_entities.len() >= 4 {
                if let Some(mut q) = self.app.world_mut().get_mut::<Skill>(skill_entities[0]) {
                    q.level = 3;
                }
                if let Some(mut w) = self.app.world_mut().get_mut::<Skill>(skill_entities[1]) {
                    w.level = 1;
                }
                if let Some(mut e) = self.app.world_mut().get_mut::<Skill>(skill_entities[2]) {
                    e.level = 1;
                }
                if let Some(mut r) = self.app.world_mut().get_mut::<Skill>(skill_entities[3]) {
                    r.level = 1;
                }
            }
        }

        // Riven skills: Q Level 3, W Level 1, E Level 1, R Level 1
        if let Some(skills) = self.app.world().get::<Skills>(self.riven) {
            let skill_entities = skills.to_vec();
            if skill_entities.len() >= 4 {
                if let Some(mut q) = self.app.world_mut().get_mut::<Skill>(skill_entities[0]) {
                    q.level = 3;
                }
                if let Some(mut w) = self.app.world_mut().get_mut::<Skill>(skill_entities[1]) {
                    w.level = 1;
                }
                if let Some(mut e) = self.app.world_mut().get_mut::<Skill>(skill_entities[2]) {
                    e.level = 1;
                }
                if let Some(mut r) = self.app.world_mut().get_mut::<Skill>(skill_entities[3]) {
                    r.level = 1;
                }
            }
        }
    }

    pub fn reset(&mut self) -> FioraVsRivenObs {
        self.step_count = 0;

        // Reset positions
        if let Some(mut transform) = self.app.world_mut().get_mut::<Transform>(self.fiora) {
            transform.translation = self.initial_fiora_pos;
        }
        if let Some(mut transform) = self.app.world_mut().get_mut::<Transform>(self.riven) {
            transform.translation = self.initial_riven_pos;
        }

        // Reset health
        if let Some(mut hp) = self.app.world_mut().get_mut::<Health>(self.fiora) {
            hp.value = if hp.max > 0.0 { hp.max } else { 500.0 };
        }
        if let Some(mut hp) = self.app.world_mut().get_mut::<Health>(self.riven) {
            hp.value = if hp.max > 0.0 { hp.max } else { 500.0 };
        }

        // Reset skill cooldowns
        if let Some(skills) = self.app.world().get::<Skills>(self.fiora) {
            let skill_entities = skills.to_vec();
            for s_entity in skill_entities {
                if let Some(mut cd) = self
                    .app
                    .world_mut()
                    .get_mut::<lol_core::skill::CoolDown>(s_entity)
                {
                    cd.timer = None;
                }
            }
        }

        // Reset Vital state on Riven
        self.app
            .world_mut()
            .entity_mut(self.riven)
            .remove::<Vital>();

        self.get_obs()
    }

    pub fn get_obs(&self) -> FioraVsRivenObs {
        let fiora_transform = self.app.world().get::<Transform>(self.fiora);
        let fiora_hp = self.app.world().get::<Health>(self.fiora);
        let riven_transform = self.app.world().get::<Transform>(self.riven);
        let riven_hp = self.app.world().get::<Health>(self.riven);

        let fpos = fiora_transform.map(|t| t.translation).unwrap_or_default();
        let rpos = riven_transform.map(|t| t.translation).unwrap_or_default();

        let dist = fpos.distance(rpos);

        // Query Vital component on Riven
        let vital = self.app.world().get::<Vital>(self.riven);
        let (has_vital, vital_is_active, v_x, v_neg_x, v_z, v_neg_z) = match vital {
            Some(v) => {
                let (vx, vnx, vz, vnz) = match v.direction {
                    Direction::X => (1.0, 0.0, 0.0, 0.0),
                    Direction::NegX => (0.0, 1.0, 0.0, 0.0),
                    Direction::Z => (0.0, 0.0, 1.0, 0.0),
                    Direction::NegZ => (0.0, 0.0, 0.0, 1.0),
                };
                (true, v.is_active(), vx, vnx, vz, vnz)
            }
            None => (false, false, 0.0, 0.0, 0.0, 0.0),
        };

        let (q_ready, w_ready, e_ready, r_ready) = {
            let mut ready = (true, true, true, true);
            if let Some(skills) = self.app.world().get::<Skills>(self.fiora) {
                let skill_entities = skills.to_vec();
                let check_ready = |idx: usize| -> bool {
                    if idx < skill_entities.len() {
                        let s_entity = skill_entities[idx];
                        let cd = self.app.world().get::<lol_core::skill::CoolDown>(s_entity);
                        let recast = self
                            .app
                            .world()
                            .get::<lol_core::skill::SkillRecastWindow>(s_entity);
                        match cd {
                            Some(c) => lol_core::skill::is_skill_ready(c, recast),
                            None => true,
                        }
                    } else {
                        true
                    }
                };
                ready = (
                    check_ready(0),
                    check_ready(1),
                    check_ready(2),
                    check_ready(3),
                );
            }
            ready
        };

        FioraVsRivenObs {
            fiora_pos: fpos,
            fiora_hp: fiora_hp.map(|h| h.value).unwrap_or(0.0),
            fiora_max_hp: fiora_hp.map(|h| h.max).unwrap_or(500.0),
            riven_pos: rpos,
            riven_hp: riven_hp.map(|h| h.value).unwrap_or(0.0),
            riven_max_hp: riven_hp.map(|h| h.max).unwrap_or(500.0),
            distance: dist,
            q_ready,
            w_ready,
            e_ready,
            r_ready,
            has_vital,
            vital_is_active,
            vital_dir_x: v_x,
            vital_dir_neg_x: v_neg_x,
            vital_dir_z: v_z,
            vital_dir_neg_z: v_neg_z,
        }
    }

    /// Advances the environment by 1 timestep with given action
    pub fn step(&mut self, action: FioraVsRivenAction) -> StepResult {
        self.step_count += 1;

        let prev_riven_hp = self
            .app
            .world()
            .get::<Health>(self.riven)
            .map(|h| h.value)
            .unwrap_or(0.0);
        let prev_fiora_hp = self
            .app
            .world()
            .get::<Health>(self.fiora)
            .map(|h| h.value)
            .unwrap_or(0.0);
        let prev_fpos = self
            .app
            .world()
            .get::<Transform>(self.fiora)
            .map(|t| t.translation)
            .unwrap_or_default();
        let riven_pos = self
            .app
            .world()
            .get::<Transform>(self.riven)
            .map(|t| t.translation)
            .unwrap_or(self.initial_riven_pos);
        let riven_pos_2d = Vec2::new(riven_pos.x, riven_pos.z);

        // Dispatch Fiora action
        match action {
            FioraVsRivenAction::MoveEast50 => {
                if let Some(mut t) = self.app.world_mut().get_mut::<Transform>(self.fiora) {
                    t.translation = Vec3::new(riven_pos.x + 50.0, riven_pos.y, riven_pos.z);
                }
            }
            FioraVsRivenAction::MoveWest50 => {
                if let Some(mut t) = self.app.world_mut().get_mut::<Transform>(self.fiora) {
                    t.translation = Vec3::new(riven_pos.x - 50.0, riven_pos.y, riven_pos.z);
                }
            }
            FioraVsRivenAction::MoveNorth50 => {
                if let Some(mut t) = self.app.world_mut().get_mut::<Transform>(self.fiora) {
                    t.translation = Vec3::new(riven_pos.x, riven_pos.y, riven_pos.z + 50.0);
                }
            }
            FioraVsRivenAction::MoveSouth50 => {
                if let Some(mut t) = self.app.world_mut().get_mut::<Transform>(self.fiora) {
                    t.translation = Vec3::new(riven_pos.x, riven_pos.y, riven_pos.z - 50.0);
                }
            }
            FioraVsRivenAction::AttackRiven => {
                self.app.world_mut().trigger(CommandAction {
                    entity: self.fiora,
                    action: Action::Attack(self.riven),
                });
            }
            FioraVsRivenAction::CastQ => {
                self.app.world_mut().trigger(CommandAction {
                    entity: self.fiora,
                    action: Action::Skill {
                        index: 0,
                        point: riven_pos_2d,
                    },
                });
            }
            FioraVsRivenAction::CastW => {
                self.app.world_mut().trigger(CommandAction {
                    entity: self.fiora,
                    action: Action::Skill {
                        index: 1,
                        point: riven_pos_2d,
                    },
                });
            }
            FioraVsRivenAction::CastE => {
                self.app.world_mut().trigger(CommandAction {
                    entity: self.fiora,
                    action: Action::Skill {
                        index: 2,
                        point: riven_pos_2d,
                    },
                });
            }
            FioraVsRivenAction::CastR => {
                self.app.world_mut().trigger(CommandAction {
                    entity: self.fiora,
                    action: Action::Skill {
                        index: 3,
                        point: riven_pos_2d,
                    },
                });
            }
        }

        // Execute Bevy update step with frame_skip = 10
        for _ in 0..10 {
            self.app.update();
        }

        let obs = self.get_obs();
        let curr_riven_hp = obs.riven_hp;
        let curr_fiora_hp = obs.fiora_hp;

        let (reward, reward_breakdown) = compute_step_reward(
            prev_riven_hp,
            prev_fiora_hp,
            curr_riven_hp,
            curr_fiora_hp,
            prev_fpos,
            riven_pos,
            action,
            self.step_count,
            self.max_steps,
        );

        let terminated = curr_riven_hp <= 0.0 || curr_fiora_hp <= 0.0;
        let truncated = self.step_count >= self.max_steps;

        StepResult {
            obs,
            reward,
            terminated,
            truncated,
            step: self.step_count,
            reward_breakdown,
        }
    }
}

/// Compute step reward and its breakdown items.
/// Extracted to be shared between `FioraVsRivenEnv::step()` and `visual_loop.rs`.
pub fn compute_step_reward(
    prev_riven_hp: f32,
    prev_fiora_hp: f32,
    curr_riven_hp: f32,
    curr_fiora_hp: f32,
    prev_fpos: Vec3,
    riven_pos: Vec3,
    action: FioraVsRivenAction,
    step_count: usize,
    max_steps: usize,
) -> (f32, Vec<RewardBreakdownItem>) {
    let damage_dealt = (prev_riven_hp - curr_riven_hp).max(0.0);
    let damage_taken = (prev_fiora_hp - curr_fiora_hp).max(0.0);

    // Repeat move penalty
    let is_repeat_move = match action {
        FioraVsRivenAction::MoveEast50 => {
            (prev_fpos.x - (riven_pos.x + 50.0)).abs() < 5.0
                && (prev_fpos.z - riven_pos.z).abs() < 5.0
        }
        FioraVsRivenAction::MoveWest50 => {
            (prev_fpos.x - (riven_pos.x - 50.0)).abs() < 5.0
                && (prev_fpos.z - riven_pos.z).abs() < 5.0
        }
        FioraVsRivenAction::MoveNorth50 => {
            (prev_fpos.x - riven_pos.x).abs() < 5.0
                && (prev_fpos.z - (riven_pos.z + 50.0)).abs() < 5.0
        }
        FioraVsRivenAction::MoveSouth50 => {
            (prev_fpos.x - riven_pos.x).abs() < 5.0
                && (prev_fpos.z - (riven_pos.z - 50.0)).abs() < 5.0
        }
        _ => false,
    };
    let repeat_move_penalty = if is_repeat_move { -5.0 } else { 0.0 };

    // Close move penalty
    let prev_dist = prev_fpos.distance(riven_pos);
    let is_movement_action = matches!(
        action,
        FioraVsRivenAction::MoveEast50
            | FioraVsRivenAction::MoveWest50
            | FioraVsRivenAction::MoveNorth50
            | FioraVsRivenAction::MoveSouth50
    );
    let close_move_penalty = if prev_dist < 80.0 && is_movement_action {
        -3.0
    } else {
        0.0
    };

    // Skill bonus
    let is_skill_action = matches!(
        action,
        FioraVsRivenAction::CastQ
            | FioraVsRivenAction::CastW
            | FioraVsRivenAction::CastE
            | FioraVsRivenAction::CastR
    );
    let skill_bonus = if is_skill_action { 50.0 } else { 0.0 };

    // Vital break bonus
    let vital_break_bonus = if damage_dealt > 25.0 { 300.0 } else { 0.0 };

    // Step penalty
    let step_penalty = -0.2;

    // Speed-boosted combat reward
    let steps_left = (max_steps - step_count) as f32;
    let speed_multiplier = 1.0 + steps_left.max(0.0) / 100.0 * 2.0;
    let damage_reward = damage_dealt * 4.0 * speed_multiplier;

    let mut breakdown = vec![
        RewardBreakdownItem {
            name: "伤害造成 (Damage Dealt)".to_string(),
            value: damage_reward,
        },
        RewardBreakdownItem {
            name: "破绽打击 (Vital Break)".to_string(),
            value: vital_break_bonus,
        },
        RewardBreakdownItem {
            name: "技能施放奖励 (Skill Bonus)".to_string(),
            value: skill_bonus,
        },
        RewardBreakdownItem {
            name: "重复移动惩罚 (Repeat Move Penalty)".to_string(),
            value: repeat_move_penalty,
        },
        RewardBreakdownItem {
            name: "贴脸走位惩罚 (Close Move Penalty)".to_string(),
            value: close_move_penalty,
        },
        RewardBreakdownItem {
            name: "受到伤害 (Damage Taken)".to_string(),
            value: -damage_taken * 0.1,
        },
        RewardBreakdownItem {
            name: "每步消耗 (Step Penalty)".to_string(),
            value: step_penalty,
        },
    ];

    let mut reward =
        damage_reward + vital_break_bonus + skill_bonus + repeat_move_penalty + close_move_penalty
            - damage_taken * 0.1
            + step_penalty;

    if curr_riven_hp <= 0.0 {
        let speed_kill_bonus = steps_left.max(0.0) * 10.0;
        reward += 500.0 + speed_kill_bonus;
        breakdown.push(RewardBreakdownItem {
            name: "击杀瑞雯 (Solo Kill)".to_string(),
            value: 500.0 + speed_kill_bonus,
        });
    } else if curr_fiora_hp <= 0.0 {
        reward -= 100.0;
        breakdown.push(RewardBreakdownItem {
            name: "剑姬阵亡 (Death Penalty)".to_string(),
            value: -100.0,
        });
    }

    (reward, breakdown)
}
