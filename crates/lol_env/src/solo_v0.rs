use std::collections::HashMap;

use bevy::prelude::*;
use lol_core::action::{Action, CommandAction};
use lol_core::base::stats::ChampionStats;
use lol_core::entities::minion::Minion;
use lol_core::life::Health;
use lol_core::team::Team;
use lol_rl_protocol::{ActionSpace, ObsFeaturePayload, RewardFormulaSpec, RewardTermSpec};

pub use crate::fiora_riven_common::{
    ATTACK_MASK_DISTANCE, AttackEventTracker, FioraRivenBaseEnv, FioraRivenEntities,
    setup_skill_levels_world, unpause_virtual_time,
};
pub use crate::flash_plugin::{
    FLASH_COOLDOWN_SECS, FLASH_DISTANCE, FlashCooldown, dispatch_flash, extract_flash_obs,
    register_flash_plugin, tick_flash_cooldown,
};
use crate::modifier_obs::{ModifierNameId, ModifierSlotObs, extract_entity_modifiers};
use crate::obs_plugins::{extract_attack_state, extract_champion_base, extract_skill_cds};
use crate::raycast_plugin::raycast_ground_plane;
use crate::traits::{EnvConfig, EnvMeta, RenderMode, RlEnvironment, StepResult, VisualEnvironment};

// ── 常量定义 ─────────────────────────────────────────────────────────────────

pub const SOLO_V0_OFFSET_SCALE: f32 = 100.0;
pub const SOLO_V0_OBS_DIM: usize = 60;
pub const SOLO_V0_OBS_DISTANCE_SCALE: f32 = 100.0;

// ── 初始化与重置 ─────────────────────────────────────────────────────────────

pub fn setup_solo_v0_health_world(world: &mut World, fiora: Entity, riven: Entity) {
    for champion in [fiora, riven] {
        if let Some(mut hp) = world.get_mut::<Health>(champion) {
            hp.value = hp.max;
        }
        if let Some(mut flash) = world.get_mut::<FlashCooldown>(champion) {
            flash.reset();
        } else {
            world.entity_mut(champion).insert(FlashCooldown::default());
        }
        if let Some(mut stats) = world.get_mut::<ChampionStats>(champion) {
            stats.kills = 0;
            stats.deaths = 0;
            stats.assists = 0;
            stats.minion_kills = 0;
        }
    }
}

// ── 动作空间 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SoloV0DiscreteAction {
    NoOp = 0,
    Move = 1,
    Attack = 2,
    CastQ = 3,
    CastW = 4,
    CastE = 5,
    CastR = 6,
    CastFlash = 7,
}

impl SoloV0DiscreteAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::NoOp,
            1 => Self::Move,
            2 => Self::Attack,
            3 => Self::CastQ,
            4 => Self::CastW,
            5 => Self::CastE,
            6 => Self::CastR,
            7 => Self::CastFlash,
            _ => Self::NoOp,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoloV0Action {
    pub offset_x: f32,
    pub offset_z: f32,
    pub discrete: SoloV0DiscreteAction,
}

impl SoloV0Action {
    pub const fn new(offset_x: f32, offset_z: f32, discrete: SoloV0DiscreteAction) -> Self {
        Self {
            offset_x,
            offset_z,
            discrete,
        }
    }

    pub fn from_encoding(encoded: &[f32]) -> Self {
        let offset_x = encoded.first().copied().unwrap_or(0.0);
        let offset_z = encoded.get(1).copied().unwrap_or(0.0);
        let discrete_idx = encoded.get(2).copied().unwrap_or(0.0) as u8;
        Self {
            offset_x,
            offset_z,
            discrete: SoloV0DiscreteAction::from_u8(discrete_idx),
        }
    }

    pub fn to_encoding(&self) -> Vec<f32> {
        vec![self.offset_x, self.offset_z, self.discrete.to_u8() as f32]
    }

    pub fn preset_from_index(index: usize) -> Self {
        match index {
            0 => Self::new(0.0, 0.0, SoloV0DiscreteAction::NoOp),
            1 => Self::new(0.5, 0.0, SoloV0DiscreteAction::Move),
            2 => Self::new(0.0, 0.0, SoloV0DiscreteAction::Attack),
            3 => Self::new(0.5, 0.0, SoloV0DiscreteAction::CastQ),
            4 => Self::new(0.0, 0.0, SoloV0DiscreteAction::CastW),
            5 => Self::new(0.0, 0.0, SoloV0DiscreteAction::CastE),
            6 => Self::new(0.0, 0.0, SoloV0DiscreteAction::CastR),
            7 => Self::new(1.0, 0.0, SoloV0DiscreteAction::CastFlash),
            _ => Self::new(0.0, 0.0, SoloV0DiscreteAction::NoOp),
        }
    }

    pub fn preset_index(&self) -> usize {
        match self.discrete {
            SoloV0DiscreteAction::NoOp => 0,
            SoloV0DiscreteAction::Move => 1,
            SoloV0DiscreteAction::Attack => 2,
            SoloV0DiscreteAction::CastQ => 3,
            SoloV0DiscreteAction::CastW => 4,
            SoloV0DiscreteAction::CastE => 5,
            SoloV0DiscreteAction::CastR => 6,
            SoloV0DiscreteAction::CastFlash => 7,
        }
    }

    pub fn desc(&self) -> &'static str {
        match self.discrete {
            SoloV0DiscreteAction::NoOp => "保持当前 (NoOp)",
            SoloV0DiscreteAction::Move => "移动",
            SoloV0DiscreteAction::Attack => "普通攻击",
            SoloV0DiscreteAction::CastQ => "施放 Q",
            SoloV0DiscreteAction::CastW => "施放 W",
            SoloV0DiscreteAction::CastE => "施放 E",
            SoloV0DiscreteAction::CastR => "施放 R",
            SoloV0DiscreteAction::CastFlash => "闪现",
        }
    }
}

// ── 自我中心化观测数据结构 ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SoloV0Obs {
    pub role_id: f32,

    pub self_pos: Vec3,
    pub self_hp: f32,
    pub self_max_hp: f32,
    pub target_pos: Vec3,
    pub target_hp: f32,
    pub target_max_hp: f32,
    pub distance: f32,

    pub attack_state: u8,
    pub attack_is_windup: bool,
    pub attack_is_cooldown: bool,
    pub attack_timer_remaining: f32,

    pub q_ready: bool,
    pub q_cd_remaining: f32,
    pub w_ready: bool,
    pub w_cd_remaining: f32,
    pub e_ready: bool,
    pub e_cd_remaining: f32,
    pub r_ready: bool,
    pub r_cd_remaining: f32,

    pub flash_ready: bool,
    pub flash_cd_remaining: f32,

    pub self_modifiers: Vec<ModifierSlotObs>,
    pub target_modifiers: Vec<ModifierSlotObs>,
}

impl SoloV0Obs {
    pub fn to_vector(&self) -> Vec<f32> {
        let rel_x = self.self_pos.x - self.target_pos.x;
        let rel_z = self.self_pos.z - self.target_pos.z;
        let b2f = |b: bool| if b { 1.0 } else { 0.0 };

        let mut v = Vec::with_capacity(SOLO_V0_OBS_DIM);

        // 1. 角色标识 (兼容 hero embedding)
        v.push(self.role_id);

        // 2. 空间相对特征 (3维)
        v.push(rel_x / SOLO_V0_OBS_DISTANCE_SCALE);
        v.push(rel_z / SOLO_V0_OBS_DISTANCE_SCALE);
        v.push(self.distance / SOLO_V0_OBS_DISTANCE_SCALE);

        // 3. 普攻状态机 (4维)
        v.push(b2f(self.attack_state == 0));
        v.push(b2f(self.attack_is_windup));
        v.push(b2f(self.attack_is_cooldown));
        v.push(self.attack_timer_remaining / 1.0);

        // 4. 技能与闪现冷却 (10维: Q, W, E, R, Flash)
        v.push(b2f(self.q_ready));
        v.push(self.q_cd_remaining / 10.0);
        v.push(b2f(self.w_ready));
        v.push(self.w_cd_remaining / 15.0);
        v.push(b2f(self.e_ready));
        v.push(self.e_cd_remaining / 10.0);
        v.push(b2f(self.r_ready));
        v.push(self.r_cd_remaining / 60.0);
        v.push(b2f(self.flash_ready));
        v.push(self.flash_cd_remaining / 300.0);

        // 5. 双方血量百分比 (2维)
        v.push(self.self_hp / self.self_max_hp.max(1.0));
        v.push(self.target_hp / self.target_max_hp.max(1.0));

        // 6. 自身修饰符 (4 槽位 × 5 = 20维)
        for i in 0..4 {
            if let Some(slot) = self.self_modifiers.get(i) {
                v.extend_from_slice(&slot.to_vector());
            } else {
                v.extend_from_slice(&[0.0; 5]);
            }
        }

        // 7. 目标修饰符 (4 槽位 × 5 = 20维)
        for i in 0..4 {
            if let Some(slot) = self.target_modifiers.get(i) {
                v.extend_from_slice(&slot.to_vector());
            } else {
                v.extend_from_slice(&[0.0; 5]);
            }
        }

        v
    }

    pub fn dim() -> usize {
        SOLO_V0_OBS_DIM
    }

    pub fn to_payload(&self) -> ObsFeaturePayload {
        let (fiora_hp, riven_hp, f_max, r_max) = if self.role_id < 0.5 {
            (
                self.self_hp,
                self.target_hp,
                self.self_max_hp,
                self.target_max_hp,
            )
        } else {
            (
                self.target_hp,
                self.self_hp,
                self.target_max_hp,
                self.self_max_hp,
            )
        };

        let primary_vital = self
            .target_modifiers
            .iter()
            .chain(self.self_modifiers.iter())
            .find(|m| m.name_id == ModifierNameId::FioraPassiveVital);
        let has_vital = primary_vital.is_some();
        let vital_is_active = primary_vital.map(|v| v.stack_count > 0.5).unwrap_or(false);
        let vital_dir = if let Some(v) = primary_vital {
            if v.param0 > 0.5 {
                "+X (东)".to_string()
            } else if v.param0 < -0.5 {
                "-X (西)".to_string()
            } else if v.param1 > 0.5 {
                "+Z (北)".to_string()
            } else if v.param1 < -0.5 {
                "-Z (南)".to_string()
            } else {
                "无".to_string()
            }
        } else {
            "无".to_string()
        };

        ObsFeaturePayload {
            fiora_hp_pct: if f_max > 0.0 { fiora_hp / f_max } else { 1.0 },
            riven_hp_pct: if r_max > 0.0 { riven_hp / r_max } else { 1.0 },
            distance: self.distance,
            q_ready: self.q_ready,
            w_ready: self.w_ready,
            e_ready: self.e_ready,
            r_ready: self.r_ready,
            has_vital,
            vital_is_active,
            vital_direction: vital_dir,
            tags: HashMap::from([
                (
                    "role".to_string(),
                    if self.role_id < 0.5 {
                        "剑姬 (Fiora)".to_string()
                    } else {
                        "瑞雯 (Riven)".to_string()
                    },
                ),
                ("q_cd".to_string(), format!("{:.1}s", self.q_cd_remaining)),
                ("w_cd".to_string(), format!("{:.1}s", self.w_cd_remaining)),
                ("e_cd".to_string(), format!("{:.1}s", self.e_cd_remaining)),
                ("r_cd".to_string(), format!("{:.1}s", self.r_cd_remaining)),
                (
                    "flash_cd".to_string(),
                    format!("{:.1}s", self.flash_cd_remaining),
                ),
                (
                    "atk_state".to_string(),
                    match self.attack_state {
                        0 => "Ready".to_string(),
                        1 => format!("前摇中({:.2}s)", self.attack_timer_remaining),
                        2 => format!("后摇中({:.2}s)", self.attack_timer_remaining),
                        _ => "未知".to_string(),
                    },
                ),
                (
                    "modifiers_count".to_string(),
                    format!(
                        "Self:{}, Target:{}",
                        self.self_modifiers
                            .iter()
                            .filter(|m| m.name_id != ModifierNameId::None)
                            .count(),
                        self.target_modifiers
                            .iter()
                            .filter(|m| m.name_id != ModifierNameId::None)
                            .count(),
                    ),
                ),
            ]),
            ..Default::default()
        }
    }
}

// ── 环境主体 ─────────────────────────────────────────────────────────────────

/// 统一的有头/无头世界初始化与重置逻辑（双方满血与闪现重置）
pub fn setup_solo_v0_env_world(fiora: Entity, riven: Entity, world: &mut World) {
    setup_solo_v0_health_world(world, fiora, riven);
}

pub struct SoloV0Env {
    pub base: FioraRivenBaseEnv,
}

impl std::ops::Deref for SoloV0Env {
    type Target = FioraRivenBaseEnv;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for SoloV0Env {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl SoloV0Env {
    pub const DEFAULT_MAX_STEPS: usize = 160;

    pub fn new() -> Self {
        Self::with_config(EnvConfig::default())
    }

    pub fn new_with_max_steps(max_steps: usize) -> Self {
        Self::with_config(EnvConfig {
            max_steps,
            render_mode: RenderMode::Headless,
        })
    }

    pub fn with_config(config: EnvConfig) -> Self {
        let base = FioraRivenBaseEnv::builder(config, Self::DEFAULT_MAX_STEPS)
            .window_title("Solo 1v1 V0 (Self-Play RL Viewer)")
            .map_name("solo")
            .enable_barrack(true)
            .initial_positions(
                Vec3::new(2200.0, 0.0, 12650.0),
                Vec3::new(2500.0, 0.0, 12910.0),
            )
            .initial_skill_levels([1, 0, 0, 0])
            .warmup_secs(40.0)
            .with_plugin(register_flash_plugin)
            .on_ready(setup_solo_v0_env_world)
            .on_reset(setup_solo_v0_env_world)
            .build();

        Self { base }
    }

    pub fn meta() -> EnvMeta {
        EnvMeta {
            name: <Self as RlEnvironment>::env_name().to_string(),
            display_name: <Self as RlEnvironment>::display_name().to_string(),
            description: <Self as RlEnvironment>::description().to_string(),
            action_dim: <Self as RlEnvironment>::action_dim(),
            state_dim: <Self as RlEnvironment>::state_dim(),
            action_labels: <Self as RlEnvironment>::action_labels()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn render_mode(&self) -> RenderMode {
        self.base.render_mode()
    }

    pub fn app(&self) -> &App {
        self.base.app()
    }

    pub fn app_mut(&mut self) -> &mut App {
        self.base.app_mut()
    }

    pub fn fiora(&self) -> Entity {
        self.base.fiora()
    }

    pub fn riven(&self) -> Entity {
        self.base.riven()
    }

    pub fn max_steps(&self) -> usize {
        self.base.max_steps()
    }

    pub fn step_count(&self) -> usize {
        self.base.step_count()
    }

    pub fn reset_both(&mut self) -> Vec<SoloV0Obs> {
        self.base.reset_base();
        vec![
            get_ego_obs_from_world(self.base.world(), self.base.fiora, self.base.riven, 0.0),
            get_ego_obs_from_world(self.base.world(), self.base.riven, self.base.fiora, 1.0),
        ]
    }

    pub fn step_both(
        &mut self,
        act_fiora: SoloV0Action,
        act_riven: SoloV0Action,
    ) -> (StepResult<SoloV0Obs>, StepResult<SoloV0Obs>) {
        self.base.increment_step();
        step_solo_v0_world(
            &mut self.base.app,
            self.base.fiora,
            self.base.riven,
            act_fiora,
            act_riven,
            self.base.step_count,
            self.base.max_steps,
        )
    }
}

// ── RlEnvironment Trait 实现 ─────────────────────────────────────────────────

impl RlEnvironment for SoloV0Env {
    type Action = SoloV0Action;
    type Obs = SoloV0Obs;

    fn num_agents() -> usize {
        2
    }

    fn agent_names() -> &'static [&'static str] {
        &["Fiora", "Riven"]
    }

    fn env_name() -> &'static str {
        "SoloV0"
    }

    fn display_name() -> &'static str {
        "Solo 1v1 (自博弈 V0)"
    }

    fn description() -> &'static str {
        "剑姬 vs 瑞雯 Solo 1v1 双智能体自博弈环境（真实地图召唤师峡谷上路对线，1级Q技能+闪现，40s兵线交汇起手）"
    }

    fn action_space() -> ActionSpace {
        ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 8,
        }
    }

    fn action_dim() -> usize {
        Self::action_space().actor_head_dim()
    }

    fn state_dim() -> usize {
        SOLO_V0_OBS_DIM
    }

    fn action_labels() -> &'static [&'static str] {
        &[
            "保持当前 (NoOp)",
            "移动 (Move)",
            "普通攻击 (Attack)",
            "施放 Q",
            "施放 W",
            "施放 E",
            "施放 R",
            "闪现",
        ]
    }

    fn obs_dim_labels() -> &'static [&'static str] {
        &[
            "角色标识(0=Fiora,1=Riven)",
            "目标相对X(归一化)",
            "目标相对Z(归一化)",
            "相对距离(归一化)",
            "普攻就绪(Ready)",
            "普攻前摇中(Windup)",
            "普攻后摇中(Cooldown)",
            "普攻状态倒计时",
            "Q就绪",
            "Q剩余CD",
            "W就绪",
            "W剩余CD",
            "E就绪",
            "E剩余CD",
            "R就绪",
            "R剩余CD",
            "闪现就绪",
            "闪现剩余CD",
            "自身血量百分比",
            "目标血量百分比",
            "自身修饰符1_类型ID",
            "自身修饰符1_剩余时长",
            "自身修饰符1_层数",
            "自身修饰符1_参数0",
            "自身修饰符1_参数1",
            "自身修饰符2_类型ID",
            "自身修饰符2_剩余时长",
            "自身修饰符2_层数",
            "自身修饰符2_参数0",
            "自身修饰符2_参数1",
            "自身修饰符3_类型ID",
            "自身修饰符3_剩余时长",
            "自身修饰符3_层数",
            "自身修饰符3_参数0",
            "自身修饰符3_参数1",
            "自身修饰符4_类型ID",
            "自身修饰符4_剩余时长",
            "自身修饰符4_层数",
            "自身修饰符4_参数0",
            "自身修饰符4_参数1",
            "目标修饰符1_类型ID",
            "目标修饰符1_剩余时长",
            "目标修饰符1_层数",
            "目标修饰符1_参数0(X)",
            "目标修饰符1_参数1(Z)",
            "目标修饰符2_类型ID",
            "目标修饰符2_剩余时长",
            "目标修饰符2_层数",
            "目标修饰符2_参数0(X)",
            "目标修饰符2_参数1(Z)",
            "目标修饰符3_类型ID",
            "目标修饰符3_剩余时长",
            "目标修饰符3_层数",
            "目标修饰符3_参数0(X)",
            "目标修饰符3_参数1(Z)",
            "目标修饰符4_类型ID",
            "目标修饰符4_剩余时长",
            "目标修饰符4_层数",
            "目标修饰符4_参数0(X)",
            "目标修饰符4_参数1(Z)",
        ]
    }

    fn action_from_index(idx: usize) -> Self::Action {
        SoloV0Action::preset_from_index(idx)
    }

    fn action_to_index(action: Self::Action) -> usize {
        action.preset_index()
    }

    fn action_from_encoding(encoded: &[f32]) -> Self::Action {
        SoloV0Action::from_encoding(encoded)
    }

    fn action_to_encoding(action: Self::Action) -> Vec<f32> {
        action.to_encoding()
    }

    fn action_name(action: Self::Action) -> &'static str {
        action.desc()
    }

    fn default_max_steps() -> usize {
        Self::DEFAULT_MAX_STEPS
    }

    fn max_steps(&self) -> usize {
        self.base.max_steps()
    }

    fn new() -> Self {
        Self::new()
    }

    fn with_config(config: EnvConfig) -> Self {
        Self::with_config(config)
    }

    fn reset(&mut self) -> Vec<Self::Obs> {
        self.reset_both()
    }

    fn step(&mut self, actions: &[Self::Action]) -> Vec<StepResult<Self::Obs>> {
        let fiora_action = actions.first().copied().unwrap_or(SoloV0Action::new(
            0.0,
            0.0,
            SoloV0DiscreteAction::NoOp,
        ));
        let riven_action = if actions.len() > 1 {
            actions[1]
        } else {
            get_default_riven_combat_action(self.base.world(), self.base.riven, self.base.fiora)
        };

        let (f_res, r_res) = self.step_both(fiora_action, riven_action);
        vec![f_res, r_res]
    }

    fn obs_to_vector(obs: &Self::Obs) -> Vec<f32> {
        obs.to_vector()
    }

    fn obs_to_payload(obs: &Self::Obs) -> Option<ObsFeaturePayload> {
        Some(obs.to_payload())
    }

    fn is_action_masked(obs: &Self::Obs, action_idx: usize) -> bool {
        let is_windup = obs.attack_is_windup;
        let dist_ok = obs.distance <= ATTACK_MASK_DISTANCE;

        match action_idx {
            2 => !dist_ok || is_windup,
            3 => !obs.q_ready || is_windup,
            4 => !obs.w_ready || is_windup,
            5 => !obs.e_ready || is_windup,
            6 => !obs.r_ready || is_windup,
            7 => !obs.flash_ready,
            _ => false,
        }
    }

    fn action_mask(obs: &Self::Obs) -> Option<Vec<bool>> {
        let is_windup = obs.attack_is_windup;
        let dist_ok = obs.distance <= ATTACK_MASK_DISTANCE;

        Some(vec![
            true,
            true,
            dist_ok && !is_windup,
            obs.q_ready && !is_windup,
            obs.w_ready && !is_windup,
            obs.e_ready && !is_windup,
            obs.r_ready && !is_windup,
            obs.flash_ready,
        ])
    }

    fn reward_formula_spec() -> Option<RewardFormulaSpec> {
        use lol_rl_protocol::RewardExpr;
        Some(RewardFormulaSpec {
            name: "Solo 1v1 对决与补兵公式 (SoloV0)".to_string(),
            terms: vec![
                RewardTermSpec::new(
                    "damage_dealt",
                    "造成伤害收益(低)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(0.5 / 1000.0)),
                        Box::new(RewardExpr::Variable("self_dmg".to_string())),
                    ),
                ),
                RewardTermSpec::new(
                    "damage_taken",
                    "承受伤害惩罚(低)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(-0.5 / 1000.0)),
                        Box::new(RewardExpr::Variable("target_dmg".to_string())),
                    ),
                ),
                RewardTermSpec::new(
                    "last_hit",
                    "补兵成功奖励(高)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(5.0)),
                        Box::new(RewardExpr::Variable("self_cs".to_string())),
                    ),
                ),
                RewardTermSpec::new(
                    "enemy_last_hit",
                    "敌方补兵惩罚(高)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(-5.0)),
                        Box::new(RewardExpr::Variable("target_cs".to_string())),
                    ),
                ),
                RewardTermSpec::new(
                    "minion_damage_shaping",
                    "小兵伤害诱导",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(0.5 / 1000.0)),
                        Box::new(RewardExpr::Variable("self_minion_dmg".to_string())),
                    ),
                ),
                RewardTermSpec::new(
                    "enemy_minion_damage_shaping",
                    "敌方小兵伤害抵消",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(-0.5 / 1000.0)),
                        Box::new(RewardExpr::Variable("target_minion_dmg".to_string())),
                    ),
                ),
                RewardTermSpec::new(
                    "last_hit_window",
                    "残血斩杀窗口诱导",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(0.2)),
                        Box::new(RewardExpr::Variable("self_cs_window".to_string())),
                    ),
                ),
                RewardTermSpec::new(
                    "enemy_last_hit_window",
                    "敌方斩杀窗口抵消",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(-0.2)),
                        Box::new(RewardExpr::Variable("target_cs_window".to_string())),
                    ),
                ),
                RewardTermSpec::new(
                    "kill_win",
                    "击杀获胜奖励",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(10.0)),
                        Box::new(RewardExpr::Variable("is_kill_win".to_string())),
                    ),
                ),
            ],
        })
    }
}

// ── VisualEnvironment Trait 实现 ─────────────────────────────────────────────

impl VisualEnvironment for SoloV0Env {
    fn take_app(&mut self) -> App {
        std::mem::replace(&mut self.base.app, App::new())
    }

    fn window_title(&self) -> &'static str {
        "Solo 1v1 V0 (Self-Play RL Viewer)"
    }

    fn is_assets_loaded(&self, world: &World) -> bool {
        self.base.is_assets_loaded(world)
    }

    fn on_assets_loaded(&mut self, world: &mut World) {
        setup_skill_levels_world(world, self.base.fiora, self.base.riven);
        setup_solo_v0_env_world(self.base.fiora, self.base.riven, world);

        if self.base.warmup_secs > 0.0 {
            let warmup_ticks = (self.base.warmup_secs * 64.0).round() as usize;
            for _ in 0..warmup_ticks {
                world.run_schedule(FixedUpdate);
            }
        }
    }

    fn reset_world(&mut self, world: &mut World) -> Vec<Self::Obs> {
        let (fiora, riven) = self.base.reset_world_base(world);
        vec![
            get_ego_obs_from_world(world, fiora, riven, 0.0),
            get_ego_obs_from_world(world, riven, fiora, 1.0),
        ]
    }

    fn get_current_obs_all(&self, world: &World) -> Vec<Self::Obs> {
        vec![
            get_ego_obs_from_world(world, self.base.fiora, self.base.riven, 0.0),
            get_ego_obs_from_world(world, self.base.riven, self.base.fiora, 1.0),
        ]
    }

    fn action_from_screen_click(
        &mut self,
        world: &mut World,
        screen_pos: Vec2,
    ) -> Option<SoloV0Action> {
        let rpos = world.get::<Transform>(self.base.riven)?.translation;
        let hit = raycast_ground_plane(world, screen_pos, rpos.y)?;

        let dx = hit.x - rpos.x;
        let dz = hit.z - rpos.z;
        let dist = (dx * dx + dz * dz).sqrt();

        if dist < 60.0 {
            Some(SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::Attack))
        } else {
            let nx = (dx / SOLO_V0_OFFSET_SCALE).clamp(-1.0, 1.0);
            let nz = (dz / SOLO_V0_OFFSET_SCALE).clamp(-1.0, 1.0);
            Some(SoloV0Action::new(nx, nz, SoloV0DiscreteAction::Move))
        }
    }

    fn step_world(
        &mut self,
        app: &mut App,
        actions: &[Self::Action],
    ) -> Vec<StepResult<Self::Obs>> {
        let fiora_action = actions.first().copied().unwrap_or(SoloV0Action::new(
            0.0,
            0.0,
            SoloV0DiscreteAction::NoOp,
        ));
        let riven_action = if actions.len() > 1 {
            actions[1]
        } else {
            get_default_riven_combat_action(app.world(), self.base.riven, self.base.fiora)
        };

        self.base.increment_step();
        let (f_res, r_res) = step_solo_v0_world(
            app,
            self.base.fiora,
            self.base.riven,
            fiora_action,
            riven_action,
            self.base.step_count,
            self.base.max_steps,
        );
        vec![f_res, r_res]
    }
}

// ── 自由函数 ─────────────────────────────────────────────────────────────────

pub fn get_default_riven_combat_action(
    world: &World,
    riven: Entity,
    fiora: Entity,
) -> SoloV0Action {
    let r_base = extract_champion_base(world, riven);
    let f_base = extract_champion_base(world, fiora);
    let dist = r_base.pos.distance(f_base.pos);
    let atk = extract_attack_state(world, riven);
    let skills = extract_skill_cds(world, riven);

    if atk.is_windup {
        return SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::NoOp);
    }

    let target_modifiers = extract_entity_modifiers(world, riven, 4);
    let primary_vital = target_modifiers
        .iter()
        .find(|m| m.name_id == ModifierNameId::FioraPassiveVital);
    let (offset_x, offset_z) = if let Some(v) = primary_vital {
        if v.stack_count > 0.5 {
            (-v.param0 * 0.5, -v.param1 * 0.5)
        } else {
            (0.0, 0.0)
        }
    } else {
        (0.0, 0.0)
    };

    if dist <= ATTACK_MASK_DISTANCE {
        if skills[1].ready {
            SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::CastW)
        } else if skills[0].ready {
            SoloV0Action::new(offset_x, offset_z, SoloV0DiscreteAction::CastQ)
        } else if !atk.is_cooldown {
            SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::Attack)
        } else if skills[2].ready {
            SoloV0Action::new(offset_x, offset_z, SoloV0DiscreteAction::CastE)
        } else if skills[3].ready {
            SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::CastR)
        } else {
            SoloV0Action::new(offset_x, offset_z, SoloV0DiscreteAction::Move)
        }
    } else if skills[2].ready {
        SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::CastE)
    } else if skills[0].ready {
        SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::CastQ)
    } else {
        SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::Move)
    }
}

pub fn get_ego_obs_from_world(
    world: &World,
    self_entity: Entity,
    target_entity: Entity,
    role_id: f32,
) -> SoloV0Obs {
    let self_base = extract_champion_base(world, self_entity);
    let target_base = extract_champion_base(world, target_entity);
    let dist = self_base.pos.distance(target_base.pos);

    let atk = extract_attack_state(world, self_entity);
    let skills = extract_skill_cds(world, self_entity);
    let (flash_ready, flash_cd) = extract_flash_obs(world, self_entity);

    SoloV0Obs {
        role_id,
        self_pos: self_base.pos,
        self_hp: self_base.hp,
        self_max_hp: self_base.max_hp,
        target_pos: target_base.pos,
        target_hp: target_base.hp,
        target_max_hp: target_base.max_hp,
        distance: dist,
        attack_state: atk.state_code,
        attack_is_windup: atk.is_windup,
        attack_is_cooldown: atk.is_cooldown,
        attack_timer_remaining: atk.timer_remaining,
        q_ready: skills[0].ready,
        q_cd_remaining: skills[0].cd_remaining,
        w_ready: skills[1].ready,
        w_cd_remaining: skills[1].cd_remaining,
        e_ready: skills[2].ready,
        e_cd_remaining: skills[2].cd_remaining,
        r_ready: skills[3].ready,
        r_cd_remaining: skills[3].cd_remaining,
        flash_ready,
        flash_cd_remaining: flash_cd,
        self_modifiers: extract_entity_modifiers(world, self_entity, 4),
        target_modifiers: extract_entity_modifiers(world, target_entity, 4),
    }
}

pub fn dispatch_single_action(
    world: &mut World,
    self_entity: Entity,
    target_entity: Entity,
    action: SoloV0Action,
) {
    let tpos = world
        .get::<Transform>(target_entity)
        .map(|t| t.translation)
        .unwrap_or_default();
    let spos = world
        .get::<Transform>(self_entity)
        .map(|t| t.translation)
        .unwrap_or_default();

    let target_offset_pos = Vec3::new(
        tpos.x + action.offset_x.clamp(-1.0, 1.0) * SOLO_V0_OFFSET_SCALE,
        tpos.y,
        tpos.z + action.offset_z.clamp(-1.0, 1.0) * SOLO_V0_OFFSET_SCALE,
    );

    match action.discrete {
        SoloV0DiscreteAction::NoOp => {}
        SoloV0DiscreteAction::Move => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Move(Vec2::new(target_offset_pos.x, target_offset_pos.z)),
            });
        }
        SoloV0DiscreteAction::Attack => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Attack(target_entity),
            });
        }
        SoloV0DiscreteAction::CastQ => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 0,
                    point: Vec2::new(target_offset_pos.x, target_offset_pos.z),
                },
            });
        }
        SoloV0DiscreteAction::CastW => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 1,
                    point: Vec2::new(spos.x, spos.z),
                },
            });
        }
        SoloV0DiscreteAction::CastE => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 2,
                    point: Vec2::new(target_offset_pos.x, target_offset_pos.z),
                },
            });
        }
        SoloV0DiscreteAction::CastR => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 3,
                    point: Vec2::new(tpos.x, tpos.z),
                },
            });
        }
        SoloV0DiscreteAction::CastFlash => {
            let offset_dir = Vec3::new(action.offset_x, 0.0, action.offset_z);
            let dir = if offset_dir.length_squared() > 1e-4 {
                offset_dir.normalize()
            } else {
                let to_target = tpos - spos;
                if to_target.length_squared() > 1e-4 {
                    to_target.normalize()
                } else {
                    Vec3::X
                }
            };
            dispatch_flash(world, self_entity, dir, FLASH_DISTANCE);
        }
    }
}

pub fn step_solo_v0_world(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    act_fiora: SoloV0Action,
    act_riven: SoloV0Action,
    step_count: usize,
    max_steps: usize,
) -> (StepResult<SoloV0Obs>, StepResult<SoloV0Obs>) {
    let prev_f_obs = get_ego_obs_from_world(app.world(), fiora, riven, 0.0);
    let prev_r_obs = get_ego_obs_from_world(app.world(), riven, fiora, 1.0);
    let prev_f_hp = prev_f_obs.self_hp;
    let prev_r_hp = prev_r_obs.self_hp;
    let prev_f_cs = app
        .world()
        .get::<ChampionStats>(fiora)
        .map(|s| s.minion_kills)
        .unwrap_or(0);
    let prev_r_cs = app
        .world()
        .get::<ChampionStats>(riven)
        .map(|s| s.minion_kills)
        .unwrap_or(0);

    // 记录更新前的小兵血量与队伍
    let mut prev_minion_hps: HashMap<Entity, (Team, f32)> = HashMap::new();
    {
        let mut q_minions =
            app.world_mut().query_filtered::<(Entity, &Team, &Health), With<Minion>>();
        for (e, team, hp) in q_minions.iter(app.world()) {
            prev_minion_hps.insert(e, (*team, hp.value));
        }
    }

    dispatch_single_action(app.world_mut(), fiora, riven, act_fiora);
    dispatch_single_action(app.world_mut(), riven, fiora, act_riven);
    unpause_virtual_time(app.world_mut());

    for _ in 0..10 {
        app.update();
    }

    let curr_f_obs = get_ego_obs_from_world(app.world(), fiora, riven, 0.0);
    let curr_r_obs = get_ego_obs_from_world(app.world(), riven, fiora, 1.0);
    let curr_f_hp = curr_f_obs.self_hp;
    let curr_r_hp = curr_r_obs.self_hp;
    let curr_f_cs = app
        .world()
        .get::<ChampionStats>(fiora)
        .map(|s| s.minion_kills)
        .unwrap_or(0);
    let curr_r_cs = app
        .world()
        .get::<ChampionStats>(riven)
        .map(|s| s.minion_kills)
        .unwrap_or(0);

    let fiora_cs_diff = curr_f_cs.saturating_sub(prev_f_cs) as f32;
    let riven_cs_diff = curr_r_cs.saturating_sub(prev_r_cs) as f32;

    // 统计小兵血量变化与残血斩杀窗口诱导
    let mut fiora_minion_dmg = 0.0f32;
    let mut riven_minion_dmg = 0.0f32;
    let mut fiora_near_low_hp_minion = false;
    let mut riven_near_low_hp_minion = false;

    let f_pos = curr_f_obs.self_pos;
    let r_pos = curr_r_obs.self_pos;

    {
        let mut q_minions = app
            .world_mut()
            .query_filtered::<(Entity, &Team, &Health, &Transform), With<Minion>>();
        for (e, team, hp, tf) in q_minions.iter(app.world()) {
            let m_pos = tf.translation;
            if let Some(&(prev_team, prev_hp)) = prev_minion_hps.get(&e) {
                let dmg = (prev_hp - hp.value).max(0.0);
                match prev_team {
                    Team::Chaos => fiora_minion_dmg += dmg,
                    Team::Order => riven_minion_dmg += dmg,
                    _ => {}
                }
            }

            // 斩杀窗口诱导：敌方小兵残血(<=120且存活)且在斩杀距离(<=450)内
            if hp.value > 0.0 && hp.value <= 120.0 {
                match team {
                    Team::Chaos => {
                        if f_pos.distance(m_pos) <= 450.0 {
                            fiora_near_low_hp_minion = true;
                        }
                    }
                    Team::Order => {
                        if r_pos.distance(m_pos) <= 450.0 {
                            riven_near_low_hp_minion = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 1. 英雄伤害收益（调低至 0.5 / 1000.0）
    let fiora_dmg_dealt = (prev_r_hp - curr_r_hp).max(0.0) / 1000.0;
    let riven_dmg_dealt = (prev_f_hp - curr_f_hp).max(0.0) / 1000.0;
    let r_hero_dmg = (fiora_dmg_dealt - riven_dmg_dealt) * 0.5;

    // 2. 补兵成功奖励（提很高：5.0 / 刀）
    let r_cs = (fiora_cs_diff - riven_cs_diff) * 5.0;

    // 3. 补兵诱导奖励（小兵伤害诱导 0.5/1000 + 残血斩杀窗口 0.2）
    let r_minion_dmg = ((fiora_minion_dmg - riven_minion_dmg) / 1000.0) * 0.5;
    let f_cs_window = if fiora_near_low_hp_minion { 1.0 } else { 0.0 };
    let r_cs_window = if riven_near_low_hp_minion { 1.0 } else { 0.0 };
    let r_cs_window_diff = (f_cs_window - r_cs_window) * 0.2;

    let fiora_killed = curr_r_hp <= 0.0 && prev_r_hp > 0.0;
    let riven_killed = curr_f_hp <= 0.0 && prev_f_hp > 0.0;
    let kill_bonus_fiora = if fiora_killed {
        10.0
    } else if riven_killed {
        -10.0
    } else {
        0.0
    };

    let r_fiora = r_hero_dmg + r_cs + r_minion_dmg + r_cs_window_diff + kill_bonus_fiora;
    let r_riven = -r_fiora;

    let terminated = curr_f_hp <= 0.0 || curr_r_hp <= 0.0;
    let truncated = step_count >= max_steps;

    let f_vars = HashMap::from([
        ("self_dmg".to_string(), fiora_dmg_dealt * 1000.0),
        ("target_dmg".to_string(), riven_dmg_dealt * 1000.0),
        ("self_cs".to_string(), fiora_cs_diff),
        ("target_cs".to_string(), riven_cs_diff),
        ("self_minion_dmg".to_string(), fiora_minion_dmg),
        ("target_minion_dmg".to_string(), riven_minion_dmg),
        ("self_cs_window".to_string(), f_cs_window),
        ("target_cs_window".to_string(), r_cs_window),
        (
            "is_kill_win".to_string(),
            if fiora_killed { 1.0 } else { 0.0 },
        ),
    ]);

    let r_vars = HashMap::from([
        ("self_dmg".to_string(), riven_dmg_dealt * 1000.0),
        ("target_dmg".to_string(), fiora_dmg_dealt * 1000.0),
        ("self_cs".to_string(), riven_cs_diff),
        ("target_cs".to_string(), fiora_cs_diff),
        ("self_minion_dmg".to_string(), riven_minion_dmg),
        ("target_minion_dmg".to_string(), fiora_minion_dmg),
        ("self_cs_window".to_string(), r_cs_window),
        ("target_cs_window".to_string(), f_cs_window),
        (
            "is_kill_win".to_string(),
            if riven_killed { 1.0 } else { 0.0 },
        ),
    ]);

    (
        StepResult {
            obs: curr_f_obs,
            reward: r_fiora,
            terminated,
            truncated,
            step: step_count,
            reward_breakdown: Vec::new(),
            reward_variables: f_vars,
        },
        StepResult {
            obs: curr_r_obs,
            reward: r_riven,
            terminated,
            truncated,
            step: step_count,
            reward_breakdown: Vec::new(),
            reward_variables: r_vars,
        },
    )
}
