pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, skill_dash,
    spawn_skill_particle,
};

use crate::kayn::buffs::BuffKaynRActive;

#[derive(Default)]
pub struct PluginKayn;

impl Plugin for PluginKayn {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kayn_skill_cast);
        app.add_observer(on_kayn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kayn"))]
#[reflect(Component)]
pub struct Kayn;

/// Kayn has two forms: Blue (assassin) and Red (bruiser)
/// The form is determined by which enemy champion is damaged first with R
#[derive(Component, Reflect, Default, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
pub enum KaynForm {
    #[default]
    None,
    Blue, // Assassin form
    Red,  // Bruiser form
}

fn on_kayn_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayn: Query<(), With<Kayn>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kayn.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_kayn_q(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::W => cast_kayn_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_kayn_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_kayn_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_kayn_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Kayn_Q_Cast"));
    // Q is a dash that deals damage
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Fixed(250.0),
            damage: Some(DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 700.0,
        },
    );
}

fn cast_kayn_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Kayn_W_Cast"));
    // W is an upward slash that slows
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 300.0,
            angle: 60.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kayn_W_Hit")),
    );
}

fn cast_kayn_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Kayn_E_Cast"));
    // E is a ghost-like dash that allows passing through terrain
    // Movement speed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(0.4, 1.5));
}

fn cast_kayn_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Kayn_R_Cast"));
    // R 寄生：给自身挂 BuffKaynRActive（不可选中状态）
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKaynRActive::new(Entity::PLACEHOLDER, 2.5));
}

/// 监听 Kayn 造成的伤害，W 命中时减速
fn on_kayn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kayn: Query<(), With<Kayn>>,
) {
    let source = trigger.source;
    if q_kayn.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // W 命中时减速
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.6, 1.5));
}
