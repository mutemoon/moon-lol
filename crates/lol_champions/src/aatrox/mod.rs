pub mod buffs;

#[cfg(test)]
mod e_tests;
#[cfg(test)]
mod p_tests;
#[cfg(test)]
mod q_tests;
#[cfg(test)]
mod r_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod w_tests;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::{DebuffKnockup, DebuffSlow};
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_cast_radius,
    get_skill_data_value, get_skill_value,
};
use lol_core::team::Team;

use crate::aatrox::buffs::{AatroxPassiveState, AatroxRState, DebuffAatroxWMark};

// ── 被动（死亡镰刀）── 就绪时下次普攻附带目标最大生命值 15% 额外魔法伤害 + 治疗；冷却 22s。
pub const AATROX_P_TAG: u32 = 10;

// ── Q（暗裔之刃）── 三段重施，每段 +25% 伤害；边缘（sweet spot）+70% + 击飞。
pub const AATROX_Q_TAG: u32 = 11;
pub const AATROX_Q_RECAST_WINDOW: f32 = 4.0;
pub const AATROX_Q_RADIUS: f32 = 300.0;
pub const AATROX_Q_SWEET_SPOT_MIN: f32 = 200.0;

// ── W（冥府之链）── 命中伤害 + 减速 + 标记；1.5s 后引爆二次伤害 + 击飞。
pub const AATROX_W_TAG: u32 = 12;
pub const AATROX_W_MARK_DURATION: f32 = 1.5;

// ── E（暗影冲锋）── 向指针方向突进（最大 300）。
pub const AATROX_E_MAX_RANGE: f32 = 300.0;
pub const AATROX_E_DASH_SPEED: f32 = 800.0;

// ── R（世界终结者）── 10s 内 +AD + 移速；到期移除。
pub const AATROX_R_TAG: u32 = 13;

#[derive(Default)]
pub struct PluginAatrox;

impl Plugin for PluginAatrox {
    fn build(&self, app: &mut App) {
        app.add_observer(on_aatrox_q);
        app.add_observer(on_aatrox_w);
        app.add_observer(on_aatrox_e);
        app.add_observer(on_aatrox_r);
        app.add_observer(on_aatrox_attack_end);
        app.add_systems(FixedUpdate, update_aatrox_passive);
        app.add_systems(FixedUpdate, update_aatrox_w_marks);
        app.add_systems(FixedUpdate, update_aatrox_r);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Aatrox"), AatroxPassiveState)]
#[reflect(Component)]
pub struct Aatrox;

// ════════════════════════ 被动 ════════════════════════

/// 普攻命中时：若被动就绪，对目标造成其最大生命值 15% 额外魔法伤害并治疗自身，随后进入冷却。
fn on_aatrox_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    mut q_aatrox: Query<(&mut AatroxPassiveState, &Team), With<Aatrox>>,
    q_team: Query<&Team>,
    q_health: Query<&Health>,
) {
    let attacker = trigger.event_target();
    let Ok((mut passive, caster_team)) = q_aatrox.get_mut(attacker) else {
        return;
    };
    if !passive.ready {
        return;
    }
    let target = trigger.event().target;
    let Ok(target_team) = q_team.get(target) else {
        return;
    };
    if target_team == caster_team {
        return;
    }
    let Ok(target_hp) = q_health.get(target) else {
        return;
    };

    let bonus = target_hp.max * AatroxPassiveState::DAMAGE_RATIO;
    if bonus > 0.0 {
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: attacker,
            damage_type: DamageType::Magic,
            amount: bonus,
            tag: Some(AATROX_P_TAG),
        });
    }
    let heal = bonus * AatroxPassiveState::HEAL_RATIO;
    if heal > 0.0 {
        commands
            .entity(attacker)
            .queue(move |mut e: EntityWorldMut| {
                if let Some(mut health) = e.get_mut::<Health>() {
                    health.value = (health.value + heal).min(health.max);
                }
            });
    }

    passive.ready = false;
    passive.timer.reset();
}

/// 被动冷却倒计时：到时再次就绪。
fn update_aatrox_passive(
    time: Res<Time>,
    mut q_aatrox: Query<&mut AatroxPassiveState, With<Aatrox>>,
) {
    for mut passive in q_aatrox.iter_mut() {
        if passive.ready {
            continue;
        }
        passive.timer.tick(time.delta());
        if passive.timer.just_finished() {
            passive.ready = true;
        }
    }
}

// ════════════════════════ Q ════════════════════════

fn on_aatrox_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
        return;
    }
    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let caster_pos = q_transform
        .get(entity)
        .map(|t| t.translation.xz())
        .unwrap_or(Vec2::ZERO);
    let Ok(caster_team) = q_team.get(entity) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);

    cast_aatrox_q(
        &mut commands,
        entity,
        skill.level,
        spell_obj,
        trigger.skill_entity,
        cooldown,
        recast,
        caster_pos,
        caster_team,
        ad,
        &q_enemies,
        &q_team,
    );
}

/// Q：以施法者为圆心的圆环 AoE。中心区（< sweet_spot_min）普通伤害；
/// 边缘区（sweet spot）1.7 倍伤害 + 击飞。每段重施 +25%。
fn cast_aatrox_q(
    commands: &mut Commands,
    entity: Entity,
    skill_level: usize,
    spell_obj: &Spell,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    caster_pos: Vec2,
    caster_team: &Team,
    ad: f32,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
    q_team: &Query<&Team>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);
    let anim = match stage {
        2 => "Q2IntoIdle",
        3 => "Q3IntoPassiveIdle",
        _ => "Q1IntoIdle",
    };
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: anim.to_string(),
        repeat: false,
        duration: None,
    });

    let base = get_skill_value(spell_obj, "q_damage", skill_level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);
    let ramp = get_skill_data_value(spell_obj, "QRampBonus", skill_level).unwrap_or(0.25);
    let sweet = get_skill_data_value(spell_obj, "QSweetSpotBonus", skill_level).unwrap_or(0.7);
    let knockup_dur =
        get_skill_data_value(spell_obj, "QKnockupDuration", skill_level).unwrap_or(0.25);
    let extension = get_skill_data_value(spell_obj, "QExtensionTime", skill_level)
        .unwrap_or(AATROX_Q_RECAST_WINDOW);

    let ramp_mult = 1.0 + (stage as f32 - 1.0) * ramp;
    let sweet_mult = 1.0 + sweet;
    let center_dmg = base * ramp_mult;
    let sweet_dmg = center_dmg * sweet_mult;

    for (enemy, enemy_tf) in q_enemies.iter() {
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == caster_team {
            continue;
        }
        let dist = enemy_tf.translation.xz().distance(caster_pos);
        if dist > AATROX_Q_RADIUS {
            continue;
        }
        let (dmg, is_sweet) = if dist >= AATROX_Q_SWEET_SPOT_MIN {
            (sweet_dmg, true)
        } else {
            (center_dmg, false)
        };
        if dmg > 0.0 {
            commands.entity(enemy).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount: dmg,
                tag: Some(AATROX_Q_TAG),
            });
        }
        if is_sweet {
            commands
                .entity(enemy)
                .with_related::<BuffOf>(DebuffKnockup::new(knockup_dur));
        }
    }

    if stage >= 3 {
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        });
    } else {
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(stage + 1, 3, extension));
    }
}

// ════════════════════════ W ════════════════════════

fn on_aatrox_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_skill: Query<&Skill>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
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

    let caster_pos = q_transform
        .get(entity)
        .map(|t| t.translation.xz())
        .unwrap_or(Vec2::ZERO);
    let Ok(caster_team) = q_team.get(entity) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);

    cast_aatrox_w(
        &mut commands,
        entity,
        skill.level,
        spell_obj,
        caster_pos,
        caster_team,
        ad,
        &q_enemies,
        &q_team,
    );
}

/// W：命中范围内所有敌人--首段物理伤害 + 减速 + 标记（1.5s 后引爆）。
fn cast_aatrox_w(
    commands: &mut Commands,
    entity: Entity,
    skill_level: usize,
    spell_obj: &Spell,
    caster_pos: Vec2,
    caster_team: &Team,
    ad: f32,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
    q_team: &Query<&Team>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    let dmg = get_skill_value(spell_obj, "w_damage", skill_level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);
    let slow_pct = get_skill_data_value(spell_obj, "WSlowPercentage", skill_level)
        .map(|v| v.abs())
        .unwrap_or(0.25);
    let slow_dur = get_skill_data_value(spell_obj, "WSlowDuration", skill_level).unwrap_or(1.5);
    let range = get_skill_cast_radius(spell_obj, skill_level).unwrap_or(825.0);

    for (enemy, enemy_tf) in q_enemies.iter() {
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == caster_team {
            continue;
        }
        let dist = enemy_tf.translation.xz().distance(caster_pos);
        if dist > range {
            continue;
        }
        if dmg > 0.0 {
            commands.entity(enemy).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount: dmg,
                tag: Some(AATROX_W_TAG),
            });
        }
        commands
            .entity(enemy)
            .with_related::<BuffOf>(DebuffSlow::new(slow_pct, slow_dur));
        commands
            .entity(enemy)
            .with_related::<BuffOf>(DebuffAatroxWMark::new(
                entity,
                enemy,
                dmg,
                AATROX_W_MARK_DURATION,
            ));
    }
}

/// W 标记引爆：到时造成等额二次伤害 + 击飞，并移除标记。
fn update_aatrox_w_marks(
    time: Res<Time>,
    mut commands: Commands,
    mut q_marks: Query<(Entity, &mut DebuffAatroxWMark)>,
) {
    for (buff_entity, mut mark) in q_marks.iter_mut() {
        mark.timer.tick(time.delta());
        if mark.timer.just_finished() {
            let target = mark.target;
            let source = mark.source;
            let damage = mark.damage;
            if damage > 0.0 {
                commands.entity(target).trigger(|e| CommandDamageCreate {
                    entity: e,
                    source,
                    damage_type: DamageType::Physical,
                    amount: damage,
                    tag: Some(AATROX_W_TAG),
                });
            }
            commands
                .entity(target)
                .with_related::<BuffOf>(DebuffKnockup::new(0.5));
            commands.entity(buff_entity).despawn();
        }
    }
}

// ════════════════════════ E ════════════════════════

fn on_aatrox_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
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
    cast_aatrox_e(&mut commands, entity, skill.level, spell_obj, trigger.point);
}

/// E：向指针方向突进（最大 300）。
fn cast_aatrox_e(
    commands: &mut Commands,
    entity: Entity,
    skill_level: usize,
    spell_obj: &Spell,
    point: Vec2,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    let max_range =
        get_skill_data_value(spell_obj, "EMaxRange", skill_level).unwrap_or(AATROX_E_MAX_RANGE);
    let speed =
        get_skill_data_value(spell_obj, "EDashSpeed", skill_level).unwrap_or(AATROX_E_DASH_SPEED);
    commands.trigger(ActionDash {
        entity,
        point,
        move_type: DashMoveType::Pointer { max: max_range },
        speed,
    });
}

// ════════════════════════ R ════════════════════════

fn on_aatrox_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
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
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    cast_aatrox_r(&mut commands, entity, skill.level, spell_obj, ad);
}

/// R：10s 内 +额外 AD（= AD × 10%）+ 移速；到期由 `update_aatrox_r` 扣除 AD。
fn cast_aatrox_r(
    commands: &mut Commands,
    entity: Entity,
    skill_level: usize,
    spell_obj: &Spell,
    ad: f32,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    let duration = get_skill_data_value(spell_obj, "RDuration", skill_level).unwrap_or(10.0);
    let ms_bonus =
        get_skill_data_value(spell_obj, "RMovementSpeedBonus", skill_level).unwrap_or(0.4);
    let ad_amp = get_skill_data_value(spell_obj, "RTotalADAmp", skill_level).unwrap_or(0.1);
    let bonus_ad = ad * ad_amp;

    commands
        .entity(entity)
        .insert(AatroxRState::new(duration, bonus_ad));
    commands.entity(entity).queue(move |mut e: EntityWorldMut| {
        if let Some(mut damage) = e.get_mut::<Damage>() {
            damage.0 += bonus_ad;
        }
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(ms_bonus, duration));
}

/// R 持续期：到时移除额外 AD 与状态。
fn update_aatrox_r(
    time: Res<Time>,
    mut commands: Commands,
    mut q_r: Query<(Entity, &mut AatroxRState), With<Aatrox>>,
) {
    for (entity, mut state) in q_r.iter_mut() {
        state.timer.tick(time.delta());
        if state.timer.just_finished() {
            let bonus_ad = state.bonus_ad;
            commands.entity(entity).queue(move |mut e: EntityWorldMut| {
                if let Some(mut damage) = e.get_mut::<Damage>() {
                    damage.0 = (damage.0 - bonus_ad).max(0.0);
                }
            });
            commands.entity(entity).remove::<AatroxRState>();
        }
    }
}
