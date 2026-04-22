use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, skill_dash,
    spawn_skill_particle,
};

#[derive(Default)]
pub struct PluginGnar;

impl Plugin for PluginGnar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_gnar_skill_cast);
        app.add_observer(on_gnar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Gnar"))]
#[reflect(Component)]
pub struct Gnar;

/// Gnar transforms into Mega Gnar at 100 rage
#[derive(Component, Reflect, Default, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
pub enum GnarForm {
    #[default]
    Mini,
    Mega,
}

fn on_gnar_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gnar: Query<(), With<Gnar>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gnar.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_gnar_q(&mut commands, entity, skill.key_spell_object.clone()),
        SkillSlot::W => cast_gnar_w(&mut commands, entity, skill.key_spell_object.clone()),
        SkillSlot::E => cast_gnar_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill.key_spell_object.clone(),
        ),
        SkillSlot::R => cast_gnar_r(&mut commands, entity, skill.key_spell_object.clone()),
        _ => {}
    }
}

fn cast_gnar_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Gnar_Q_Cast"));
    // Q 回旋镖：Sector 模拟直线飞行
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 500.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Gnar_Q_Hit")),
    );
}

fn cast_gnar_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Gnar_W_Cast"));
    // Mega 形态 W：AOE 伤害 + 眩晕
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 250.0,
            angle: 60.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Gnar_W_Hit")),
    );
}

fn cast_gnar_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Gnar_E_Cast"));
    // E 是跳跃，可以二段跳
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell.clone(),
            move_type: DashMoveType::Pointer { max: 300.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 600.0,
        },
    );
}

fn cast_gnar_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Gnar_R_Cast"));
    // R is only available in Mega form - throws enemies and stuns
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Gnar_R_Hit")),
    );
}

/// 监听 Gnar 造成的伤害，所有伤害命中施加减速
fn on_gnar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_gnar: Query<(), With<Gnar>>,
) {
    let source = trigger.source;
    if q_gnar.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // 所有伤害命中施加减速
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 1.5));
}
