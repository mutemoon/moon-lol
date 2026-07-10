use bevy::prelude::*;
use lol_core::base::buff::{Buff, BuffOf};
use lol_core::base::direction::{Direction, is_in_direction};
use lol_core::damage::{CommandDamageCreate, DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::life::{Death, Health};
use lol_core::team::Team;

use crate::fiora::passive::BuffFioraMS;

const VITAL_R_TIMEOUT: f32 = 1.5;
const FIORA_R_ACTIVE_DURATION: f32 = 0.5;
/// R 要害持续时间（ron MarkDuration = 8s）。
const FIORA_R_DURATION: f32 = 8.0;
/// R 施法距离（ron castRange = 500）。
const FIORA_R_RANGE: f32 = 500.0;
/// R 期间移速加成比例（wiki：30%）。
const FIORA_R_MS_PERCENT: f32 = 0.3;

/// 每个要害的最大生命值真实伤害比例（wiki：3/3.5/4%，按技能等级 1-3）。
const FIORA_R_VITAL_PCT: [f32; 3] = [0.03, 0.035, 0.04];
/// 治疗光环每秒治疗量（ron HealPerSecond = 50/75/100，按技能等级 1-3）。
const FIORA_R_HEAL_PER_SECOND: [f32; 3] = [50.0, 75.0, 100.0];
/// 治疗光环半径（ron HealRingRadius = 550）。
const FIORA_R_HEAL_RADIUS: f32 = 550.0;
/// 治疗光环持续时间（ron HealDuration = 5s）。
const FIORA_R_HEAL_DURATION: f32 = 5.0;

#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "FioraR" })]
pub struct BuffFioraR {
    pub vitals: Vec<Direction>,
    pub level: usize,
    pub active_timer: Timer,
    pub remove_timer: Timer,
    pub timeout_red_triggered: bool,
}

impl BuffFioraR {
    pub fn is_active(&self) -> bool {
        self.active_timer.is_finished()
    }
}

impl Default for BuffFioraR {
    fn default() -> Self {
        Self::new(1)
    }
}

impl BuffFioraR {
    pub fn new(level: usize) -> Self {
        Self {
            vitals: vec![
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ],
            level,
            active_timer: Timer::from_seconds(FIORA_R_ACTIVE_DURATION, TimerMode::Once),
            remove_timer: Timer::from_seconds(FIORA_R_DURATION, TimerMode::Once),
            timeout_red_triggered: false,
        }
    }
}

/// R 治疗光环：在中心点周围每秒治疗同阵营友军。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "FioraRHeal" })]
pub struct BuffFioraRHeal {
    /// 光环中心（目标被击破时的世界坐标）。
    pub center: Vec3,
    pub team: Team,
    pub timer: Timer,
    pub tick: Timer,
    pub heal_per_second: f32,
}

pub fn fixed_update(
    mut commands: Commands,
    mut q_buff_fiora_r: Query<(Entity, &BuffOf, &mut BuffFioraR)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, _buff_of, mut buff) in q_buff_fiora_r.iter_mut() {
        if !buff.is_active() {
            buff.active_timer.tick(time.delta());
            continue;
        }

        if !buff.timeout_red_triggered && buff.remove_timer.remaining_secs() <= VITAL_R_TIMEOUT {
            buff.timeout_red_triggered = true;
        }

        buff.remove_timer.tick(time.delta());

        if buff.remove_timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// 计时治疗光环：每秒治疗范围内同阵营友军，到期移除。
pub fn update_fiora_r_heal(
    mut commands: Commands,
    mut q_heal: Query<(Entity, &mut BuffFioraRHeal)>,
    mut q_allies: Query<(&Transform, &Team, &mut Health), Without<Death>>,
    time: Res<Time<Fixed>>,
) {
    for (buff_entity, mut heal) in q_heal.iter_mut() {
        heal.timer.tick(time.delta());
        heal.tick.tick(time.delta());
        if heal.tick.just_finished() {
            for (transform, team, mut hp) in q_allies.iter_mut() {
                if team != &heal.team {
                    continue;
                }
                if transform.translation.distance(heal.center) > FIORA_R_HEAL_RADIUS {
                    continue;
                }
                hp.value = (hp.value + heal.heal_per_second).min(hp.max);
            }
        }
        if heal.timer.is_finished() {
            commands.entity(buff_entity).despawn();
        }
    }
}

/// R 施法：选定 500 范围内朝向指针的最近敌方英雄，把四要害挂在「目标」身上
/// （修复原本挂在菲奥娜自身导致 on_r_damage_create 永不匹配的 bug），
/// 并给菲奥娜自身挂 30% 移速 buff。
pub fn cast_fiora_r(
    commands: &mut Commands,
    caster: Entity,
    point: Vec2,
    q_transform: &Query<&Transform>,
    q_team: &Query<&Team>,
    q_targets: &Query<(Entity, &Transform, &Team), (With<Champion>, Without<Death>)>,
    skill_level: usize,
) {
    let Ok(caster_tf) = q_transform.get(caster) else {
        return;
    };
    let Ok(caster_team) = q_team.get(caster) else {
        return;
    };
    let caster_xz = caster_tf.translation.xz();

    // 500 范围内、朝向指针最近的敌方英雄
    let mut best: Option<(Entity, f32)> = None;
    for (target, t_tf, t_team) in q_targets.iter() {
        if t_team == caster_team {
            continue;
        }
        let t_xz = t_tf.translation.xz();
        if t_xz.distance(caster_xz) > FIORA_R_RANGE {
            continue;
        }
        let d = t_xz.distance(point);
        if best.map_or(true, |(_, bd)| d < bd) {
            best = Some((target, d));
        }
    }
    let Some((target, _)) = best else {
        return;
    };

    // 四要害挂在目标身上（holder = 目标）
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffFioraR::new(skill_level));

    // R 期间 30% 移速挂在菲奥娜自身
    commands.entity(caster).with_related::<BuffOf>(BuffFioraMS {
        percent: FIORA_R_MS_PERCENT,
        timer: Timer::from_seconds(FIORA_R_DURATION, TimerMode::Once),
        applied: false,
        applied_bonus: 0.0,
    });
}

/// 监听伤害事件：从匹配方向击破一个 R 要害并造成最大生命值真实伤害；
/// 四要害全破或目标死亡（已破≥1）时触发治疗光环。
pub fn on_r_damage_create(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_target: Query<(&Transform, &Team, &Health)>,
    q_source: Query<(&Transform, &Team)>,
    mut q_buff_fiora_r: Query<(Entity, &BuffOf, &mut BuffFioraR)>,
) {
    let target_entity = trigger.event_target();
    let Ok((source_tf, source_team)) = q_source.get(trigger.source) else {
        return;
    };

    let Some((buff_entity, mut buff_fiora_r)) =
        q_buff_fiora_r
            .iter_mut()
            .find_map(|(entity, buff_of, buff_fiora_r)| {
                if buff_of.0 == target_entity {
                    Some((entity, buff_fiora_r))
                } else {
                    None
                }
            })
    else {
        return;
    };

    let Ok((target_tf, target_team, hp)) = q_target.get(target_entity) else {
        return;
    };

    if target_team == source_team {
        return;
    }

    if !buff_fiora_r.is_active() {
        return;
    }

    let source_position = source_tf.translation.xz();
    let target_position = target_tf.translation.xz();

    let mut hit_direction: Option<Direction> = None;
    buff_fiora_r.vitals.retain(|direction| {
        if hit_direction.is_none() && is_in_direction(source_position, target_position, direction) {
            hit_direction = Some(direction.clone());
            false // 移除此方向
        } else {
            true // 保留此方向
        }
    });

    let Some(_direction) = hit_direction else {
        return;
    };

    // 击破要害：最大生命值真实伤害
    let vital_pct = FIORA_R_VITAL_PCT
        .get(buff_fiora_r.level.saturating_sub(1))
        .copied()
        .unwrap_or(FIORA_R_VITAL_PCT[0]);
    let true_damage = hp.max * vital_pct;
    commands
        .entity(target_entity)
        .trigger(|e| CommandDamageCreate {
            entity: e,
            source: trigger.source,
            damage_type: DamageType::True,
            amount: true_damage,
            tag: None,
        });

    // 四要害全破，或目标死亡（已破≥1）-> 治疗光环
    let all_broken = buff_fiora_r.vitals.is_empty();
    let target_dead = hp.value <= 0.0;
    if all_broken || target_dead {
        let heal_per_second = FIORA_R_HEAL_PER_SECOND
            .get(buff_fiora_r.level.saturating_sub(1))
            .copied()
            .unwrap_or(FIORA_R_HEAL_PER_SECOND[0]);
        commands
            .entity(trigger.source)
            .with_related::<BuffOf>(BuffFioraRHeal {
                center: Vec3::new(target_position.x, 0.0, target_position.y),
                team: *source_team,
                timer: Timer::from_seconds(FIORA_R_HEAL_DURATION, TimerMode::Once),
                tick: Timer::from_seconds(1.0, TimerMode::Repeating),
                heal_per_second,
            });
        commands.entity(buff_entity).despawn();
    }
}
