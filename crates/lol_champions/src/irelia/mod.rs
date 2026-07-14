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
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::buffs::cc_debuffs::{DebuffSlow, DebuffStun};
use lol_core::buffs::damage_reduction::BuffDamageReduction;
use lol_core::damage::{Damage, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};
use lol_core::team::Team;

use crate::irelia::buffs::DebuffIreliaUnsteady;
use crate::irelia::w::BuffIreliaW;

// ── 伤害标签：区分 Irelia 不同伤害来源，供 on_irelia_damage_hit 识别 ──
pub const IRELIA_Q_DAMAGE_TAG: u32 = 1;
pub const IRELIA_E2_DAMAGE_TAG: u32 = 2;
pub const IRELIA_R_DAMAGE_TAG: u32 = 3;

// ── Q ──
/// Q 命中判定距离（ron `castRange`）
pub const IRELIA_Q_RANGE: f32 = 600.0;

// ── E2 命中附加 ──
/// E2 眩晕时长（ron `StunDuration`）
pub const IRELIA_E_STUN_DURATION: f32 = 0.75;
/// E2/R 不稳标记时长（ron `MarkDuration` 真实值为 5s，idx0 占位为 0）
pub const IRELIA_MARK_DURATION: f32 = 5.0;

// ── R 命中附加 ──
/// R 减速比例（ron `SlowAmount` 90%）
pub const IRELIA_R_SLOW_PERCENT: f32 = 0.9;
/// R 减速时长（ron `CCDuration`）
pub const IRELIA_R_SLOW_DURATION: f32 = 1.5;

#[derive(Default)]
pub struct PluginIrelia;

impl Plugin for PluginIrelia {
    fn build(&self, app: &mut App) {
        app.add_observer(on_irelia_q);
        app.add_observer(on_irelia_w);
        app.add_observer(on_irelia_e);
        app.add_observer(on_irelia_r);
        app.add_observer(passive::on_irelia_skill_cast_stack_passive);
        app.add_observer(passive::on_irelia_passive_attack_end);
        app.add_observer(on_irelia_damage_hit);
        app.add_systems(FixedUpdate, buffs::update_irelia_unsteady);
        app.add_systems(FixedUpdate, passive::update_irelia_fervor);
        app.add_systems(FixedUpdate, w::update_irelia_w);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Irelia"))]
#[reflect(Component)]
pub struct Irelia;

fn on_irelia_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_irelia: Query<&Team, With<Irelia>>,
    q_skill: Query<(&Skill, &CoolDown)>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_buffs: Query<&Buffs>,
    q_unsteady: Query<&DebuffIreliaUnsteady>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(team) = q_irelia.get(entity) else {
        return;
    };
    let Ok((skill, cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }
    let Some(spell) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);

    q::cast_irelia_q(
        &mut commands,
        entity,
        trigger.point,
        trigger.skill_entity,
        skill.level,
        spell,
        &q_enemies,
        *team,
        &q_buffs,
        &q_unsteady,
        cooldown,
        ad,
    );
}

fn on_irelia_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_irelia: Query<&Team, With<Irelia>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_buffs: Query<&Buffs>,
    q_w: Query<&BuffIreliaW>,
    q_dr: Query<&BuffDamageReduction>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(team) = q_irelia.get(entity) else {
        return;
    };
    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }
    let Some(spell) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    w::cast_irelia_w(
        &mut commands,
        entity,
        trigger.skill_entity,
        stage,
        trigger.point,
        spell,
        skill.level,
        cooldown,
        &q_enemies,
        *team,
        &q_buffs,
        &q_w,
        &q_dr,
        ad,
    );
}

fn on_irelia_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_irelia: Query<&Team, With<Irelia>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(team) = q_irelia.get(entity) else {
        return;
    };
    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }
    let Some(spell) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    e::cast_irelia_e(
        &mut commands,
        entity,
        trigger.skill_entity,
        stage,
        trigger.point,
        spell,
        skill.level,
        cooldown,
        &q_enemies,
        *team,
        ad,
    );
}

fn on_irelia_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_irelia: Query<&Team, With<Irelia>>,
    q_skill: Query<&Skill>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(team) = q_irelia.get(entity) else {
        return;
    };
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);

    r::cast_irelia_r(
        &mut commands,
        entity,
        trigger.point,
        skill.level,
        spell,
        &q_enemies,
        *team,
        ad,
    );
}

/// 监听 Irelia 造成的伤害：仅 E2/R 标签命中施加控制与标记，Q 与普攻附加伤害不触发。
fn on_irelia_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_irelia: Query<(), With<Irelia>>,
) {
    let source = trigger.source;
    if q_irelia.get(source).is_err() {
        return;
    }
    let Some(tag) = trigger.event().tag else {
        return;
    };
    let target = trigger.event_target();

    if tag == IRELIA_E2_DAMAGE_TAG {
        // E2：眩晕 + 不稳标记
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffStun::new(IRELIA_E_STUN_DURATION));
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffIreliaUnsteady::new(IRELIA_MARK_DURATION));
    } else if tag == IRELIA_R_DAMAGE_TAG {
        // R：不稳标记 + 减速
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffIreliaUnsteady::new(IRELIA_MARK_DURATION));
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffSlow::new(
                IRELIA_R_SLOW_PERCENT,
                IRELIA_R_SLOW_DURATION,
            ));
    }
}
