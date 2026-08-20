use bevy::prelude::*;
use lol_champions::fiora::e::BuffFioraE;
use lol_champions::fiora::r::BuffFioraR;
use lol_champions::fiora::passive::Vital;
use lol_core::attack::{Attack, AttackState, AttackStatus};
use lol_core::base::buff::Buffs;
use lol_core::base::direction::Direction;
use lol_core::life::Health;
use lol_core::skill::{CoolDown, SkillRecastWindow, Skills, is_skill_ready};

/// 基础英雄属性与空间状态
#[derive(Debug, Clone, Default)]
pub struct ChampionBaseObs {
    pub pos: Vec3,
    pub hp: f32,
    pub max_hp: f32,
}

/// 从 World 提取英雄基础信息（坐标、当前血量、最大血量）
pub fn extract_champion_base(world: &World, entity: Entity) -> ChampionBaseObs {
    let pos = world
        .get::<Transform>(entity)
        .map(|t| t.translation)
        .unwrap_or_default();
    let hp = world.get::<Health>(entity);
    ChampionBaseObs {
        pos,
        hp: hp.map(|h| h.value).unwrap_or(0.0),
        max_hp: hp.map(|h| h.max).unwrap_or(500.0),
    }
}

/// 被动要害破绽状态
#[derive(Debug, Clone, Default)]
pub struct PassiveVitalObs {
    pub has_vital: bool,
    pub is_active: bool,
    pub active_timer_remaining: f32,
    pub remove_timer_remaining: f32,
    pub dir_x: f32,
    pub dir_neg_x: f32,
    pub dir_z: f32,
    pub dir_neg_z: f32,
}

/// 从目标实体提取菲奥娜被动破绽信息
pub fn extract_passive_vital(world: &World, target: Entity) -> PassiveVitalObs {
    let vital = world.get::<Vital>(target);
    match vital {
        Some(v) => {
            let (vx, vnx, vz, vnz) = match v.direction {
                Direction::X => (1.0, 0.0, 0.0, 0.0),
                Direction::NegX => (0.0, 1.0, 0.0, 0.0),
                Direction::Z => (0.0, 0.0, 1.0, 0.0),
                Direction::NegZ => (0.0, 0.0, 0.0, 1.0),
            };
            let active_rem = if v.is_active() {
                0.0
            } else {
                v.active_timer.remaining_secs()
            };
            let remove_rem = v.remove_timer.remaining_secs();
            PassiveVitalObs {
                has_vital: true,
                is_active: v.is_active(),
                active_timer_remaining: active_rem,
                remove_timer_remaining: remove_rem,
                dir_x: vx,
                dir_neg_x: vnx,
                dir_z: vz,
                dir_neg_z: vnz,
            }
        }
        None => PassiveVitalObs::default(),
    }
}

/// 大招四破绽状态 (BuffFioraR)
#[derive(Debug, Clone, Default)]
pub struct RVitalObs {
    pub has_r_vital: bool,
    pub is_active: bool,
    pub active_timer_remaining: f32,
    pub remove_timer_remaining: f32,
    pub vital_east: bool,
    pub vital_west: bool,
    pub vital_north: bool,
    pub vital_south: bool,
}

/// 从目标实体的 Buff 列表中提取大招四破绽状态
pub fn extract_r_vital(world: &World, target: Entity) -> RVitalObs {
    if let Some(buffs) = world.get::<Buffs>(target) {
        for buff_entity in buffs.iter() {
            if let Some(buff_r) = world.get::<BuffFioraR>(buff_entity) {
                let is_active = buff_r.is_active();
                let active_rem = if is_active {
                    0.0
                } else {
                    buff_r.active_timer.remaining_secs()
                };
                let remove_rem = buff_r.remove_timer.remaining_secs();
                let has_e = buff_r.vitals.contains(&Direction::X);
                let has_w = buff_r.vitals.contains(&Direction::NegX);
                let has_n = buff_r.vitals.contains(&Direction::Z);
                let has_s = buff_r.vitals.contains(&Direction::NegZ);
                return RVitalObs {
                    has_r_vital: true,
                    is_active,
                    active_timer_remaining: active_rem,
                    remove_timer_remaining: remove_rem,
                    vital_east: has_e,
                    vital_west: has_w,
                    vital_north: has_n,
                    vital_south: has_s,
                };
            }
        }
    }
    RVitalObs::default()
}

/// 普攻状态机状态
#[derive(Debug, Clone, Default)]
pub struct AttackStateObs {
    pub state_code: u8, // 0: Ready, 1: Windup, 2: Cooldown
    pub is_windup: bool,
    pub is_cooldown: bool,
    pub timer_remaining: f32,
    pub windup_duration: f32,
    pub total_duration: f32,
}

/// 从实体提取当前普攻状态机信息
pub fn extract_attack_state(world: &World, entity: Entity) -> AttackStateObs {
    let now = world
        .get_resource::<Time<Fixed>>()
        .map(|t| t.elapsed_secs())
        .unwrap_or(0.0);
    let attack_state = world.get::<AttackState>(entity);
    let attack_prop = world.get::<Attack>(entity);

    let windup_dur = attack_prop
        .map(|a| a.windup_duration_secs())
        .unwrap_or(0.25);
    let total_dur = attack_prop.map(|a| a.total_duration_secs()).unwrap_or(0.8);

    if let Some(state) = attack_state {
        match &state.status {
            AttackStatus::Windup { end_time, .. } => {
                let rem = (*end_time - now).max(0.0);
                AttackStateObs {
                    state_code: 1,
                    is_windup: true,
                    is_cooldown: false,
                    timer_remaining: rem,
                    windup_duration: windup_dur,
                    total_duration: total_dur,
                }
            }
            AttackStatus::Cooldown { end_time } => {
                let rem = (*end_time - now).max(0.0);
                AttackStateObs {
                    state_code: 2,
                    is_windup: false,
                    is_cooldown: true,
                    timer_remaining: rem,
                    windup_duration: windup_dur,
                    total_duration: total_dur,
                }
            }
        }
    } else {
        AttackStateObs {
            state_code: 0,
            is_windup: false,
            is_cooldown: false,
            timer_remaining: 0.0,
            windup_duration: windup_dur,
            total_duration: total_dur,
        }
    }
}

/// 单个技能的冷却与就绪状态
#[derive(Debug, Clone, Default)]
pub struct SkillCdObs {
    pub ready: bool,
    pub cd_remaining: f32,
}

/// 从实体提取 Q(0), W(1), E(2), R(3) 四个槽位的技能 CD 状态
pub fn extract_skill_cds(world: &World, entity: Entity) -> [SkillCdObs; 4] {
    let mut result = [
        SkillCdObs {
            ready: true,
            cd_remaining: 0.0,
        },
        SkillCdObs {
            ready: true,
            cd_remaining: 0.0,
        },
        SkillCdObs {
            ready: true,
            cd_remaining: 0.0,
        },
        SkillCdObs {
            ready: true,
            cd_remaining: 0.0,
        },
    ];

    if let Some(skills) = world.get::<Skills>(entity) {
        let skill_entities = skills.to_vec();
        for (i, &s_entity) in skill_entities.iter().enumerate().take(4) {
            let cd = world.get::<CoolDown>(s_entity);
            let recast = world.get::<SkillRecastWindow>(s_entity);
            let ready = match cd {
                Some(c) => is_skill_ready(c, recast),
                None => true,
            };
            let rem = cd
                .and_then(|c| c.timer.as_ref())
                .map(|t| {
                    if t.is_finished() {
                        0.0
                    } else {
                        t.remaining_secs()
                    }
                })
                .unwrap_or(0.0);
            result[i] = SkillCdObs {
                ready,
                cd_remaining: rem,
            };
        }
    }

    result
}

/// 剑姬 E 技能 Buff 状态
#[derive(Debug, Clone, Default)]
pub struct BuffEObs {
    pub has_buff: bool,
    pub left: i32,
}

/// 从实体提取剑姬 E 技能的 Buff 强化状态
pub fn extract_buff_e(world: &World, entity: Entity) -> BuffEObs {
    if let Some(buffs) = world.get::<Buffs>(entity) {
        for buff_entity in buffs.iter() {
            if let Some(buff_e) = world.get::<BuffFioraE>(buff_entity) {
                return BuffEObs {
                    has_buff: true,
                    left: buff_e.left,
                };
            }
        }
    }
    BuffEObs::default()
}
