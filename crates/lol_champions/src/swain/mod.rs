pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage,
    spawn_skill_particle,
};

use crate::swain::buffs::BuffSwainW;

#[derive(Default)]
pub struct PluginSwain;

impl Plugin for PluginSwain {
    fn build(&self, app: &mut App) {
        app.add_observer(on_swain_skill_cast);
        app.add_observer(on_swain_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Swain"))]
#[reflect(Component)]
pub struct Swain;

fn on_swain_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_swain: Query<(), With<Swain>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_swain.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_swain_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_swain_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_swain_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_swain_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_swain_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Swain_Q_Cast"));

    // Q is death flare - damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 700.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Swain_Q_Hit")),
    );
}

fn cast_swain_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Swain_W_Cast"));

    // W is vision of empire - damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Swain_W_Hit")),
    );
}

fn cast_swain_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Swain_E_Cast"));

    // E is nevermove - root
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 650.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Swain_E_Hit")),
    );
}

fn cast_swain_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Swain_R_Cast"));

    // R is demonic ascension - transformation
}

fn on_swain_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_swain: Query<(), With<Swain>>,
) {
    let source = trigger.source;
    if q_swain.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSwainW::new(0.75, 1.0));
}
