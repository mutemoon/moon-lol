use std::collections::HashMap;

use bevy::prelude::*;
use bevy::world_serialization::DynamicWorld;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_champions::fiora::Fiora;
use lol_champions::fiora::passive::Vital;
use lol_champions::riven::Riven;
use lol_core::base::direction::Direction;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::life::Health;
use lol_core::skill::{CoolDown, Skill, SkillRecastWindow, Skills, is_skill_ready};
use lol_core::team::Team;

use crate::reward::{FioraRewardContext, FioraVsRivenRewardModel, RewardModel};
use crate::traits::RewardBreakdownItem;

/// 攻击类动作的掩码距离阈值：超过该距离不允许攻击（单一事实来源）。
pub const ATTACK_MASK_DISTANCE: f32 = 220.0;

/// obs 向量中「相对距离归一化列」的下标，与 [`FioraVsRivenObs::to_vector`] 的布局一致。
pub const OBS_DISTANCE_IDX: usize = 8;
/// obs 向量中距离的归一化缩放：`to_vector` 写入 `distance / OBS_DISTANCE_SCALE`。
pub const OBS_DISTANCE_SCALE: f32 = 100.0;

// ── 观测 ────────────────────────────────────────────────────────────────────

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
    pub has_vital: bool,
    pub vital_is_active: bool,
    pub vital_dir_x: f32,
    pub vital_dir_neg_x: f32,
    pub vital_dir_z: f32,
    pub vital_dir_neg_z: f32,
}

impl FioraVsRivenObs {
    /// 转换为强化学习策略网络输入向量。
    pub fn to_vector(&self) -> Vec<f32> {
        let rel_x = self.fiora_pos.x - self.riven_pos.x;
        let rel_z = self.fiora_pos.z - self.riven_pos.z;

        vec![
            // 破绽四方位 (4维)
            self.vital_dir_x,
            self.vital_dir_neg_x,
            self.vital_dir_z,
            self.vital_dir_neg_z,
            // 破绽状态 (2维：是否存在、是否已激活)
            if self.has_vital { 1.0 } else { 0.0 },
            if self.vital_is_active { 1.0 } else { 0.0 },
            // 剑姬相对于瑞雯的相对位置与距离 (3维，归一化/OBS_DISTANCE_SCALE)
            rel_x / OBS_DISTANCE_SCALE,
            rel_z / OBS_DISTANCE_SCALE,
            self.distance / OBS_DISTANCE_SCALE,
        ]
    }

    pub fn dim() -> usize {
        9
    }

    pub fn to_payload(&self) -> lol_rl_protocol::ObsFeaturePayload {
        let vital_dir = if self.vital_dir_x > 0.5 {
            "+X (东)".to_string()
        } else if self.vital_dir_neg_x > 0.5 {
            "-X (西)".to_string()
        } else if self.vital_dir_z > 0.5 {
            "+Z (北)".to_string()
        } else if self.vital_dir_neg_z > 0.5 {
            "-Z (南)".to_string()
        } else {
            "无".to_string()
        };

        lol_rl_protocol::ObsFeaturePayload {
            fiora_hp_pct: if self.fiora_max_hp > 0.0 {
                self.fiora_hp / self.fiora_max_hp
            } else {
                1.0
            },
            riven_hp_pct: if self.riven_max_hp > 0.0 {
                self.riven_hp / self.riven_max_hp
            } else {
                1.0
            },
            distance: self.distance,
            q_ready: self.q_ready,
            w_ready: self.w_ready,
            e_ready: self.e_ready,
            r_ready: self.r_ready,
            has_vital: self.has_vital,
            vital_is_active: self.vital_is_active,
            vital_direction: vital_dir,
            ..Default::default()
        }
    }
}

// ── 事件追踪 ────────────────────────────────────────────────────────────────

/// 普攻事件追踪：攻击命中（EventAttackEnd）与攻击就绪（EventAttackReady）。
#[derive(Resource, Default, Debug, Clone)]
pub struct AttackEventTracker {
    pub attack_hit: bool,
    pub attack_ready: bool,
}

pub fn on_attack_end(
    _trigger: On<lol_core::attack::EventAttackEnd>,
    mut tracker: ResMut<AttackEventTracker>,
) {
    tracker.attack_hit = true;
}

pub fn on_attack_ready(
    _trigger: On<lol_core::attack::EventAttackReady>,
    mut tracker: ResMut<AttackEventTracker>,
) {
    tracker.attack_ready = true;
}

/// 环境中的 Fiora / Riven 实体，供观察者过滤事件来源。
#[derive(Resource)]
pub struct FioraRivenEntities {
    pub fiora: Entity,
    pub riven: Entity,
}

/// 真实破绽击破信号：菲奥娜被动击破要害会对目标造成一次真实伤害
/// （`DamageType::True`，5% 最大生命值），这是「破绽被真正击破」的权威判据。
#[derive(Resource, Default, Debug, Clone)]
pub struct VitalBreakTracker {
    pub hit: bool,
}

pub fn on_vital_break_damage(
    trigger: On<EventDamageCreate>,
    entities: Res<FioraRivenEntities>,
    mut tracker: ResMut<VitalBreakTracker>,
) {
    if trigger.source == entities.fiora && trigger.damage_type == DamageType::True {
        tracker.hit = true;
    }
}

/// 注册两个环境共用的资源与观察者（在 `App::finish()` 之前调用）。
pub fn add_common_observers(app: &mut App) {
    app.init_resource::<AttackEventTracker>();
    app.init_resource::<VitalBreakTracker>();
    app.add_observer(on_attack_end);
    app.add_observer(on_attack_ready);
    app.add_observer(on_vital_break_damage);
}

// ── 世界读写 ────────────────────────────────────────────────────────────────

/// Extract observation from the Bevy ECS world.
pub fn get_obs_from_world(world: &World, fiora: Entity, riven: Entity) -> FioraVsRivenObs {
    let fpos = world
        .get::<Transform>(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();
    let rpos = world
        .get::<Transform>(riven)
        .map(|t| t.translation)
        .unwrap_or_default();
    let dist = fpos.distance(rpos);

    let fhp = world.get::<Health>(fiora);
    let rhp = world.get::<Health>(riven);

    let vital = world.get::<Vital>(riven);
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
        if let Some(skills) = world.get::<Skills>(fiora) {
            let skill_entities = skills.to_vec();
            let check_ready = |idx: usize| -> bool {
                if idx < skill_entities.len() {
                    let s_entity = skill_entities[idx];
                    let cd = world.get::<CoolDown>(s_entity);
                    let recast = world.get::<SkillRecastWindow>(s_entity);
                    match cd {
                        Some(c) => is_skill_ready(c, recast),
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
        fiora_hp: fhp.map(|h| h.value).unwrap_or(0.0),
        fiora_max_hp: fhp.map(|h| h.max).unwrap_or(500.0),
        riven_pos: rpos,
        riven_hp: rhp.map(|h| h.value).unwrap_or(0.0),
        riven_max_hp: rhp.map(|h| h.max).unwrap_or(500.0),
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

/// 销毁世界中的英雄实体及其附带技能
pub fn despawn_entities_world(world: &mut World, fiora: Entity, riven: Entity) {
    for champion in [fiora, riven] {
        if let Ok(entity_ref) = world.get_entity(champion) {
            if let Some(skills) = entity_ref.get::<Skills>() {
                for s in skills.to_vec() {
                    if let Ok(s_mut) = world.get_entity_mut(s) {
                        s_mut.despawn();
                    }
                }
            }
        }
        if let Ok(entity_mut) = world.get_entity_mut(champion) {
            entity_mut.despawn();
        }
    }
}

/// 在世界中重新生成 Fiora 和 Riven 实体
pub fn spawn_champions_world(
    world: &mut World,
    fiora_config_handle: Handle<DynamicWorld>,
    riven_config_handle: Handle<DynamicWorld>,
    fiora_skin_handle: Option<Handle<DynamicWorld>>,
    riven_skin_handle: Option<Handle<DynamicWorld>>,
    initial_fiora_pos: Vec3,
    initial_riven_pos: Vec3,
    render: bool,
) -> (Entity, Entity) {
    let mut fiora_builder = world.spawn((
        Fiora::default(),
        Transform::from_translation(initial_fiora_pos),
        Team::Order,
        ConfigCharacterRecord {
            character_record: fiora_config_handle,
        },
        Health::new(500.0),
        lol_core::damage::Armor(35.0),
        lol_core::movement::Movement { speed: 345.0 },
    ));

    if render {
        if let Some(skin) = fiora_skin_handle {
            fiora_builder.insert((
                lol_render::controller::SelfPlayer,
                lol_base_render::camera::Focus,
                ConfigSkin { skin },
            ));
        }
    }

    let fiora = fiora_builder.id();

    let mut riven_builder = world.spawn((
        Riven::default(),
        Transform::from_translation(initial_riven_pos),
        Team::Chaos,
        ConfigCharacterRecord {
            character_record: riven_config_handle,
        },
        Health::new(500.0),
        lol_core::damage::Armor(33.0),
        lol_core::movement::Movement { speed: 340.0 },
    ));

    if render {
        if let Some(skin) = riven_skin_handle {
            riven_builder.insert(ConfigSkin { skin });
        }
    }

    let riven = riven_builder.id();

    (fiora, riven)
}

/// Reset entities in the Bevy ECS world for a new episode by despawning and respawning.
pub fn reset_episode_world(
    world: &mut World,
    fiora: Entity,
    riven: Entity,
    fiora_config_handle: &Handle<DynamicWorld>,
    riven_config_handle: &Handle<DynamicWorld>,
    fiora_skin_handle: &Option<Handle<DynamicWorld>>,
    riven_skin_handle: &Option<Handle<DynamicWorld>>,
    initial_fiora_pos: Vec3,
    initial_riven_pos: Vec3,
    render: bool,
) -> (Entity, Entity) {
    // 1. 销毁旧实体
    despawn_entities_world(world, fiora, riven);

    // 2. 重新生成新实体
    let (new_fiora, new_riven) = spawn_champions_world(
        world,
        fiora_config_handle.clone(),
        riven_config_handle.clone(),
        fiora_skin_handle.clone(),
        riven_skin_handle.clone(),
        initial_fiora_pos,
        initial_riven_pos,
        render,
    );

    // 3. 更新实体引用资源
    world.insert_resource(FioraRivenEntities {
        fiora: new_fiora,
        riven: new_riven,
    });

    // 4. 重置事件追踪器
    if let Some(mut tracker) = world.get_resource_mut::<AttackEventTracker>() {
        tracker.attack_hit = false;
        tracker.attack_ready = false;
    }
    if let Some(mut tracker) = world.get_resource_mut::<VitalBreakTracker>() {
        tracker.hit = false;
    }

    // 5. 随机为目标生成一个初始已激活的破绽 (Active Vital)
    let random_dir = match rand::random::<u8>() % 4 {
        0 => Direction::X,
        1 => Direction::NegX,
        2 => Direction::Z,
        _ => Direction::NegZ,
    };
    let mut initial_vital = Vital::new(random_dir, 0.0, 10.0);
    initial_vital
        .active_timer
        .tick(std::time::Duration::from_millis(1));
    world.entity_mut(new_riven).insert(initial_vital);

    (new_fiora, new_riven)
}

/// Set skill levels for Fiora and Riven in the Bevy ECS world.
pub fn setup_skill_levels_world(world: &mut World, fiora: Entity, riven: Entity) {
    for champion in [fiora, riven] {
        if let Some(skills) = world.get::<Skills>(champion) {
            let skill_entities = skills.to_vec();
            if skill_entities.len() >= 4 {
                if let Some(mut q) = world.get_mut::<Skill>(skill_entities[0]) {
                    q.level = 3;
                }
                if let Some(mut w) = world.get_mut::<Skill>(skill_entities[1]) {
                    w.level = 1;
                }
                if let Some(mut e) = world.get_mut::<Skill>(skill_entities[2]) {
                    e.level = 1;
                }
                if let Some(mut r) = world.get_mut::<Skill>(skill_entities[3]) {
                    r.level = 1;
                }
            }
        }
    }
}

/// Helper function to check if a 3D position is aligned with the vital's direction quadrant relative to target.
pub fn is_position_aligned_with_vital(fpos: Vec3, rpos: Vec3, obs: &FioraVsRivenObs) -> bool {
    let delta_x = fpos.x - rpos.x;
    let delta_z = fpos.z - rpos.z;
    let abs_delta_x = delta_x.abs();
    let abs_delta_z = delta_z.abs();

    if obs.vital_dir_x > 0.5 {
        delta_x > 0.0 && abs_delta_x > abs_delta_z
    } else if obs.vital_dir_neg_x > 0.5 {
        delta_x < 0.0 && abs_delta_x > abs_delta_z
    } else if obs.vital_dir_z > 0.5 {
        delta_z > 0.0 && abs_delta_z > abs_delta_x
    } else if obs.vital_dir_neg_z > 0.5 {
        delta_z < 0.0 && abs_delta_z > abs_delta_x
    } else {
        false
    }
}

/// Compute step reward and its breakdown items using the structured RewardModel.
/// `is_vital_break` 来自世界真实事件（菲奥娜被动真实伤害），而非位置启发式。
pub fn compute_step_reward(
    prev_riven_hp: f32,
    curr_riven_hp: f32,
    prev_fpos: Vec3,
    curr_fpos: Vec3,
    riven_pos: Vec3,
    is_attack: bool,
    is_vital_break: bool,
    prev_obs: &FioraVsRivenObs,
    elapsed_secs: f32,
) -> (f32, Vec<RewardBreakdownItem>, HashMap<String, f32>) {
    let prev_aligned =
        prev_obs.has_vital && is_position_aligned_with_vital(prev_fpos, riven_pos, prev_obs);
    let curr_aligned =
        prev_obs.has_vital && is_position_aligned_with_vital(curr_fpos, riven_pos, prev_obs);

    let ctx = FioraRewardContext {
        prev_aligned,
        curr_aligned,
        is_vital_break,
        is_attack,
        prev_riven_hp,
        curr_riven_hp,
        elapsed_secs,
    };

    let model = FioraVsRivenRewardModel;
    let (reward, items, vars) = model.evaluate(&ctx);

    let breakdown = items
        .into_iter()
        .map(|it| RewardBreakdownItem {
            name: it.name,
            value: it.value,
        })
        .collect();

    (reward, breakdown, vars)
}

/// Helper functions for controlling Virtual time during visual stepping.
pub fn pause_virtual_time(world: &mut World) {
    if let Some(mut time) = world.get_resource_mut::<Time<Virtual>>() {
        time.pause();
    }
}

pub fn unpause_virtual_time(world: &mut World) {
    if let Some(mut time) = world.get_resource_mut::<Time<Virtual>>() {
        time.unpause();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{App, Vec3};
    use lol_core::damage::{DamageResult, DamageType, EventDamageCreate};

    use super::*;

    fn obs_with_vital(dir_x: f32, dir_neg_x: f32, dir_z: f32, dir_neg_z: f32) -> FioraVsRivenObs {
        FioraVsRivenObs {
            fiora_pos: Vec3::ZERO,
            fiora_hp: 500.0,
            fiora_max_hp: 500.0,
            riven_pos: Vec3::ZERO,
            riven_hp: 500.0,
            riven_max_hp: 500.0,
            distance: 0.0,
            q_ready: true,
            w_ready: true,
            e_ready: true,
            r_ready: true,
            has_vital: true,
            vital_is_active: true,
            vital_dir_x: dir_x,
            vital_dir_neg_x: dir_neg_x,
            vital_dir_z: dir_z,
            vital_dir_neg_z: dir_neg_z,
        }
    }

    #[test]
    fn test_is_position_aligned_with_vital() {
        let riven = Vec3::ZERO;
        // +X 破绽 → 剑姬在 +X 侧（且 |x| > |z|）对齐
        let obs = obs_with_vital(1.0, 0.0, 0.0, 0.0);
        assert!(is_position_aligned_with_vital(
            Vec3::new(60.0, 0.0, 10.0),
            riven,
            &obs
        ));
        assert!(!is_position_aligned_with_vital(
            Vec3::new(-60.0, 0.0, 10.0),
            riven,
            &obs
        ));
        // -X 破绽
        let obs = obs_with_vital(0.0, 1.0, 0.0, 0.0);
        assert!(is_position_aligned_with_vital(
            Vec3::new(-60.0, 0.0, 10.0),
            riven,
            &obs
        ));
        // +Z 破绽
        let obs = obs_with_vital(0.0, 0.0, 1.0, 0.0);
        assert!(is_position_aligned_with_vital(
            Vec3::new(10.0, 0.0, 60.0),
            riven,
            &obs
        ));
        // 无破绽方向 → 永远不对齐
        let obs = obs_with_vital(0.0, 0.0, 0.0, 0.0);
        assert!(!is_position_aligned_with_vital(
            Vec3::new(60.0, 0.0, 0.0),
            riven,
            &obs
        ));
    }

    #[test]
    fn test_obs_vector_layout() {
        let mut obs = obs_with_vital(1.0, 0.0, 0.0, 0.0);
        obs.fiora_pos = Vec3::new(250.0, 0.0, 0.0);
        obs.riven_pos = Vec3::ZERO;
        obs.distance = 250.0;
        let v = obs.to_vector();
        assert_eq!(v.len(), FioraVsRivenObs::dim());
        assert_eq!(v.len(), 9);
        assert_eq!(v[0], 1.0); // vital_dir_x
        assert_eq!(v[4], 1.0); // has_vital
        assert_eq!(v[5], 1.0); // vital_is_active
        // 相对位置与距离的归一化列
        assert_eq!(v[6], 250.0 / OBS_DISTANCE_SCALE);
        assert_eq!(v[OBS_DISTANCE_IDX], 250.0 / OBS_DISTANCE_SCALE);
    }

    #[test]
    fn test_compute_step_reward_kill_and_vital() {
        // 无破绽方向 → 对齐项为 0；击杀（第 4 秒击杀，时效奖励严格为 0）+ 破绽命中
        let obs = obs_with_vital(0.0, 0.0, 0.0, 0.0);
        let (reward, _breakdown, vars) = compute_step_reward(
            100.0,
            0.0, // 击杀
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
            false,
            true, // 破绽命中
            &obs,
            4.0,  // 第 4 秒击杀
        );
        let expected = -0.002 + 0.8 + 2.0 + 0.0;
        assert!(
            (reward - expected).abs() < 1e-4,
            "reward={reward} expected={expected}"
        );
        assert_eq!(vars["is_vital_break"], 1.0);
        assert_eq!(vars["is_kill"], 1.0);
        assert_eq!(vars["is_attack_missed"], 0.0);
        assert_eq!(vars["quick_kill_reward"], 0.0);

        // 第 1 秒极速击杀：时效奖励达到 ~15.15（高于击杀基础分 2.0）
        let (reward_1s, _, vars_1s) = compute_step_reward(
            100.0, 0.0, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, false, true, &obs, 1.0,
        );
        let quick_1s = vars_1s["quick_kill_reward"];
        assert!(quick_1s > 15.0, "1s击杀时效奖励应 > 15.0，实际为 {quick_1s}");
        assert!(reward_1s > 17.5, "1s击杀总奖励应 > 17.5，实际为 {reward_1s}");

        // 第 5 秒击杀：时效奖励严格为负（扣分）
        let (_, _, vars_5s) = compute_step_reward(
            100.0, 0.0, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, false, true, &obs, 5.0,
        );
        assert!(vars_5s["quick_kill_reward"] < 0.0, "5s击杀时效奖励应为负数");
    }

    #[test]
    fn test_compute_step_reward_attack_miss() {
        let obs = obs_with_vital(0.0, 0.0, 0.0, 0.0);
        let (reward, _breakdown, vars) = compute_step_reward(
            500.0,
            490.0,
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
            true,  // 攻击
            false, // 未击破破绽
            &obs,
            0.5,
        );
        // -0.002 (time) + -0.1 (attack_miss)
        assert!((reward - (-0.102)).abs() < 1e-4, "reward={reward}");
        assert_eq!(vars["is_attack_missed"], 1.0);
        assert_eq!(vars["is_kill"], 0.0);
    }

    #[test]
    fn test_vital_break_tracker() {
        let mut app = App::new();
        let fiora = app.world_mut().spawn_empty().id();
        let riven = app.world_mut().spawn_empty().id();
        app.world_mut()
            .insert_resource(FioraRivenEntities { fiora, riven });
        add_common_observers(&mut app);

        let damage_result = || DamageResult {
            final_damage: 0.0,
            white_shield_absorbed: 0.0,
            magic_shield_absorbed: 0.0,
            reduced_damage: 0.0,
            armor_reduced_damage: 0.0,
            original_damage: 0.0,
        };

        // 剑姬造成的真实伤害 → 判定破绽被击破
        app.world_mut().trigger(EventDamageCreate {
            entity: riven,
            source: fiora,
            damage_type: DamageType::True,
            damage_result: damage_result(),
            tag: None,
        });
        assert!(app.world().resource::<VitalBreakTracker>().hit);

        // 物理伤害（非真实）→ 不判定
        app.world_mut().resource_mut::<VitalBreakTracker>().hit = false;
        app.world_mut().trigger(EventDamageCreate {
            entity: riven,
            source: fiora,
            damage_type: DamageType::Physical,
            damage_result: damage_result(),
            tag: None,
        });
        assert!(!app.world().resource::<VitalBreakTracker>().hit);

        // 真实伤害但来源不是剑姬 → 不判定
        app.world_mut().trigger(EventDamageCreate {
            entity: fiora,
            source: riven,
            damage_type: DamageType::True,
            damage_result: damage_result(),
            tag: None,
        });
        assert!(!app.world().resource::<VitalBreakTracker>().hit);
    }
}
