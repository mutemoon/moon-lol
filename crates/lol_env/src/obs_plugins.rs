use bevy::prelude::*;
use lol_core::attack::{Attack, AttackState, AttackStatus};
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
