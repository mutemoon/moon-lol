pub mod buffs;
pub mod e;
pub mod passive;
pub mod q;
pub mod r;
pub mod w;

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
use lol_base::spell::Spell;
use lol_core::base::buff::Buffs;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::{AbilityPower, CommandDamageCreate, DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::movement::Movement;
use lol_core::skill::{EventSkillCast, Skill, SkillRecastWindow, SkillSlot};
use lol_core::team::Team;

use crate::mordekaiser::buffs::{
    BuffMordekaiserWShield, MORDE_W_DECAY_PER_SECOND, MORDE_W_DURATION, MORDE_W_TIME_BEFORE_DECAY,
    MordekaiserRealm, MordekaiserWStorage,
};
use crate::mordekaiser::e::cast_mordekaiser_e as execute_mordekaiser_e;
use crate::mordekaiser::passive::{
    MORDE_PASSIVE_AUTO_AP_RATIO, MORDE_PASSIVE_AUTO_TAG, MORDE_PASSIVE_DOT_AP_RATIO,
    MORDE_PASSIVE_DOT_RADIUS, MORDE_PASSIVE_DOT_TAG, MORDE_PASSIVE_MAX_STACKS,
    MORDE_PASSIVE_MS_BONUS, MordekaiserDarkness,
};
use crate::mordekaiser::q::cast_mordekaiser_q as execute_mordekaiser_q;
use crate::mordekaiser::r::cast_mordekaiser_r as execute_mordekaiser_r;
use crate::mordekaiser::w::cast_mordekaiser_w as execute_mordekaiser_w;

#[derive(Default)]
pub struct PluginMordekaiser;

impl Plugin for PluginMordekaiser {
    fn build(&self, app: &mut App) {
        app.add_observer(on_mordekaiser_q);
        app.add_observer(on_mordekaiser_w);
        app.add_observer(on_mordekaiser_e);
        app.add_observer(on_mordekaiser_r);
        app.add_observer(on_mordekaiser_damage_hit);
        app.add_observer(on_mordekaiser_damage_taken);
        app.add_systems(FixedUpdate, update_mordekaiser_passive);
        app.add_systems(FixedUpdate, update_mordekaiser_w_shield);
        app.add_systems(FixedUpdate, crate::mordekaiser::r::update_mordekaiser_realm);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Mordekaiser"))]
#[reflect(Component)]
pub struct Mordekaiser;

fn on_mordekaiser_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    execute_mordekaiser_q(
        &mut commands,
        entity,
        skill.spell.clone(),
        skill.level,
        trigger.point,
        spell_obj,
    );
}

fn on_mordekaiser_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
    q_health: Query<&Health>,
    q_buffs: Query<&Buffs>,
    q_w_shield: Query<&BuffMordekaiserWShield>,
    q_shield_white: Query<&BuffShieldWhite>,
    q_storage: Query<&MordekaiserWStorage>,
    q_recast: Query<&SkillRecastWindow>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
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

    execute_mordekaiser_w(
        &mut commands,
        entity,
        trigger.skill_entity,
        skill.level,
        spell_obj,
        &q_health,
        &q_buffs,
        &q_w_shield,
        &q_shield_white,
        &q_storage,
        &q_recast,
    );
}

fn on_mordekaiser_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_ap: Query<&AbilityPower>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
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

    execute_mordekaiser_e(
        &mut commands,
        entity,
        skill.spell.clone(),
        trigger.point,
        skill.level,
        spell_obj,
        &q_transform,
        &q_team,
        &q_enemies,
        &q_ap,
    );
}

fn on_mordekaiser_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_realm: Query<&MordekaiserRealm>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
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

    execute_mordekaiser_r(
        &mut commands,
        entity,
        trigger.point,
        skill.level,
        spell_obj,
        &q_transform,
        &q_team,
        &q_enemies,
        &q_realm,
    );
}

/// 监听莫德凯撒造成的伤害：叠加被动 Darkness，满层激活后普攻附带 40% AP 魔法伤害。
fn on_mordekaiser_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    mut q_darkness: Query<&mut MordekaiserDarkness>,
    q_ap: Query<&AbilityPower>,
    mut q_movement: Query<&mut Movement>,
) {
    let morde = trigger.source;
    if q_morde.get(morde).is_err() {
        return;
    }
    // DoT 与普攻附伤不再叠层，避免递归
    let tag = trigger.event().tag;
    if tag == Some(MORDE_PASSIVE_DOT_TAG) || tag == Some(MORDE_PASSIVE_AUTO_TAG) {
        return;
    }

    let now_active = match q_darkness.get_mut(morde) {
        Ok(mut d) => {
            d.stacks = (d.stacks + 1).min(MORDE_PASSIVE_MAX_STACKS);
            d.combat_timer.reset();
            if d.stacks >= MORDE_PASSIVE_MAX_STACKS && !d.active {
                d.active = true;
                d.dot_timer.reset();
                // 激活：+3% 移速（基于当前移速）
                if let Ok(mut mv) = q_movement.get_mut(morde) {
                    let bonus = mv.speed * MORDE_PASSIVE_MS_BONUS;
                    d.ms_bonus = bonus;
                    mv.speed += bonus;
                }
            }
            d.active
        }
        Err(_) => {
            // 首次命中：插入 1 层
            commands.entity(morde).insert(MordekaiserDarkness::new());
            false
        }
    };

    // 激活期间普攻（物理）附带 40% AP 魔法伤害
    if now_active && trigger.event().damage_type == DamageType::Physical {
        let ap = q_ap.get(morde).map(|a| a.0).unwrap_or(0.0);
        let bonus = MORDE_PASSIVE_AUTO_AP_RATIO * ap;
        if bonus > 0.0 {
            let target = trigger.event_target();
            commands.entity(target).trigger(|e| CommandDamageCreate {
                entity: e,
                source: morde,
                damage_type: DamageType::Magic,
                amount: bonus,
                tag: Some(MORDE_PASSIVE_AUTO_TAG),
            });
        }
    }
}

/// 监听莫德凯撒受到的伤害：按 7.5% 储存为 W 护盾原料（上限 30% 最大生命）。
fn on_mordekaiser_damage_taken(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_storage: Query<&MordekaiserWStorage>,
    q_health: Query<&Health>,
) {
    let morde = trigger.event_target();
    if q_morde.get(morde).is_err() {
        return;
    }
    let final_dmg = trigger.event().damage_result.final_damage;
    if final_dmg <= 0.0 {
        return;
    }
    let max_hp = q_health.get(morde).map(|h| h.max).unwrap_or(0.0);
    let cap = crate::mordekaiser::buffs::MORDE_W_MAX_HEALTH_CAP * max_hp;
    let gained = final_dmg * crate::mordekaiser::buffs::MORDE_W_DAMAGE_TAKEN_CONVERSION;
    let new_stored = match q_storage.get(morde) {
        Ok(s) => (s.stored + gained).min(cap),
        Err(_) => gained.min(cap),
    };
    commands
        .entity(morde)
        .insert(MordekaiserWStorage { stored: new_stored });
}

/// 被动 Darkness：脱战 4 秒失效；激活期间每 0.5 秒对半径内敌人造成 30% AP×层数 魔法伤害。
fn update_mordekaiser_passive(
    mut commands: Commands,
    time: Res<Time>,
    mut q_morde: Query<(Entity, &mut MordekaiserDarkness, &Transform, &Team), With<Mordekaiser>>,
    q_ap: Query<&AbilityPower>,
    mut q_movement: Query<&mut Movement>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
) {
    for (morde, mut darkness, transform, team) in q_morde.iter_mut() {
        darkness.combat_timer.tick(time.delta());
        if darkness.combat_timer.just_finished() {
            // 脱战：失效并还原移速
            if darkness.active {
                if let Ok(mut mv) = q_movement.get_mut(morde) {
                    mv.speed -= darkness.ms_bonus;
                }
            }
            darkness.stacks = 0;
            darkness.active = false;
            darkness.ms_bonus = 0.0;
            continue;
        }

        if darkness.active {
            darkness.dot_timer.tick(time.delta());
            if darkness.dot_timer.just_finished() {
                let ap = q_ap.get(morde).map(|a| a.0).unwrap_or(0.0);
                let amount = MORDE_PASSIVE_DOT_AP_RATIO * ap * darkness.stacks as f32;
                if amount > 0.0 {
                    let pos = transform.translation;
                    for (enemy, enemy_tf, enemy_team) in q_enemies.iter() {
                        if enemy_team == team {
                            continue;
                        }
                        if enemy_tf.translation.distance(pos) > MORDE_PASSIVE_DOT_RADIUS {
                            continue;
                        }
                        commands.entity(enemy).trigger(|e| CommandDamageCreate {
                            entity: e,
                            source: morde,
                            damage_type: DamageType::Magic,
                            amount,
                            tag: Some(MORDE_PASSIVE_DOT_TAG),
                        });
                    }
                }
            }
        }
    }
}

/// W 护盾：1 秒后每秒衰减 0.5% 最大生命，5 秒到期移除。
fn update_mordekaiser_w_shield(
    mut commands: Commands,
    time: Res<Time>,
    mut q_shield: Query<(Entity, &mut BuffShieldWhite, &mut BuffMordekaiserWShield)>,
) {
    let dt = time.delta().as_secs_f32();
    for (entity, mut shield, mut tracker) in q_shield.iter_mut() {
        tracker.elapsed += dt;
        if tracker.elapsed > MORDE_W_TIME_BEFORE_DECAY {
            let decay = MORDE_W_DECAY_PER_SECOND * tracker.max_health * dt;
            shield.current = (shield.current - decay).max(0.0);
        }
        if tracker.elapsed >= MORDE_W_DURATION {
            commands.entity(entity).despawn();
        }
    }
}
