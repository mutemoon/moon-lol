pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage,
    spawn_skill_particle,
};

use crate::brand::buffs::BuffBrandPassive;

#[derive(Default)]
pub struct PluginBrand;

impl Plugin for PluginBrand {
    fn build(&self, app: &mut App) {
        app.add_observer(on_brand_skill_cast);
        app.add_observer(on_brand_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Brand"))]
#[reflect(Component)]
pub struct Brand;

fn on_brand_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_brand: Query<(), With<Brand>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_brand.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_brand_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_brand_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_brand_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_brand_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_brand_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Brand_Q_Cast"));

    // Q is a fireball that stuns ablazed targets
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1100.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Brand_Q_Hit")),
    );
}

fn cast_brand_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Brand_W_Cast"));

    // W is a ground targeted area
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Brand_W_Hit")),
    );
}

fn cast_brand_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Brand_E_Cast"));

    // E spreads to nearby enemies
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 675.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Brand_E_Hit")),
    );
}

fn cast_brand_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Brand_R_Cast"));

    // R bounces between enemies
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 750.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Brand_R_Hit")),
    );
}

fn on_brand_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_brand: Query<(), With<Brand>>,
) {
    let source = trigger.source;
    if q_brand.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply blaze passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBrandPassive::new());
    // R slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}
