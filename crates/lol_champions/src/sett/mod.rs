pub mod buffs;

#[cfg(test)]
mod e_tests;
#[cfg(test)]
mod passive_tests;
#[cfg(test)]
mod q_tests;
#[cfg(test)]
mod r_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod w_tests;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{ActionDamageEffect, DamageShape, TargetDamage, TargetFilter};
use lol_core::action::delayed_damage::{ActionDelayedDamage, AoEIndicator, AoEOrigin};
use lol_core::action::knockback::{CommandKnockback, DisplaceDirection};
use lol_core::attack::{CommandAttackReset, EventAttackEnd};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::{DebuffSlow, DebuffStun};
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::buffs::on_hit::{BuffOnHitCounter, BuffOnHitTargetMaxHp};
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, delay_from_cast_frame, get_skill_value};
use lol_core::team::Team;

use crate::sett::buffs::{SettGrit, SettPunchState, SettWShield};

// ── 被动（沙场战心）── 左右拳交替，右拳附带 0.55×AD 额外物理伤害。
pub const SETT_PASSIVE_RIGHT_PUNCH_RATIO: f32 = 0.55;

// ── Q（屈人之威）── 强化下 2 次普攻：附带目标最大生命百分比物理伤害 + 30% 移速 4s。
/// Q 每级目标最大生命百分比（ron 缺失该 dataValue，按 wiki 硬编码）。
pub const SETT_Q_MAX_HP_RATIO: [f32; 5] = [0.03, 0.035, 0.04, 0.045, 0.05];
pub const SETT_Q_ATTACKS: u8 = 2;
pub const SETT_Q_DURATION: f32 = 4.0;
pub const SETT_Q_MS_BONUS: f32 = 0.30;

// ── W（强腕重击）── 被动储存"灰心"（受到伤害累积，上限 50% 最大生命，脱战 4s 衰减）；
///     主动把灰心转化为白盾（0.75s），并挥出锥形（中心真实 / 两侧物理）。
pub const SETT_GRIT_CAP_RATIO: f32 = 0.50;
pub const SETT_GRIT_COMBAT_DURATION: f32 = 4.0;
pub const SETT_W_SHIELD_DURATION: f32 = 0.75;

// ── E（迎面痛击）── 锥形拉回敌人到脚下 + 击飞 + 物理伤害 + 0.5s 眩晕。
pub const SETT_E_RANGE: f32 = 490.0;
pub const SETT_E_CONE_ANGLE: f32 = 90.0;
pub const SETT_E_KNOCKUP_DURATION: f32 = 0.5;
pub const SETT_E_PULL_SPEED: f32 = 1200.0;
pub const SETT_E_STUN_DURATION: f32 = 0.5;
/// E 伤害标签：on_sett_damage_hit 据此施加眩晕（仅 E 眩晕）。
pub const SETT_E_TAG: u32 = 1;

// ── R（消防官）── 在施法点 AoE 砸地 + 物理伤害 + 40% 减速 1.5s（位移/搬运简化为定点 AoE）。
pub const SETT_R_RADIUS: f32 = 200.0;
pub const SETT_R_SLOW_PERCENT: f32 = 0.40;
pub const SETT_R_SLOW_DURATION: f32 = 1.5;
/// R 伤害标签：on_sett_damage_hit 据此施加减速（仅 R 减速）。
pub const SETT_R_TAG: u32 = 2;

#[derive(Default)]
pub struct PluginSett;

impl Plugin for PluginSett {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sett_q);
        app.add_observer(on_sett_w);
        app.add_observer(on_sett_e);
        app.add_observer(on_sett_r);
        app.add_observer(on_sett_damage_hit);
        app.add_observer(on_sett_attack_end);
        app.add_observer(on_sett_damage_taken);
        app.add_systems(FixedUpdate, update_sett_grit);
        app.add_systems(FixedUpdate, update_sett_w_shield);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Sett"))]
#[reflect(Component)]
pub struct Sett;

// ════════════════════════ Q ════════════════════════

fn on_sett_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }
    cast_sett_q(&mut commands, entity, skill.level);
}

/// Q：强化下 2 次普攻——每次附带目标最大生命百分比物理伤害 + 30% 移速 4s。
fn cast_sett_q(commands: &mut Commands, entity: Entity, skill_level: usize) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandAttackReset { entity });

    let lvl_idx = skill_level
        .saturating_sub(1)
        .min(SETT_Q_MAX_HP_RATIO.len() - 1);
    let ratio = SETT_Q_MAX_HP_RATIO[lvl_idx];

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOnHitCounter::new(SETT_Q_ATTACKS, SETT_Q_DURATION))
        .with_related::<BuffOf>(BuffOnHitTargetMaxHp { ratio })
        .with_related::<BuffOf>(BuffMoveSpeed::new(SETT_Q_MS_BONUS, SETT_Q_DURATION));
}

// ════════════════════════ W ════════════════════════

fn on_sett_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
    q_grit: Query<&SettGrit>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    // 读取已储存的灰心，主动施放时全部转化为护盾并清零
    let grit_stored = q_grit.get(entity).map(|g| g.stored).unwrap_or(0.0);
    if grit_stored > 0.0 {
        commands.entity(entity).queue(move |mut e: EntityWorldMut| {
            if let Some(mut grit) = e.get_mut::<SettGrit>() {
                grit.stored = 0.0;
            }
        });
    }

    cast_sett_w(
        &mut commands,
        entity,
        skill.spell.clone(),
        skill.level,
        spell_obj,
        trigger.point,
        grit_stored,
    );
}

/// W：灰心转化为白盾（0.75s）+ 锥形（中心 30° 真实 / 两侧 75° 物理，exclude 分区）。
fn cast_sett_w(
    commands: &mut Commands,
    entity: Entity,
    skill_spell: Handle<Spell>,
    skill_level: usize,
    spell_obj: &Spell,
    point: Vec2,
    grit_stored: f32,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    // 灰心 -> 白盾子 buff（吸收交给通用 BuffShieldWhite，harness 可读）
    if grit_stored > 0.0 {
        let shield_entity = commands
            .spawn((BuffShieldWhite::new(grit_stored), SettWShield::new()))
            .id();
        commands
            .entity(entity)
            .add_related::<BuffOf>(&[shield_entity]);
    }

    // 锥形：中心窄扇形（30°）真实伤害，两侧（全 75° 扇形排除中心）物理伤害。
    let full_sector = DamageShape::Sector {
        radius: 350.0,
        angle: 75.0,
    };
    let center_sector = DamageShape::Sector {
        radius: 350.0,
        angle: 30.0,
    };
    commands.trigger(ActionDelayedDamage {
        entity,
        skill: skill_spell,
        skill_level,
        delay: delay_from_cast_frame(spell_obj),
        point,
        origin: AoEOrigin::Caster,
        effects: vec![
            ActionDamageEffect {
                shape: full_sector,
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "damage_calc".to_string(),
                    damage_type: DamageType::Physical,
                    ..Default::default()
                }],
                exclude: vec![center_sector.clone()],
                ..Default::default()
            },
            ActionDamageEffect {
                shape: center_sector,
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "damage_calc".to_string(),
                    damage_type: DamageType::True,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        indicator: AoEIndicator {
            color: Color::srgba(1.0, 0.84, 0.0, 0.35),
            pulse: true,
            grow_from_zero: false,
            impact_burst_scale: 1.4,
            fade_duration: 0.3,
        },
    });
}

// ════════════════════════ E ════════════════════════

fn on_sett_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    cast_sett_e(
        &mut commands,
        entity,
        trigger.point,
        skill.level,
        spell_obj,
        &q_transform,
        &q_team,
        &q_enemies,
        &q_damage,
    );
}

/// E：朝施法方向锥形拉回敌人到脚下（CommandKnockback{Toward}，自动击飞），
///    并造成 base+0.6×AD 物理伤害（带 SETT_E_TAG，由 on_sett_damage_hit 施加 0.5s 眩晕）。
fn cast_sett_e(
    commands: &mut Commands,
    entity: Entity,
    point: Vec2,
    skill_level: usize,
    spell_obj: &Spell,
    q_transform: &Query<&Transform>,
    q_team: &Query<&Team>,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
    q_damage: &Query<&Damage>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let Ok(team) = q_team.get(entity) else {
        return;
    };
    let pos = transform.translation.xz();
    let forward = {
        let f = (point - pos).normalize_or_zero();
        if f == Vec2::ZERO {
            transform.forward().xz()
        } else {
            f
        }
    };
    let half_angle = SETT_E_CONE_ANGLE.to_radians() / 2.0;

    // 物理伤害 = base + 0.6×AD（spell ron damage_calc；stat 2 = AD）
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let damage = get_skill_value(spell_obj, "damage_calc", skill_level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    let mut hit = 0u32;
    for (enemy, enemy_transform) in q_enemies.iter() {
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == team {
            continue;
        }
        let diff = enemy_transform.translation.xz() - pos;
        let distance = diff.length();
        if distance > SETT_E_RANGE || distance == 0.0 {
            continue;
        }
        let dir = diff.normalize();
        let angle = forward.dot(dir).clamp(-1.0, 1.0).acos();
        if angle > half_angle {
            continue;
        }

        // 拉回脚下（Toward 钳制不越过 source）+ 自动击飞
        commands.entity(enemy).trigger(|e| CommandKnockback {
            entity: e,
            source: entity,
            distance: SETT_E_RANGE,
            speed: SETT_E_PULL_SPEED,
            duration: Some(SETT_E_KNOCKUP_DURATION),
            direction: DisplaceDirection::Toward,
        });
        // 物理伤害（带 E 标签 -> on_sett_damage_hit 施加眩晕）
        if damage > 0.0 {
            commands.entity(enemy).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount: damage,
                tag: Some(SETT_E_TAG),
            });
        }
        hit += 1;
    }

    debug!(
        "Sett E: 迎面痛击，锥形拉回 {} 个敌人 + 击飞 + 伤害 + 眩晕",
        hit
    );
}

// ════════════════════════ R ════════════════════════

fn on_sett_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_team: Query<&Team>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    cast_sett_r(
        &mut commands,
        entity,
        trigger.point,
        skill.level,
        spell_obj,
        &q_enemies,
        &q_team,
        &q_damage,
    );
}

/// R：在施法点 AoE 砸地（半径 200），造成 base+1.2×AD 物理伤害（带 SETT_R_TAG，
///    由 on_sett_damage_hit 施加 40% 减速 1.5s）。位移/搬运简化为定点 AoE。
fn cast_sett_r(
    commands: &mut Commands,
    entity: Entity,
    point: Vec2,
    skill_level: usize,
    spell_obj: &Spell,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
    q_team: &Query<&Team>,
    q_damage: &Query<&Damage>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    let Ok(team) = q_team.get(entity) else { return };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let damage = get_skill_value(spell_obj, "damage_calc", skill_level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    let mut hit = 0u32;
    for (enemy, enemy_transform) in q_enemies.iter() {
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == team {
            continue;
        }
        let distance = enemy_transform.translation.xz().distance(point);
        if distance > SETT_R_RADIUS {
            continue;
        }
        if damage > 0.0 {
            commands.entity(enemy).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount: damage,
                tag: Some(SETT_R_TAG),
            });
        }
        hit += 1;
    }

    debug!("Sett R: 消防官，施法点 AoE 砸中 {} 个敌人 + 减速", hit);
}

// ════════════════════════ 被动 / 灰心 / 标签副作用 ════════════════════════

/// 监听 Sett 造成的伤害：仅 E 标签眩晕、仅 R 标签减速（Q/被动/W 不附带 CC）。
fn on_sett_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
) {
    let source = trigger.source;
    if q_sett.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    match trigger.event().tag {
        Some(SETT_E_TAG) => {
            commands
                .entity(target)
                .with_related::<BuffOf>(DebuffStun::new(SETT_E_STUN_DURATION));
        }
        Some(SETT_R_TAG) => {
            commands
                .entity(target)
                .with_related::<BuffOf>(DebuffSlow::new(SETT_R_SLOW_PERCENT, SETT_R_SLOW_DURATION));
        }
        _ => {}
    }
}

/// 被动（沙场战心）：每次普攻命中左右拳交替，右拳（偶数次）附带 0.55×AD 额外物理伤害。
fn on_sett_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_punch: Query<&SettPunchState>,
    q_damage: Query<&Damage>,
) {
    let attacker = trigger.event_target();
    if q_sett.get(attacker).is_err() {
        return;
    }
    let target = trigger.target;

    let count = q_punch.get(attacker).map(|p| p.count).unwrap_or(0);
    let new_count = count + 1;

    // 延迟写入拳序（避免与 FixedUpdate 里的 &mut 冲突，同 morde 灰心写法）
    commands
        .entity(attacker)
        .queue(move |mut e: EntityWorldMut| {
            if let Some(mut punch) = e.get_mut::<SettPunchState>() {
                punch.count = new_count;
            } else {
                e.insert(SettPunchState { count: new_count });
            }
        });

    // 右拳（偶数次）附带 0.55×AD 物理伤害（tag None -> 不触发 E/R 的 CC）
    if new_count % 2 == 0 {
        let ad = q_damage.get(attacker).map(|d| d.0).unwrap_or(0.0);
        let bonus = ad * SETT_PASSIVE_RIGHT_PUNCH_RATIO;
        if bonus > 0.0 {
            commands.entity(target).trigger(|e| CommandDamageCreate {
                entity: e,
                source: attacker,
                damage_type: DamageType::Physical,
                amount: bonus,
                tag: None,
            });
        }
    }
}

/// 监听 Sett 受到的伤害：累积"灰心"（= final_damage，上限 50% 最大生命），重置脱战计时。
fn on_sett_damage_taken(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_health: Query<&Health>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }
    let final_dmg = trigger.event().damage_result.final_damage;
    if final_dmg <= 0.0 {
        return;
    }
    let max_hp = q_health.get(entity).map(|h| h.max).unwrap_or(0.0);
    let cap = SETT_GRIT_CAP_RATIO * max_hp;

    commands.entity(entity).queue(move |mut e: EntityWorldMut| {
        if let Some(mut grit) = e.get_mut::<SettGrit>() {
            grit.stored = (grit.stored + final_dmg).min(cap);
            grit.combat_timer.reset();
        } else {
            let mut grit = SettGrit::new();
            grit.stored = final_dmg.min(cap);
            e.insert(grit);
        }
    });
}

/// 灰心脱战衰减：脱战计时器（4s）结束后清零灰心。
fn update_sett_grit(time: Res<Time>, mut q_grit: Query<&mut SettGrit, With<Sett>>) {
    for mut grit in q_grit.iter_mut() {
        grit.combat_timer.tick(time.delta());
        if grit.combat_timer.is_finished() {
            grit.stored = 0.0;
        }
    }
}

/// W 护盾到期移除（0.75s；被打空时由通用 update_shield_white 提前 despawn）。
fn update_sett_w_shield(
    mut commands: Commands,
    time: Res<Time>,
    mut q_shield: Query<(Entity, &mut SettWShield)>,
) {
    for (entity, mut shield) in q_shield.iter_mut() {
        shield.timer.tick(time.delta());
        if shield.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
