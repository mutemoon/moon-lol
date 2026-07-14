pub mod e;
pub mod passive;
pub mod r;

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
use lol_core::action::dash::{ActionDash, DashDamage, DashDamageIntent, DashMoveType};
use lol_core::action::delayed_damage::{ActionDelayedDamage, AoEIndicator, AoEOrigin};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::on_hit::{BuffOnHitBonusDamage, BuffOnHitCounter};
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_data_value,
};
use lol_core::team::Team;

const CAMILLE_E_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginCamille;

impl Plugin for PluginCamille {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                passive::update_camille_passive,
                e::update_camille_e,
                r::update_camille_r_mark,
            ),
        );
        app.add_observer(on_camille_q);
        app.add_observer(on_camille_w);
        app.add_observer(on_camille_e);
        app.add_observer(on_camille_r);
        app.add_observer(on_camille_damage_hit);
        app.add_observer(passive::on_camille_attack_end);
        app.add_observer(r::on_camille_r_attack_end);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Camille"))]
#[reflect(Component)]
pub struct Camille;

fn on_camille_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_buffof: Query<(Entity, &BuffOf)>,
    q_onhit_counter: Query<&BuffOnHitCounter>,
    q_onhit_bonus: Query<&BuffOnHitBonusDamage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
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
    let level = skill.level;
    let tad_ratio = get_skill_data_value(spell_obj, "TADRatio", level).unwrap_or(0.15);
    let empowered_amp = get_skill_data_value(spell_obj, "QEmpoweredAmp", level).unwrap_or(2.0);
    let q2_duration = get_skill_data_value(spell_obj, "Q2Duration", level).unwrap_or(2.0);
    let recast_time = get_skill_data_value(spell_obj, "QTotalRecastTime", level).unwrap_or(3.5);

    cast_camille_q(
        &mut commands,
        entity,
        trigger.skill_entity,
        cooldown,
        recast,
        &q_buffof,
        &q_onhit_counter,
        &q_onhit_bonus,
        tad_ratio,
        empowered_amp,
        q2_duration,
        recast_time,
    );
}

fn on_camille_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
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
    cast_camille_w(
        &mut commands,
        entity,
        skill.spell.clone(),
        skill.level,
        spell_obj,
        trigger.point,
    );
}

fn on_camille_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    let level = skill.level;
    let as_buff = get_skill_data_value(spell_obj, "ASBuff", level).unwrap_or(0.35);
    let as_duration = get_skill_data_value(spell_obj, "ASDuration", level).unwrap_or(5.0);

    cast_camille_e(
        &mut commands,
        &q_transform,
        entity,
        trigger.skill_entity,
        trigger.point,
        cooldown,
        recast,
        skill.spell.clone(),
        as_buff,
        as_duration,
    );
}

fn on_camille_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<&Team, With<Camille>>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(camille_team) = q_camille.get(entity) else {
        return;
    };

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    let level = skill.level;
    let percent = get_skill_data_value(spell_obj, "RPercentCurrentHPDamage", level).unwrap_or(2.0);
    let duration = get_skill_data_value(spell_obj, "RDuration", level).unwrap_or(1.75);

    cast_camille_r(
        &mut commands,
        entity,
        trigger.point,
        &q_enemies,
        *camille_team,
        percent,
        duration,
    );
}

/// 清除角色上既存的强化普攻计数器与额外伤害 buff（Q1→Q2 切换时刷新用）。
fn clear_camille_on_hit(
    commands: &mut Commands,
    camille: Entity,
    q_buffof: &Query<(Entity, &BuffOf)>,
    q_counter: &Query<&BuffOnHitCounter>,
    q_bonus: &Query<&BuffOnHitBonusDamage>,
) {
    for (e, bo) in q_buffof.iter() {
        if bo.0 != camille {
            continue;
        }
        if q_counter.get(e).is_ok() || q_bonus.get(e).is_ok() {
            commands.entity(e).despawn();
        }
    }
}

fn cast_camille_q(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    q_buffof: &Query<(Entity, &BuffOf)>,
    q_onhit_counter: &Query<&BuffOnHitCounter>,
    q_onhit_bonus: &Query<&BuffOnHitBonusDamage>,
    tad_ratio: f32,
    empowered_amp: f32,
    q2_duration: f32,
    recast_time: f32,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    // 切换阶段时刷新既存的强化普攻 buff（避免两份计数器叠加）
    clear_camille_on_hit(commands, entity, q_buffof, q_onhit_counter, q_onhit_bonus);

    // 重置普攻：Q 命中由下次普攻触发
    commands.trigger(CommandAttackReset { entity });

    if stage == 1 {
        // 第一段：强化下次普攻（bonus = AD × TADRatio），开启重施窗口
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffOnHitCounter::new(1, q2_duration))
            .with_related::<BuffOf>(BuffOnHitBonusDamage {
                flat: 0.0,
                ratio: tad_ratio,
            });
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, recast_time));
    } else {
        // 第二段（蓄力后）：强化下次普攻（bonus = AD × TADRatio × QEmpoweredAmp）
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffOnHitCounter::new(1, q2_duration))
            .with_related::<BuffOf>(BuffOnHitBonusDamage {
                flat: 0.0,
                ratio: tad_ratio * empowered_amp,
            });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    }
}

fn cast_camille_w(
    commands: &mut Commands,
    entity: Entity,
    skill_spell: Handle<Spell>,
    skill_level: usize,
    spell_obj: &Spell,
    point: Vec2,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    // W 蓄力锥形（扇形）延迟伤害：以施法者为顶点，朝施法点方向，蓄力后造成物理伤害 + 减速。
    // 组合表达（验证 AoEOrigin::Caster + Sector 原语）：
    // - 原点 = Caster（扇形顶点在施法者）
    // - 朝向 = 施法者 → 施法点（forward，由系统快照）
    // - 形状 = Sector{ radius=BlastLength=650, angle=ConeAngle=35 }
    // - 延迟 = dataValue ChargeDuration=0.75（蓄力）
    // - 伤害 = spell ron 的 base_damage_total（物理）
    // - 减速由 on_camille_damage_hit observer 在伤害结算时施加
    let radius = get_skill_data_value(spell_obj, "BlastLength", skill_level).unwrap_or(650.0);
    let angle = get_skill_data_value(spell_obj, "ConeAngle", skill_level).unwrap_or(35.0);
    let delay = get_skill_data_value(spell_obj, "ChargeDuration", skill_level).unwrap_or(0.75);

    commands.trigger(ActionDelayedDamage {
        entity,
        skill: skill_spell,
        skill_level,
        delay,
        point,
        origin: AoEOrigin::Caster,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector { radius, angle },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "base_damage_total".to_string(),
                damage_type: DamageType::Physical,
                ..Default::default()
            }],
            ..Default::default()
        }],
        indicator: AoEIndicator {
            color: Color::srgba(0.9, 0.3, 0.5, 0.4),
            pulse: true,
            grow_from_zero: false,
            impact_burst_scale: 1.4,
            fade_duration: 0.3,
        },
    });
}

fn cast_camille_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    skill_spell: Handle<Spell>,
    as_buff: f32,
    as_duration: f32,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: Hookshot - launches toward terrain
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, CAMILLE_E_RECAST_WINDOW));
    } else {
        // Second cast: Dash toward hooked terrain + 攻速加成
        commands.entity(entity).insert(DashDamageIntent {
            damage: DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Physical,
                    ..Default::default()
                },
            },
            skill: skill_spell,
        });
        commands.trigger(ActionDash {
            entity,
            point,
            move_type: DashMoveType::Pointer { max: 400.0 },
            speed: 900.0,
        });
        e::apply_camille_e_as(commands, entity, as_buff, as_duration);
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    }
}

fn cast_camille_r(
    commands: &mut Commands,
    entity: Entity,
    point: Vec2,
    q_enemies: &Query<(Entity, &Transform, &Team), With<Champion>>,
    camille_team: Team,
    percent: f32,
    duration: f32,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    // 标记距施法点最近的敌方英雄（R 目标）
    let nearest = q_enemies
        .iter()
        .filter(|(_, _, team)| **team != camille_team)
        .min_by(|a, b| {
            let da = (a.1.translation.xz() - point).length_squared();
            let db = (b.1.translation.xz() - point).length_squared();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(e, _, _)| e);
    if let Some(target) = nearest {
        r::apply_camille_r_mark(commands, target, percent, duration);
    }

    // R 的伤害来自标记后的普攻额外伤害（% 当前生命值魔法），位移本身只管运动。
    // CamilleR.ron 无 flat 伤害计算式，故不挂 DashDamageIntent（否则 get_skill_value 会 panic）。
    commands.trigger(ActionDash {
        entity,
        point,
        move_type: DashMoveType::Pointer { max: 350.0 },
        speed: 800.0,
    });
}

/// 监听 Camille 造成的伤害，给目标施加减速
fn on_camille_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
) {
    let source = trigger.source;
    if q_camille.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.6, 2.0));
}
